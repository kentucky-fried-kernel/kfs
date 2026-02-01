SRC_DIR := src
BUILD_DIR := build
NAME := kernel
BINARY := $(NAME).bin
ISO := $(NAME).iso

LD_SCRIPT := ./src/arch/x86/linker.ld
TARGET_CONFIG := ./src/arch/x86/i386-unknown-none.json

QEMU_FLAGS := -boot d -device isa-debug-exit,iobase=0xf4,iosize=0x04 -m 4G
DEBUG_QEMU_FLAGS := $(QEMU_FLAGS) -serial stdio

BIN := target/i386-unknown-none/release/kfs

RUST_SRCS := $(shell find $(SRC_DIR) -type f -name "*.rs")
CARGO_TOML := Cargo.toml

all: $(BUILD_DIR)/$(BINARY) $(LD_SCRIPT) $(TARGET_CONFIG)

$(BUILD_DIR)/$(BINARY): $(BIN)

$(BIN): $(RUST_SRCS) $(CARGO_TOML) $(MULTIBOOT_HEADER) $(LD_SCRIPT) $(TARGET_CONFIG)
	@cargo build --release -Zjson-target-spec
	@touch $(BIN)

$(BUILD_DIR):
	@mkdir -p $@

iso: all
	@mkdir -p $(BUILD_DIR)/iso/boot/grub
	@cp grub/grub.cfg $(BUILD_DIR)/iso/boot/grub
	@cp $(BIN) $(BUILD_DIR)/iso/boot/kernel.bin
	@grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso --compress=xz --locale-directory=/dev/null --fonts=ascii

run: iso
	@./scripts/run.sh $(BUILD_DIR)/$(NAME).iso $(QEMU_FLAGS)

debug-iso: all
	@mkdir -p $(BUILD_DIR)/iso/boot/grub
	@cp grub/grub.cfg $(BUILD_DIR)/iso/boot/grub
	@cp $(BIN) $(BUILD_DIR)/iso/boot/kernel.bin
	@grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso

debug: debug-iso
	./scripts/run.sh $(BUILD_DIR)/$(NAME).iso $(DEBUG_QEMU_FLAGS)

test:
	@LOGLEVEL=INFO ./x.py --end-to-end-tests
	@echo
	@LOGLEVEL=INFO ./x.py --unit-tests

debug-test:
	@LOGLEVEL=DEBUG ./x.py --end-to-end-tests
	@echo
	@LOGLEVEL=DEBUG ./x.py --unit-tests

fclean:
	@cargo clean
	@$(RM) -rf $(BUILD_DIR)
	@$(RM) -f $(GDT_OBJ)

clippy:
	cargo clippy -Zjson-target-spec -- --D warnings

re: fclean all

.PHONY: all run re fclean iso debug
