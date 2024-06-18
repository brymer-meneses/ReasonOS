pub const arch = @This();

const builtin = @import("builtin");

pub const cpu = switch (builtin.cpu.arch) {
    .x86_64 => @import("x86_64/cpu.zig"),
    else => unreachable,
};

pub fn init() void {
    switch (builtin.cpu.arch) {
        .x86_64 => {
            const gdt = @import("x86_64/gdt.zig");
            const idt = @import("x86_64/idt.zig");

            gdt.install();
            idt.install();
        },
        else => unreachable,
    }
}
