DEBUG ?= 0
ifeq ($(DEBUG), 1)
	CONFIG := debug
else
	CONFIG := release
endif

# Toolset

QEMU := qemu-system-x86_64

LD := ld
LDFLAGS := -z max-page-size=0x1000 --whole-archive

CARGO := cargo
CARGOFLAGS := --target x86_64-learnos
ifeq ($(DEBUG), 0)
	CARGOFLAGS += --release
endif

# Build inputs
GRUB_CFG := ./image/grub.cfg
LDSCRIPT := learnos_multiboot2/linker.ld

# Build artifacts

ROOT_BUILD_DIR := ./build
BUILD_DIR := $(ROOT_BUILD_DIR)/$(CONFIG)
MULTIBOOT_NAME := learnos_multiboot2

BOOT_ISO := $(BUILD_DIR)/boot.iso
MULTIBOOT_BIN := $(BUILD_DIR)/$(MULTIBOOT_NAME)
MULTIBOOT_LIB := ./target/x86_64-learnos/$(CONFIG)/lib$(MULTIBOOT_NAME).a

build: $(BOOT_ISO)

run: build
	qemu-system-x86_64 -cdrom $(BOOT_ISO)

rungdb: build
	qemu-system-x86_64 -cdrom $(BOOT_ISO) -gdb tcp::9000

clean:
	rm -rf $(ROOT_BUILD_DIR)
	cargo clean

$(BOOT_ISO): $(MULTIBOOT_BIN) $(GRUB_CFG)
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp $(GRUB_CFG) $(BUILD_DIR)/iso/boot/grub/grub.cfg
	cp $(MULTIBOOT_BIN) $(BUILD_DIR)/iso/boot/$(MULTIBOOT_NAME)
	grub-mkrescue -o $(BOOT_ISO) $(BUILD_DIR)/iso

$(MULTIBOOT_BIN): $(MULTIBOOT_LIB) $(LDSCRIPT)
	mkdir -p $(BUILD_DIR)
	ld $(LDFLAGS) -T $(LDSCRIPT) -o $(MULTIBOOT_BIN) $(MULTIBOOT_LIB)

$(MULTIBOOT_LIB):
	$(CARGO) xbuild $(CARGOFLAGS)

.PHONY: run rungdb build clean $(MULTIBOOT_LIB)
