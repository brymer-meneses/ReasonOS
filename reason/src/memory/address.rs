#![allow(dead_code)]

use crate::arch::paging::PAGE_SIZE;
use core::{fmt, ops::Add, ops::AddAssign, ops::Sub};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq)]
pub struct PhysicalAddress(u64);

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq)]
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

    pub const fn align_up_offset(&self, alignment: u64) -> u64 {
        (alignment - (self.0 % alignment)) % alignment
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

    pub const fn align_up_offset(&self, alignment: u64) -> u64 {
        (alignment - (self.0 % alignment)) % alignment
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

impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl fmt::Debug for VirtualAddress {
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

impl Sub<VirtualAddress> for VirtualAddress {
    type Output = VirtualAddress;
    fn sub(self, rhs: VirtualAddress) -> Self::Output {
        VirtualAddress::new(self.0 - rhs.0)
    }
}

impl Add<VirtualAddress> for VirtualAddress {
    type Output = VirtualAddress;
    fn add(self, rhs: VirtualAddress) -> Self::Output {
        VirtualAddress::new(self.0 + rhs.0)
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
