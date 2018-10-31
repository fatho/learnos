OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start32)

SECTIONS
{   
    /* VGA buffer location */
    vga_buffer = 0xB8000;

    /* Kernel code should be loaded at 1M */
    . = 1M;
    
    .text ALIGN(4K) :
    {   
        KEEP(*(.multiboot2))
        *(.boot32)
        *(.text*)
    }

    .rodata ALIGN(4K) :
    {
        *(.boot32.rodata)
        *(.rodata*)
    }

    .data ALIGN(4K) :
    {
        *(.data*)
    }

    .bss ALIGN(4K) :
    {
        /* reserve page tables */
        . += 4K;
        page_id_pml4 = .;
        . += 4K;
        page_id_pdp = .;
        . += 4K;
        page_id_pd = .;
        /* reserve 8K of stack space */
        . += 8K;
        stack_top = .;
    }
}
