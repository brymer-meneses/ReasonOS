pub mod colored;
pub mod log;

use core::cell::OnceCell;
use spin::{Mutex, MutexGuard};

#[inline]
pub const fn align_up(addr: u64, align: u64) -> u64 {
    ((addr + align - 1) / align) * align
}

#[inline]
pub const fn align_down(addr: u64, align: u64) -> u64 {
    (addr / align) * align
}

#[derive(Debug)]
pub struct OnceCellMutex<T>(OnceCell<Mutex<T>>);

impl<T> OnceCellMutex<T> {
    pub const fn new() -> Self {
        Self(OnceCell::new())
    }

    pub fn set(&mut self, param: T) {
        self.0.get_or_init(|| Mutex::new(param));
    }

    pub unsafe fn lock(&self) -> MutexGuard<T> {
        self.0.get().unwrap().lock()
    }
}

unsafe impl<T> Send for OnceCellMutex<T> {}
unsafe impl<T> Sync for OnceCellMutex<T> {}
