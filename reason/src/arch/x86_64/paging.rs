use crate::arch::cpu::{self, Context};
use crate::arch::interrupt;
use crate::boot::HHDM_OFFSET;

use crate::memory::VirtualMemoryFlags;
use crate::memory::PHYSICAL_MEMORY_MANAGER;
use crate::memory::{PhysicalAddress, VirtualAddress};

use core::ptr::NonNull;

use crate::misc::log;

use bitflags::bitflags;
use core::arch::asm;
use core::intrinsics::write_bytes;

pub const PAGE_SIZE: u64 = 4096;

const PTE_PRESENT: u64 = 1 << 0;
const PTE_WRITEABLE: u64 = 1 << 1;
const PTE_USER_ACCESSIBLE: u64 = 1 << 2;
const PTE_NOT_EXECUTABLE: u64 = 1 << 63;
const PTE_ADDRESS_MASK: u64 = 0x000ffffffffff000;

impl VirtualAddress {
    const fn get_indexes(&self) -> (usize, usize, usize, usize) {
        let addr = self.as_addr();
        (
            ((addr >> 39) & 0x1ff) as usize,
            ((addr >> 30) & 0x1ff) as usize,
            ((addr >> 21) & 0x1ff) as usize,
            ((addr >> 12) & 0x1ff) as usize,
        )
    }
}

pub unsafe fn map(
    pml4: VirtualAddress,
    virtual_addr: VirtualAddress,
    physical_addr: PhysicalAddress,
    flags: VirtualMemoryFlags,
) {
    assert!(virtual_addr.is_page_aligned());
    assert!(physical_addr.is_page_aligned());

    let (pml4_index, pml3_index, pml2_index, pml1_index) = virtual_addr.get_indexes();

    let pml3 = get_next_level(pml4, pml4_index, flags, true);
    let pml2 = get_next_level(pml3, pml3_index, flags, true);
    let pml1 = get_next_level(pml2, pml2_index, flags, true);

    let entry = set_flags(physical_addr, flags);

    pml1.as_ptr().add(pml1_index).write(entry);

    log::debug!(
        "[paging] Successfully mapped physical address {} to virtual address {}",
        physical_addr,
        virtual_addr
    );
}

pub unsafe fn unmap(
    pml4: VirtualAddress,
    virtual_addr: VirtualAddress,
    physical_addr: PhysicalAddress,
) {
    assert!(virtual_addr.is_page_aligned());
    assert!(physical_addr.is_page_aligned());

    let flags = VirtualMemoryFlags::empty();

    let (pml4_index, pml3_index, pml2_index, pml1_index) = virtual_addr.get_indexes();

    let pml3 = get_next_level(pml4, pml4_index, flags, false);
    let pml2 = get_next_level(pml3, pml3_index, flags, false);
    let pml1 = get_next_level(pml2, pml2_index, flags, false);

    let entry = set_flags(physical_addr, flags);

    let addr = pml1.as_ptr().add(pml1_index).as_ref().unwrap() & PTE_ADDRESS_MASK;

    PHYSICAL_MEMORY_MANAGER
        .lock()
        .free_page(PhysicalAddress::new(addr));

    pml1.as_ptr().add(pml1_index).write(0);

    invalidate_tlb_cache(virtual_addr);
}

pub fn initialize() {
    interrupt::set_interrupt_handler(0xE, page_fault_handler);
    interrupt::set_interrupt_handler(0xD, general_page_fault_handler);
}

fn page_fault_handler(ctx: *const Context) {
    unsafe {
        let ctx = ctx.read();
        panic!(
            "Page Fault accessed memory: 0x{:016X} at RIP: 0x{:016X}",
            cpu::read_cr2(),
            ctx.iret_rip
        );
    }
}

fn general_page_fault_handler(ctx: *const Context) {
    unsafe {
        let ctx = ctx.read();
        panic!(
            "General Page Fault accessed memory: 0x{:016X} at RIP: 0x{:016X}",
            cpu::read_cr2(),
            ctx.iret_rip
        );
    }
}

fn invalidate_tlb_cache(addr: VirtualAddress) {
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) addr.as_addr(),
        );
    }
}

fn set_flags(addr: PhysicalAddress, flags: VirtualMemoryFlags) -> u64 {
    let mut addr = addr.as_addr() | PTE_PRESENT;

    if flags.contains(VirtualMemoryFlags::Writeable) {
        addr |= PTE_WRITEABLE;
    }
    if flags.contains(VirtualMemoryFlags::UserAccessible) {
        addr |= PTE_USER_ACCESSIBLE;
    }
    if !flags.contains(VirtualMemoryFlags::Executable) {
        addr |= PTE_NOT_EXECUTABLE;
    }

    addr
}

unsafe fn get_next_level(
    root: VirtualAddress,
    index: usize,
    flags: VirtualMemoryFlags,
    should_allocate: bool,
) -> VirtualAddress {
    let root = root.as_ptr();
    let entry = root.add(index).read();

    if entry & PTE_PRESENT != 0 {
        return VirtualAddress::new(((entry & PTE_ADDRESS_MASK) + HHDM_OFFSET));
    }

    if !should_allocate {
        panic!("Tried to get the next level of a page table");
    }

    let page = PHYSICAL_MEMORY_MANAGER
        .lock()
        .allocate_page()
        .expect("Failed to allocate memory");

    assert!(page.is_page_aligned());
    assert!(!page.is_null());

    // zero out the newly allocated page-table
    write_bytes(
        (page.as_addr() + HHDM_OFFSET) as *mut u8,
        0,
        PAGE_SIZE as usize,
    );

    let new_level = set_flags(page, flags);

    root.add(index).write(new_level);

    VirtualAddress::new(((new_level & PTE_ADDRESS_MASK) + HHDM_OFFSET))
}

pub fn get_initial_pagemap() -> VirtualAddress {
    unsafe { VirtualAddress::new(((cpu::read_cr3() & PTE_ADDRESS_MASK) + HHDM_OFFSET)) }
}
