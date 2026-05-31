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
pub mod ipc;
pub mod memory;
pub mod panic;
pub mod process;
pub mod scheduler;
pub mod timer;

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

    // Enable hardware interrupts — timer fires immediately
    arch::enable_interrupts();
    kprintln!("[arch] Interrupts enabled — scheduler is LIVE");
    kprintln!();
    kprintln!("NexusOS v{} — Phase 3: IPC message-passing active.",
              env!("CARGO_PKG_VERSION"));

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

// ─── Kernel tasks (Phase 3 IPC demo) ──────────────────────────────────────────────

/// Echo server — registers port "nexus.echo", receives any message, echoes back.
/// This is the prototype for AI Core, filesystem, and network servers.
extern "C" fn task_echo_server() -> ! {
    use ipc::{ipc_recv, ipc_send, Message, ANY, MSG_PING, MSG_PONG};
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
    use ipc::{ipc_recv, ipc_send, Message, ANY, MSG_PING, MSG_PONG};
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
