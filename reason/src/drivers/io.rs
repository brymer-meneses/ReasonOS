
use core::arch::asm;

#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
     );
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let data: u8;
    asm!(
        "in dx, al",
        in("dx") port,
        out("al") data,
     );
    return data;
}
