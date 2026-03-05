/* Userland linker script for 32-bit user processes */

ENTRY(_start)

SECTIONS
{
    /* User code starts at 0x08048000 (typical for 32-bit user space) */
    . = 0x08048000;
    
    .text : {
        *(.text*)
    }
    
    .data : {
        *(.data*)
    }
    
    .rodata : {
        *(.rodata*)
    }
    
    .bss : {
        *(.bss*)
        *(COMMON)
    }
    
    /DISCARD/ : {
        *(.note.GNU-stack)
        *(.gnu_debuglink)
        *(.gnu_debugdata)
    }
}
