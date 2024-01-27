use core::mem::size_of;
use core::ptr::NonNull;

use super::{VirtualAddress, VirtualMemoryFlags};
use crate::arch;
use crate::arch::paging::PAGE_SIZE;
use crate::data_structures::{SinglyLinkedList, SinglyLinkedListNode};
use crate::memory::PHYSICAL_MEMORY_MANAGER;

#[derive(PartialEq, Eq)]
pub struct VirtualMemoryObject {
    base: VirtualAddress,
    flags: VirtualMemoryFlags,
    length: u64,
    is_used: bool,
}

pub struct VirtualMemoryManager {
    pagemap: VirtualAddress,
    flags: VirtualMemoryFlags,
    base_address: VirtualAddress,
    current_address: VirtualAddress,
    objects: SinglyLinkedList<VirtualMemoryObject>,
}

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
            objects: SinglyLinkedList::new(),
            flags,
        }
    }

    pub unsafe fn allocate_object(&mut self, pages: usize) -> Option<&VirtualMemoryObject> {
        for i in 0..pages {
            let page = PHYSICAL_MEMORY_MANAGER
                .lock()
                .allocate_page()
                .expect("Page Allocation Failed");

            let virtual_address = VirtualAddress::new(
                self.current_address.as_addr() + size_of::<VirtualMemoryObject>() as u64,
            );

            arch::paging::map(
                self.pagemap,
                virtual_address,
                page,
                VirtualMemoryFlags::Writeable | VirtualMemoryFlags::Executable,
            )
        }

        let object = {
            let ptr =
                self.current_address.as_addr() as *mut SinglyLinkedListNode<VirtualMemoryObject>;

            self.objects.append(
                VirtualMemoryObject {
                    is_used: true,
                    length: pages as u64 * PAGE_SIZE,
                    flags: self.flags,
                    base: VirtualAddress::new(
                        self.current_address.as_addr() + size_of::<VirtualMemoryObject>() as u64,
                    ),
                },
                ptr,
            );

            self.objects.tail()
        };

        object
    }

    pub unsafe fn free_object(&mut self, object: &VirtualMemoryObject) {
        self.objects.remove(
            |o| object.base == o.base,
            |node| {
                (*node).data.is_used = false;
            },
        );

        todo!()
    }
}

impl VirtualMemoryObject {}
