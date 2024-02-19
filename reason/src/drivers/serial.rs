use core::fmt;
use core::fmt::Write;

use crate::arch::cpu;
use crate::misc::utils::OnceCellMutex;

static mut SERIAL_WRITER: OnceCellMutex<Writer> = OnceCellMutex::new();

struct Writer {
    port: u16,
}

impl Writer {
    fn new(port: u16) -> Self {
        cpu::outb(port + 1, 0x00);
        cpu::outb(port + 3, 0x80);
        cpu::outb(port, 0x03);
        cpu::outb(port + 1, 0x00);
        cpu::outb(port + 3, 0x03);
        cpu::outb(port + 2, 0xC7);
        cpu::outb(port + 4, 0x0B);
        cpu::outb(port + 4, 0x1E);
        cpu::outb(port, 0xAE);
        cpu::outb(port + 4, 0x0F);

        Writer { port }
    }

    fn write_character(&self, character: char) {
        if character == '\0' {
            return;
        }
        cpu::outb(self.port, character as u8);
    }

    fn write_string(&self, string: &str) {
        for character in string.chars() {
            self.write_character(character);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn initialize() {
    unsafe {
        SERIAL_WRITER.set(Writer::new(0x3f8));
    }
}

#[doc(hidden)]
pub fn _write(args: fmt::Arguments) {
    unsafe {
        let _ = SERIAL_WRITER.lock().write_fmt(args);
    }
}

macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_write( format_args!($($arg)*)));
}

macro_rules! println {
    () => ($crate::serial::print!("\n"));
    ($($arg:tt)*) => ($crate::serial::print!("{}\n", format_args!($($arg)*)));
}

pub(crate) use print;
pub(crate) use println;
