use bitflags::bitflags;

use core::mem;
use core::ptr::NonNull;

use crate::arch::paging::{self, PAGE_SIZE};
use crate::memory::address::VirtualAddress;
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
    pub base: VirtualAddress,
    pub total_pages: usize,
    pub flags: VirtualMemoryFlags,
    pub is_used: bool,
    pub next: Option<NonNull<VirtualMemoryObject>>,
}

pub struct VirtualMemoryManager {
    root_object: Option<NonNull<VirtualMemoryObject>>,
    current_object: Option<NonNull<VirtualMemoryObject>>,
    pagemap: VirtualAddress,

    base_address: VirtualAddress,
    current_address: VirtualAddress,
    flags: VirtualMemoryFlags,
}

unsafe impl Send for VirtualMemoryManager {}
unsafe impl Send for VirtualMemoryObject {}

impl VirtualMemoryManager {
    pub fn new(
        pagemap: VirtualAddress,
        base_address: VirtualAddress,
        flags: VirtualMemoryFlags,
    ) -> Self {
        Self {
            pagemap,
            base_address,
            current_address: base_address,
            flags,
            root_object: None,
            current_object: None,
        }
    }

    pub unsafe fn allocate_object(&mut self, pages: usize) -> Option<NonNull<VirtualMemoryObject>> {
        for i in 0..pages {
            let page = PHYSICAL_MEMORY_MANAGER
                .lock()
                .allocate_page()
                .expect("Can't allocated page");

            let virtual_address =
                VirtualAddress::new(self.current_address.as_addr() + i as u64 * PAGE_SIZE);
            paging::map(self.pagemap, virtual_address, page, self.flags);
        }

        let object = {
            let object = self.current_address.as_addr() as *mut VirtualMemoryObject;

            object.write(VirtualMemoryObject {
                next: None,
                is_used: true,
                total_pages: pages,
                base: VirtualAddress::new(self.current_address.as_addr() + mem::size_of::<VirtualMemoryObject>() as u64),
                flags: self.flags
            });

            Some(NonNull::new_unchecked(object))
        };

        if self.root_object.is_none() {
            self.root_object = object;
            self.current_object = object;
        } else {
            self.current_object.unwrap().as_mut().next = object;
            self.current_object = object;
        }

        self.current_address =
            VirtualAddress::new(self.current_address.as_addr() + pages as u64 * PAGE_SIZE);
        object
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

