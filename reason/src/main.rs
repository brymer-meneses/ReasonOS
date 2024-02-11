#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(non_null_convenience)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod arch;
mod boot;
mod data_structures;
mod drivers;
mod memory;
mod misc;

use alloc::boxed::Box;
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

    let mut memory = Box::new(50);
    *memory = 5;

    log::info!("{}", memory);

    cpu::halt();
}

use crate::misc::qemu::{self, QemuExitCode};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::println!("{}", info.red());

    qemu::exit(QemuExitCode::Failed);
    cpu::hcf();
}

#[cfg(test)]
use crate::misc::Testable;

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial::println!("========= Running {} tests ======", tests.len());

    for test in tests {
        test.run();
    }

    qemu::exit(QemuExitCode::Success);
}
