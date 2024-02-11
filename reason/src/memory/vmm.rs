#![allow(dead_code)]
use core::ptr::NonNull;

use super::{VirtualAddress, VirtualMemoryFlags};
use crate::arch;
use crate::arch::paging::PAGE_SIZE;
use crate::data_structures::SinglyLinkedList;
use crate::memory::PHYSICAL_MEMORY_MANAGER;
use crate::misc::log;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtualMemoryObject {
    /// start of the `free` virtual memory address that we can put data from this is
    /// guaranteed to **NOT** be where the instance of `VirtualMemoryRegion` is stored
    pub base: VirtualAddress,

    /// flags of the virtual memory
    pub flags: VirtualMemoryFlags,

    /// total length of the virtual memory object this includes the part where this instance
    /// of `VirtualMemoryObject` is stored
    pub length: u64,

    /// is the virtual memory object used
    pub is_used: bool,
}

/// Manages memory with greater than page size length
pub struct VirtualMemoryManager {
    pagemap: VirtualAddress,
    flags: VirtualMemoryFlags,
    base_address: VirtualAddress,
    current_address: VirtualAddress,

    pub allocated_objects: SinglyLinkedList<VirtualMemoryObject>,
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
            allocated_objects: SinglyLinkedList::new(),
            flags,
        }
    }

    pub unsafe fn allocate_object(&mut self, size: u64) -> NonNull<VirtualMemoryObject> {
        // TODO: check for free object first

        let node_size = self.allocated_objects.list_node_size();
        let pages = (size + node_size).div_ceil(PAGE_SIZE);

        for i in 0..pages {
            let current_address = self.current_address + i * PAGE_SIZE;
            let page = PHYSICAL_MEMORY_MANAGER
                .lock()
                .allocate_page()
                .expect("Failed to allocate page");
            arch::paging::map(self.pagemap, current_address, page, self.flags)
        }

        let vm_object_base = self.current_address + node_size;

        log::info!("vm_object_base {vm_object_base}");

        self.allocated_objects.append_to_address(
            self.current_address,
            VirtualMemoryObject::new(vm_object_base, pages * PAGE_SIZE, self.flags),
        );

        self.current_address += pages * PAGE_SIZE;
        self.allocated_objects.tail().unwrap_unchecked()
    }

    pub unsafe fn free_object(&mut self, address: VirtualAddress) {
        self.allocated_objects
            .remove(|node| node.ptr_to_data().as_ref().base == address);
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
