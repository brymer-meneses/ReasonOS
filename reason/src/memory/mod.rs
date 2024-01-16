
pub mod pmm;
pub mod vmm;

use crate::misc::log;
use crate::boot::MEMORY_MAP_REQUEST;
use crate::arch::paging::{self, PAGE_SIZE};
use crate::misc::align_down;

use core::ptr::NonNull;
use lazy_static::lazy_static;

use pmm::BitmapAllocator;
use vmm::{VirtualMemoryManager, VirtualMemoryFlags};

use spin::Mutex;

lazy_static! {
    pub static ref PHYSICAL_MEMORY_MANAGER: Mutex<BitmapAllocator<'static>> = Mutex::new(BitmapAllocator::new());
    pub static ref VIRTUAL_MEMORY_MANAGER: Mutex<VirtualMemoryManager> = Mutex::new(VirtualMemoryManager::NULL);
}

extern "C" {
    static __kernel_end_address: u8;
    static __kernel_start_address: u8;
}

pub unsafe fn initialize() {

    let memmap = MEMORY_MAP_REQUEST
        .get_response()
        .get()
        .expect("Failed to get memory map.");

    PHYSICAL_MEMORY_MANAGER.lock().initialize(memmap);

    log::info!("Initialized PMM");

    let pagemap = NonNull::new_unchecked(paging::get_initial_pagemap());
    let kernel_heap_start = align_down(&__kernel_end_address as *const _ as u64, PAGE_SIZE);

    VIRTUAL_MEMORY_MANAGER.lock().initialize(pagemap, kernel_heap_start, VirtualMemoryFlags::Writeable | VirtualMemoryFlags::Executable );

    log::info!("Initialized VMM");
}
