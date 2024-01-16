use bitflags::bitflags;

use core::ptr::NonNull;
use core::mem;

use crate::arch::paging::{self, PAGE_SIZE};
use crate::memory::PHYSICAL_MEMORY_MANAGER;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct VirtualMemoryFlags: u8 {
        const Writeable = 1 << 0;
        const Executable = 1 << 1;
        const UserAccessible = 1 << 2;
    }
}


pub struct VirtualMemoryObject {
    pub base: u64,
    pub total_pages: usize,
    pub flags: VirtualMemoryFlags,
    pub is_used: bool,
    pub next: Option<NonNull<VirtualMemoryObject>>,
}

pub struct VirtualMemoryManager {
    root_object: Option<NonNull<VirtualMemoryObject>>,
    current_object: Option<NonNull<VirtualMemoryObject>>,
    pagemap: NonNull<u64>,

    base_address: u64,
    current_address: u64,
    flags: VirtualMemoryFlags
}

unsafe impl Send for VirtualMemoryManager {}
unsafe impl Send for VirtualMemoryObject {}

impl VirtualMemoryManager {

    pub const NULL: Self = Self {
        root_object: None,
        current_object: None,

        pagemap: NonNull::dangling(),
        base_address: 0,
        current_address: 0,
        flags: VirtualMemoryFlags::empty()
    };

    pub fn initialize(&mut self, pagemap: NonNull<u64>, base_address: u64, flags: VirtualMemoryFlags) {
        self.pagemap = pagemap;
        self.base_address = base_address;
        self.current_address = base_address;
        self.flags = flags;
    }

    pub unsafe fn allocate_object(&mut self, pages: usize) -> Option<NonNull<VirtualMemoryObject>> {
        for i in 0..pages {
            let page = PHYSICAL_MEMORY_MANAGER.lock().allocate_page().expect("Can't allocated page");
            let virtual_address = self.current_address + i as u64 * PAGE_SIZE;
            paging::map(self.pagemap.as_ptr(), virtual_address, page.as_ptr() as u64, self.flags);
        }
        
        let object = { 
            let object = self.current_address as *mut VirtualMemoryObject;
            let object = object.as_mut().unwrap();
            object.next = None;
            object.is_used = true;
            object.total_pages = pages;
            object.base = self.current_address + mem::size_of::<VirtualMemoryObject>() as u64;
            Some(NonNull::new_unchecked(object))
        };

        if self.root_object.is_none() {
            self.root_object = object;
            self.current_object = object;
        } else {
            self.current_object.unwrap().as_mut().next = object;
            self.current_object = object;
        }

        self.current_address += pages as u64 * PAGE_SIZE;
        return object;
    }

    pub unsafe fn free_object(&mut self, address: u64) {
        let mut node = self.root_object;
        while let Some(mut object) = node {
            let object = object.as_mut();
            if object.base == address {
                object.is_used = false;
                return;
            }
            node = object.next;
        }

        panic!("tried to free an invalid object");
    }


}
