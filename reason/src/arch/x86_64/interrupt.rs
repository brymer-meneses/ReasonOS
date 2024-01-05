
use lazy_static::lazy_static;
use spin::Mutex;

use crate::arch::x86_64::cpu::Context;
use crate::serial::println;

pub type InterruptHandler = unsafe fn(*const Context);

lazy_static! {
    static ref HANDLERS: Mutex<[InterruptHandler; 256]> = Mutex::new([dump_context; 256]);
}

pub fn set_interrupt_handler(vector: usize, handler: InterruptHandler) {
    (HANDLERS.lock())[vector] = handler;
}

const fn get_exception_message(vector: usize) -> &'static str {
    return match vector {
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
        _ => panic!("Got an ISR somehow??")
    }
}

unsafe fn dump_context(context: *const Context) {
    let context = context.read_volatile(); 
    let vector = context.vector;
    let error_code = context.error;
    let rax = context.rax;
    let rbx = context.rbx;
    let rcx = context.rcx;
    let rdx = context.rdx;
    let rsi = context.rsi;
    let rdi = context.rdi;
    let r8 = context.r8;
    let r9 = context.r9;
    let r10 = context.r10;
    let r11 = context.r11;
    let r12 = context.r12;
    let r13 = context.r13;
    let r14 = context.r14;
    let r15 = context.r15;
    let iret_rip = context.iret_rip;
    let iret_cs = context.iret_cs;
    let iret_flags = context.iret_flags;
    let iret_rsp = context.iret_rsp;
    let iret_ss = context.iret_ss;

    println!("----------------------------");
    println!("Received Exception {}", vector);
    println!("Description: {}", get_exception_message(vector as usize));
    println!("Error Code: {:016x}", error_code);
    println!("----------------------------");
    println!("rax: 0x{:016X}", rax);
    println!("rbx: 0x{:016X}", rbx);
    println!("rcx: 0x{:016X}", rcx);
    println!("rdx: 0x{:016X}", rdx);
    println!("rsi: 0x{:016X}", rsi);
    println!("rdi: 0x{:016X}", rdi);
    println!("r8: 0x{:016x}", r8);
    println!("r9: 0x{:016x}", r9);
    println!("r10: 0x{:016x}", r10);
    println!("r11: 0x{:016x}", r11);
    println!("r12: 0x{:016x}", r12);
    println!("r13: 0x{:016x}", r13);
    println!("r14: 0x{:016x}", r14);
    println!("r15: 0x{:016x}", r15);
    println!("rip: 0x{:016x}", iret_rip);
    println!("cs: 0x{:016x}", iret_cs);
    println!("flags: 0x{:016x}", iret_flags);
    println!("rsp: 0x{:016X}", iret_rsp);
    println!("ss: 0x{:016X}", iret_ss);
    println!("----------------------------");
}

#[no_mangle]
extern "C" fn interrupt_dispatch(context: *const Context) {
    unsafe {
        let vector = context.read_volatile().vector;
        (HANDLERS.lock()[vector as usize])(context);
    }
}
