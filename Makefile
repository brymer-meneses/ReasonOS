
QEMUFLAGS := \
	-serial stdio \
	-no-reboot \
	-display none \
	-D qemu-log.txt \
	-d int -M smm=off \

all: run

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v6.x-branch-binary --depth=1 build/limine || true
	$(MAKE) -C build/limine

kernel:
	$(MAKE) -C reason RUST_PROFILE=release

iso: limine kernel
	rm -rf build/iso_root
	mkdir -p build/iso_root
	cp -v build/kernel \
		limine.cfg build/limine/limine-bios.sys  \
		build/limine/limine-bios-cd.bin build/limine/limine-uefi-cd.bin \
		build/iso_root/
	mkdir -p build/iso_root/EFI/BOOT
	cp -v build/limine/BOOTX64.EFI build/iso_root/EFI/BOOT/
	cp -v build/limine/BOOTIA32.EFI build/iso_root/EFI/BOOT/
	xorriso -as mkisofs -b limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		build/iso_root -o build/reason.iso
	./build/limine/limine bios-install build/reason.iso
	rm -rf build/iso_root


run: iso
	qemu-system-x86_64 $(QEMUFLAGS) -cdrom build/reason.iso

clean:
	rm -rf build
