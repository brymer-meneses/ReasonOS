use core::arch::asm;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Context {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub vector: u64,
    pub error: u64,

    pub iret_rip: u64,
    pub iret_cs: u64,
    pub iret_flags: u64,
    pub iret_rsp: u64,
    pub iret_ss: u64,
}

pub fn halt() -> ! {
    unsafe {
        loop {
            asm!("hlt");
        }
    }
}

pub fn hcf() -> ! {
    unsafe {
        asm!("cli");
    }
    halt();
}

#[inline(always)]
pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
           "out dx, al",
           in("dx") port,
           in("al") value,
        );
    }
}

#[inline(always)]
pub fn read_cr3() -> u64 {
    let value;
    unsafe {
        asm!(
            "mov {}, cr3",
            out(reg) value
        );
    }
    value
}

#[inline(always)]
pub fn read_cr2() -> u64 {
    let value;
    unsafe {
        asm!(
            "mov {}, cr2",
            out(reg) value
        );
    }
    value
}

#[inline(always)]
pub fn inb(port: u16) -> u8 {
    let data: u8;
    unsafe {
        asm!(
           "in dx, al",
           in("dx") port,
           out("al") data,
        );
    }
    data
}
