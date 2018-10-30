section .boot64
align 4
bits 64

global _start64
extern vga_buffer

_start64:
    ; expects multiboot information table in EBX
    call cls64
    mov esi, msg_so_long
    call print64
    jmp halt64

print64:
    ; Print the status message pointed to by ESI to the VGA buffer
    ; clobbers EAX, EDI
    mov ah, 0X0F  ; setup color
    mov edi, vga_buffer  ; setup destination
print64_loop:
    lodsb         ; load [ESI] into AL
    cmp al, 0     ; check for 0 terminator
    je print64_done
    stosw         ; write AX to [EDI]
    jmp print64_loop
print64_done:
    ret

cls64:
    ; clears the VGA buffer
    mov ecx, 80 * 25
    mov ax, 0
    mov edi, vga_buffer
    rep stosw
    ret

halt64:
    ; stop doing anything useful
    hlt
    jmp halt64


section .boot64.rodata

msg_so_long:
    db 'So long!', 0