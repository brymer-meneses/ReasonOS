const std = @import("std");
const log = @import("../../log.zig");

const GdtEntry = packed struct {
    limit_low: u16,
    base_low: u24,
    access: Access,
    limit_high: u4,
    flags: Flags,
    base_high: u8,

    const Access = packed struct {
        accessed: u1,
        readble_writable: u1,
        direction_conforming: u1,
        executable: u1,
        descriptor_type: u1,
        descriptor_privilege: u2,
        present: u1,
    };

    const Flags = packed struct {
        reserved: u1,
        long_mode: u1,
        size: u1,
        granularity: u1,
    };

    const Self = @This();

    pub fn init(base: u32, limit: u24, access: Access, flags: Flags) Self {
        return .{
            .limit_low = @truncate(limit),
            .limit_high = @truncate(limit >> 16),
            .base_low = @truncate(base),
            .base_high = @truncate(base >> 24),
            .access = access,
            .flags = flags,
        };
    }
};

const GdtPtr = packed struct {
    limit: u16,
    base: u64,
};

var Gdt: [5]GdtEntry = undefined;

extern fn load_gdt(gdtptr: *const GdtPtr) callconv(.C) void;

pub fn install() void {

    // null descriptor
    Gdt[0] = GdtEntry.init(
        0,
        0,
        .{
            .accessed = 0,
            .readble_writable = 0,
            .direction_conforming = 0,
            .executable = 0,
            .descriptor_type = 0,
            .descriptor_privilege = 0,
            .present = 0,
        },
        .{
            .reserved = 0,
            .long_mode = 0,
            .size = 0,
            .granularity = 0,
        },
    );

    // kernel code segment
    Gdt[1] = GdtEntry.init(
        0,
        0xFFFFF,
        .{
            .accessed = 0,
            .readble_writable = 1,
            .direction_conforming = 0,
            .executable = 1,
            .descriptor_type = 1,
            .descriptor_privilege = 0,
            .present = 1,
        },
        .{
            .reserved = 0,
            .long_mode = 1,
            .size = 0,
            .granularity = 1,
        },
    );

    // kernel mode data segment
    Gdt[2] = GdtEntry.init(
        0,
        0xFFFFF,
        .{
            .accessed = 0,
            .readble_writable = 1,
            .direction_conforming = 0,
            .executable = 0,
            .descriptor_type = 1,
            .descriptor_privilege = 0,
            .present = 1,
        },
        .{
            .reserved = 0,
            .long_mode = 0,
            .size = 1,
            .granularity = 1,
        },
    );

    // user mode code segment
    Gdt[3] = GdtEntry.init(
        0,
        0xFFFFF,
        .{
            .accessed = 0,
            .readble_writable = 1,
            .direction_conforming = 0,
            .executable = 1,
            .descriptor_type = 1,
            .descriptor_privilege = 0b11,
            .present = 1,
        },
        .{
            .reserved = 0,
            .long_mode = 1,
            .size = 0,
            .granularity = 1,
        },
    );

    // user mode code segment
    Gdt[4] = GdtEntry.init(
        0,
        0xFFFFF,
        .{
            .accessed = 0,
            .readble_writable = 1,
            .direction_conforming = 0,
            .executable = 0,
            .descriptor_type = 1,
            .descriptor_privilege = 0b11,
            .present = 1,
        },
        .{
            .reserved = 0,
            .long_mode = 0,
            .size = 1,
            .granularity = 1,
        },
    );

    const gdtptr = GdtPtr{
        .limit = @sizeOf(GdtEntry) * Gdt.len - 1,
        .base = @intFromPtr(&Gdt),
    };

    load_gdt(&gdtptr);

    log.info("Loaded GDT!", .{});
}
