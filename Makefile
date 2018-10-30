.PHONY: build clean

AS	:= nasm
ASFLAGS	:= -f elf64

LD := ld
LDFLAGS	:= -z max-page-size=0x1000

LINK_FILE := linker.ld
TARGET_NAME := boot.elf
ISO_NAME := boot.iso

BUILD_DIR := ./build

BOOTLOADER_SRC_DIR := ./bootloader
BOOTLOADER_BUILD_DIR := $(BUILD_DIR)/bootloader

BOOTLOADER_SRC_LIST := $(wildcard $(BOOTLOADER_SRC_DIR)/*.asm)
BOOTLOADER_OBJ_LIST := $(patsubst $(BOOTLOADER_SRC_DIR)/%.asm,$(BOOTLOADER_BUILD_DIR)/%.asm.o,$(BOOTLOADER_SRC_LIST))

build: $(BUILD_DIR)/$(ISO_NAME)

clean:
	rm -rf $(BUILD_DIR)

$(BUILD_DIR)/$(ISO_NAME): $(BUILD_DIR)/$(TARGET_NAME) $(BOOTLOADER_SRC_DIR)/grub.cfg
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp $(BOOTLOADER_SRC_DIR)/grub.cfg $(BUILD_DIR)/iso/boot/grub/grub.cfg
	cp $(BUILD_DIR)/$(TARGET_NAME) $(BUILD_DIR)/iso/boot/$(TARGET_NAME)
	grub-mkrescue -o $(BUILD_DIR)/$(ISO_NAME) $(BUILD_DIR)/iso

$(BUILD_DIR)/$(TARGET_NAME): $(BOOTLOADER_OBJ_LIST) $(LINK_FILE)
	echo $(BOOTLOADER_SRC_LIST)
	echo $(BOOTLOADER_OBJ_LIST)
	mkdir -p $(@D)
	$(LD) $(LDFLAGS) -T $(LINK_FILE) -o $(BUILD_DIR)/$(TARGET_NAME) $(BOOTLOADER_OBJ_LIST)


$(BOOTLOADER_BUILD_DIR)/%.asm.o: $(BOOTLOADER_SRC_DIR)/%.asm
	mkdir -p $(@D)
	$(AS) $(ASFLAGS) -i$(BOOTLOADER_SRC_DIR)/ $< -o $@
