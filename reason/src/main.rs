#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod acpi;
mod arch;
mod boot;
mod data_structures;
mod drivers;
mod memory;
mod misc;

mod panic;

#[cfg(test)]
mod tests;

use drivers::framebuffer;
use drivers::serial;

use arch::cpu;
use misc::log;

#[no_mangle]
extern "C" fn _start() -> ! {
    serial::initialize();

    boot::initialize();
    framebuffer::initialize();

    arch::initialize();
    memory::initialize();

    acpi::initialize();

    #[cfg(test)]
    test_main();

    cpu::halt();
}
