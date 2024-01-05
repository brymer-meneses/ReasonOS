
ifeq ($(RUST_PROFILE),)
    override RUST_PROFILE := dev
endif

override RUST_PROFILE_SUBDIR := $(RUST_PROFILE)
ifeq ($(RUST_PROFILE),dev)
    override RUST_PROFILE_SUBDIR := debug
endif

all:
	cargo build --profile release
	cp ../build/reason/x86_64-reason_os/$(RUST_PROFILE_SUBDIR)/reason ../build/kernel

clean:
	cargo clean
	rm -rf kernel
