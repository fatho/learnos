section .multiboot2
align 4

MB2_MAGIC        EQU 0xE85250D6  ; Multiboot2 Magic Value
MB2_ARCHITECTURE EQU 0           ; x86 protected mode
MB2_HEADER_LEN   EQU Multiboot2HeaderEnd - Multiboot2Header
MB2_CHECKSUM     EQU -(MB2_MAGIC + MB2_ARCHITECTURE + MB2_HEADER_LEN)

MB2_TAG_FLAG_OPTIONAL  EQU 0x0001  ; marks an optional tag

Multiboot2Header:
   dd MB2_MAGIC
   dd MB2_ARCHITECTURE
   dd MB2_HEADER_LEN
   dd MB2_CHECKSUM
   ; module alignment tag
   dw 6  ; type
   dw 0  ; flags
   dd 8  ; size
   ; end of header tag
   dw 0  ; type
   dw 0  ; flags
   dd 8  ; size

Multiboot2HeaderEnd: