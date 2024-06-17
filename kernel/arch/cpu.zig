const builtin = @import("builtin");

pub usingnamespace switch (builtin.cpu.arch) {
    .x86_64 => @import("x86_64/cpu.zig"),
    else => @compileError("Unsupported Architecture"),
};
