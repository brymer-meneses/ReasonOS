use crate::arch::cpu;
use crate::boot::KERNEL_FILE_REQUEST;
use crate::drivers::serial;
use crate::memory::{IntoAddress, VirtualAddress};
use crate::misc::colored::Colorize;
use crate::misc::elf::{Elf64, SymbolType};
use crate::misc::qemu::{self, QemuExitCode};

use core::arch::asm;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::println!("{}", info.red());

    unsafe {
        let mut frame = core::ptr::null();

        StackFrame::from(&mut frame);

        serial::println!("=========== stack trace ===========");

        for (i, rip) in (*frame).enumerate() {
            let name = resolve_symbol_name(rip.as_virtual());

            serial::print!("{}{}{}\t", "[".blue(), i.blue(), "]".blue());

            if let Some(name) = name {
                serial::println!("{:#}", rustc_demangle::demangle(name));
            } else {
                serial::println!("unknown@{:016X}", rip.cyan());
            }
        }
    }

    qemu::exit(QemuExitCode::Failed);
    cpu::hcf();
}

#[derive(Clone, Copy)]
struct StackFrame {
    rbp: *const StackFrame,
    rip: u64,
}

impl StackFrame {
    pub fn from(to: &mut *const StackFrame) {
        unsafe { asm!("mov {}, rbp", out(reg)(*to)) };
    }
}

impl Iterator for StackFrame {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.rbp.is_null() {
            return None;
        }

        self.rbp = unsafe { (*self.rbp).rbp };
        self.rip = unsafe { (*self.rbp).rip };

        if self.rip == 0 {
            return None;
        }

        Some(self.rip)
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
    let string_table = kernel_elf.symbol_table().sh_link;

    kernel_elf
        .symbols()
        .filter(|symbol| symbol.r#type() == SymbolType::Func && symbol.st_size != 0)
        .find(|symbol| symbol.within(ip))
        .map(|symbol| kernel_elf.get_name(symbol.st_name, string_table))
}
