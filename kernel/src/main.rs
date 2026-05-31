//! NexusOS Kernel — Entry Point
//!
//! Three build targets:
//!   laptop   — Intel i9-13900HX, x86_64, full (framebuffer + AI hooks)
//!   tiamat   — x86_64 server, headless
//!   bahamut  — AArch64, 2 GB edge node

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]      // x86_64 interrupt handlers
#![feature(naked_functions)]        // AArch64 exception stubs

extern crate alloc;

// ─── Sub-modules ─────────────────────────────────────────────────────────────

pub mod arch;
pub mod io;
pub mod memory;
pub mod panic;

// ─── Limine boot protocol requests ───────────────────────────────────────────
// These static variables are scanned by the Limine bootloader before handing
// control to _start. Limine fills in the response pointers.

use limine::{
    HhdmRequest,
    MemmapRequest,
    KernelAddressRequest,
};

#[cfg(feature = "framebuffer")]
use limine::FramebufferRequest;

/// Higher-Half Direct Map: Limine maps all physical memory at this offset.
#[used]
#[link_section = ".limine_requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new(0);

/// Physical memory map.
#[used]
#[link_section = ".limine_requests"]
static MMAP_REQUEST: MemmapRequest = MemmapRequest::new(0);

/// Kernel physical + virtual base addresses.
#[used]
#[link_section = ".limine_requests"]
static KADDR_REQUEST: KernelAddressRequest = KernelAddressRequest::new(0);

/// Framebuffer — laptop only.
#[cfg(feature = "framebuffer")]
#[used]
#[link_section = ".limine_requests"]
static FB_REQUEST: FramebufferRequest = FramebufferRequest::new(0);

// ─── Kernel entry point ───────────────────────────────────────────────────────

/// Called by Limine after setting up paging, HHDM, and GDT stubs.
/// CPU is in 64-bit long mode (x86_64) or EL1 (AArch64).
/// Interrupts are DISABLED on entry.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // ── 1. Early serial/UART output ─────────────────────────────────────────
    // x86_64: COM1 port I/O — works immediately, no mapping needed.
    // aarch64: PL011 UART — MMIO, needs physical address (Limine identity-maps
    //          low memory before calling us, so physical == virtual here).
    io::init_early();
    kprintln!();
    kprintln!("┌─────────────────────────────────────────┐");
    kprintln!("│  NexusOS Kernel v{}  [{:^10}]  │",
              env!("CARGO_PKG_VERSION"), build_label());
    kprintln!("│  World's First AI-Native OS             │");
    kprintln!("└─────────────────────────────────────────┘");
    kprintln!();

    // ── 2. Collect Limine boot responses ────────────────────────────────────
    let hhdm = HHDM_REQUEST
        .get_response()
        .get()
        .expect("Limine: no HHDM response");
    let hhdm_offset = hhdm.offset;

    let mmap = MMAP_REQUEST
        .get_response()
        .get()
        .expect("Limine: no memory map response");

    let kaddr = KADDR_REQUEST
        .get_response()
        .get()
        .expect("Limine: no kernel address response");

    kprintln!("[boot] HHDM offset       : {:#018x}", hhdm_offset);
    kprintln!("[boot] Kernel phys base  : {:#018x}", kaddr.physical_base);
    kprintln!("[boot] Kernel virt base  : {:#018x}", kaddr.virtual_base);

    // ── 3. Architecture initialisation (GDT/IDT on x86_64, VBAR on AArch64)
    arch::init();
    kprintln!("[arch] CPU structures loaded");

    // ── 4. Physical memory manager ──────────────────────────────────────────
    memory::physical::init(mmap, hhdm_offset);
    kprintln!("[mem]  Physical frame allocator online");

    // ── 5. Virtual memory / page tables ─────────────────────────────────────
    memory::paging::init(hhdm_offset);
    kprintln!("[mem]  Paging initialised");

    // ── 6. Kernel heap ──────────────────────────────────────────────────────
    memory::heap::init();
    kprintln!("[mem]  Kernel heap ({} MB) ready",
              memory::heap::HEAP_SIZE / (1024 * 1024));

    // ── 7. Framebuffer text console (laptop only) ────────────────────────────
    #[cfg(feature = "framebuffer")]
    {
        let fb = FB_REQUEST
            .get_response()
            .get()
            .expect("Limine: no framebuffer response");
        io::framebuffer::init(fb);
        kprintln!("[fb]   Framebuffer console active");
    }

    // ── 8. Feature-specific init ─────────────────────────────────────────────
    #[cfg(feature = "ai-hooks")]
    {
        kprintln!("[ai]   AI interface hooks registered — ready for AI Core server");
    }

    #[cfg(feature = "server-mode")]
    {
        kprintln!("[srv]  Server mode: headless, service management hooks active");
    }

    // ── 9. Enable interrupts and idle ────────────────────────────────────────
    arch::enable_interrupts();
    kprintln!("[arch] Interrupts enabled");
    kprintln!();
    kprintln!("NexusOS kernel idle — waiting for scheduler (Phase 2).");

    loop {
        arch::halt();
    }
}

/// Human-readable build label for banner.
const fn build_label() -> &'static str {
    if cfg!(feature = "laptop") {
        "laptop"
    } else if cfg!(feature = "tiamat") {
        "tiamat"
    } else if cfg!(feature = "bahamut") {
        "bahamut"
    } else {
        "dev"
    }
}

// ─── Global print macros ─────────────────────────────────────────────────────

/// Print without newline — routes to serial (x86_64) or UART (aarch64).
/// On laptop, also mirrors to framebuffer when available.
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::io::_kprint(format_args!($($arg)*))
    };
}

/// Print with newline.
#[macro_export]
macro_rules! kprintln {
    ()              => ($crate::kprint!("\n"));
    ($($arg:tt)*)   => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

// ─── Global allocator error handler ──────────────────────────────────────────

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Kernel heap allocation failed: size={} align={}",
           layout.size(), layout.align())
}
