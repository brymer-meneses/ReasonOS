pub const PAGE_SIZE: u64 = 4096;
use crate::arch::cpu::{self, Context};
use crate::arch::interrupt;

fn page_fault_handler(_ctx: *const Context) {
    panic!("Page Fault accessed memory: 0x{:016X}", cpu::read_cr3());
}

fn general_page_fault_handler(_ctx: *const Context) {
    panic!("General Page Fault");
}

pub fn initialize() {
    interrupt::set_interrupt_handler(0xE, page_fault_handler);
    interrupt::set_interrupt_handler(0xD, general_page_fault_handler);
}
