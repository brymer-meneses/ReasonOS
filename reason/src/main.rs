#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(non_null_convenience)]

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

use misc::colored::Colorize;

#[no_mangle]
extern "C" fn _start() -> ! {
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
    serial::print!("{}", info.red());
    cpu::hcf();
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial::println!("===== Running {} tests ======", tests.len());
    for test in tests {
        test();
    }
}
