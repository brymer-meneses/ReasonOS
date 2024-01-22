#![allow(unused)]

use core::mem;
use core::ptr::NonNull;
use limine::{MemmapEntry, MemmapResponse, MemoryMapEntryType};
use spin::Mutex;

use crate::arch::paging::PAGE_SIZE;
use crate::boot::HHDM_OFFSET;
use crate::misc::log;

#[derive(Debug)]
struct Bitmap {
    last_index_used: usize,
    used_pages: usize,
    total_pages: usize,
    data: *mut u8,
}

impl Bitmap {
    unsafe fn set_used(&mut self, index: usize) {
        let row = index / 8;
        let col = index % 8;

        let value = self.data.add(row);

        *value |= 1 << col;

        self.used_pages += 1;
    }

    unsafe fn set_free(&mut self, index: usize) {
        let row = index / 8;
        let col = index % 8;

        let value = self.data.add(row);
        *value &= !(1 << col);

        self.used_pages -= 1;
    }

    fn is_used(&self, index: usize) -> bool {
        let row = index / 8;
        let col = index % 8;

        let value = unsafe { self.data.add(row).read() };
        (value >> col) & 1 == 1
    }

    fn is_full(&self) -> bool {
        self.total_pages == self.used_pages
    }

    fn get_free_index(&mut self) -> Option<usize> {
        while self.used_pages < self.total_pages {
            if self.last_index_used > self.used_pages {
                self.last_index_used = 0;
            }

            if !self.is_used(self.last_index_used) {
                unsafe {
                    self.set_used(self.last_index_used);
                }

                self.last_index_used += 1;
                return Some(self.last_index_used - 1);
            }

            self.last_index_used += 1;
        }

        None
    }
}

static BITMAP_SIZE: u64 = mem::size_of::<Bitmap>() as u64;

trait BitmapInstallable {
    unsafe fn get_bitmap(&self) -> NonNull<Bitmap>;
    unsafe fn install(&mut self);
}

impl BitmapInstallable for MemmapEntry {
    unsafe fn install(&mut self) {
        let bitmap = self.get_bitmap().as_mut();
        let total_pages = self.len / PAGE_SIZE;
        let reserved_pages_for_bitmap = (total_pages + BITMAP_SIZE.div_ceil(PAGE_SIZE)).div_ceil(8);

        bitmap.total_pages = total_pages as usize;
        bitmap.last_index_used = reserved_pages_for_bitmap as usize;
        bitmap.used_pages = 0;
        bitmap.data = (self.base + BITMAP_SIZE + HHDM_OFFSET) as *mut u8;

        for i in 0..reserved_pages_for_bitmap as usize {
            bitmap.set_used(i);
        }
    }

    unsafe fn get_bitmap(&self) -> NonNull<Bitmap> {
        NonNull::new_unchecked((self.base + HHDM_OFFSET) as *mut Bitmap)
    }
}

pub struct BitmapAllocator(&'static MemmapResponse);

unsafe impl Send for BitmapAllocator {}

impl BitmapAllocator {
    pub fn new(memmap: &'static MemmapResponse) -> Self {
        let entry_count = memmap.entry_count;
        let entries = &memmap.entries;

        for i in 0..entry_count as usize {
            unsafe {
                let entry = entries.as_ptr().add(i).as_mut().unwrap();
                if entry.typ == MemoryMapEntryType::Usable {
                    entry.install();
                }
            }
        }

        BitmapAllocator(memmap)
    }

    pub unsafe fn allocate_page(&mut self) -> Option<NonNull<u64>> {
        let response = self.0;
        let entry_count = response.entry_count;
        let entries = &response.entries;

        for i in 0..entry_count as usize {
            let entry = entries.as_ptr().add(i).as_mut().unwrap();

            // skip unusable entries
            if entry.typ != MemoryMapEntryType::Usable {
                continue;
            }

            // skip full entries
            let bitmap = entry.get_bitmap().as_mut();
            if bitmap.is_full() {
                continue;
            }

            let index = bitmap.get_free_index().expect("Failed to get free index");
            let address = entry.base + index as u64 * PAGE_SIZE;
            return Some(NonNull::new_unchecked(address as *mut u64));
        }

        None
    }

    pub unsafe fn free_page(&mut self, addr: NonNull<u64>) {
        let addr: u64 = addr.as_ptr() as u64;

        let response = self.0;
        let entry_count = response.entry_count;
        let entries = &response.entries;

        for i in 0..entry_count as usize {
            let entry = entries.as_ptr().add(i).as_mut().unwrap();

            if entry.typ != MemoryMapEntryType::Usable {
                continue;
            }

            let bitmap = entry.get_bitmap().as_mut();

            // can't be equal since bitmap is `installed` at the beginning of each entry
            if entry.base > addr && addr <= entry.base + entry.len {
                if let Some(index) = get_index_from_address(entry, addr) {
                    bitmap.set_free(index);
                    return;
                }
            }
        }

        panic!("Tried to free an invalid address 0x{:016X?}", addr);
    }
}

const fn get_index_from_address(entry: &MemmapEntry, addr: u64) -> Option<usize> {
    let mut index = 0;
    while index * PAGE_SIZE < entry.base + entry.len {
        if index * PAGE_SIZE == addr {
            return Some(index as usize);
        }
        index += 1;
    }
    None
}
