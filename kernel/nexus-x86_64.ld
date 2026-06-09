/* NexusOS x86_64 Kernel Linker Script
 * Higher-half kernel: virtual base 0xffffffff80000000
 * Compatible with Limine boot protocol
 */

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS {
    text    PT_LOAD FLAGS(5);   /* R-X: code */
    rodata  PT_LOAD FLAGS(4);   /* R--: read-only data */
    data    PT_LOAD FLAGS(6);   /* RW-: writable data + BSS */
}

SECTIONS {
    /* Higher half: kernel lives at -2GB in virtual space */
    . = 0xffffffff80000000;

    /* ── Executable code ───────────────────────────────────────────────── */
    .text ALIGN(4096) : {
        *(.text .text.*)
    } :text

    /* ── Read-only data + Limine requests ─────────────────────────────── */
    . = ALIGN(4096);
    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    /* Limine boot protocol request records — must be in a loadable segment */
    .limine_requests ALIGN(8) : {
        KEEP(*(.limine_requests_start))
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests_end))
    } :rodata

    /* ── Writable data ─────────────────────────────────────────────────── */
    . = ALIGN(4096);
    .data : {
        *(.data .data.*)
    } :data

    /* Zero-initialised data — not stored in the ELF, zeroed by bootloader */
    .bss ALIGN(4096) : {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    /* End-of-kernel marker — used by the installer to calculate kernel size */
    PROVIDE(__kernel_end = .);

    /* ── Discard unwanted sections ─────────────────────────────────────── */
    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note .note.*)
        *(.gnu.hash)
    }
}
