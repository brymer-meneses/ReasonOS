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
pub struct VirtualMemoryRegion {
    /// start of the `free` virtual memory address that we can put data from this is
    /// guaranteed to **NOT** be where the instance of `VirtualMemoryRegion` is stored
    pub base: VirtualAddress,

    /// flags of the virtual memory
    pub flags: VirtualMemoryFlags,

    /// total length of the virtual memory region this includes the part where this instance
    /// of `VirtualMemoryRegion` is stored
    pub length: u64,

    /// is the virtual memory region used
    pub is_used: bool,
}

/// Manages memory with greater than page size length
pub struct VirtualMemoryManager {
    pagemap: VirtualAddress,
    flags: VirtualMemoryFlags,
    base_address: VirtualAddress,
    current_address: VirtualAddress,
    pub regions: SinglyLinkedList<VirtualMemoryRegion>,
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
            regions: SinglyLinkedList::new(),
            flags,
        }
    }

    pub unsafe fn allocate_region(&mut self, size: u64) -> Option<*mut VirtualMemoryRegion> {
        // TODO: check for free region first

        const NODE_SIZE: u64 = size!(SinglyLinkedListNode<VirtualMemoryRegion>);
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
        let vm_region_base = self.current_address + NODE_SIZE;

        self.regions.append(
            self.current_address,
            VirtualMemoryRegion::new(vm_region_base, pages * PAGE_SIZE, self.flags),
        );

        // log::debug!("[vmm] Allocate vm region with size {}", pages * PAGE_SIZE);

        self.current_address += pages * PAGE_SIZE;
        self.regions.tail_mut()
    }

    pub unsafe fn free_region(&mut self, region: &VirtualMemoryRegion) {
        self.regions.remove(
            |other| region.base == other.base,
            |node| {
                (*node).data.is_used = false;
            },
        );
        todo!()
    }
}

impl VirtualMemoryRegion {
    pub fn new(base: VirtualAddress, length: u64, flags: VirtualMemoryFlags) -> Self {
        Self {
            is_used: true,
            length,
            flags,
            base,
        }
    }
}
