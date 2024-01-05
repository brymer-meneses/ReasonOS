pub mod cpu;
pub mod interrupt;

mod idt;
mod gdt;

pub fn initialize() {
    gdt::initialize();
    idt::initialize();
}
