#![no_std]
#![no_main]

mod drivers;
mod arch;

use drivers::serial;
use drivers::framebuffer;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    serial::initialize();
    framebuffer::initialize();

    serial::write_string("Hello kernel!");
    arch::halt();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    arch::hcf();
}

