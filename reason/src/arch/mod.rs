pub mod io;

#[cfg(target_arch = "x86_64")]
mod x86_64;

pub fn halt() -> ! {
    #[cfg(target_arch = "x86_64")]
    x86_64::halt();
}

pub fn hcf() -> ! {
    #[cfg(target_arch = "x86_64")]
    x86_64::halt();
}
