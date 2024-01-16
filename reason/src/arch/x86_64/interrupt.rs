use lazy_static::lazy_static;
use spin::Mutex;

use crate::arch::cpu::{self, Context};
use crate::log;
use crate::serial::println;

pub type InterruptHandler = unsafe fn(*const Context);

lazy_static! {
    static ref HANDLERS: Mutex<[Option<InterruptHandler>; 256]> = Mutex::new([None; 256]);
}

pub fn set_interrupt_handler(vector: usize, handler: InterruptHandler) {
    let mut handlers = HANDLERS.lock();
    handlers[vector] = Some(handler);
    log::debug!("Set handler for vector 0x{:0X}", vector);
}

const fn get_exception_message(vector: usize) -> &'static str {
    match vector {
        0 => "Division Error",
        1 => "Debug",
        2 => "Non-Maskable Interrupt",
        3 => "Breakpoint",
        4 => "Overflow",
        5 => "Bound Range Exceeded",
        6 => "Invalid Opcode",
        7 => "Device not Available",
        8 => "Double Fault",
        10 => "Invalid TSS",
        11 => "Segment Not Present",
        12 => "Stack-Segment Fault",
        13 => "General Protection Fault",
        14 => "Page Fault",
        16 => "x87 Floating-Point Exception",
        17 => "Aligment Check",
        18 => "Machine Check",
        19 => "SIMD Floating-Point Exception",
        20 => "Virtualization Exception",
        21 => "Control Protection Exception",
        30 => "Hypervisor Injection Exception",
        31 => "VMM Communication Exception",
        32 => "Security Exception",
        _ => panic!("Got an ISR somehow??"),
    }
}

unsafe fn dump_context(ctx: *const Context) {
    let ctx = ctx.read_volatile();

    println!("----------------------------");
    println!("Received Exception {}", ctx.vector);
    println!("Description: {}", get_exception_message(ctx.error as usize));
    println!("Error Code: {}", ctx.error);
    println!("----------------------------");
    println!("rax: 0x{:016X}", ctx.rax);
    println!("rbx: 0x{:016X}", ctx.rbx);
    println!("rcx: 0x{:016X}", ctx.rcx);
    println!("rdx: 0x{:016X}", ctx.rdx);
    println!("rsi: 0x{:016X}", ctx.rsi);
    println!("rdi: 0x{:016X}", ctx.rdi);
    println!("r8: 0x{:016x}", ctx.r8);
    println!("r9: 0x{:016x}", ctx.r9);
    println!("r10: 0x{:016x}", ctx.r10);
    println!("r11: 0x{:016x}", ctx.r11);
    println!("r12: 0x{:016x}", ctx.r12);
    println!("r13: 0x{:016x}", ctx.r13);
    println!("r14: 0x{:016x}", ctx.r14);
    println!("r15: 0x{:016x}", ctx.r15);
    println!("rip: 0x{:016x}", ctx.iret_rip);
    println!("cs: 0x{:016x}", ctx.iret_cs);
    println!("flags: 0x{:016x}", ctx.iret_flags);
    println!("rsp: 0x{:016X}", ctx.iret_rsp);
    println!("ss: 0x{:016X}", ctx.iret_ss);
    println!("----------------------------");

    cpu::halt();
}

#[no_mangle]
extern "C" fn interrupt_dispatch(context: *const Context) {
    let handlers = HANDLERS.lock();
    unsafe {
        let vector = context.read_volatile().vector;
        if let Some(handler) = handlers[vector as usize] {
            (handler)(context);
        } else {
            dump_context(context);
        }
    }
}
