; this should be in EAX when being booted by a compliant bootloader
%define MULTIBOOT2_BOOTLOADER_MAGIC 0x36d76289

%macro check_long_mode 1
    ; ensure that functionality for checking if long mode is available is available
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb %1
    ; ensure that long mode is available
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz %1
%endmacro


%macro check_multiboot 1
    ; ensure we've been booted by a compliant bootloader
    cmp eax, MULTIBOOT2_BOOTLOADER_MAGIC
    jne %1
%endmacro


%macro check_cpuid 1
    ; ensure that CPUID is available
    pushfd            ; set EAX to FLAGS
    pop eax           ; 
    mov ecx, eax      ; save for later
    xor eax, 1 << 21  ; flip ID bit
    push eax          ; set FLAGS to EAX
    popfd             ; 
    pushfd            ; set EAX to FLAGS
    pop eax           ; 
    push ecx          ; restore FLAGS from ECX
    popfd             ;
    xor eax, ecx      ; check whether we managed to flip the bit
    jz %1
%endmacro


%macro enable_long_mode_feature 0
    ; enable LM (long mode) bit in EFER MSR
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr
%endmacro


%macro enable_paging 0
    ; enable PG (paging) bit in CR0
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
%endmacro


%macro enable_pae 0
    ; enable PAE (physical address extension) bit in CR4
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
%endmacro


%macro load_page_table 1
    mov eax, %1
    mov cr3, eax
%endmacro


%macro clear4 2
    mov eax, 0
    mov edi, %1
    mov ecx, %2
    rep stosd
%endmacro