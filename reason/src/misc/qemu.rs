use core::arch::asm;

pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub const QEMU_PORT: u16 = 0xf4;

pub fn exit(exit_code: QemuExitCode) {
    unsafe {
        asm!("out dx, al", in("dx") QEMU_PORT, in("al") exit_code as u8, options(nomem, nostack, preserves_flags));
    }
}
