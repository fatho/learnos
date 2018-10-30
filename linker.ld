OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{   
    . = 4K;
    
    .boot ALIGN(4K) :
    {   
        *(.multiboot2)
        *(.boot32)
    }

    . += 1M;
    STACK_TOP = .;
}
