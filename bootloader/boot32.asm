section .boot32
align 4
bits 32
global _start
extern STACK_TOP

VGA EQU 0xB8000  ; start of VGA buffer

STATUS_OK EQU 0x0F4F ; O

; this is the multiboot entry point, we are in 32 bit protected mode now
_start:
    ; setup temporary stack
    mov esp, STACK_TOP
    mov ebp, esp
    ; save multiboot info table, we need it later
    push ebx

    
    ; inform that we're done
    mov esi, status_ok
    call print32
    jmp halt



print32:
    ; Print the status message pointed to by ESI to the VGA buffer
    ; clobbers EAX, EDI
    mov ah, 0X0F  ; setup color
    mov edi, VGA  ; setup destination
print32_loop:
    lodsb         ; load [ESI] into AL
    cmp al, 0     ; check for 0 terminator
    je print32_done
    stosw         ; write AX to [EDI]
    jmp print32_loop
print32_done:
    ret


halt:
    ; stop doing anything useful
    hlt
    jmp halt


status_ok:
    db 'OK', 0