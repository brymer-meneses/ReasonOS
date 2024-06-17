const std = @import("std");

const AtomicBool = std.atomic.Value(bool);

pub const SpinLock = struct {
    state: AtomicBool,

    const Self = @This();

    pub fn init() Self {
        return .{
            .state = AtomicBool.init(false),
        };
    }

    pub fn acquire(self: *Self) void {
        while (true) {
            if (!self.state.swap(true, .acquire)) {
                return;
            }
            while (self.state.load(.unordered)) {}
        }
    }

    pub fn release(self: *Self) void {
        self.state.store(false, .release);
    }
};
