DEBUG ?= 0
ifeq ($(DEBUG), 1)
	CONFIG := debug
else
	CONFIG := release
endif

# Toolset

QEMU := qemu-system-x86_64
QEMUFLAGS := -serial stdio -smp cores=2 

LD := ld
LDFLAGS := -z max-page-size=0x1000 --whole-archive

CARGO := cargo
CARGOFLAGS := --target x86_64-learnos

# Special flags depending on debug mode

ifeq ($(DEBUG), 0)
	CARGOFLAGS += --release
else
	QEMUFLAGS += -gdb tcp::9000 -S
endif

# Build inputs
GRUB_CFG := ./image/grub.cfg
TEST_MODULE := ./image/test-module.txt
LDSCRIPT := learnos_kernel/linker.ld

# Build artifacts

ROOT_BUILD_DIR := ./build
BUILD_DIR := $(ROOT_BUILD_DIR)/$(CONFIG)
MULTIBOOT_NAME := learnos_kernel

BOOT_ISO := $(BUILD_DIR)/boot.iso
MULTIBOOT_BIN := $(BUILD_DIR)/$(MULTIBOOT_NAME)
MULTIBOOT_LIB := ./target/x86_64-learnos/$(CONFIG)/lib$(MULTIBOOT_NAME).a


build: $(BOOT_ISO)

run: build
	qemu-system-x86_64 -cdrom $(BOOT_ISO) $(QEMUFLAGS)

test:
	cargo test

clean:
	rm -rf $(ROOT_BUILD_DIR)
	cargo clean

$(BOOT_ISO): $(MULTIBOOT_BIN) $(GRUB_CFG) $(TEST_MODULE)
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp $(GRUB_CFG) $(BUILD_DIR)/iso/boot/grub/grub.cfg
	cp $(MULTIBOOT_BIN) $(BUILD_DIR)/iso/boot/$(MULTIBOOT_NAME)
	cp $(TEST_MODULE) $(BUILD_DIR)/iso/boot/test-module.txt
	grub-mkrescue -o $(BOOT_ISO) $(BUILD_DIR)/iso

$(MULTIBOOT_BIN): $(MULTIBOOT_LIB) $(LDSCRIPT)
	mkdir -p $(BUILD_DIR)
	ld $(LDFLAGS) -T $(LDSCRIPT) -o $(MULTIBOOT_BIN) $(MULTIBOOT_LIB)

$(MULTIBOOT_LIB):
	RUST_TARGET_PATH="$(CURDIR)" RUSTFLAGS="-C code-model=kernel" $(CARGO) xbuild $(CARGOFLAGS)

.PHONY: run test build clean $(MULTIBOOT_LIB)
