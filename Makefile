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

ifeq ($(OS),macos)
	AS=i386-elf-as
	LD=i386-elf-ld
else ifeq (($OS),linux)
	AS=as
	LD=ld
endif

MULTIBOOT_HEADER := src/arch/x86/boot.s
MULTIBOOT_HEADER_OBJ := boot.o
GDT := src/arch/x86/gdt.s
GDT_OBJ := gdt.o

LIB := target/i386-unknown-none/release/libkfs.a

RUST_SRCS := $(shell find $(SRC_DIR) -type f -name "*.rs")
CARGO_TOML := Cargo.toml

all: $(BUILD_DIR)/$(BINARY)

$(BUILD_DIR)/$(BINARY): $(BUILD_DIR)/$(MULTIBOOT_HEADER_OBJ) $(BUILD_DIR)/$(GDT_OBJ) $(LIB)
	$(LD) -m elf_i386 -T src/arch/x86/linker.ld -o $@ $^

$(BUILD_DIR)/$(MULTIBOOT_HEADER_OBJ): $(MULTIBOOT_HEADER) | $(BUILD_DIR)
	$(AS) --32 -o $@ $<

$(BUILD_DIR)/$(GDT_OBJ): $(GDT) | $(BUILD_DIR)
	$(AS) --32 -o $@ $<

$(LIB): $(RUST_SRCS) $(CARGO_TOML) $(MULTIBOOT_HEADER)
	cargo build-kernel
	touch $(LIB)

$(BUILD_DIR):
	mkdir -p $@

iso: all
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp grub/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso --compress=xz --locale-directory=/dev/null --fonts=ascii

run: iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -device isa-debug-exit,iobase=0xf4,iosize=0x04

debug-iso: all
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp grub/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso

debug: debug-iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -curses -device isa-debug-exit,iobase=0xf4,iosize=0x04

crash: debug-iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -d int -no-reboot -no-shutdown

test: all
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp grub/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso
	qemu-system-i386 -s -S -kernel build/kernel.bin -append "root=/dev/hda"

fclean:
	cargo clean
	$(RM) -rf $(BUILD_DIR)
	$(RM) -f $(GDT_OBJ)

re: fclean all

.PHONY: all run re fclean iso debug
