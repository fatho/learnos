OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{   
    /* VGA buffer location */
    vga_buffer = 0xB8000;

    /* Kernel code should be loaded at 1M */
    . = 1M;
    
    .boot ALIGN(4K) :
    {   
        KEEP(*(.multiboot2))
        *(.boot32)
        *(.boot32.rodata)
        *(.boot64)
        *(.boot64.rodata)
    }

    .bss ALIGN(4K) :
    {
        /* reserve 4K of stack space */
        . += 4K;
        stack_top = .;
        /* reserve page tables */
        . += 4K;
        page_id_pml4 = .;
        . += 4K;
        page_id_pdp = .;
        . += 4K;
        page_id_pd = .;
    }
}
