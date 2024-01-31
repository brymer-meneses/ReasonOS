#![allow(unused)]

use crate::{arch::paging::PAGE_SIZE, misc::colored::Colorize};
use core::{fmt, ops::Add, ops::AddAssign};

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

pub trait IntoAddress {
    fn as_virtual(&self) -> VirtualAddress;
    fn as_physical(&self) -> PhysicalAddress;
}

impl<T> IntoAddress for T
where
    T: Into<u64> + Copy,
{
    fn as_virtual(&self) -> VirtualAddress {
        let value = (*self).into();
        VirtualAddress::new(value)
    }
    fn as_physical(&self) -> PhysicalAddress {
        let value = (*self).into();
        PhysicalAddress::new(value)
    }
}

use crate::misc::colored::Color;

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

impl AddAssign<u64> for VirtualAddress {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl AddAssign<u64> for PhysicalAddress {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}
impl Add<u64> for VirtualAddress {
    type Output = VirtualAddress;
    fn add(self, rhs: u64) -> Self::Output {
        VirtualAddress::new(self.0 + rhs)
    }
}

impl Add<u64> for PhysicalAddress {
    type Output = PhysicalAddress;
    fn add(self, rhs: u64) -> Self::Output {
        PhysicalAddress::new(self.0 + rhs)
    }
}
