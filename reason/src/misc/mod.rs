#![allow(unused, unused_macros)]

pub mod colored;
pub mod log;
pub mod qemu;
pub mod utils;

use crate::drivers::serial;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        use crate::misc::colored::Colorize;
        serial::println!("Running {}", core::any::type_name::<T>().yellow());
        self();
    }
}
