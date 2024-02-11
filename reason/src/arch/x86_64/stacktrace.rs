use core::arch::asm;
use core::ptr::{null, NonNull};

use crate::misc::log;

struct StackFrame {
    rbp: *const StackFrame,
    rip: u64,
}

pub fn run() {
    unsafe {
        let mut frame: *const StackFrame;

        asm!("mov {}, rbp", out(reg) frame);

        while !(*frame).rbp.is_null() {
            log::info!("0x{:016X}", (*frame).rip);
            frame = (*frame).rbp;
        }
    }
}
