%include "src/bootcode/boot.inc"

section .boot32
align 4
bits 32
global _start32

extern vga_buffer
extern stack_end
extern kernel_start
extern kernel_end
extern page_tbl_pml4
extern page_tbl_pdp_low
extern page_tbl_pdp_high
extern page_tbl_pd_1
extern page_tbl_pdp_direct
extern kernel_virtual_base

extern kernel_main

; this is the multiboot entry point, we are in 32 bit protected mode now
_start32:
    ; setup temporary stack
    mov esp, stack_end
    mov ebp, esp
    ; save multiboot info table, we need it later
    push ebx

    check_multiboot fail_no_multiboot
    check_cpuid fail_no_cpuid
    check_long_mode fail_no_long_mode
    check_2mb_pages fail_no_2mb_pages

    ; TODO: future-proof this by checking for 5th page level

    call setup_page_tables
    
    enable_long_mode_feature
    enable_pae
    load_page_table page_tbl_pml4
    enable_paging
    
    ; inform that we're done and jump to long mode
    mov esi, msg_ok
    call print32
    ; Restore multiboot pointer into EDI, passing it as the first argument to kernel_main
    ; This is because `kernel_main` expects System V AMD64 ABI calling convention
    pop ebx
    lgdt [gdt_data.pointer]
    jmp gdt_data.kernel_code:kernel_main_trampoline

; jump to the trampoline, otherwise, we could not make the large jump from lowest 2 GiB to highest 2 GiB
bits 64
kernel_main_trampoline:
    ; Set data segments (didn't seem to be necessary in QEMU though)
    mov ax, GDT64.Data
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    ; move stack pointer to higher half, still pointing to the same physical memory
    add rsp, kernel_virtual_base
    mov rbp, rsp
    ; write KernelArgs structure
    sub rsp, 32
    mov qword [rsp], kernel_start
    mov qword [rsp+8], kernel_end
    mov qword [rsp+16], rbx
    mov eax, dword [rbx] ; read length field of multiboot header
    add rax, rbx
    mov qword [rsp+24], rax
    ; pass pointer to that structure to the kernel
    mov rdi, rsp
    ; reset all other registers
    xor rax, rax
    xor rbx, rbx
    xor rcx, rcx
    xor rdx, rdx
    xor rsi, rsi
    xor r8, r8
    xor r9, r9
    xor r10, r10
    xor r11, r11
    xor r12, r12
    xor r13, r13
    xor r14, r14
    xor r15, r15
    jmp kernel_main
bits 32


setup_page_tables:
    ; clear page tables (1024 * 4 bytes)
    clear4 page_tbl_pml4, 1024
    clear4 page_tbl_pdp_low, 1024
    clear4 page_tbl_pd_1, 1024
    clear4 page_tbl_pdp_direct, 1024

    ; PML4[0] -> page_tbl_pdp_low
    mov eax, page_tbl_pdp_low
    or  eax, 0b11
    mov DWORD [page_tbl_pml4], eax
    ; PML4[256] -> page_tbl_pdp_direct
    mov eax, page_tbl_pdp_direct
    or  eax, 0b11
    mov DWORD [page_tbl_pml4 + 256 * 8], eax
    ; PML4[510] -> page_tbl_pml4
    mov eax, page_tbl_pml4
    or  eax, 0b11
    mov DWORD [page_tbl_pml4 + 510 * 8], eax
    ; PML4[511] -> page_tbl_pdp_high
    mov eax, page_tbl_pdp_high
    or  eax, 0b11
    mov DWORD [page_tbl_pml4 + 511 * 8], eax

    ; page_tbl_pdp_low[0] -> page_tbl_pd_1
    mov eax, page_tbl_pd_1
    or  eax, 0b11
    mov DWORD [page_tbl_pdp_low], eax

    ; page_tbl_pdp_high[510] -> page_tbl_pd_1
    mov eax, page_tbl_pd_1
    or  eax, 0b11
    mov DWORD [page_tbl_pdp_high + 510 * 8], eax

    ; map 2 MiB pages in 1st PD to first GiB of physical memory
    mov ecx, 512
    mov edi, page_tbl_pd_1
    mov eax, (1 << 7) | 3 ; huge page (7), writable (1), present (0)
    .next_pd_1:
        stosd                    ; write lower DWORD of PD entry
        add edi, 4               ; skip higher DWORD of PD entry
        add eax, 2 * 1024 * 1024 ; advance physical address by 2 MiB
        loop .next_pd_1
    
    check_1gb_pages .no_1gb_pages
    ; map 1 GiB pages in direct mapping
    mov ecx, 512
    mov edi, page_tbl_pdp_direct
    mov eax, (1 << 7) | 3 ; huge page (7), writable (1), present (0)
    .next_pdp_direct:
        stosd                       ; write lower DWORD of PD entry
        add edi, 4                  ; skip higher DWORD of PD entry
        add eax, 1024 * 1024 * 1024 ; advance physical address by 1 GiB
        loop .next_pdp_direct
    jmp .pdp_direct_done
    ; as a fallback, map the 1st GB in page_tbl_pdp_direct
    .no_1gb_pages:
    mov eax, page_tbl_pd_1
    or  eax, 0b11
    mov DWORD [page_tbl_pdp_direct], eax
    .pdp_direct_done:
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
fail no_2mb_pages

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
msg_no_2mb_pages:
    db 'Huge pages are not available', 0

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
    db 10011010b                 ; Access. Pr=1, Prvl=0, 1, Ex=1, DC=0, RW=1, Ac=0
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

; see section 4.8.1, page 88, of AMD64 Architecture Programmer's Manual, Volume 2 System Programming
gdt_data:
    .null: equ $ - gdt_data
        dd 0x0
        dd 0x0
    .kernel_code: equ $ - gdt_data
        dd 0x0
        dd 0x00209800
    .kernel_data: equ $ - gdt_data
        dd 0x0
        dd 0x00209200
    .user_code: equ $ - gdt_data
        dd 0x0
        dd 0x0020F800
    .user_data: equ $ - gdt_data
        dd 0x0
        dd 0x0020F200
    .pointer:
        dw $ - gdt_data - 1
        dq gdt_data