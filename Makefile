UNAME := $(shell uname -s)

ifeq ($(UNAME),Linux)
	OS := linux
else ifeq ($(UNAME),Darwin)
	OS := macos
else
	exit 1
endif

SRC_DIR := src
OBJ_DIR := obj
BUILD_DIR := build
NAME := kernel
BINARY := $(NAME).bin
ISO := $(NAME).iso

LD_SCRIPT := ./src/arch/x86/linker.ld
TARGET_CONFIG := ./src/arch/x86/i386-unknown-none.json


LIB := target/i386-unknown-none/release/libkfs.a

RUST_SRCS := $(shell find $(SRC_DIR) -type f -name "*.rs")
CARGO_TOML := Cargo.toml

all: $(BUILD_DIR)/$(BINARY)

$(BUILD_DIR)/$(BINARY): $(LIB)

$(LIB): $(RUST_SRCS) $(CARGO_TOML) $(MULTIBOOT_HEADER)
	cargo build --release
	touch $(LIB)

$(BUILD_DIR):
	mkdir -p $@

iso: all
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp grub/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp target/i386-unknown-none/release/kfs $(BUILD_DIR)/iso/boot/kernel.bin
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso --compress=xz --locale-directory=/dev/null --fonts=ascii

run: iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -d int -device isa-debug-exit,iobase=0xf4,iosize=0x04 -no-reboot -no-shutdown

debug-iso: all
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp grub/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp target/i386-unknown-none/release/kfs $(BUILD_DIR)/iso/boot/kernel.bin
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso

debug: debug-iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -d int -no-reboot -no-shutdown

test:
	@LOGLEVEL=INFO ./x.py --end-to-end-tests
	echo
	@LOGLEVEL=INFO ./x.py --unit-tests

debug-test:
	@LOGLEVEL=DEBUG ./x.py --end-to-end-tests
	echo
	@LOGLEVEL=DEBUG ./x.py --unit-tests

fclean:
	cargo clean
	$(RM) -rf $(BUILD_DIR)
	$(RM) -f $(GDT_OBJ)

re: fclean all

.PHONY: all run re fclean iso debug
