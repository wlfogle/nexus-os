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
pub mod drivers;
pub mod fs;
pub mod installer;
pub mod io;
pub mod ipc;
pub mod memory;
pub mod panic;
pub mod process;
pub mod scheduler;
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

/// Original kernel ELF file — used by the installer to write the correct
/// binary to the installed disk (rather than a raw memory image).
#[used]
#[link_section = ".limine_requests"]
static KFILE_REQUEST: KernelFileRequest = KernelFileRequest::new(0);

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

    // Store original kernel ELF pointer + size for the installer task.
    // KernelFileRequest gives us the exact bytes Limine read from disk, so
    // the installer writes a proper ELF rather than a raw memory image.
    // KernelFileRequest: Ptr<T>::get() → Option<&T>, Ptr<u8>::as_ptr() → Option<*mut u8>
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

    // ── 7.5. VirtIO-blk disk driver (after framebuffer so output is visible) ──
    // VirtIO vendor 0x1AF4; device 0x1001 = legacy blk, 0x1042 = transitional
    const VIRTIO_VENDOR: u16 = 0x1AF4;
    match drivers::pci::find(&[(VIRTIO_VENDOR, 0x1001), (VIRTIO_VENDOR, 0x1042)]) {
        Some(dev) => {
            kprintln!("[disk] PCI {:02x}:{:02x}.{} vendor={:#06x} device={:#06x} BAR0={:#010x}",
                      dev.bus, dev.dev, dev.func,
                      dev.vendor_id, dev.device_id, dev.bar0);
            if dev.bar0 & 1 == 0 {
                kprintln!("[disk] BAR0 is MMIO (not I/O port) — legacy driver incompatible");
                kprintln!("[disk] MMIO addr={:#010x} — needs MMIO VirtIO transport",
                          dev.bar0 & !0xF);
            } else {
                dev.enable_io_and_busmaster();
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

    // ── 7.6. FAT32 filesystem ─────────────────────────────────────────────────
    // Must run after framebuffer (to display the [fs] line) and after disk driver.
    let fs_msg = fs::fat::init();
    kprintln!("[fs]   {}", fs_msg);

    // ── 8. Feature-specific init ─────────────────────────────────────────────
    #[cfg(feature = "ai-hooks")]
    {
        kprintln!("[ai]   AI interface hooks registered — ready for AI Core server");
    }

    #[cfg(feature = "server-mode")]
    {
        kprintln!("[srv]  Server mode: headless, service management hooks active");
    }

    // ── 9. Phase 2: PIC + PIT + Scheduler ───────────────────────────────
    timer::init();                          // PIC remap + PIT 100 Hz
    kprintln!("[timer] PIC remapped, PIT running at {} Hz", timer::TIMER_HZ);

    scheduler::init();                      // register idle process

    // ── Phase 3: IPC echo server + client ────────────────────────────
    scheduler::spawn(b"echo-server", task_echo_server)
        .expect("failed to spawn echo-server");
    scheduler::spawn(b"echo-client", task_echo_client)
        .expect("failed to spawn echo-client");
    kprintln!("[ipc]  echo-server and echo-client spawned");

    // ── Phase 4: Syscall interface + user-space process ───────────────
    syscall::init();
    let user_pid = userspace::spawn_user_init();
    kprintln!("[user] nexus-init spawned as pid={} (ring 3)", user_pid);

    // ── Phase 5: AI Core kernel thread ──────────────────
    scheduler::spawn(b"nexus-ai", task_nexus_ai)
        .expect("failed to spawn nexus-ai");
    kprintln!("[ai]   nexus-ai AI Core daemon spawned");
    scheduler::spawn(b"kbd-echo", task_keyboard_echo)
        .expect("failed to spawn kbd-echo");
    kprintln!("[kbd]  keyboard echo task spawned");

    // ── NexusOS Installer ───────────────────────────────────────────────
    // Runs only when disk is unformatted (first boot from ISO).
    if !fs::fat::is_mounted() {
        scheduler::spawn(b"installer", installer::task_installer)
            .expect("failed to spawn installer");
        kprintln!("[inst] NexusOS Installer spawned");
    }

    // Enable hardware interrupts — timer fires immediately
    arch::enable_interrupts();
    kprintln!("[arch] Interrupts enabled — scheduler is LIVE");
    kprintln!();
    kprintln!("NexusOS v{} — Phase 5: AI Core + PS/2 keyboard active.",
              env!("CARGO_PKG_VERSION"));
    kprintln!("[kbd]  PS/2 keyboard online — type to interact");

    // Idle loop — preempted every 10 ms
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

// ─── Phase 5: Keyboard echo kernel task ──────────────────────────────────────────

/// Kernel thread that echoes keystrokes to serial + framebuffer.
/// Proves the full ring-0 keyboard input pipeline:
///   IRQ1 fires → scancode translated → key buffered → BlockedOnKey process woken
///   → this task reads the char → prints it
extern "C" fn task_keyboard_echo() -> ! {
    kprintln!("[kbd]  keyboard echo task running — type something!");
    loop {
        // Block until a key arrives (IRQ1 wakes us)
        while !io::keyboard::has_key() {
            process::set_state(scheduler::current_id(),
                               process::ProcessState::BlockedOnKey);
            unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
            process::set_state(scheduler::current_id(),
                               process::ProcessState::Running);
        }
        if let Some(ch) = io::keyboard::try_read() {
            match ch {
                b'\n' | 13 => kprintln!(),
                8          => kprint!("\x08 \x08"),  // backspace: erase
                27         => kprintln!("[ESC]"),
                c if c >= 32 => kprint!("{}", c as char),
                _          => {}
            }
        }
    }
}

// ─── Phase 5: AI Core kernel thread ────────────────────────────────────────────

/// AI Core daemon — registers the `nexus.ai` port and serves inference requests.
///
/// Phase 5.0: responds with a structured mock reply so the IPC pipeline and
/// port-discovery path are fully exercised.  The Ollama HTTP client that sends
/// real prompts ships in Phase 5.1 once the network stack is available.
extern "C" fn task_nexus_ai() -> ! {
    use ipc::{ipc_recv, ipc_send, Message, ANY, MSG_AI_RESPONSE};
    use ipc::ports::port_register;

    port_register(b"nexus.ai").expect("nexus-ai: failed to register port");
    kprintln!("[nexus-ai] AI Core online — port 'nexus.ai' registered");
    kprintln!("[nexus-ai] Phase 5.0: mock inference active (Ollama HTTP in Phase 5.1)");

    let mut req = Message::new(0, 0);
    loop {
        ipc_recv(ANY, &mut req).expect("nexus-ai: recv failed");

        let query = req.as_str();
        kprintln!("[nexus-ai] request from pid={}: {}", req.from, query);

        // Phase 5.0: canned response so IPC round-trip is proven.
        // Phase 5.1: issue HTTP POST to http://localhost:11434/api/generate
        let reply_text = alloc::format!(
            "AI Core v0.5 [Phase 5.0]: received '{}' \
             (Ollama HTTP client ships in Phase 5.1)",
            if query.len() > 40 { &query[..40] } else { query }
        );
        let reply = Message::with_str(req.from, MSG_AI_RESPONSE, &reply_text);
        ipc_send(req.from, reply).ok();
        kprintln!("[nexus-ai] response sent to pid={}", req.from);
    }
}

// ─── Kernel tasks (Phase 3 IPC demo) ──────────────────────────────────────────────

/// Echo server — registers port "nexus.echo", receives any message, echoes back.
/// This is the prototype for AI Core, filesystem, and network servers.
extern "C" fn task_echo_server() -> ! {
    use ipc::{ipc_recv, ipc_send, Message, ANY, MSG_PONG};
    use ipc::ports::port_register;

    port_register(b"nexus.echo").expect("echo-server: port register failed");
    kprintln!("[echo-server] registered port 'nexus.echo'");

    let mut req = Message::new(0, 0);
    loop {
        // Block until a message arrives from any sender
        ipc_recv(ANY, &mut req).expect("echo-server: recv failed");

        kprintln!("[echo-server] got MSG {:04x} from pid={} len={}: {}",
                  req.msg_type, req.from, req.len, req.as_str());

        // Echo back: type PONG, same payload
        let reply = Message::with_str(req.from, MSG_PONG, req.as_str());
        ipc_send(req.from, reply).ok();
    }
}

/// Echo client — looks up the echo server by port, sends pings every 2 seconds.
extern "C" fn task_echo_client() -> ! {
    use ipc::{ipc_recv, ipc_send, Message, MSG_PING};
    use ipc::ports::port_find;

    // Wait a moment for the server to register its port
    for _ in 0..50 {
        unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
    }

    let mut seq: u32 = 0;
    let mut reply = Message::new(0, 0);

    loop {
        // Find the echo server's PID via the port registry
        let server_pid = loop {
            if let Some(pid) = port_find(b"nexus.echo") {
                break pid;
            }
            unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
        };

        // Build ping with a sequence number in the payload
        seq = seq.wrapping_add(1);
        // Write seq number as text into a small buffer
        let mut buf = [0u8; 16];
        let s = seq;
        let digits = [
            b'p', b'i', b'n', b'g', b'#',
            b'0' + ((s / 10000) % 10) as u8,
            b'0' + ((s / 1000)  % 10) as u8,
            b'0' + ((s / 100)   % 10) as u8,
            b'0' + ((s / 10)    % 10) as u8,
            b'0' + (s % 10)           as u8,
        ];
        buf[..digits.len()].copy_from_slice(&digits);
        let text = core::str::from_utf8(&buf[..digits.len()]).unwrap_or("?");

        let ping = Message::with_str(server_pid, MSG_PING, text);
        kprintln!("[echo-client] -> pid={} MSG_PING '{}'", server_pid, text);

        ipc_send(server_pid, ping).expect("echo-client: send failed");

        // Wait for the echo reply
        ipc_recv(server_pid, &mut reply).expect("echo-client: recv failed");
        kprintln!("[echo-client] <- MSG_PONG '{}' (round-trip OK)", reply.as_str());

        // Pause ~2 seconds (200 ticks at 100 Hz)
        let start = timer::ticks();
        while timer::ticks().wrapping_sub(start) < 200 {
            unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
        }
    }
}

// ─── Global allocator error handler ──────────────────────────────────────────

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Kernel heap allocation failed: size={} align={}",
           layout.size(), layout.align())
}
