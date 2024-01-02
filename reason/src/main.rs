#![no_std]
#![no_main]

mod arch;
mod drivers;
mod misc;

use drivers::framebuffer;
use drivers::serial;
use misc::log;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    serial::initialize();
    framebuffer::initialize();

    log::info!("hello {}", "kernel!");

    arch::halt();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    arch::hcf();
}
