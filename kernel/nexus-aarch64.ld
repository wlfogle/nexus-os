/* NexusOS AArch64 Kernel Linker Script (Bahamut)
 * Higher-half kernel: virtual base 0xffffffff80000000
 *
 * Boots via the custom boot stub in kernel/src/arch/aarch64/boot.rs, which
 * implements the Linux/arm64 kernel image boot protocol. Limine is no longer
 * used on AArch64.
 *
 * RECONSTRUCTED 2026-10-14. The previous version of this file was lost to an
 * accidental `git reset --hard` before it was ever committed. It has been
 * rebuilt from the contract that `boot.rs` documents explicitly -- every
 * symbol below is required by a named reference in that file, so the
 * *requirements* are certain even though the original wording is not
 * recoverable. Cross-check against boot.rs before trusting it on hardware.
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

    /* Start of the loaded image. `aarch64_boot_stub_main` measures the image
     * as (__kernel_end - __kernel_load_base) and asserts it fits inside the
     * trampoline's fixed 16 MiB TTBR1 block mapping. */
    __kernel_load_base = .;

    .text ALIGN(4096) : {
        /* The Linux/arm64 64-byte boot header MUST be the very first bytes of
         * the image: `code0` branches to _boot_trampoline, and U-Boot's
         * `booti` / QEMU's `-kernel Image` parse the fields behind it. */
        KEEP(*(.boot_header))
        KEEP(*(.boot_trampoline))

        /* Exception vectors must be 2048-byte aligned (VBAR_EL1). The boot
         * header and trampoline now precede them, so this alignment has to be
         * stated explicitly rather than inherited from the section start. */
        . = ALIGN(2048);
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

        /* Boot page tables, reserved statically so the pre-MMU trampoline can
         * reach them with `adrp`/`add :lo12:` before any Rust -- including the
         * allocator -- has run. Each must be its own 4 KiB-aligned, named
         * 4096-byte region; boot.rs declares them as `static [u8; 4096]`.
         *
         * Zeroing is free: the Makefile's raw-Image extraction
         * (`objcopy --pad-to`) zero-fills the whole .bss range in the flat
         * image, so the trampoline never zeroes them itself. */
        . = ALIGN(4096);
        __ttbr0_l0 = .;
        . += 4096;
        __ttbr0_l1 = .;
        . += 4096;
        __ttbr1_l0 = .;
        . += 4096;
        __ttbr1_l1 = .;
        . += 4096;
        __ttbr1_l2 = .;
        . += 4096;

        /* HHDM L1 table (Increment C3). The trampoline's TTBR1 mapping above
         * covers only the 16 MiB kernel image, and TTBR0's identity map is
         * Device-nGnRnE -- correct for MMIO, wrong for RAM, since Device
         * memory forbids the unaligned accesses ordinary Rust performs and
         * is uncached.
         *
         * memory::paging resolves every page-table walk through
         * phys_to_virt(), so a direct map of physical RAM as Normal WB is a
         * hard prerequisite for map_page(), and therefore for heap::init()
         * and everything downstream of it. Mapped at L0[256] of TTBR1
         * (0xffff800000000000), which is a different L0 slot from the kernel
         * image at L0[511], so the two never collide. */
        __ttbr1_hhdm_l1 = .;
        . += 4096;

        /* Boot stack. The trampoline loads __boot_stack_top into SP *after*
         * enabling the MMU, via its high-VA literal, so these frames are never
         * aliased through both the Device identity map and the Normal kernel
         * map. AArch64 requires SP to be 16-byte aligned. */
        . = ALIGN(4096);
        __boot_stack_bottom = .;
        . += 64K;
        . = ALIGN(16);
        __boot_stack_top = .;
    } :data

    /* End of the loaded image, and the image size the boot header advertises.
     * boot.rs emits `.quad __image_size` directly in the header because the
     * assembler cannot resolve `a - b` for two symbols external to that
     * translation unit -- so it must be computed here. */
    __kernel_end = .;
    __image_size = __kernel_end - __kernel_load_base;

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note .note.*)
        *(.gnu.hash)
    }
}
