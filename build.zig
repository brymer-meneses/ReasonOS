const std = @import("std");

const SupportedArchs = enum {
    x86_64,
};

pub fn build(b: *std.Build) void {
    const arch = b.option(SupportedArchs, "arch", "Target Architecture") orelse .x86_64;
    const optimize = b.standardOptimizeOption(.{});

    const kernel = configure_kernel(b, arch, optimize);
    const iso = prepare_iso(b, kernel, arch, optimize);

    {
        const compile_kernel = b.step("kernel", "Compile the kernel");
        compile_kernel.dependOn(&kernel.step);
    }

    {
        const create_iso = b.step("iso", "Create the iso");
        create_iso.dependOn(&iso.step);
    }

    {
        const run_iso = b.step("run", "Run the ISO in QEMU");
        const qemu = b.addSystemCommand(&.{
            "qemu-system-" ++ @tagName(arch),
            "-serial",
            "stdio",
            "-D",
            "qemu-log.txt",
            "-d",
            "int",
            "-M",
            "smm=off",
            "-device",
            "isa-debug-exit,iobase=0xf4,iosize=0x0f",
            "-cdrom",
        });
        qemu.addFileArg(iso.source);
        run_iso.dependOn(&qemu.step);
    }
}

pub fn configure_kernel(b: *std.Build, arch: SupportedArchs, optimize: std.builtin.OptimizeMode) *std.Build.Step.Compile {
    const limine_zig = b.dependency("limine_zig", .{});
    const target = configure_target(b, arch);

    switch (arch) {
        .x86_64 => {
            const kernel = b.addExecutable(.{
                .name = "kernel",
                .root_source_file = b.path("kernel/main.zig"),
                .target = target,
                .optimize = optimize,
                .code_model = .kernel,
                .pic = true,
            });

            kernel.addAssemblyFile(b.path("kernel/arch/x86_64/load_gdt.S"));

            kernel.root_module.addImport("limine", limine_zig.module("limine"));
            kernel.want_lto = false; // Disable LTO. This prevents issues with limine requests
            kernel.setLinkerScriptPath(b.path("kernel/arch/x86_64/linker.ld"));
            return kernel;
        },
    }
}

pub fn configure_target(b: *std.Build, arch: SupportedArchs) std.Build.ResolvedTarget {
    var target: std.zig.CrossTarget = .{
        .cpu_arch = .x86_64,
        .os_tag = .freestanding,
        .abi = .none,
    };

    switch (arch) {
        .x86_64 => {
            target.cpu_arch = .x86_64;

            // Disable CPU features that require additional initialization
            // like MMX, SSE/2 and AVX. That requires us to enable the soft-float feature.
            const Features = std.Target.x86.Feature;
            target.cpu_features_sub.addFeature(@intFromEnum(Features.mmx));
            target.cpu_features_sub.addFeature(@intFromEnum(Features.sse));
            target.cpu_features_sub.addFeature(@intFromEnum(Features.sse2));
            target.cpu_features_sub.addFeature(@intFromEnum(Features.avx));
            target.cpu_features_sub.addFeature(@intFromEnum(Features.avx2));
            target.cpu_features_add.addFeature(@intFromEnum(Features.soft_float));

            return b.resolveTargetQuery(target);
        },
    }
}

pub fn prepare_iso(b: *std.Build, kernel: *std.Build.Step.Compile, arch: SupportedArchs, optimize: std.builtin.OptimizeMode) *std.Build.Step.InstallFile {
    const limine = b.dependency("limine", .{});
    const limine_exe = b.addExecutable(.{
        .name = "limine",
        .target = b.standardTargetOptions(.{}),
        .optimize = optimize,
    });
    limine_exe.addCSourceFile(.{ .file = limine.path("limine.c"), .flags = &.{"-std=c99"} });

    const iso_root = b.addWriteFiles();

    _ = iso_root.addCopyFile(limine.path("limine-bios.sys"), "boot/limine/limine-bios.sys");
    _ = iso_root.addCopyFile(limine.path("limine-bios-cd.bin"), "boot/limine/limine-bios-cd.bin");
    _ = iso_root.addCopyFile(limine.path("limine-uefi-cd.bin"), "boot/limine/limine-uefi-cd.bin");
    _ = iso_root.addCopyFile(limine.path("BOOTX64.EFI"), "boot/EFI/BOOT/BOOTX64.EFI");
    _ = iso_root.addCopyFile(limine.path("BOOTIA32.EFI"), "boot/EFI/BOOT/BOOTIA32.EFI");
    _ = iso_root.addCopyFile(kernel.getEmittedBin(), "boot/kernel");
    _ = iso_root.addCopyFile(b.path("limine.cfg"), "limine.cfg");

    const xorriso = b.addSystemCommand(&.{
        "xorriso",
        "-as",
        "mkisofs",
        "-b",
        "boot/limine/limine-bios-cd.bin",
        "-no-emul-boot",
        "-boot-load-size",
        "4",
        "-boot-info-table",
        "--efi-boot",
        "boot/limine/limine-uefi-cd.bin",
        "-efi-boot-part",
        "--efi-boot-image",
        "--protective-msdos-label",
    });
    xorriso.addDirectoryArg(iso_root.getDirectory());
    xorriso.addArg("-o");

    const iso_path = xorriso.addOutputFileArg("reason-os.iso");
    const limine_installed_iso = b.addRunArtifact(limine_exe);
    limine_installed_iso.addArg("bios-install");
    limine_installed_iso.addFileArg(iso_path);

    const iso_name = "reason-os-" ++ @tagName(arch) ++ ".iso";
    const iso = b.addInstallFileWithDir(
        iso_path,
        .prefix,
        iso_name,
    );

    return iso;
}
