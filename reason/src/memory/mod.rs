#![allow(unused)]

use bitflags::bitflags;

mod address;
mod heap;
mod pmm;
mod vmm;

use crate::arch::paging::{self, PAGE_SIZE};
use crate::boot::MEMORY_MAP_REQUEST;
use crate::misc::log;
use crate::misc::utils::{align_down, OnceCellMutex};

use pmm::BitmapAllocator;
use vmm::VirtualMemoryManager;

pub use address::{PhysicalAddress, VirtualAddress};

pub static mut PHYSICAL_MEMORY_MANAGER: OnceCellMutex<BitmapAllocator> = OnceCellMutex::new();
pub static mut VIRTUAL_MEMORY_MANAGER: OnceCellMutex<VirtualMemoryManager> = OnceCellMutex::new();

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VirtualMemoryFlags: u8 {
        const Writeable = 1 << 0;
        const Executable = 1 << 1;
        const UserAccessible = 1 << 2;
    }
}

pub fn initialize() {
    // Physical Memory Manager
    unsafe {
        let memmap = MEMORY_MAP_REQUEST
            .get_response()
            .get()
            .expect("Failed to get memory map.");
        PHYSICAL_MEMORY_MANAGER.set(BitmapAllocator::new(memmap));
        log::info!("Initialized PMM");

        let pagemap = paging::get_initial_pagemap();

        let kernel_heap_start = VirtualAddress::new(align_down(
            &__kernel_end_address as *const _ as u64,
            PAGE_SIZE,
        ));

        VIRTUAL_MEMORY_MANAGER.set(VirtualMemoryManager::new(
            pagemap,
            kernel_heap_start,
            VirtualMemoryFlags::Writeable | VirtualMemoryFlags::Executable,
        ));

        log::info!("Initialized VMM");
    }
}

extern "C" {
    static __kernel_end_address: u8;
    static __kernel_start_address: u8;
}
