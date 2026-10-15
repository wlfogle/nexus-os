//! AArch64 Custom Boot Stub — replaces Limine for Bahamut (Increment C2/C3)
//!
//! Implements the Linux/arm64 kernel image boot protocol (the same protocol
//! U-Boot's `booti` command and every real Raspberry Pi 4 boot chain use),
//! so this kernel no longer depends on Limine (or any UEFI implementation)
//! on AArch64. See the "Custom AArch64 Boot Stub" section of the Bahamut
//! plan for the full design rationale.
//!
//! # What this file does
//! 1. Emits the mandatory 64-byte Linux/arm64 boot header as the very first
//!    bytes of the image (`code0` branches past it) — this is what U-Boot's
//!    `booti` and QEMU's `-kernel Image -dtb ...` loader both recognise.
//! 2. A minimal pre-MMU assembly trampoline (`_boot_trampoline`):
//!    - Drops from EL2 to EL1 if the bootloader started us at EL2.
//!    - Builds *minimal* static page tables (statically reserved in `.bss`,
//!      referenced only via PC-relative `adrp`/`add` — never an absolute
//!      linked address — since absolute addresses baked in at link time
//!      against the 0xffffffff80000000 link base are only valid once the
//!      MMU is on and TTBR1 actually maps that range to wherever the image
//!      was physically loaded):
//!        * TTBR0_EL1: identity-maps the low 4 GiB via four 1 GiB L1 block
//!          descriptors, Device-nGnRnE. This covers both the physical
//!          location this trampoline is actually executing from (wherever
//!          the bootloader placed it) and every real MMIO window this
//!          kernel currently accesses by raw physical address (QEMU virt's
//!          PL011/virtio-mmio, and — once Increment B2+ lands — real Pi4's
//!          UART0/GPIO/mailbox/GIC-400/EMMC2, all comfortably within 4 GiB).
//!          Executing the handful of instructions between enabling the MMU
//!          and jumping away from this identity-mapped region is
//!          architecturally permitted from Device memory (merely
//!          discouraged for steady-state code, which this is not).
//!        * TTBR1_EL1: maps the kernel image's linked range
//!          (0xffffffff80000000 upward) to wherever it was actually loaded,
//!          using 2 MiB L2 block descriptors, Normal Write-Back
//!          Read/Write-Allocate. The Linux/arm64 boot protocol guarantees
//!          2 MiB alignment, so block descriptors (not 4 KiB pages) are
//!          valid here. Fixed at 16 MiB (8 blocks) — enormously generous
//!          for this kernel's actual size; [`aarch64_boot_stub_main`]
//!          asserts this at runtime so any future growth past the cap
//!          fails loudly instead of silently corrupting memory.
//!    - Programs `MAIR_EL1` (index 0 = Device-nGnRnE, index 1 = Normal WB
//!      RW-Allocate), `TCR_EL1` (4 KiB granule both TTBRs, T0SZ=T1SZ=16 for
//!      48-bit VA matching the "4-level, 48-bit VA" convention already
//!      documented in `memory::paging`, IPS read from
//!      `ID_AA64MMFR0_EL1.PARange` rather than assumed), then
//!      read-modify-writes `SCTLR_EL1` to set only M/C/I (its RES1 bits are
//!      already architecturally correct at hardware reset and are
//!      deliberately never overwritten with a hand-derived constant).
//!    - Jumps to [`aarch64_boot_stub_main`] at its normal linked (high VA)
//!      address — the one deliberate place an absolute address is used
//!      pre-jump, since by construction TTBR1 makes it valid the instant
//!      the branch lands. The stack pointer is likewise only ever set
//!      *after* this jump, directly to its linked high-VA address, so the
//!      boot stack is never aliased through two different memory
//!      attributes (Device via the low identity map vs. Normal via the
//!      kernel mapping) for the same physical frames.
//! 3. [`aarch64_boot_stub_main`] (Increment C3 extends this further): runs
//!    as ordinary Rust with normal absolute addressing, safe again now that
//!    TTBR1 is live.

use core::arch::global_asm;

// ─── Linux/arm64 boot image header (docs/arch/arm64/booting.rst upstream) ────
//
// Must be the very first 64 bytes of the loaded image. `code0` is a single
// branch instruction skipping the rest of the header; every other bootloader
// (U-Boot's `booti`, QEMU's `-kernel Image -dtb ...`) parses these fields
// without ever executing them.
global_asm!(
    ".section .boot_header, \"ax\", @progbits",
    ".global _image_start",
    "_image_start:",
    "b _boot_trampoline",              // code0: branch past the header
    ".word 0xd503201f",                // code1: nop (dead space, never executed)
    ".quad 0",                         // text_offset = 0
    ".quad __image_size",              // image_size -- computed by the linker
                                        //   script (see nexus-aarch64.ld), since a
                                        //   plain `.quad (a - b)` cannot be resolved
                                        //   by the assembler for two symbols that are
                                        //   external to this translation unit.
    ".quad 0xa",                       // flags: bit3=1 (place anywhere in DRAM,
                                        //   full linear map), bits[2:1]=01 (4K pages)
    ".quad 0",                         // res2
    ".quad 0",                         // res3
    ".quad 0",                         // res4
    ".word 0x644d5241",                // magic = "ARM\x64" (LE)
    ".word 0",                         // res5 (PE/COFF offset — unused, no EFI stub)
);

// ─── Pre-MMU trampoline ───────────────────────────────────────────────────────
//
// Register usage is fixed and deliberate throughout this whole block:
//   x0      — dtb physical pointer, per the Linux/arm64 boot protocol's own
//             calling convention (bootloader sets x0 = dtb, x1..x3 = 0 on
//             entry). Never used as scratch anywhere below, so it arrives
//             at `aarch64_boot_stub_main`'s first argument unmodified.
//   x20     — runtime physical load bias: the actual physical address
//             `_image_start` was loaded at (computed once via PC-relative
//             `adrp`/`add`, since the bootloader may not place it at any
//             particular address — the Linux/arm64 boot protocol only
//             guarantees 2 MiB alignment). Held live across both page-table
//             build sections.
//   x1..x10 — scratch, freely reused between sections (no values need to
//             survive across the TTBR0 and TTBR1 build sections besides x20).
global_asm!(
    ".section .boot_trampoline, \"ax\", @progbits",
    ".global _boot_trampoline",
    "_boot_trampoline:",

    // ── EL2 -> EL1 drop, only if the bootloader started us at EL2 ──────────
    "mrs x1, CurrentEL",
    "and x1, x1, #0xc",                // bits[3:2]
    "cmp x1, #0x8",                    // 0b10 << 2 == EL2
    "b.ne 1f",

    "mov  x1, #0x80000000",            // HCR_EL2.RW (bit 31): EL1 is AArch64
    "msr  hcr_el2, x1",
    "mov  x1, #0x3",                   // CNTHCTL_EL2: EL1PCTEN | EL1PCEN
    "msr  cnthctl_el2, x1",
    "msr  cntvoff_el2, xzr",
    "mov  x1, #0x3c5",                 // SPSR_EL2: EL1h, DAIF all masked
    "msr  spsr_el2, x1",
    "adr  x1, 1f",
    "msr  elr_el2, x1",
    "eret",

    "1:",
    // ── EL1 from here on (either dropped from EL2 above, or started here) ──
    "msr daifset, #0xf",

    // Runtime physical load bias: the actual address this code is executing
    // from right now, computed purely via PC-relative addressing so it is
    // correct regardless of where the bootloader placed the image.
    "adrp x20, _image_start",
    "add  x20, x20, :lo12:_image_start",

    // ── Build TTBR0 identity map: low 4 GiB, Device-nGnRnE, four 1 GiB blocks ──
    "adrp x2, __ttbr0_l0",
    "add  x2, x2, :lo12:__ttbr0_l0",
    "adrp x3, __ttbr0_l1",
    "add  x3, x3, :lo12:__ttbr0_l1",
    "orr  x4, x3, #0x3",               // Table descriptor (Valid + Table)
    "str  x4, [x2]",                   // L0[0] -> TTBR0 L1 table

    "mov  x6, #0x601",                 // AF | SH=Outer Shareable | Block, AttrIndx=0 (Device)
    "mov  x7, #0",                     // physical base of this block (0, 1G, 2G, 3G)
    "mov  x8, #0",                     // loop index / L1 slot
    "mov  x10, #0x40000000",           // 1 GiB step (ADD's immediate encoding cannot
                                        //   hold this directly -- imm12[,lsl#12] tops
                                        //   out at 0xFFF000 -- so it is loaded into a
                                        //   register once via MOV's wider immediate
                                        //   range and added in register form instead)
    "2:",
    "orr  x9, x7, x6",
    "str  x9, [x3, x8, lsl #3]",
    "add  x7, x7, x10",                // += 1 GiB
    "add  x8, x8, #1",
    "cmp  x8, #4",
    "b.lt 2b",

    // ── Build TTBR1 kernel mapping: 0xffffffff80000000+, Normal WB, 2 MiB blocks ──
    "adrp x2, __ttbr1_l0",
    "add  x2, x2, :lo12:__ttbr1_l0",
    "adrp x3, __ttbr1_l1",
    "add  x3, x3, :lo12:__ttbr1_l1",
    "adrp x4, __ttbr1_l2",
    "add  x4, x4, :lo12:__ttbr1_l2",

    "orr  x5, x3, #0x3",
    "mov  x9, #511",
    "str  x5, [x2, x9, lsl #3]",       // L0[511] -> TTBR1 L1 table
    "orr  x5, x4, #0x3",
    "mov  x9, #510",
    "str  x5, [x3, x9, lsl #3]",       // L1[510] -> TTBR1 L2 table (0xffffffff80000000 is 1 GiB-aligned)

    "mov  x6, #0x705",                 // AF | AttrIndx=1 (Normal) | SH=Inner Shareable | Block
    "mov  x7, x20",                    // physical base = actual runtime load address
    "mov  x8, #0",                     // loop index / L2 slot
    "mov  x10, #0x200000",             // 2 MiB step -- same ADD-immediate-range reason as above
    "3:",
    "orr  x9, x7, x6",
    "str  x9, [x4, x8, lsl #3]",
    "add  x7, x7, x10",                // += 2 MiB
    "add  x8, x8, #1",
    "cmp  x8, #8",                     // 8 * 2 MiB = 16 MiB kernel-image mapping cap
    "b.lt 3b",

    // ── Build the HHDM: physical RAM, Normal WB, at 0xffff800000000000 ────
    // Increment C3. `memory::paging` resolves every page-table walk through
    // `phys_to_virt()`, so without a direct map of physical memory
    // `map_page()` cannot write a table entry at all -- which blocks
    // `heap::init()` and everything downstream of it.
    //
    // This cannot reuse the TTBR0 identity map: that is Device-nGnRnE, which
    // is correct for MMIO but wrong for RAM, since Device memory forbids the
    // unaligned accesses ordinary Rust emits and is uncached.
    //
    // Four 1 GiB blocks cover the low 4 GiB, spanning all of QEMU virt's RAM.
    // L0[256] of TTBR1 is 0xffff800000000000; the kernel image lives at
    // L0[511], so the two mappings never overlap.
    "adrp x2, __ttbr1_l0",
    "add  x2, x2, :lo12:__ttbr1_l0",
    "adrp x3, __ttbr1_hhdm_l1",
    "add  x3, x3, :lo12:__ttbr1_hhdm_l1",
    "orr  x5, x3, #0x3",               // Table descriptor (Valid + Table)
    "mov  x9, #256",
    "str  x5, [x2, x9, lsl #3]",       // TTBR1 L0[256] -> HHDM L1 table

    "mov  x6, #0x705",                 // AF | AttrIndx=1 (Normal) | SH=Inner | Block
    "mov  x7, #0",                     // physical base of this block
    "mov  x8, #0",                     // loop index / L1 slot
    "mov  x10, #0x40000000",           // 1 GiB step (same ADD-immediate limit as above)
    "4:",
    "orr  x9, x7, x6",
    "str  x9, [x3, x8, lsl #3]",
    "add  x7, x7, x10",
    "add  x8, x8, #1",
    "cmp  x8, #4",                     // 4 * 1 GiB = low 4 GiB direct-mapped
    "b.lt 4b",

    // ── Program TTBR0_EL1 / TTBR1_EL1 (physical addresses, PC-relative) ────
    "adrp x1, __ttbr0_l0",
    "add  x1, x1, :lo12:__ttbr0_l0",
    "msr  ttbr0_el1, x1",
    "adrp x1, __ttbr1_l0",
    "add  x1, x1, :lo12:__ttbr1_l0",
    "msr  ttbr1_el1, x1",

    // ── MAIR_EL1: index 0 = Device-nGnRnE (0x00), index 1 = Normal WB RW-Allocate (0xFF) ──
    "mov  x1, #0xff00",
    "msr  mair_el1, x1",

    // ── TCR_EL1: T0SZ=T1SZ=16 (48-bit VA), 4K granule both TTBRs, WB/Inner-Shareable
    //    table walks, IPS from ID_AA64MMFR0_EL1.PARange ─────────────────────
    "mov  x1, #0x3510",                // bits[15:0]: TG0=00,SH0=11,ORGN0=01,IRGN0=01,T0SZ=16
    "movk x1, #0xb510, lsl #16",       // bits[31:16]: TG1=10,SH1=11,ORGN1=01,IRGN1=01,T1SZ=16
    "mrs  x2, id_aa64mmfr0_el1",
    "and  x2, x2, #0xf",               // PARange field
    "lsl  x2, x2, #32",                // TCR_EL1.IPS is at bits[34:32]
    "orr  x1, x1, x2",
    "msr  tcr_el1, x1",

    // ── Enable the MMU: read-modify-write SCTLR_EL1, touching only M/C/I ────
    // (RES1 bits are already correct at hardware reset; never overwritten
    // with a hand-derived constant.)
    "isb",
    "mrs  x1, sctlr_el1",
    "orr  x1, x1, #0x1",               // M — MMU enable
    "orr  x1, x1, #0x4",               // C — data cache enable
    "orr  x1, x1, #0x1000",            // I — instruction cache enable
    "msr  sctlr_el1, x1",
    "isb",

    // ── Set SP to its final, proper high-VA address, then jump to the high-VA
    // Rust continuation. Both addresses are loaded from a small in-image
    // literal pool (plain `.quad symbol` -- an ordinary, universally
    // supported single-symbol ABS64 relocation) reached via normal
    // PC-relative `adrp`/`add` (the literal pool itself is part of this
    // same object, at a fixed offset from this code, regardless of where
    // the image was actually loaded). This is deliberately simpler than
    // the AArch64 `movz/movk` `:abs_gN:` relocation-specifier idiom, which
    // is not reliably supported by every assembler.
    //
    // Doing this *before* jumping matters for SP specifically: it is safe
    // despite PC still being physically in the low range, because the MMU
    // is already fully enabled at this point (SCTLR_EL1.M=1 above), and
    // once enabled it governs *every* memory access uniformly by address
    // range -- only instruction *fetch* needs PC itself to be in a mapped
    // range (still true here, via the TTBR0 identity map), while ordinary
    // data accesses through a high-VA register value (like `mov sp, x2`,
    // and everything the Rust prologue below it does) already correctly
    // resolve through TTBR1. A normal Rust function's own prologue can
    // touch the stack before any hand-written inline `asm!` inside its
    // body would ever run, so SP must already be valid *before*
    // `aarch64_boot_stub_main` is ever entered -- and setting it to a
    // proper high-VA address (rather than the trampoline's physical/
    // identity addressing) also keeps the boot stack's physical frames
    // from ever being aliased through two different memory attributes
    // (Device via the low identity map vs. Normal via the kernel mapping).
    "adrp x2, __boot_stack_top_lit",
    "add  x2, x2, :lo12:__boot_stack_top_lit",
    "ldr  x2, [x2]",
    "mov  sp, x2",

    "adrp x1, __boot_stub_main_lit",
    "add  x1, x1, :lo12:__boot_stub_main_lit",
    "ldr  x1, [x1]",
    "br   x1",

    ".balign 8",
    "__boot_stack_top_lit:",
    ".quad __boot_stack_top",
    "__boot_stub_main_lit:",
    ".quad aarch64_boot_stub_main",
);

// ─── Statically-reserved boot page tables + stack ────────────────────────────
//
// Declared here (not as ordinary Rust `static` arrays) because the linker
// script must place them inside `.bss` at fixed, individually-named,
// 4 KiB-aligned offsets that the trampoline above can reach via
// `adrp`/`add :lo12:` before any Rust code — including the allocator — has
// run. Zero-initialised for free: the Makefile's raw-Image extraction
// (`objcopy --pad-to`) physically zero-fills the entire `.bss` range in the
// flat image, exactly like a normal ELF loader would, so the trampoline
// never needs to explicitly zero them.
extern "C" {
    static __ttbr0_l0: [u8; 4096];
    static __ttbr0_l1: [u8; 4096];
    static __ttbr1_l0: [u8; 4096];
    static __ttbr1_l1: [u8; 4096];
    static __ttbr1_l2: [u8; 4096];
    static __ttbr1_hhdm_l1: [u8; 4096];
    static __boot_stack_top: u8;
    static __kernel_load_base: u8;
    static __kernel_end: u8;
}

/// Fixed cap on the TTBR1 kernel-image block mapping built by the trampoline
/// above (`3:` loop, 8 * 2 MiB). Checked at runtime against the linker's own
/// measurement of the actual image size so any future growth past this cap
/// fails loudly (a clear panic) instead of silently running off the mapped
/// range and faulting on unrelated, hard-to-diagnose memory accesses.
const TRAMPOLINE_KERNEL_MAP_CAP: u64 = 16 * 1024 * 1024;

/// Base of the direct map built by the trampoline's `4:` loop (Increment C3).
///
/// `memory::paging::phys_to_virt` adds this to a physical address, so it is
/// the offset the rest of the kernel must be initialised with. Chosen to sit
/// at TTBR1 L0[256], well clear of the kernel image at L0[511].
pub const HHDM_BASE: u64 = 0xffff_8000_0000_0000;

/// How much physical memory the trampoline's direct map covers (4 * 1 GiB).
/// Anything beyond this is not reachable through `phys_to_virt` until a
/// later increment extends the map.
pub const HHDM_COVERAGE: u64 = 4 * 1024 * 1024 * 1024;

/// Rust continuation, entered by the trampoline above at the kernel's normal
/// linked (high VA) address with `x0` = the DTB physical pointer, per the
/// Linux/arm64 boot protocol's own calling convention. Ordinary absolute
/// addressing is safe again here since TTBR1 now correctly maps this range.
///
/// Increment C2 scope: set up the stack (at its proper high-VA address —
/// never the trampoline's physical/identity addressing, so the boot stack's
/// physical frames are never aliased through two different memory
/// attributes), bring up early UART output, and prove the whole chain
/// (EL2->EL1 drop if applicable, MMU bring-up, jump to high VA) actually
/// works end-to-end. Increment C3 replaces the tail of this function with a
/// full DTB-derived memory map and a handoff into `_start()`'s existing
/// continuation logic.
#[no_mangle]
extern "C" fn aarch64_boot_stub_main(dtb_phys: u64) -> ! {
    // SP is already correctly set (to its high-VA address) by the
    // trampoline before this function was ever entered -- see the comment
    // above the `mov sp, x2` sequence in the `global_asm!` block above.

    let image_memsz = unsafe {
        (&__kernel_end as *const u8 as u64) - (&__kernel_load_base as *const u8 as u64)
    };
    assert!(
        image_memsz <= TRAMPOLINE_KERNEL_MAP_CAP,
        "aarch64 boot stub: kernel image ({} bytes) exceeds the trampoline's \
         fixed {} byte TTBR1 mapping cap -- raise TRAMPOLINE_KERNEL_MAP_CAP \
         and the matching loop bound (`cmp x8, #N`) in the trampoline above",
        image_memsz,
        TRAMPOLINE_KERNEL_MAP_CAP,
    );

    // Increment B2: which PL011 to talk to depends on the board, so the DTB
    // has to be classified before there is any console to report it on.
    // `detect_quiet` is the silent variant for exactly this ordering; the
    // result is logged a few lines below, once the UART is live.
    //
    // The DTB pointer arrives as a *physical* address in x0 per the
    // Linux/arm64 boot protocol, and is readable here because the
    // trampoline's TTBR0 identity-maps the low 4 GiB.
    let platform = crate::arch::aarch64::platform::detect_quiet(dtb_phys);
    let (ibrd, fbrd) = crate::arch::aarch64::platform::uart_divisors(platform);
    crate::io::uart::configure(
        crate::arch::aarch64::platform::uart_base(platform),
        ibrd,
        fbrd,
    );

    crate::io::init_early();
    crate::kprintln!();
    crate::arch::aarch64::platform::log_detected();
    crate::kprintln!("[boot] AArch64 custom boot stub: EL/MMU bring-up complete, running at high VA");
    crate::kprintln!("[boot] dtb_phys           : {:#018x}", dtb_phys);
    crate::kprintln!("[boot] kernel image size  : {} KiB (cap {} KiB)",
        image_memsz / 1024, TRAMPOLINE_KERNEL_MAP_CAP / 1024);

    // Prove the HHDM actually works before anything depends on it.
    //
    // The DTB is a known-value probe: its first four bytes are the FDT magic
    // 0xd00dfeed, and it was just read successfully through the *identity*
    // map during platform detection. Reading the same bytes through the
    // direct map must therefore yield the same answer. A wrong L0 slot, a
    // bad block descriptor or wrong memory attributes all show up here as
    // garbage or a fault, rather than silently much later inside the first
    // `map_page()` call.
    if dtb_phys != 0 && dtb_phys < HHDM_COVERAGE {
        let via_hhdm = unsafe {
            core::ptr::read_volatile((HHDM_BASE + dtb_phys) as *const u32)
        };
        let ok = u32::from_be(via_hhdm) == 0xd00d_feed;
        crate::kprintln!(
            "[mem]  HHDM at {:#018x} covering {} GiB: DTB magic via direct map = {:#010x} ({})",
            HHDM_BASE,
            HHDM_COVERAGE / (1024 * 1024 * 1024),
            u32::from_be(via_hhdm),
            if ok { "OK" } else { "MISMATCH" },
        );
        assert!(ok, "aarch64 boot stub: HHDM direct map is not mapping physical memory correctly");
    }

    loop {
        crate::arch::halt();
    }
}
