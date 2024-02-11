use crate::drivers::serial;
use crate::misc::colored::Colorize;
use crate::misc::qemu::{self, QemuExitCode};

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial::println!("Running {}", core::any::type_name::<T>().yellow());
        self();
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial::println!("========= Running {} tests ======", tests.len());

    for test in tests {
        test.run();
    }

    qemu::exit(QemuExitCode::Success);
}
