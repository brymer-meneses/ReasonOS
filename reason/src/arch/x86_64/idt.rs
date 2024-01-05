
use core::ptr::NonNull;
use core::mem;
use core::arch::asm;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::arch::x86_64::interrupt::InterruptHandler;
use crate::misc::log;

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    isr_low: u16,
    kernel_code_segment: u16,
    ist: u8,
    attributes: u8,
    isr_mid: u16,
    isr_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn new() -> Self {
        IdtEntry {
            isr_low: 0,
            kernel_code_segment: 0,
            ist: 0,
            attributes: 0,
            isr_mid: 0,
            isr_high: 0,
            reserved: 0
        }
    }
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

impl IdtPtr {
    const fn new() -> Self {
        IdtPtr {
            limit: 0,
            base: 0
        }
    }
}

static mut INTERRUPT_DESCRIPTOR_TABLE: [IdtEntry; 256] = [IdtEntry::new(); 256];

extern "C" {
    static INTERRUPT_HANDLER_TABLE: [u64; 32];
} 


pub fn set_entry(vector: u8, handler: u64, flags: u8) {
    unsafe {
        let idt = &mut INTERRUPT_DESCRIPTOR_TABLE;
        let vector = vector as usize;

        idt[vector].isr_low = (handler & 0xffff) as u16;
        idt[vector].isr_mid = ((handler >> 16) & 0xffff) as u16;
        idt[vector].isr_high = ((handler >> 32) & 0xFFFFFFFF) as u32;

        idt[vector].kernel_code_segment = 0x08;
        idt[vector].ist = 0;
        idt[vector].attributes = flags;
        idt[vector].reserved = 0;
    }
}

pub fn initialize() {
    let mut idtptr = IdtPtr::new();
    unsafe {
        idtptr.base = &INTERRUPT_DESCRIPTOR_TABLE as *const _ as u64;
        idtptr.limit = (mem::size_of::<IdtEntry>() * 256 - 1) as u16;
        
        for i in 0..32 {
            let addr = &INTERRUPT_HANDLER_TABLE[i as usize] as *const _ as u64;
            log::info!("Address {i} 0x{:016X}", addr);
            set_entry(i, addr, 0x8E);
        }

        asm!("lidt [{0}]", in(reg) &idtptr)
    }

    log::info!("Initialized IDT");
}

