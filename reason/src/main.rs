#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

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

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    serial::initialize();
    boot::initialize();
    framebuffer::initialize();

    arch::initialize();
    memory::initialize();

    #[cfg(test)]
    test_main();

    cpu::halt();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::print!("\x1b[31m{}\x1B[0m\n", info);
    cpu::hcf();
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial::println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
