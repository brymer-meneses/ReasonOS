use crate::arch::cpu;
use crate::drivers::serial;
use crate::misc::colored::Colorize;
use crate::misc::qemu::{self, QemuExitCode};

use crate::arch::stacktrace;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::println!("{}", info.red());

    stacktrace::run();

    qemu::exit(QemuExitCode::Failed);
    cpu::hcf();
}
