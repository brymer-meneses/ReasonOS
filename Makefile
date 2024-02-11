
MODE := default
PROFILE := debug

QEMUFLAGS := \
	-serial stdio \
	-no-reboot \
	-display none \
	-D qemu-log.txt \
	-d int -M smm=off \
	-device isa-debug-exit,iobase=0xf4,iosize=0x0f

CARGO_FLAGS := \
	--manifest-path=reason/Cargo.toml \
	--target=configs/x86_64/reason-os.json \
	--target-dir=build/target \
	-Z build-std=core,compiler_builtins,alloc \
	-Z build-std-features=compiler-builtins-mem \

ifeq ($(PROFILE), debug) 
	CARGO_FLAGS += --profile dev
else ifeq ($(PROFILE), release)
	CARGO_FLAGS += --profile release
else
	$(error Invalid argument $(PROFILE) for `PROFILE`. Must be either `release` or `debug`.)
endif

ifeq ($(MODE), test) 
	CARGO_FLAGS += --tests
else ifneq ($(MODE), default)
	$(error Invalid argument $(MODE) for `MODE`. Must be either `test` or `default`.)
endif

setup:
	@mkdir -p build/bin
ifeq ($(wildcard build/limine/.),)
	git clone https://github.com/limine-bootloader/limine.git --branch=v6.x-branch-binary --depth=1 build/limine
	@$(MAKE) -C build/limine
endif

check:
	@cargo check $(CARGO_FLAGS)

kernel: check setup
	@export KERNEL_EXECUTABLE=$$(cargo build $(CARGO_FLAGS) --message-format=json | jq -r 'select(.executable) | .executable'); \
	cp $$KERNEL_EXECUTABLE build/bin/kernel

run: iso 
	@qemu-system-x86_64 $(QEMUFLAGS) -cdrom build/reason.iso

iso: setup kernel
	@rm -rf build/iso_root
	@mkdir -p build/iso_root
	@cp -v build/bin/kernel \
		configs/limine.cfg build/limine/limine-bios.sys  \
		build/limine/limine-bios-cd.bin build/limine/limine-uefi-cd.bin \
		build/iso_root/
	mkdir -p build/iso_root/EFI/BOOT
	@cp -v build/limine/BOOTX64.EFI build/iso_root/EFI/BOOT/
	@cp -v build/limine/BOOTIA32.EFI build/iso_root/EFI/BOOT/
	@xorriso -as mkisofs -b limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		build/iso_root -o build/reason.iso
	@./build/limine/limine bios-install build/reason.iso

clean:
	$(RM) -rf build
