#![allow(unused)]

pub mod cpu;
pub mod interrupt;
pub mod paging;

mod gdt;
mod idt;

pub fn initialize() {
    gdt::initialize();
    idt::initialize();

    paging::initialize();
}
