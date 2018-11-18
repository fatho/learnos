OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start32)

SECTIONS
{
    /* VGA buffer location */
    vga_buffer = 0xB8000;

    . = 4K;

    .bootcode ALIGN(4K) :
    {   
        KEEP(*(.multiboot2))
        *(.boot32)
        *(.boot32.rodata)
    }

    /* This memory is allocated at boot-time, but the structures allocated
       here should be replaced when the kernel is loaded  */
    .bootmem ALIGN(4K) :
    {
        /* reserve page tables */
        page_tbl_pml4 = .; . += 4K;      /*  */
        page_tbl_pdp_low = .; . += 4K;   /* 0-2 GiB identity mapping */
        page_tbl_pdp_direct = .; . += 4K;  /* mapping -1.5TB to -1TB virtual to 0B to 512GB physical */
        page_tbl_pdp_high = .; . += 4K;  /* mapping -2GB to 0 physical */
        page_tbl_pd_1 = .; . += 4K;      /* mapping 1st physical GiB */
        page_tbl_pd_2 = .; . += 4K;      /* mapping 2nd physical GiB */
        page_tbl_pd_3 = .; . += 4K;      /* mapping 3rd physical GiB */
        page_tbl_pd_4 = .; . += 4K;      /* mapping 4th physical GiB */
        /* reserve 8K of stack space */
        stack_start = .;
        . += 8K;
        stack_end = .;
    }

    /* Kernel starts at this virtual address.  */
    /* Physically, it should be loaded at 1M */
    . = 1M;    

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