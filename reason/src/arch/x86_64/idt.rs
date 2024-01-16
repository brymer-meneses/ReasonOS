use core::arch::asm;
use core::mem;
use core::ptr::NonNull;

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
    const NULL: Self = Self {
        isr_low: 0,
        kernel_code_segment: 0,
        ist: 0,
        attributes: 0,
        isr_mid: 0,
        isr_high: 0,
        reserved: 0,
    };
}

impl IdtEntry {
    fn set_entry(&mut self, handler: u64, flags: u8) {
        self.isr_low = (handler & 0xffff) as u16;
        self.isr_mid = ((handler >> 16) & 0xffff) as u16;
        self.isr_high = ((handler >> 32) & 0xFFFFFFFF) as u32;

        self.kernel_code_segment = 0x08;
        self.ist = 0;
        self.attributes = flags;
        self.reserved = 0;
    }
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

impl IdtPtr {
    const NULL: Self = Self { limit: 0, base: 0 };
}

const IDT_ENTRIES: usize = 256;

static mut INTERRUPT_DESCRIPTOR_TABLE: [IdtEntry; IDT_ENTRIES] = [IdtEntry::NULL; IDT_ENTRIES];

extern "C" {
    static INTERRUPT_HANDLERS: [extern "C" fn() -> !; IDT_ENTRIES];
}

pub fn initialize() {
    let mut idtptr = IdtPtr::NULL;
    unsafe {
        idtptr.base = &INTERRUPT_DESCRIPTOR_TABLE as *const _ as u64;
        idtptr.limit = (mem::size_of::<IdtEntry>() * IDT_ENTRIES - 1) as u16;
        for (index, &handler) in INTERRUPT_HANDLERS.iter().enumerate() {
            INTERRUPT_DESCRIPTOR_TABLE[index].set_entry(handler as u64, 0x8E);
        }
        load_idt(&idtptr);

        log::info!("Initialized IDT at 0x{:016X}", &INTERRUPT_DESCRIPTOR_TABLE as *const _ as u64);
    }

}

#[inline(always)]
unsafe fn load_idt(ptr: &IdtPtr) {
    asm!("lidt [{0}]", in(reg) ptr, options(nostack));
}
