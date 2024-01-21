pub mod pmm;
pub mod vmm;

use crate::arch::paging::{self, PAGE_SIZE};
use crate::boot::MEMORY_MAP_REQUEST;
use crate::misc::align_down;
use crate::misc::log;

use core::ptr::NonNull;

use pmm::BitmapAllocator;
use vmm::{VirtualMemoryFlags, VirtualMemoryManager};

use crate::misc::OnceCellMutex;

pub static mut PHYSICAL_MEMORY_MANAGER: OnceCellMutex<BitmapAllocator> = OnceCellMutex::new();
pub static mut VIRTUAL_MEMORY_MANAGER: OnceCellMutex<VirtualMemoryManager> = OnceCellMutex::new();

extern "C" {
    static __kernel_end_address: u8;
    static __kernel_start_address: u8;
}

pub unsafe fn initialize() {
    let memmap = MEMORY_MAP_REQUEST
        .get_response()
        .get()
        .expect("Failed to get memory map.");

    PHYSICAL_MEMORY_MANAGER.set(BitmapAllocator::new(memmap));

    log::info!("Initialized PMM");

    let pagemap = NonNull::new_unchecked(paging::get_initial_pagemap());
    let kernel_heap_start = align_down(&__kernel_end_address as *const _ as u64, PAGE_SIZE);

    VIRTUAL_MEMORY_MANAGER.set(
        VirtualMemoryManager::new(
            pagemap,
            kernel_heap_start,
            VirtualMemoryFlags::Writeable | VirtualMemoryFlags::Executable,
        )
    );

    log::info!("Initialized VMM");
}
