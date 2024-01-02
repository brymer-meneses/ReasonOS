#![no_std]
#![no_main]

mod arch;
mod drivers;
mod misc;

use drivers::framebuffer;
use drivers::serial;

use misc::log;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::print!("\x1b[31m{}\x1B[0m\n", info);
    arch::hcf();
}

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    serial::initialize();
    framebuffer::initialize();

    log::info!("hello {}", "kernel!");

    assert!(1 + 1 == 3);

    arch::halt();
}
