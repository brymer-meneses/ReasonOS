# ReasonOS

Yet another Operating System in Zig. Motivating myself to live, I guess?

## Running

This project makes use of the zig build system. You need to have
[xorriso](https://www.gnu.org/software/xorriso/) to build the iso, 
and [qemu](https://www.qemu.org/) to emulate it on your host operating system.

To run it on QEMU, invoke the following commands
```bash
# fetch dependences
zig build --fetch
# run on qemu
zig build run
```

