
use lazy_static::lazy_static;
use spin::Mutex;

use crate::arch::x86_64::cpu::Context;
use crate::serial::println;

pub type InterruptHandler = fn(*const Context);

lazy_static! {
    static ref HANDLERS: Mutex<[InterruptHandler; 256]> = Mutex::new([dump_context; 256]);
}

pub fn set_interrupt_handler(vector: u16, handler: InterruptHandler) {
    (HANDLERS.lock())[vector as usize] = handler;
}

fn dump_context(context: *const Context) {
}

#[no_mangle]
extern "C" fn interrupt_dispatch(context: *const Context) {
    panic!("PANIC!");
}
