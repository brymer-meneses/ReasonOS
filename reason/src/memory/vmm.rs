use core::mem::size_of;
use core::ptr::NonNull;

use super::{VirtualAddress, VirtualMemoryFlags};
use crate::arch;
use crate::arch::paging::PAGE_SIZE;
use crate::data_structures::{SinglyLinkedList, SinglyLinkedListNode};
use crate::memory::PHYSICAL_MEMORY_MANAGER;
use crate::misc::log;
use crate::misc::utils::size;

#[derive(Clone, Copy)]
pub struct VirtualMemoryObject {
    pub base: VirtualAddress,
    pub flags: VirtualMemoryFlags,
    pub length: u64,
    pub is_used: bool,
}

pub struct VirtualMemoryManager {
    pagemap: VirtualAddress,
    flags: VirtualMemoryFlags,
    base_address: VirtualAddress,
    current_address: VirtualAddress,
    pub objects: SinglyLinkedList<VirtualMemoryObject>,
}

impl VirtualMemoryManager {
    pub fn new(
        pagemap: VirtualAddress,
        base_address: VirtualAddress,
        flags: VirtualMemoryFlags,
    ) -> Self {
        assert!(base_address.is_page_aligned());
        assert!(pagemap.is_page_aligned());

        Self {
            pagemap,
            base_address,
            current_address: base_address,
            objects: SinglyLinkedList::new(),
            flags,
        }
    }

    pub unsafe fn allocate_object(&mut self, size: u64) -> Option<&mut VirtualMemoryObject> {
        // TODO: check for free object first

        const NODE_SIZE: u64 = size!(SinglyLinkedListNode<VirtualMemoryObject>);
        let pages = (size + NODE_SIZE).div_ceil(PAGE_SIZE);

        for i in 0..pages {
            let current_address = self.current_address + i * PAGE_SIZE;
            let page = PHYSICAL_MEMORY_MANAGER
                .lock()
                .allocate_page()
                .expect("Failed to allocate page");
            arch::paging::map(self.pagemap, current_address, page, self.flags)
        }

        // we add `NODE_SIZE` to `self.current_address` since this is where we store the
        // singly-list node
        let vm_object_base = self.current_address + NODE_SIZE;

        self.objects.append(
            VirtualMemoryObject::new(vm_object_base, pages * PAGE_SIZE, self.flags),
            self.current_address,
        );

        self.current_address += pages * PAGE_SIZE;
        self.objects.tail_mut()
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

impl VirtualMemoryObject {
    pub fn new(base: VirtualAddress, length: u64, flags: VirtualMemoryFlags) -> Self {
        Self {
            is_used: true,
            length,
            flags,
            base,
        }
    }
}
