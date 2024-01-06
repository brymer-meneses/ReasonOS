use lazy_static::lazy_static;
use spin::Mutex;
use core::mem;

use crate::log;

#[derive(Clone, Copy, Default)]
#[repr(packed, C)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    flags: u8,
    base_high: u8,
}

impl GdtEntry {
    const NULL: Self = Self {
        limit_low: 0,
        base_low: 0,
        base_middle: 0,
        access: 0,
        flags: 0,
        base_high: 0
    };
}

#[repr(packed, C)]
struct GdtPtr {
    limit: u16,
    base: u64,
}

impl GdtPtr {
    const NULL: Self = Self {
        limit: 0,
        base: 0
    };
}

static mut GLOBAL_DESCRIPTOR_TABLE: [GdtEntry; 5] = [GdtEntry::NULL; 5];

fn set_entry(vector: u8, access: u8, flags: u8) {
    unsafe {
        let gdt = &mut GLOBAL_DESCRIPTOR_TABLE;
        let vector = vector as usize;
        gdt[vector].base_low = 0x0000;
        gdt[vector].base_middle = 0x00;
        gdt[vector].base_high = 0x00;
        gdt[vector].limit_low = 0x0000;
        gdt[vector].access = access;
        gdt[vector].flags = flags;
    }
}

extern "C" {
    fn load_gdt(ptr: *const GdtPtr);
}

pub fn initialize() {

    unsafe {
        let mut gdtpr = GdtPtr::NULL;

        gdtpr.limit = (mem::size_of::<GdtEntry>() * 5 - 1) as u16;
        gdtpr.base = &GLOBAL_DESCRIPTOR_TABLE as *const _ as u64;

        set_entry(0, 0, 0);
        set_entry(1, 0x9A, 0xA0);
        set_entry(2, 0x92, 0xC0);
        set_entry(3, 0xFA, 0xA0);
        set_entry(4, 0xF2, 0xC0);

        load_gdt(&gdtpr); 
        log::info!("Initialized GDT at 0x{:016X}", &GLOBAL_DESCRIPTOR_TABLE as *const _ as u64);
    };


}
