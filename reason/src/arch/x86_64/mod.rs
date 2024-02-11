#![allow(dead_code)]

pub mod cpu;
pub mod interrupt;
pub mod paging;
pub mod stacktrace;

mod gdt;
mod idt;

pub fn initialize() {
    gdt::initialize();
    idt::initialize();

    paging::initialize();
}
