
ifeq ($(RUST_PROFILE),)
    override RUST_PROFILE := dev
endif

override RUST_PROFILE_SUBDIR := $(RUST_PROFILE)
ifeq ($(RUST_PROFILE),dev)
    override RUST_PROFILE_SUBDIR := debug
endif

all:
	cargo build --target x86_64-unknown-none --profile $(RUST_PROFILE) --target-dir=../build/reason
	cp ../build/reason/x86_64-unknown-none/$(RUST_PROFILE_SUBDIR)/reason ../build/kernel

clean:
	cargo clean
	rm -rf kernel
