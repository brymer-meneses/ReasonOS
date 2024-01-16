#![allow(unused)]

use core::ptr::NonNull;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::boot::MEMORY_MAP_REQUEST;
use crate::memory::bitmap_allocator::BitmapAllocator;
use crate::misc::log;

lazy_static! {
    static ref ALLOCATOR: Mutex<BitmapAllocator<'static>> = Mutex::new(BitmapAllocator::new());
}

pub fn initialize() {
    let response = MEMORY_MAP_REQUEST
        .get_response()
        .get()
        .expect("Failed to get memory map.");

    let mut allocator = ALLOCATOR.lock();
    allocator.initialize(response);

    log::info!("Initialized PMM");
}

pub fn allocate_page() -> Option<NonNull<u64>> {
    let mut allocator = ALLOCATOR.lock();
    unsafe { allocator.allocate_page() }
}

pub fn free_page(address: NonNull<u64>) {
    let mut allocator = ALLOCATOR.lock();
    unsafe {
        allocator.free_page(address);
    }
}
