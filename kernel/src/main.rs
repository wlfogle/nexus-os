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

extern crate alloc;

// ─── Kernel end symbol (set by linker, used by installer) ────────────────────────

extern "C" {
    static __kernel_end: u8;
}

// ─── Sub-modules ─────────────────────────────────────────────────────────────

pub mod arch;
#[cfg(target_arch = "x86_64")]
pub mod acpi;
#[cfg(target_arch = "x86_64")]
pub mod drivers;
// AArch64 VirtIO-MMIO block driver (QEMU virt machine)
#[cfg(target_arch = "aarch64")]
pub mod virtio_mmio;
#[cfg(all(target_arch = "aarch64", feature = "bahamut"))]
pub mod bahamut_tui;
pub mod exec;
pub mod fs;
pub mod installer;
pub mod io;
pub mod ipc;
pub mod memory;
#[cfg(target_arch = "x86_64")]
pub mod net;
pub mod panic;
pub mod process;
pub mod scheduler;
#[cfg(target_arch = "x86_64")]
pub mod syscall;
pub mod timer;
pub mod userspace;

// ─── Limine boot protocol requests ───────────────────────────────────────────
// These static variables are scanned by the Limine bootloader before handing
// control to _start. Limine fills in the response pointers.

use limine::{
    HhdmRequest,
    MemmapRequest,
    KernelAddressRequest,
    KernelFileRequest,
    BaseRevision,
};

#[cfg(target_arch = "x86_64")]
use limine::RsdpRequest;

#[cfg(feature = "framebuffer")]
use limine::FramebufferRequest;

/// Limine base revision — minimum 6 required for AArch64 UEFI.
#[used]
#[link_section = ".limine_requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new(6);

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

/// Original kernel ELF file — used by the installer to write the correct
/// binary to the installed disk (rather than a raw memory image).
#[used]
#[link_section = ".limine_requests"]
static KFILE_REQUEST: KernelFileRequest = KernelFileRequest::new(0);

/// ACPI RSDP pointer (Phase K5) — x86_64 only; AArch64 (bahamut) has no
/// ACPI tables to discover.
#[cfg(target_arch = "x86_64")]
#[used]
#[link_section = ".limine_requests"]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new(0);

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
    // ── 1a. AArch64: re-enable TTBR0_EL1 so physical UART is accessible.
    // Limine sets TCR_EL1.EPD0=1, disabling TTBR0 (lower VA range).
    // UART MMIO at 0x09000000 is not in HHDM — only in TTBR0 identity map.
    // Clear EPD0 to restore access to the identity-mapped lower VA range.
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "mrs x0, tcr_el1",
            "bic x0, x0, #(1 << 7)",  // Clear EPD0 (bit 7) — enable TTBR0
            "msr tcr_el1, x0",
            "isb",
            options(nostack, nomem)
        );
    }

    // ── 1b. Early serial/UART output ────────────────────────────────────────
    // x86_64: COM1 port I/O — works immediately, no mapping needed.
    // aarch64: PL011 UART — remapped via HHDM above.
    io::init_early();
    kprintln!();
    kprintln!("\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}");
    kprintln!("\u{2502}  NexusOS Kernel v{}  [{:^10}]  \u{2502}",
              env!("CARGO_PKG_VERSION"), build_label());
    kprintln!("\u{2502}  World's First AI-Native OS             \u{2502}");
    kprintln!("\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}");
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

    // Store original kernel ELF pointer + size for the installer task.
    // KernelFileRequest gives us the exact bytes Limine read from disk, so
    // the installer writes a proper ELF rather than a raw memory image.
    // KernelFileRequest: Ptr<T>::get() → Option<&T>, Ptr<u8>::as_ptr() → Option<*mut u8>
    #[cfg(target_arch = "x86_64")]
    {
        if let Some(kfile_resp) = KFILE_REQUEST.get_response().get() {
            if let Some(kfile) = kfile_resp.kernel_file.get() {
                if let Some(base_ptr) = kfile.base.as_ptr() {
                    installer::KERNEL_ELF_BASE.store(
                        base_ptr as u64,
                        core::sync::atomic::Ordering::Relaxed,
                    );
                    installer::KERNEL_ELF_SIZE.store(
                        kfile.length,
                        core::sync::atomic::Ordering::Relaxed,
                    );
                }
            }
        }
    }

    // ── 3. Architecture initialisation (GDT/IDT on x86_64, VBAR on AArch64)
    arch::init();
    kprintln!("[arch] CPU structures loaded");

    // ── 4. Physical memory manager ──────────────────────────────────────────
    memory::physical::init(mmap, hhdm_offset);
    kprintln!("[mem]  Physical frame allocator online");

    // ── 5. Virtual memory / page tables ─────────────────────────────
    memory::paging::init(hhdm_offset);
    kprintln!("[mem]  Paging initialised");

    // ── 5.5. ACPI table discovery (Phase K5) ────────────────────
    // Needs paging::phys_to_virt (just initialised above) to walk the
    // RSDT/XSDT and every table it points to. Purely additive: nothing
    // downstream depends on this yet, and a missing/corrupt table just
    // means later phases fall back to their pre-ACPI behavior.
    #[cfg(target_arch = "x86_64")]
    {
        let rsdp_virt = RSDP_REQUEST
            .get_response()
            .get()
            .and_then(|r| r.address.as_ptr())
            .map(|p| p as u64)
            .unwrap_or(0);
        acpi::init(rsdp_virt);
    }

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
        // Reprint the full boot log to the framebuffer now that it's ready
        kprintln!();
        kprintln!("==========================================");
        kprintln!("  NexusOS Kernel v{}  [  laptop  ]", env!("CARGO_PKG_VERSION"));
        kprintln!("  World's First AI-Native OS");
        kprintln!("==========================================");
        kprintln!();
        kprintln!("[boot] HHDM offset       : {:#018x}", hhdm_offset);
        kprintln!("[boot] Kernel phys base  : {:#018x}", kaddr.physical_base);
        kprintln!("[boot] Kernel virt base  : {:#018x}", kaddr.virtual_base);
        kprintln!("[arch] CPU structures loaded");
        kprintln!("[pmem] {} MiB usable", memory::physical::free_frames() * 4096 / (1024*1024));
        kprintln!("[mem]  Physical frame allocator online");
        kprintln!("[mem]  Paging initialised");
        kprintln!("[mem]  Kernel heap ({} MB) ready", memory::heap::HEAP_SIZE / (1024*1024));
        kprintln!("[fb]   Framebuffer console active");
    }

    #[cfg(target_arch = "x86_64")]
    {
        // ── 7.5. VirtIO-blk disk driver (after framebuffer so output is visible) ──
        // VirtIO vendor 0x1AF4; device 0x1001 = legacy blk, 0x1042 = transitional
        const VIRTIO_VENDOR: u16 = 0x1AF4;
        match drivers::pci::find(&[(VIRTIO_VENDOR, 0x1001), (VIRTIO_VENDOR, 0x1042)]) {
            Some(mut dev) => {
                dev.enable_io_and_busmaster();
                dev.bar0 = drivers::pci::read32(dev.bus, dev.dev, dev.func, 0x10);
                kprintln!("[disk] PCI {:02x}:{:02x}.{} vendor={:#06x} device={:#06x} BAR0={:#010x}",
                          dev.bus, dev.dev, dev.func,
                          dev.vendor_id, dev.device_id, dev.bar0);
                if dev.bar0 & 1 == 0 {
                    kprintln!("[disk] BAR0 is MMIO (not I/O port) — legacy driver incompatible");
                    kprintln!("[disk] MMIO addr={:#010x} — needs MMIO VirtIO transport",
                              dev.bar0 & !0xF);
                } else {
                    match drivers::virtio::blk::init(dev.io_base()) {
                        Ok(sectors) => {
                            let gib = sectors / (2 * 1024 * 1024);
                            kprintln!("[disk] VirtIO-blk: {} GiB ({} sectors)", gib, sectors);
                        }
                        Err(e) => kprintln!("[disk] VirtIO-blk init failed: {}", e),
                    }
                }
            }
            None => kprintln!("[disk] no VirtIO-blk device found"),
        }

        if !drivers::nvme::init() {
            kprintln!("[nvme] no NVMe controller found");
        }
        if !drivers::ahci::init() {
            kprintln!("[ahci] no AHCI SATA controller found");
        }

        let fs_msg = fs::fat::init();
        kprintln!("[fs]   {}", fs_msg);
    }

    #[cfg(target_arch = "aarch64")]
    {
        kprintln!("[bahamut] AArch64 network-edge: UART, memory, paging, heap online");

        // ── AArch64 VirtIO-MMIO disk driver ─────────────────────────────────
        let disk_sectors = virtio_mmio::init();

        // ── FAT32 filesystem (shared with x86_64) ───────────────────────────
        let fs_msg = fs::fat::init();
        kprintln!("[fs]   {}", fs_msg);

        // ── Installer (first boot from ISO — no formatted disk) ─────────────
        if disk_sectors > 0 && !fs::fat::is_mounted() {
            kprintln!("[inst] No formatted disk — spawning installer");
            installer::task_installer_run();
        } else if fs::fat::is_mounted() {
            kprintln!("[inst] Disk already installed — booting normally");
        }
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

    #[cfg(target_arch = "x86_64")]
    {
        // ── 9. Phase 2: PIC + PIT + Scheduler ───────────────────────────────
        timer::init();                          // PIC remap + PIT 100 Hz
        kprintln!("[timer] PIC remapped, PIT running at {} Hz", timer::TIMER_HZ);

        io::keyboard::init();
        kprintln!("[kbd]  i8042 initialised — scanning enabled, IRQ1 unmasked");

        io::mouse::init();
        kprintln!("[mouse] PS/2 mouse initialised — IRQ12 unmasked");

        // ── Phase K5 increment 3: Local APIC + I/O APIC ─────────────────
        // Supersedes the legacy 8259 PIC as the interrupt-routing path for
        // the same three IRQs (timer/keyboard/mouse) whenever the MADT
        // gave us both a Local APIC and an I/O APIC address; every handler
        // stays installed at the exact same IDT vectors either way (see
        // arch::x86_64::{lapic,ioapic}). Falls back to leaving the PIC path
        // untouched (already fully set up above) if ACPI/MADT is
        // unavailable or incomplete — exactly the K1-K4 degrade-gracefully
        // precedent this whole ACPI subsystem was built on.
        match acpi::madt() {
            Some(m) if m.local_apic_addr != 0 && m.ioapic_addr != 0 => {
                arch::x86_64::lapic::init(m.local_apic_addr);
                arch::x86_64::ioapic::init(m.ioapic_addr, m.ioapic_gsi_base);
                let bsp_id = arch::x86_64::lapic::id();
                // Phase K5 increment 4: record core 0 (the BSP) -> its real
                // hardware LAPIC ID, so future code can go either direction
                // between the dense core index and the LAPIC identity.
                arch::x86_64::lapic::register_core(0);

                let route = |isa_irq: u8, vector: u8| {
                    let gsi = m.isa_irq_to_gsi(isa_irq);
                    let (active_low, level_triggered) = m.isa_irq_polarity_trigger(isa_irq);
                    arch::x86_64::ioapic::set_redirection(
                        gsi, vector, bsp_id, false, active_low, level_triggered,
                    );
                    gsi
                };
                let gsi_timer = route(timer::pic::IRQ_TIMER, timer::pic::PIC1_OFFSET + timer::pic::IRQ_TIMER);
                let gsi_kbd   = route(timer::pic::IRQ_KEYBOARD, timer::pic::PIC1_OFFSET + timer::pic::IRQ_KEYBOARD);
                let gsi_mouse = route(timer::pic::IRQ_MOUSE, timer::pic::PIC2_OFFSET + (timer::pic::IRQ_MOUSE - 8));

                // Now that the I/O APIC owns these lines, the 8259 must
                // never be allowed to also raise them — two controllers
                // both live for the same wire would double-deliver.
                timer::pic::mask_all();
                kprintln!(
                    "[ioapic] routed via I/O APIC (bsp lapic id={}): timer->gsi{} kbd->gsi{} mouse->gsi{}; legacy 8259 PIC masked",
                    bsp_id, gsi_timer, gsi_kbd, gsi_mouse
                );
            }
            Some(_) => kprintln!("[ioapic] MADT missing Local APIC/IOAPIC address — staying on legacy 8259 PIC"),
            None => kprintln!("[ioapic] no MADT — staying on legacy 8259 PIC"),
        }

        if !drivers::xhci::init() {
            kprintln!("[xhci] no USB controller found");
        }

        scheduler::init();                      // register idle process

        // ── Phase 4: Syscall interface + user-space process ─────────────
        // Phase K5 increment 4: per-core GDT/TSS/PERCPU infrastructure. This
        // is core 0 (the BSP) doing its own setup; each AP will call the
        // same underlying functions with its own id in increment 5.
        syscall::init(0);
        kprintln!(
            "[cpu]  core 0 (BSP) registered, lapic id={:?}",
            arch::x86_64::lapic::lapic_id_for_core(0)
        );
        let user_pid = userspace::spawn_user_init();
        kprintln!("[user] nexus-init spawned as pid={} (ring 3)", user_pid);

        // ── Phase 5: AI Core kernel thread ──────────────────
        scheduler::spawn(b"nexus-ai", task_nexus_ai)
            .expect("failed to spawn nexus-ai");
        kprintln!("[ai]   nexus-ai AI Core daemon spawned");

        // ── Phase K4: USB HID polling kernel thread ───────────
        // drivers::xhci is polling-only (no MSI/MSI-X interrupts), so
        // something has to periodically drain its HID endpoint. Harmless
        // to always spawn this even with no USB controller/device present
        // -- usb_hid::poll() returns immediately in that case.
        scheduler::spawn(b"usb-hid", task_usb_hid_poll)
            .expect("failed to spawn usb-hid");
        kprintln!("[xhci] USB HID polling thread spawned");

        // ── NexusOS Installer ───────────────────────────────────────────────
        // Runs only when disk is unformatted (first boot from ISO).
        if !fs::fat::is_mounted() {
            scheduler::spawn(b"installer", installer::task_installer)
                .expect("failed to spawn installer");
            kprintln!("[inst] NexusOS Installer spawned");
        }
    }

    // Enable hardware interrupts — timer fires immediately
    arch::enable_interrupts();
    kprintln!("[arch] Interrupts enabled — scheduler is LIVE");
    #[cfg(target_arch = "x86_64")]
    {
        kprintln!();
        kprintln!("NexusOS v{} — Phase 5: Ring-3 shell + AI Core + PS/2 keyboard active.",
                  env!("CARGO_PKG_VERSION"));
        kprintln!("[kbd]  PS/2 keyboard online — nexus-shell ready");

        net::init();
    }

    #[cfg(target_arch = "aarch64")]
    {
        kprintln!();
        kprintln!("NexusOS v{} — Bahamut AArch64 booting NexusOS Lite TUI...",
                  env!("CARGO_PKG_VERSION"));
    }

    // AArch64 Bahamut: launch TUI (never returns)
    #[cfg(all(target_arch = "aarch64", feature = "bahamut"))]
    bahamut_tui::run();

    // x86_64 or non-bahamut AArch64: idle loop
    #[cfg(not(all(target_arch = "aarch64", feature = "bahamut")))]
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

// ─── Phase 5: AI Core kernel thread ────────────────────────────────────────────

/// AI Core daemon — registers the `nexus.ai` port and serves inference requests.
///
/// Phase 5.0: responds with a structured mock reply so the IPC pipeline and
/// port-discovery path are fully exercised.  The Ollama HTTP client that sends
/// real prompts ships in Phase 5.1 once the network stack is available.
///
/// x86_64-only: depends on `net::tcp_request`, which in turn depends on the
/// x86_64 VirtIO-net/smoltcp stack. Never spawned on AArch64 (see the boot
/// sequence above), so this — and the two helpers below it — are compiled
/// out there rather than left as a link-time trap.
#[cfg(target_arch = "x86_64")]
extern "C" fn task_nexus_ai() -> ! {
    use ipc::{ipc_recv, ipc_send, Message, ANY, MSG_AI_RESPONSE};
    use ipc::ports::port_register;

    port_register(b"nexus.ai").expect("nexus-ai: failed to register port");
    kprintln!("[nexus-ai] AI Core online — port 'nexus.ai' registered");
    kprintln!("[nexus-ai] Phase 5.1: real Ollama HTTP client active");

    let mut req = Message::new(0, 0);
    loop {
        ipc_recv(ANY, &mut req).expect("nexus-ai: recv failed");

        let query = req.as_str();
        kprintln!("[nexus-ai] request from pid={}: {}", req.from, query);

        let reply_text = ai_generate(query);
        let reply = Message::with_str(req.from, MSG_AI_RESPONSE, &reply_text);
        ipc_send(req.from, reply).ok();
        kprintln!("[nexus-ai] response sent to pid={}", req.from);
    }
}

/// Polls `drivers::usb_hid` once per timer tick (100 Hz) for a pending HID
/// report. `drivers::xhci` has no interrupt path, so this kernel thread is
/// what keeps USB keyboard/mouse input flowing -- `sti; hlt` between polls
/// yields to the scheduler until the next tick, matching `idle_entry`'s
/// own halt-until-interrupt pattern rather than busy-spinning a full core.
#[cfg(target_arch = "x86_64")]
extern "C" fn task_usb_hid_poll() -> ! {
    loop {
        drivers::usb_hid::poll();
        unsafe { core::arch::asm!("sti; hlt", options(nomem, nostack)); }
    }
}

/// Ask Ollama to generate a reply to `prompt` over a real TCP connection.
/// Falls back to a diagnostic string (never panics) if the network stack
/// isn't up, the connection fails, or the reply can't be parsed — the shell
/// and IPC round-trip must keep working even with Ollama unreachable.
#[cfg(target_arch = "x86_64")]
fn ai_generate(prompt: &str) -> alloc::string::String {
    use smoltcp::wire::Ipv4Address;

    // QEMU user-mode networking gateway; also the host's address as seen from
    // a real bridged/NAT'd interface in most other setups.
    const OLLAMA_IP: Ipv4Address = Ipv4Address::new(10, 0, 2, 2);
    const OLLAMA_PORT: u16 = 11434;
    const TIMEOUT_MS: u64 = 30_000;

    // Escape the prompt minimally for JSON (quotes/backslashes/control chars).
    let mut escaped = alloc::string::String::with_capacity(prompt.len());
    for c in prompt.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => {}
            c if (c as u32) < 0x20 => {}
            c => escaped.push(c),
        }
    }

    let body = alloc::format!(
        "{{\"model\":\"llama3.2:3b\",\"prompt\":\"{}\",\"stream\":false}}",
        escaped
    );
    let request = alloc::format!(
        "POST /api/generate HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        OLLAMA_IP, OLLAMA_PORT, body.len(), body
    );

    let response = match net::tcp_request(OLLAMA_IP, OLLAMA_PORT, request.as_bytes(), TIMEOUT_MS) {
        Ok(bytes) => bytes,
        Err(e) => return alloc::format!("[nexus-ai] Ollama unreachable: {}", e),
    };

    let text = match core::str::from_utf8(&response) {
        Ok(s) => s,
        Err(_) => return alloc::string::String::from("[nexus-ai] Ollama replied with invalid UTF-8"),
    };

    // Split HTTP headers from body (blank-line separator), then pull the
    // "response" field out of Ollama's JSON with a small hand-rolled scan —
    // no serde in this no_std kernel binary.
    let json = match text.split_once("\r\n\r\n") {
        Some((_headers, body)) => body,
        None => text,
    };
    match extract_json_string_field(json, "response") {
        Some(s) => s,
        None => alloc::format!("[nexus-ai] unexpected Ollama reply: {}",
            if json.len() > 200 { &json[..200] } else { json }),
    }
}

/// Find `"key":"value"` in a JSON object and return `value` with `\n`/`\"`/`\\`
/// escapes undone. Good enough for Ollama's flat, single-line response field;
/// does not handle nested objects/arrays in the value.
#[cfg(target_arch = "x86_64")]
fn extract_json_string_field(json: &str, key: &str) -> Option<alloc::string::String> {
    let needle = alloc::format!("\"{}\"", key);
    let key_pos = json.find(&needle)?;
    let after_key = &json[key_pos + needle.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    if !after_colon.starts_with('"') {
        return None;
    }
    let value_start = &after_colon[1..];
    let mut out = alloc::string::String::new();
    let mut chars = value_start.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other),
                None => break,
            },
            c => out.push(c),
        }
    }
    None
}

// ─── Global allocator error handler ──────────────────────────────────────────

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Kernel heap allocation failed: size={} align={}",
           layout.size(), layout.align())
}
