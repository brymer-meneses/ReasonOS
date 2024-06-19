const gdt = @import("gdt.zig");
const std = @import("std");
const cpu = @import("cpu.zig");
const log = @import("../../log.zig");

const Privilege = enum(u2) {
    ring0 = 0,
    ring1 = 1,
    ring2 = 2,
    ring3 = 3,
};

const IdtEntry = packed struct {
    // NOTE:
    // ISR stands for Interrupt Service Routine which will be invoked by the
    // CPU if it encounters an interrupt
    isr_address_low: u16,
    kernel_code_segment: u16 = gdt.KERNEL_CODE_SEGMENT,
    interrupt_stack_table: u3,
    __padding1: u5 = 0,
    flags: Flags,
    isr_address_mid: u16,
    isr_address_high: u32,
    __padding2: u32 = 0,

    const Flags = packed struct(u8) {
        gate_type: enum(u4) {
            interrupt_gate = 0xE,
            trap_gate = 0xF,
        },
        __padding: u1 = 0,
        privilege: Privilege,
        present: u1,
    };

    const Self = @This();

    pub fn init(isr_address: u64, flags: Flags) Self {
        return .{
            .isr_address_low = @truncate(isr_address),
            .isr_address_mid = @truncate(isr_address >> 16),
            .isr_address_high = @truncate(isr_address >> 32),
            .flags = flags,
            .interrupt_stack_table = 0,
        };
    }
};

const IdtPtr = packed struct {
    limit: u16,
    base: u64,
};

var Idt: [256]IdtEntry = undefined;

extern fn interrupt_handler0() void;

pub fn install() void {
    const flags: IdtEntry.Flags = .{
        .gate_type = .interrupt_gate,
        .privilege = Privilege.ring0,
        .present = 1,
    };

    for (0..256) |i| {
        Idt[i] = IdtEntry.init(@intFromPtr(&interrupt_handler0) + i * 16, flags);
    }

    const idtptr = IdtPtr{
        .limit = @sizeOf(IdtEntry) * Idt.len - 1,
        .base = @intFromPtr(&Idt),
    };

    asm volatile ("lidt (%[idtptr])"
        :
        : [idtptr] "r" (&idtptr),
    );
    //
    log.info("Loaded IDT", .{});
}

const InterruptContext = extern struct {
    cpu: cpu.Registers,
    interrupt: cpu.InterruptFrame,
};

pub export fn interrupt_dispatch(ctx: *const InterruptContext) callconv(.C) void {
    log.write("Caught an exception! 0x{x}", .{ctx.interrupt.interrupt_number});

    inline for (std.meta.fields(cpu.Registers)) |f| {
        log.write("{s}: 0x{x}", .{ f.name, @field(ctx.cpu, f.name) });
    }

    inline for (std.meta.fields(cpu.InterruptFrame)) |f| {
        log.write("{s}: 0x{x}", .{ f.name, @field(ctx.interrupt, f.name) });
    }
}
