OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start32)

SECTIONS
{   
    /* VGA buffer location */
    vga_buffer = 0xB8000;

    /* bootloader code should be loaded at 1M */
    . = 1M;
    
    .bootcode ALIGN(4K) :
    {   
        KEEP(*(.multiboot2))
        *(.boot32)
        *(.boot32.rodata)
    }

    .bootmem ALIGN(4K) :
    {
        /* reserve page tables */
        page_tbl_pml4 = .; . += 4K;      /*  */
        page_tbl_pdp_low = .; . += 4K;   /* 0-2 GiB identity mapping */
        page_tbl_pdp_high = .; . += 4K;  /* mapping highest 2 GiB virtual mem to lowest 2 GiB physical */
        page_tbl_pd_1 = .; . += 4K;      /* mapping first physical GiB */
        page_tbl_pd_2 = .; . += 4K;      /* mapping second physical GiB */
        /* reserve 8K of stack space */
        stack_start = .;
        . += 8K;
        stack_end = .;
    }

    /* Kernel starts at this virtual address. Physically, it's put right behind the bootcode */

    kernel_virtual_base = 0xFFFFFFFF80000000;
    
    . += kernel_virtual_base;

    kernel_start = . - kernel_virtual_base;

    .text ALIGN(4K) : AT(ADDR(.text) - kernel_virtual_base)
    {   
        *(.text*)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - kernel_virtual_base)
    {
        *(.rodata*)
    }

    .data ALIGN(4K) :  AT(ADDR(.data) - kernel_virtual_base)
    {
        *(.data*)
    }

    .bss ALIGN(4K) :  AT(ADDR(.bss) - kernel_virtual_base)
    {
        *(.bss*)
    }

    kernel_end = . - kernel_virtual_base;
}