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
        page_tbl_pml4 = .; . += 4K;      /*  */
        page_tbl_pdp_low = .; . += 4K;   /* 0-2 GiB identity mapping */
        page_tbl_pdp_high = .; . += 4K;  /* mapping highest 2 GiB virtual mem to lowest 2 GiB physical */
        page_tbl_pd_1 = .; . += 4K;      /* mapping first physical GiB */
        page_tbl_pd_2 = .; . += 4K;      /* mapping second physical GiB */
        /* reserve 8K of stack space */
        stack_bottom = .;
        . += 8K;
        stack_top = .;
    }
}
