use core::arch::asm;
use core::ptr::{null, NonNull};

use crate::boot::KERNEL_FILE_REQUEST;
use crate::drivers::serial;
use crate::memory::{IntoAddress, VirtualAddress};
use crate::misc::colored::Colorize;
use crate::misc::elf::{Elf64, SymbolType};
use crate::misc::log;

struct StackFrame {
    rbp: *const StackFrame,
    rip: u64,
}

pub fn run() {
    unsafe {
        let mut frame: *const StackFrame;

        asm!("mov {}, rbp", out(reg) frame);
        let mut i = 0;

        serial::println!("=========== stack trace ===========");
        while !(*frame).rbp.is_null() {
            let rip = (*frame).rip.as_virtual();
            let name = resolve_symbol_name(rip);

            serial::print!("{}{}{}\t", "[".blue(), i.blue(), "]".blue(),);

            if let Some(name) = name {
                serial::println!("{:#}", rustc_demangle::demangle(name));
            } else {
                serial::println!("unknown@{}", rip.cyan());
            }

            frame = (*frame).rbp;
            i += 1;
        }
    }
}

fn resolve_symbol_name<'a>(ip: VirtualAddress) -> Option<&'a str> {
    let kernel_file_address = {
        let response = KERNEL_FILE_REQUEST
            .get_response()
            .get()
            .expect("Failed to get Kernel Address");

        let file = response.kernel_file.get().unwrap();
        VirtualAddress::new(file.base.get().unwrap() as *const u8 as u64)
    };

    let kernel_elf = Elf64::new(kernel_file_address);
    let string_table = kernel_elf.symbol_table().link;
    let ip = ip.as_addr();

    kernel_elf
        .symbols()
        .filter(|symbol| symbol.symbol_type == SymbolType::Func && symbol.size != 0)
        .find(|symbol| ip >= symbol.value && ip < symbol.value + symbol.size)
        .map(|symbol| kernel_elf.get_name(symbol.name_offset, string_table))
}
