#![allow(unused)]

use limine::{MemmapEntry, MemmapResponse, MemoryMapEntryType};
use core::ptr::NonNull;
use core::mem;

use crate::arch::paging::PAGE_SIZE;
use crate::misc::log;
use crate::boot::HHDM_OFFSET;


#[derive(Debug)]
struct Bitmap {
    last_index_used: u64,
    used_pages: u64,
    total_pages: u64,
    data: *mut u8,
}

impl Bitmap {
    unsafe fn set_used(&mut self, index: usize) {
        let row = index / 8;
        let col = index % 8;

        let data = self.data as u64 as *mut u8;
        let value = data.add(row);

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

    unsafe fn is_full(&self) -> bool {
        self.total_pages == self.used_pages
    }

    fn get_free_index(&mut self) -> Option<usize> {
        if self.last_index_used < self.total_pages {
            self.last_index_used += 1;
            return Some(self.last_index_used as usize);
        }

        None
    }
}

static BITMAP_SIZE: u64 = mem::size_of::<Bitmap>() as u64;

trait BitmapInstallable {
    unsafe fn get_bitmap(&self) -> &mut Bitmap;
    unsafe fn install(&mut self);
}

impl BitmapInstallable for MemmapEntry {
    unsafe fn install(&mut self) {
        let bitmap = self.get_bitmap();
        let total_pages = self.len / PAGE_SIZE;
        let reserved_pages_for_bitmap = (total_pages + BITMAP_SIZE.div_ceil(PAGE_SIZE)).div_ceil(8);
        
        bitmap.total_pages = total_pages;
        bitmap.last_index_used = reserved_pages_for_bitmap;
        bitmap.used_pages = 0;
        bitmap.data = (self.base + BITMAP_SIZE + HHDM_OFFSET) as *mut u8;

        for i in 0..reserved_pages_for_bitmap as usize {
            bitmap.set_used(i);
        }
    }

    unsafe fn get_bitmap(&self) -> &mut Bitmap {
        return ((self.base + HHDM_OFFSET) as *mut Bitmap).as_mut().unwrap();
    }
}

pub struct BitmapAllocator<'a> {
    response: Option<&'a MemmapResponse>
}

unsafe impl<'a> Send for BitmapAllocator<'a> {}

impl<'a> BitmapAllocator<'a> {
    pub fn initialize(&mut self, memmap: &'a MemmapResponse) {
        let entry_count = memmap.entry_count;
        let entries = &memmap.entries;

        self.response = Some(memmap);

        for i in 0..entry_count as usize {
            unsafe {
                let entry = entries.as_ptr().add(i).as_mut().unwrap();
                if entry.typ == MemoryMapEntryType::Usable {
                    entry.install();
                    let bitmap = entry.get_bitmap();
                    log::debug!("{:?}", bitmap);
                }
            }
        }
    }

    pub unsafe fn allocate_page(&mut self) -> Option<NonNull<u64>> {
        if self.response.is_none() {
            panic!("Bitmap allocator is not initialized");
        }

        let response = self.response.unwrap();
        let entry_count = response.entry_count;
        let entries = &response.entries;

        for i in 0..entry_count as usize {
            let entry = entries.as_ptr().add(i).as_mut().unwrap();

            // skip unusable entries
            if entry.typ != MemoryMapEntryType::Usable {
                continue;
            }

            // skip full entries
            let bitmap = entry.get_bitmap();
            if bitmap.is_full() {
                continue;
            }

            let index = bitmap.get_free_index().unwrap();
            bitmap.set_used(index);

            let address = entry.base + index as u64 * PAGE_SIZE + HHDM_OFFSET;
            return Some(NonNull::new_unchecked(address as *mut u64));
        }
        return None;
    }

    pub unsafe fn free_page(&mut self, addr: NonNull<u64>) {
        let addr: u64 = addr.as_ptr() as u64;

        let response = self.response.unwrap();
        let entry_count = response.entry_count;
        let entries = &response.entries;

        for i in 0..entry_count as usize {
            let entry = entries.as_ptr().add(i).as_mut().unwrap();

            if entry.typ != MemoryMapEntryType::Usable {
                continue;
            }

            let bitmap = entry.get_bitmap();
            if entry.base > addr && addr < entry.base + entry.len {
                let mut index = 0;
                loop {
                    if addr == index * PAGE_SIZE {
                        bitmap.set_free(index as usize);
                        break;
                    }
                    index += 1;
                }
                break;
            }
        }

        panic!("Tried to free an invalid address");
    }

    pub fn new() -> BitmapAllocator<'a> {
        BitmapAllocator {
            response: None
        }
    }
}
