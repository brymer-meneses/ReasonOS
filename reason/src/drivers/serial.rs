
use crate::drivers::io;

const PORT: u16 = 0x3f8;

pub fn initialize() {
    unsafe {
      io::outb(PORT + 1, 0x00);
      io::outb(PORT + 3, 0x80);
      io::outb(PORT + 0, 0x03);
      io::outb(PORT + 1, 0x00);
      io::outb(PORT + 3, 0x03);
      io::outb(PORT + 2, 0xC7);
      io::outb(PORT + 4, 0x0B);
      io::outb(PORT + 4, 0x1E);
      io::outb(PORT + 0, 0xAE);
    }
}
