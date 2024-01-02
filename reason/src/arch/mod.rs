use core::arch::asm;

pub fn halt() -> ! {
    unsafe {
        loop {
            asm!("hlt");
        }
    }
}

pub fn hcf() -> ! {
    unsafe { asm!("cli"); }
    halt();
}
