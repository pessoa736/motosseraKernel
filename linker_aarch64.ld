ENTRY(_start)

PHDRS
{
    text   PT_LOAD FLAGS(5);   /* R+X */
    rodata PT_LOAD FLAGS(4);   /* R   */
    data   PT_LOAD FLAGS(6);   /* R+W */
}

SECTIONS
{
    . = 0xffffffff80000000;

    .text : ALIGN(4096)
    {
        *(.multboot2*)   /* herança do multiboot, pode remover se não usa mais */
        *(.text*)
    } :text

    .rodata : ALIGN(4096)
    {
        *(.rodata*)
    } :rodata

    .data : ALIGN(4096)
    {
        *(.data*)
        *(.got*)
    } :data

    /* Os requests precisam ser graváveis: o Limine escreve as respostas neles */
    .requests : ALIGN(4096)
    {
        KEEP(*(.requests))
    } :data

    .bss : ALIGN(4096)
    {
        *(COMMON)
        *(.bss*)
    } :data

    /DISCARD/ : { *(.eh_frame*) *(.note .note.*) }
}