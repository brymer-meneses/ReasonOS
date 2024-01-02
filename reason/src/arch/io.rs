
#[cfg(target_arch="x86_64")]
use crate::arch::x86_64;

#[inline(always)]
pub fn outb(port: u16, value: u8) {
    unsafe { 
        #[cfg(target_arch="x86_64")]
         x86_64::outb(port, value);
    };

}

#[inline(always)]
pub fn inb(port: u16) -> u8 {
    unsafe { 
        #[cfg(target_arch="x86_64")]
         return x86_64::inb(port);
    };
}
