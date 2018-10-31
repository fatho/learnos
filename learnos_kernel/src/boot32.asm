%include "src/boot32.inc"

section .boot32
align 4
bits 32
global _start32

extern vga_buffer
extern stack_top
extern page_tbl_pml4
extern page_tbl_pdp_low
extern page_tbl_pdp_high
extern page_tbl_pd_1
extern page_tbl_pd_2

extern rust_main

; this is the multiboot entry point, we are in 32 bit protected mode now
_start32:
    ; setup temporary stack
    mov esp, stack_top
    mov ebp, esp
    ; save multiboot info table, we need it later
    push ebx

    check_multiboot fail_no_multiboot
    check_cpuid fail_no_cpuid
    check_long_mode fail_no_long_mode

    ; TODO: check availability of 2 MB pages
    ; TODO: future-proof this by checking for 5th page level

    call setup_page_tables
    
    enable_long_mode_feature
    enable_pae
    load_page_table page_tbl_pml4
    enable_paging
    
    ; inform that we're done and jump to long mode
    mov esi, msg_ok
    call print32
    ; Restore multiboot pointer into EDI, passing it as the first argument to rust_main
    ; This is because `rust_main` expects System V AMD64 ABI calling convention
    pop edi
    jump_to_64 rust_main


setup_page_tables:
    ; clear page tables (1024 * 4 bytes)
    clear4 page_tbl_pml4, 1024
    clear4 page_tbl_pdp_low, 1024
    clear4 page_tbl_pd_1, 1024

    ; PML4[0] -> page_tbl_pdp_low
    mov   eax, page_tbl_pdp_low
    or    eax, 0b11
    mov   DWORD [page_tbl_pml4], eax
    ; PML4[511] -> page_tbl_pdp_high
    mov   eax, page_tbl_pdp_high
    or    eax, 0b11
    mov   DWORD [page_tbl_pml4 + 511 * 8], eax

    ; page_tbl_pdp_low[0] -> page_tbl_pd_1
    mov   eax, page_tbl_pd_1
    or    eax, 0b11
    mov   DWORD [page_tbl_pdp_low], eax
    ; page_tbl_pdp_low[1] -> page_tbl_pd_2
    mov   eax, page_tbl_pd_2
    or    eax, 0b11
    mov   DWORD [page_tbl_pdp_low+8], eax

    ; page_tbl_pdp_high[510] -> page_tbl_pd_1
    mov   eax, page_tbl_pd_1
    or    eax, 0b11
    mov   DWORD [page_tbl_pdp_high + 510 * 8], eax
    ; page_tbl_pdp_high[511] -> page_tbl_pd_2
    mov   eax, page_tbl_pd_2
    or    eax, 0b11
    mov   DWORD [page_tbl_pdp_high + 511 * 8], eax

    ; map 2 MiB pages in 1st PD to first GiB of physical memory
    mov ecx, 512
    mov edi, page_tbl_pd_1
    mov eax, (1 << 7) | 3 ; huge page (7), writable (1), present (0)
    .next_pd_1:
        stosd                    ; write lower DWORD of PD entry
        add edi, 4               ; skip higher DWORD of PD entry
        add eax, 2 * 1024 * 1024 ; advance physical address by 2 MiB
        loop .next_pd_1
    ; map 2 MiB pages in 2nd PD to second GiB of physical memory
    mov ecx, 512
    mov edi, page_tbl_pd_2
    .next_pd_2:
        stosd                    ; write lower DWORD of PD entry
        add edi, 4               ; skip higher DWORD of PD entry
        add eax, 2 * 1024 * 1024 ; advance physical address by 2 MiB
        loop .next_pd_2
    ret


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; MACROS
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

%macro fail 1
fail_%1:
    mov esi, msg_%1
    call print32
    jmp halt32
%endmacro

fail no_cpuid
fail no_multiboot
fail no_long_mode

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; UTILITY FUNCTIONS
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

print32:
    ; Print the status message pointed to by ESI to the VGA buffer
    ; clobbers EAX, EDI
    mov ah, 0X0F  ; setup color
    mov edi, vga_buffer  ; setup destination
print32_loop:
    lodsb         ; load [ESI] into AL
    cmp al, 0     ; check for 0 terminator
    je print32_done
    stosw         ; write AX to [EDI]
    jmp print32_loop
print32_done:
    ret

halt32:
    ; stop doing anything useful
    hlt
    jmp halt32


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; DATA
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

section .boot32.rodata

msg_ok:
    db 'Entering long mode', 0
msg_no_multiboot:
    db 'Kernel was not booted by multiboot compliant bootloader', 0
msg_no_cpuid:
    db 'CPUID instruction is not available', 0
msg_no_long_mode:
    db 'Long mode is not available', 0

GDT64:                           ; Global Descriptor Table (64-bit).
    .Null: equ $ - GDT64         ; The null descriptor.
    dw 0xFFFF                    ; Limit (low).
    dw 0                         ; Base (low).
    db 0                         ; Base (middle)
    db 0                         ; Access.
    db 1                         ; Granularity.
    db 0                         ; Base (high).
    .Code: equ $ - GDT64         ; The code descriptor.
    dw 0                         ; Limit (low).
    dw 0                         ; Base (low).
    db 0                         ; Base (middle)
    db 10011010b                 ; Access (exec/read).
    db 10101111b                 ; Granularity, 64 bits flag, limit19:16.
    db 0                         ; Base (high).
    .Data: equ $ - GDT64         ; The data descriptor.
    dw 0                         ; Limit (low).
    dw 0                         ; Base (low).
    db 0                         ; Base (middle)
    db 10010010b                 ; Access (read/write).
    db 00000000b                 ; Granularity.
    db 0                         ; Base (high).
    .Pointer:                    ; The GDT-pointer.
    dw $ - GDT64 - 1             ; Limit.
    dq GDT64                     ; Base.