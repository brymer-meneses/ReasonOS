#![allow(unused)]

use bitflags::bitflags;
use core::ptr::addr_of;

mod address;
mod heap;
mod pmm;
mod vmm;

use crate::arch::paging::{self, PAGE_SIZE};
use crate::boot::MEMORY_MAP_REQUEST;
use crate::misc::log;
use crate::misc::utils::{align_down, align_up, OnceCellMutex};

use pmm::BitmapAllocator;
use vmm::VirtualMemoryManager;

pub use address::IntoAddress;
pub use address::{PhysicalAddress, VirtualAddress};

use self::heap::ExplicitFreeList;

pub static mut PHYSICAL_MEMORY_MANAGER: OnceCellMutex<BitmapAllocator> = OnceCellMutex::new();
pub static mut VIRTUAL_MEMORY_MANAGER: OnceCellMutex<VirtualMemoryManager> = OnceCellMutex::new();
pub static mut KERNEL_HEAP_ALLOCATOR: OnceCellMutex<ExplicitFreeList> = OnceCellMutex::new();

bitflags! {
    #[derive(Clone, Copy)]
    pub struct VirtualMemoryFlags: u8 {
        const Writeable = 1 << 0;
        const Executable = 1 << 1;
        const UserAccessible = 1 << 2;
    }
}

pub fn initialize() {
    unsafe {
        let memmap = MEMORY_MAP_REQUEST
            .get_response()
            .get()
            .expect("Failed to get memory map.");

        PHYSICAL_MEMORY_MANAGER.set(BitmapAllocator::new(memmap));
        log::info!("Initialized PMM");

        let pagemap = paging::get_initial_pagemap();

        let kernel_heap_start = VirtualAddress::new(align_up(
            &__kernel_end_address as *const _ as u64,
            PAGE_SIZE,
        ));

        log::debug!("Kernel Heap Start {}", kernel_heap_start);

        VIRTUAL_MEMORY_MANAGER.set(VirtualMemoryManager::new(
            pagemap,
            kernel_heap_start,
            VirtualMemoryFlags::Writeable | VirtualMemoryFlags::Executable,
        ));

        log::info!("Initialized VMM");

        KERNEL_HEAP_ALLOCATOR.set(ExplicitFreeList::new(
            addr_of!(VIRTUAL_MEMORY_MANAGER) as *mut OnceCellMutex<VirtualMemoryManager>
        ));

        log::info!("Initialized Kernel Heap");
    }
}

extern "C" {
    static __kernel_end_address: u8;
    static __kernel_start_address: u8;
}

#[cfg(test)]
mod tests {

    use crate::{data_structures::DoublyLinkedList, misc::utils::size};

    use super::*;

    #[test_case]
    fn test_linked_list() {
        unsafe {
            let mut heap_allocator = KERNEL_HEAP_ALLOCATOR.lock();
            let mut linked_list = DoublyLinkedList::<i64>::new();

            for i in 0..1000 {
                let address = heap_allocator.alloc(size!(i64));
                linked_list.append(address, i);
            }
        }
    }
}
