#![no_std]
#![no_main]

mod arch;
mod drivers;
mod misc;

use drivers::framebuffer;
use drivers::serial;

use arch::cpu;
use misc::log;

use core::arch::asm;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::print!("\x1b[31m{}\x1B[0m\n", info);
    cpu::hcf();
}

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    serial::initialize();
    framebuffer::initialize();

    arch::initialize();

    log::info!("Successfully initialized kernel!");

    asm!("int 0");

    cpu::halt();
}
