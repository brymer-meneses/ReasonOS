#![feature(non_null_convenience)]
#![no_std]
#![no_main]

mod arch;
mod boot;
mod drivers;
mod memory;
mod misc;

use drivers::framebuffer;
use drivers::serial;

use arch::{cpu, paging};
use memory::pmm;
use memory::vmm::VirtualMemoryFlags;

use misc::log;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::print!("\x1b[31m{}\x1B[0m\n", info);
    cpu::hcf();
}

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    serial::initialize();
    boot::initialize();

    framebuffer::initialize();

    arch::initialize();
    memory::initialize();

    let phys = pmm::allocate_page().unwrap().as_ptr() as u64;
    let root = paging::get_initial_pagemap();

    let virt = 0xdeadb000;

    paging::map(
        root as *mut u64,
        virt,
        phys,
        VirtualMemoryFlags::Writeable,
    );

    // paging::unmap(
    //     root as *mut u64,
    //     virt,
    //     phys,
    // );

    unsafe {
        let addr = virt as *mut u64;
        *addr = 25;
        assert_eq!(*addr, 25);
    }

    log::info!("Successfully initialized kernel");

    cpu::halt();
}
