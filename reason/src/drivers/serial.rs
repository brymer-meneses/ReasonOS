use crate::arch::io;

const PORT: u16 = 0x3f8;

use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;
use core::fmt::Write;

lazy_static! {
    static ref SERIAL_WRITER: Mutex<Writer> = Mutex::new(Writer { port: PORT });
}

struct Writer {
    port: u16,
}

impl Writer {
    fn initialize(&self) {
        io::outb(self.port + 1, 0x00);
        io::outb(self.port + 3, 0x80);
        io::outb(self.port + 0, 0x03);
        io::outb(self.port + 1, 0x00);
        io::outb(self.port + 3, 0x03);
        io::outb(self.port + 2, 0xC7);
        io::outb(self.port + 4, 0x0B);
        io::outb(self.port + 4, 0x1E);
        io::outb(self.port + 0, 0xAE);
        io::outb(self.port + 4, 0x0F);
    }

    fn write_character(&self, character: char) {
        if character == '\0' {
            return;
        }
        io::outb(PORT, character as u8);
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
    SERIAL_WRITER.lock().initialize();
}

#[doc(hidden)]
pub fn _write(args: fmt::Arguments) {
    let _ = SERIAL_WRITER.lock().write_fmt(args);
}

macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_write( format_args!($($arg)*)));
}

pub(crate) use print;
