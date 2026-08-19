ENTRY(_start)

SECTIONS
{
    . = 0x400000;

    .text : ALIGN(4096)
    {
        *(.multboot2*)
        *(.text*)
    }
    
    .rodata : ALIGN(4096)
    {
        *(.rodata*)
    }

    .data : ALIGN(4096)
    {
        *(.data*)
        *(.got*)
    }

    /* Seção de requisições isolada e protegida pelo KEEP */
    .requests : ALIGN(4096)
    {
        KEEP(*(.requests))
    }
   
    .bss : ALIGN(4096)
    {
        *(COMMON)
        *(.bss*)
    }
}