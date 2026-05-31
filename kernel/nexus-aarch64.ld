/* NexusOS AArch64 Kernel Linker Script (Bahamut)
 * Higher-half kernel: virtual base 0xffffffff80000000
 * Compatible with Limine boot protocol on AArch64
 */

OUTPUT_FORMAT(elf64-littleaarch64)
OUTPUT_ARCH(aarch64)
ENTRY(_start)

PHDRS {
    text    PT_LOAD FLAGS(5);   /* R-X */
    rodata  PT_LOAD FLAGS(4);   /* R-- */
    data    PT_LOAD FLAGS(6);   /* RW- */
}

SECTIONS {
    . = 0xffffffff80000000;

    /* Exception vectors MUST be 2048-byte aligned — place first in .text */
    .text ALIGN(4096) : {
        KEEP(*(.exception_vectors))
        *(.text .text.*)
    } :text

    . = ALIGN(4096);
    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .limine_requests ALIGN(8) : {
        KEEP(*(.limine_requests_start))
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests_end))
    } :rodata

    . = ALIGN(4096);
    .data : {
        *(.data .data.*)
    } :data

    .bss ALIGN(4096) : {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note .note.*)
        *(.gnu.hash)
    }
}
