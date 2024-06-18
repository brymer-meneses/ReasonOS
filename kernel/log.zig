const log = @This();

const std = @import("std");
const builtin = @import("builtin");

const SpinLock = @import("lock.zig").SpinLock;

const Writer = std.io.Writer(*SpinLock, error{}, writeFn);
var writerContext = SpinLock.init();

pub const writer = Writer{ .context = &writerContext };

fn writeFn(lock: *SpinLock, bytes: []const u8) error{}!usize {
    lock.acquire();
    defer lock.release();

    const cpu = @import("arch/arch.zig").cpu;
    for (bytes) |byte| {
        cpu.writeByte(0x3f8, byte);
    }

    return bytes.len;
}

pub fn debug(comptime fmt: []const u8, args: anytype) void {
    std.fmt.format(writer, "[DEBUG]: " ++ fmt ++ "\n", args) catch return;
}

pub fn info(comptime fmt: []const u8, args: anytype) void {
    std.fmt.format(writer, "[INFO]: " ++ fmt ++ "\n", args) catch return;
}

pub fn warn(comptime fmt: []const u8, args: anytype) void {
    std.fmt.format(writer, "[WARN]: " ++ fmt ++ "\n", args) catch return;
}

pub fn write(comptime fmt: []const u8, args: anytype) void {
    std.fmt.format(writer, fmt ++ "\n", args) catch return;
}
