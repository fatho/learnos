CONFIG := release

QEMU := qemu-system-x86_64

LD := ld
LDFLAGS := -z max-page-size=0x1000 --whole-archive
LDSCRIPT := learnos_multiboot2/linker.ld

BUILD_DIR := ./build
BOOT_ISO := ./build/boot.iso

GRUB_CFG := ./image/grub.cfg

MULTIBOOT_NAME := learnos_multiboot2

MULTIBOOT_BIN := $(BUILD_DIR)/$(MULTIBOOT_NAME)
MULTIBOOT_LIB := ./target/x86_64-learnos/$(CONFIG)/lib$(MULTIBOOT_NAME).a
MULTIBOOT_LIB_DEBUG := ./target/x86_64-learnos/debug/lib$(MULTIBOOT_NAME).a
MULTIBOOT_LIB_RELEASE := ./target/x86_64-learnos/release/lib$(MULTIBOOT_NAME).a

build: $(BOOT_ISO)

run: build
	qemu-system-x86_64 -cdrom $(BOOT_ISO)

clean:
	rm -rf $(BUILD_DIR)
	cargo clean

$(BOOT_ISO): $(MULTIBOOT_BIN) $(GRUB_CFG)
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp $(GRUB_CFG) $(BUILD_DIR)/iso/boot/grub/grub.cfg
	cp $(MULTIBOOT_BIN) $(BUILD_DIR)/iso/boot/$(MULTIBOOT_NAME)
	grub-mkrescue -o $(BOOT_ISO) $(BUILD_DIR)/iso

$(MULTIBOOT_BIN): $(MULTIBOOT_LIB) $(LDSCRIPT)
	mkdir -p $(BUILD_DIR)
	ld $(LDFLAGS) -T $(LDSCRIPT) -o $(MULTIBOOT_BIN) $(MULTIBOOT_LIB)

$(MULTIBOOT_LIB_DEBUG):
	cargo xbuild --target x86_64-learnos

$(MULTIBOOT_LIB_RELEASE):
	cargo xbuild --target x86_64-learnos --release

.PHONY: run build clean $(MULTIBOOT_LIB_DEBUG) $(MULTIBOOT_LIB_RELEASE)
