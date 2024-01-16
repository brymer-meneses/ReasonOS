#![allow(unused)]

use crate::arch::cpu::{self, Context};
use crate::arch::interrupt;
use crate::boot::HHDM_OFFSET;
use crate::memory::pmm;
use crate::memory::vmm::VirtualMemoryFlags;
use crate::misc::log;

use bitflags::bitflags;
use core::arch::asm;
use core::ptr::NonNull;
use core::intrinsics::write_bytes;

pub const PAGE_SIZE: u64 = 4096;

const PTE_PRESENT: u64 = 1 << 0;
const PTE_WRITEABLE: u64 = 1 << 1;
const PTE_USER_ACCESSIBLE: u64 = 1 << 2;
const PTE_NOT_EXECUTABLE: u64 = 1 << 63;
const PTE_ADDRESS_MASK: u64 = 0x000ffffffffff000;

pub unsafe fn map(
    pml4: *mut u64,
    virtual_addr: u64,
    physical_addr: u64,
    flags: VirtualMemoryFlags,
) {
    assert_eq!(virtual_addr % PAGE_SIZE, 0);
    assert_eq!(physical_addr % PAGE_SIZE, 0);

    let pml4_index = ((virtual_addr >> 39) & 0x1ff) as usize;
    let pml3_index = ((virtual_addr >> 30) & 0x1ff) as usize;
    let pml2_index = ((virtual_addr >> 21) & 0x1ff) as usize;
    let pml1_index = ((virtual_addr >> 12) & 0x1ff) as usize;

    let pml3 = get_next_level(pml4, pml4_index, flags, true);
    let pml2 = get_next_level(pml3, pml3_index, flags, true);
    let pml1 = get_next_level(pml2, pml2_index, flags, true);

    let entry = set_flags(physical_addr, flags);

    pml1.add(pml1_index).write(entry);

    invalidate_tlb_cache(virtual_addr);

    log::debug!("Successfully mapped physical address 0x{:016X} to virtual address 0x{:016X}", physical_addr, virtual_addr);
}

pub unsafe fn unmap(pml4: *mut u64, virtual_addr: u64, physical_addr: u64) {
    assert_eq!(virtual_addr % PAGE_SIZE, 0);
    assert_eq!(physical_addr % PAGE_SIZE, 0);

    let flags = VirtualMemoryFlags::empty();

    let pml4_index = ((virtual_addr >> 39) & 0x1ff) as usize;
    let pml3_index = ((virtual_addr >> 30) & 0x1ff) as usize;
    let pml2_index = ((virtual_addr >> 21) & 0x1ff) as usize;
    let pml1_index = ((virtual_addr >> 12) & 0x1ff) as usize;

    let pml3 = get_next_level(pml4, pml4_index, flags, false);
    let pml2 = get_next_level(pml3, pml3_index, flags, false);
    let pml1 = get_next_level(pml2, pml2_index, flags, false);

    let entry = set_flags(physical_addr, flags);

    let addr = pml1.add(pml1_index).as_ref().unwrap() & PTE_ADDRESS_MASK;

    pmm::free_page(NonNull::new_unchecked(addr as *mut u64));

    pml1.add(pml1_index).write(0);

    invalidate_tlb_cache(virtual_addr);
}

pub fn initialize() {
    interrupt::set_interrupt_handler(0xE, page_fault_handler);
    interrupt::set_interrupt_handler(0xD, general_page_fault_handler);
}

fn page_fault_handler(ctx: *const Context) {
    unsafe {
        let ctx = ctx.read();
        panic!("Page Fault accessed memory: 0x{:016X} at RIP: 0x{:016X}", cpu::read_cr2(), ctx.iret_rip);
    }
}

fn general_page_fault_handler(ctx: *const Context) {
    unsafe {
        let ctx = ctx.read();
        panic!("General Page Fault accessed memory: 0x{:016X} at RIP: 0x{:016X}", cpu::read_cr2(), ctx.iret_rip);
    }
}

fn invalidate_tlb_cache(addr: u64) {
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) addr
        );
    }
}
fn set_flags(mut addr: u64, flags: VirtualMemoryFlags) -> u64 {
    addr |= PTE_PRESENT;

    if flags.contains(VirtualMemoryFlags::Writeable) {
        addr |= PTE_WRITEABLE;
    }
    if flags.contains(VirtualMemoryFlags::UserAccessible) {
        addr |= PTE_USER_ACCESSIBLE;
    }
    if !flags.contains(VirtualMemoryFlags::Executable) {
        addr |= PTE_NOT_EXECUTABLE;
    }

    return addr;
}

unsafe fn get_next_level(
    root: *mut u64,
    index: usize,
    flags: VirtualMemoryFlags,
    should_allocate: bool,
) -> *mut u64 {
    let root = root as *mut u64;
    let entry = root.add(index).read();

    if entry & PTE_PRESENT != 0 {
        return ((entry & PTE_ADDRESS_MASK) + HHDM_OFFSET) as *mut u64;
    }

    if !should_allocate {
        panic!("Tried to get the next level of a page table");
    }

    let page = pmm::allocate_page()
        .expect("Failed to allocate memory")
        .as_ptr() as u64;

    assert_eq!(page % PAGE_SIZE, 0);
    assert_ne!(page, 0);

    // zero out the newly allocated page-table
    write_bytes((page + HHDM_OFFSET) as *mut u8, 0, PAGE_SIZE as usize);

    let new_level = set_flags(page, flags);

    root.add(index).write(new_level);

    return ((new_level & PTE_ADDRESS_MASK) + HHDM_OFFSET) as *mut u64;
}

pub fn get_initial_pagemap() -> *mut u64 {
    unsafe {
        ((cpu::read_cr3() & PTE_ADDRESS_MASK) + HHDM_OFFSET) as *mut u64
    }
}
