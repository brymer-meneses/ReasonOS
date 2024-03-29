#![allow(unused_imports)]

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

pub fn initialize() {
    #[cfg(target_arch = "x86_64")]
    x86_64::initialize();
}
