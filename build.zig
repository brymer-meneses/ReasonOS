const std = @import("std");

pub fn build(b: *std.Build) void {
    var target: std.zig.CrossTarget = .{
        .cpu_arch = .x86_64,
        .os_tag = .freestanding,
        .abi = .none,
    };

    // Disable CPU features that require additional initialization
    // like MMX, SSE/2 and AVX. That requires us to enable the soft-float feature.
    const Features = std.Target.x86.Feature;
    target.cpu_features_sub.addFeature(@intFromEnum(Features.mmx));
    target.cpu_features_sub.addFeature(@intFromEnum(Features.sse));
    target.cpu_features_sub.addFeature(@intFromEnum(Features.sse2));
    target.cpu_features_sub.addFeature(@intFromEnum(Features.avx));
    target.cpu_features_sub.addFeature(@intFromEnum(Features.avx2));
    target.cpu_features_add.addFeature(@intFromEnum(Features.soft_float));

    const optimize = b.standardOptimizeOption(.{});
    const limine_zig = b.dependency("limine_zig", .{});

    const kernel = b.addExecutable(.{
        .name = "kernel",
        .root_source_file = b.path("kernel/main.zig"),
        .target = b.resolveTargetQuery(target),
        .optimize = optimize,
        .code_model = .kernel,
        .pic = true,
    });

    kernel.root_module.addImport("limine", limine_zig.module("limine"));
    kernel.setLinkerScriptPath(b.path("kernel/linker.ld"));

    // Disable LTO. This prevents issues with limine requests
    kernel.want_lto = false;

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

    const xorriso = b.addSystemCommand(&.{"xorriso"});
    xorriso.addArgs(&.{
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

    const iso = b.addInstallFileWithDir(iso_path, .prefix, "reason-os.iso");

    const compile_kernel = b.step("kernel", "Compile the kernel");
    compile_kernel.dependOn(&kernel.step);

    const create_iso = b.step("iso", "create the iso");
    create_iso.dependOn(&iso.step);
}
