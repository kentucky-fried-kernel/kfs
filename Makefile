SRC_DIR := src
OBJ_DIR := obj
BUILD_DIR := build
NAME := kernel
BINARY := $(NAME).bin
ISO := $(NAME).iso
MULTIBOOT_HEADER := assets/boot.s
MULTIBOOT_HEADER_OBJ := boot.o

LIB := target/i386-unknown-none/release/libkfs.a

RUST_SRCS := $(shell find $(SRC_DIR) -type f -name "*.rs")
CARGO_TOML := Cargo.toml

all: $(BUILD_DIR)/$(BINARY)

$(BUILD_DIR)/$(BINARY): $(BUILD_DIR)/$(MULTIBOOT_HEADER_OBJ) $(LIB)
	ld -m elf_i386 -T assets/linker.ld -o $@ $^

$(BUILD_DIR)/$(MULTIBOOT_HEADER_OBJ): $(MULTIBOOT_HEADER) | $(BUILD_DIR)
	as --32 -o $@ $<

$(LIB): $(RUST_SRCS) $(CARGO_TOML) $(MULTIBOOT_HEADER)
	cargo build-kernel
	touch $(LIB)

$(BUILD_DIR):
	mkdir -p $@

iso: all 
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp assets/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso --compress=xz --locale-directory=/dev/null --fonts=ascii

run: iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d

debug-iso: all
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp assets/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso

debug: debug-iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -curses

crash: debug-iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -d int -no-reboot -no-shutdown

test: all 
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp assets/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso
	qemu-system-i386 -s -S -kernel build/kernel.bin -append "root=/dev/hda"

fclean:
	cargo clean
	$(RM) -rf $(BUILD_DIR)

re: fclean all

.PHONY: all run re fclean iso debug
