#![allow(unused)]

use crate::arch::paging::PAGE_SIZE;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalAddress(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualAddress(u64);

impl PhysicalAddress {
    pub const fn as_ptr(&self) -> *mut u64 {
        self.0 as *mut u64
    }

    pub const fn as_addr(&self) -> u64 {
        self.0
    }

    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn is_aligned_to(&self, size: u64) -> bool {
        self.0 % size == 0
    }

    pub const fn is_page_aligned(&self) -> bool {
        self.0 % PAGE_SIZE == 0
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl VirtualAddress {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn is_aligned_to(&self, size: u64) -> bool {
        self.0 % size == 0
    }

    pub const fn as_addr(&self) -> u64 {
        self.0
    }

    pub const fn as_ptr(&self) -> *mut u64 {
        self.0 as *mut u64
    }

    pub const fn is_page_aligned(&self) -> bool {
        self.0 % PAGE_SIZE == 0
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl PartialEq<u64> for PhysicalAddress {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<u64> for VirtualAddress {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

