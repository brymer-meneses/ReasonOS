const arch = @import("kernel").arch;
const log = @import("kernel").utils.log;

const limine = @import("limine");
const std = @import("std");
const builtin_panic = @import("std").builtin.panic;

pub export var framebuffer_request: limine.FramebufferRequest = .{};
pub export var base_revision: limine.BaseRevision = .{ .revision = 2 };

inline fn done() noreturn {
    while (true) {
        asm volatile ("hlt");
    }
}

pub fn panic(message: []const u8, _: ?*std.builtin.StackTrace, _: ?usize) noreturn {
    log.write("FATAL: {s}", .{message});

    done();
}

export fn _start() callconv(.C) noreturn {
    if (!base_revision.is_supported()) {
        done();
    }

    arch.init();

    if (framebuffer_request.response) |framebuffer_response| {
        if (framebuffer_response.framebuffer_count < 1) {
            done();
        }
        const framebuffer = framebuffer_response.framebuffers()[0];

        for (0..100) |i| {
            const pixel_offset = i * framebuffer.pitch + i * 4;
            @as(*u32, @ptrCast(@alignCast(framebuffer.address + pixel_offset))).* = 0xFFFFFFFF;
        }
    }

    asm volatile ("int $0x99");

    done();
}
