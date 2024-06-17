const GdtEntry = packed struct {
    limit_low: u16,
    base_low: u24,
    access: Access,
    limit_high: u4,
    flags: Flags,
    base_high: u8,

    const Access = packed struct {
        accessed: bool,
        readble_writable: bool,
        direction_conforming: bool,
        executable: bool,
        descriptor_type: bool,
        descriptor_privilege: u2,
        present: bool,
    };

    const Flags = packed struct {
        reserved: bool,
        long_mode: bool,
        size: bool,
        granularity: bool,
    };

    const Self = @This();

    pub fn init(base: u32, limit: u24, access: Access, flags: Flags) Self {
        return .{
            .limit_low = limit & ~0x10000,
            .limit_high = limit >> 16,
            .access = access,
            .flags = flags,
            .base_high = base >> 24,
            .base_low = base & ~0x1000000,
        };
    }

    pub fn empty() Self {
        return .{
            .limit_low = 0,
            .base_low = 0,
            .base_high = 0,
            .access = .{
                .accessd = false,
                .readble_writable = false,
                .direction_conforming = false,
                .executable = false,
                .descriptor_type = false,
                .descriptor_privilege = 0,
                .present = false,
            },
            .limit_high = 0,
            .flags = .{
                .reserved = false,
                .long_mode = false,
                .size = false,
                .granularity = false,
            },
        };
    }
};

const GdtPtr = packed struct {
    limit: u16,
    base: u64,
};

var Gdt: [5]GdtEntry = undefined;

pub fn install() void {

    // null descriptor
    Gdt[0] = GdtEntry.init(
        0,
        0,
        .{
            .accessd = false,
            .readble_writable = false,
            .direction_conforming = false,
            .executable = false,
            .descriptor_type = false,
            .descriptor_privilege = 0,
            .present = false,
        },
        .{
            .reserved = false,
            .long_mode = false,
            .size = false,
            .granularity = false,
        },
    );

    // kernel code segment
    Gdt[1] = GdtEntry.init(
        0,
        0xFFFFF,
        .{
            .accessd = false,
            .readble_writable = true,
            .direction_conforming = false,
            .executable = true,
            .descriptor_type = true,
            .descriptor_privilege = 0,
            .present = true,
        },
        .{
            .reserved = false,
            .long_mode = false,
            .size = true,
            .granularity = true,
        },
    );

    // kernel mode data segment
    Gdt[2] = GdtEntry.init(
        0,
        0xFFFFF,
        .{
            .accessd = false,
            .readble_writable = true,
            .direction_conforming = false,
            .executable = false,
            .descriptor_type = true,
            .descriptor_privilege = 0,
            .present = true,
        },
        .{
            .reserved = false,
            .long_mode = false,
            .size = true,
            .granularity = true,
        },
    );

    // user mode code segment
    Gdt[3] = GdtEntry.init(
        0,
        0xFFFFF,
        .{
            .accessd = false,
            .readble_writable = true,
            .direction_conforming = false,
            .executable = true,
            .descriptor_type = true,
            .descriptor_privilege = 0b11,
            .present = true,
        },
        .{
            .reserved = false,
            .long_mode = false,
            .size = true,
            .granularity = true,
        },
    );

    // user mode code segment
    Gdt[4] = GdtEntry.init(
        0,
        0xFFFFF,
        .{
            .accessd = false,
            .readble_writable = true,
            .direction_conforming = false,
            .executable = false,
            .descriptor_type = true,
            .descriptor_privilege = 0b11,
            .present = true,
        },
        .{
            .reserved = false,
            .long_mode = false,
            .size = true,
            .granularity = true,
        },
    );

    _ = GdtPtr{
        .limit = @sizeOf(GdtEntry) * Gdt.len - 1,
        .base = @intFromPtr(&Gdt),
    };
}
