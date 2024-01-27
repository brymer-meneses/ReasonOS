#![no_std]
#![no_main]

mod arch;
mod boot;
mod data_structures;
mod drivers;
mod memory;
mod misc;

use drivers::framebuffer;
use drivers::serial;

use arch::cpu;
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

    log::info!("Successfully initialized kernel");

    cpu::halt();
}
