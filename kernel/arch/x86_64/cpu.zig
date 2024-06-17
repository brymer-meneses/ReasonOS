pub fn writeByte(port: u16, value: u8) void {
    asm volatile ("outb %[value], %[port]"
        :
        : [value] "{al}" (value),
          [port] "N{dx}" (port),
    );
}

const gdt = @import("gdt.zig");

pub fn init() void {
    gdt.install();
}
