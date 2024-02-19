#![allow(dead_code)]

use bitflags::bitflags;
use core::alloc::GlobalAlloc;
use core::ptr::{addr_of, addr_of_mut, NonNull};

mod address;
mod heap;
mod pmm;
mod vmm;

use crate::arch::paging::{self, PAGE_SIZE};
use crate::boot::MEMORY_MAP_REQUEST;
use crate::misc::log;
use crate::misc::utils::{align_up, OnceLock};

use pmm::BitmapAllocator;
use vmm::VirtualMemoryManager;

pub use address::IntoAddress;
pub use address::{PhysicalAddress, VirtualAddress};

use self::heap::ExplicitFreeList;

pub static mut PHYSICAL_MEMORY_MANAGER: OnceLock<BitmapAllocator> = OnceLock::new();
pub static mut VIRTUAL_MEMORY_MANAGER: OnceLock<VirtualMemoryManager> = OnceLock::new();

#[global_allocator]
pub static mut KERNEL_HEAP_ALLOCATOR: OnceLock<ExplicitFreeList> = OnceLock::new();

bitflags! {
    #[derive(Clone, Copy, Debug)]
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

        let kernel_heap_start =
            VirtualAddress::new(align_up(addr_of!(__kernel_end_address) as u64, PAGE_SIZE));

        log::debug!("Kernel Heap Start {}", kernel_heap_start);

        VIRTUAL_MEMORY_MANAGER.set(VirtualMemoryManager::new(
            pagemap,
            kernel_heap_start,
            VirtualMemoryFlags::Writeable | VirtualMemoryFlags::Executable,
        ));

        log::info!("Initialized VMM");

        KERNEL_HEAP_ALLOCATOR.set(ExplicitFreeList::new(NonNull::new_unchecked(addr_of_mut!(
            VIRTUAL_MEMORY_MANAGER
        ))));

        log::info!("Initialized Kernel Heap");
    }
}

use core::alloc::Layout;

unsafe impl GlobalAlloc for OnceLock<ExplicitFreeList> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        allocator
            .alloc_aligned(layout.size() as u64, layout.align() as u64)
            .as_addr() as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let mut allocator = self.lock();
        let address = VirtualAddress::new(ptr as u64);
        allocator.free(address)
    }
}

extern "C" {
    static __kernel_end_address: u8;
    static __kernel_start_address: u8;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn test_linked_list() {
        unsafe {
            let mut heap_allocator = KERNEL_HEAP_ALLOCATOR.lock();

            for _ in 0..10 {
                let addr = heap_allocator.alloc(8);
                heap_allocator.free(addr);
            }

            let address1 = heap_allocator.alloc(16);
            let address2 = heap_allocator.alloc(16);
            let address3 = heap_allocator.alloc(16);

            heap_allocator.free(address1);
            heap_allocator.free(address3);
            heap_allocator.free(address2);
        }
    }

    #[test_case]
    fn test_another_linked_list() {
        unsafe {
            let mut heap_allocator = KERNEL_HEAP_ALLOCATOR.lock();

            for _ in 0..10 {
                let addr = heap_allocator.alloc(8);
                heap_allocator.free(addr);
            }

            let address1 = heap_allocator.alloc(16);
            let address2 = heap_allocator.alloc(16);
            let address3 = heap_allocator.alloc(16);

            heap_allocator.free(address1);
            heap_allocator.free(address3);
            heap_allocator.free(address2);
        }
    }
}
