//! NexusOS Network Stack (Phase 7)
//!
//! Brings up the legacy VirtIO-net device and runs a minimal no_std TCP/IP
//! path via smoltcp.  At boot we discover the NIC, initialise the driver,
//! build a smoltcp `Interface`, and drive a DHCPv4 client to demonstrate a
//! real packet round-trip (DHCP DISCOVER/OFFER/REQUEST/ACK, plus the ARP the
//! stack performs along the way) — all logged via `kprintln!`.
//!
//! To exercise this under QEMU the VM must expose a legacy VirtIO-net device,
//! e.g. add to the `qemu-system-x86_64` command line:
//!   -netdev user,id=n0 -device virtio-net-pci,netdev=n0,disable-modern=on
//! QEMU's built-in SLIRP DHCP server (10.0.2.x, gateway 10.0.2.2) answers the
//! DISCOVER, completing the round-trip without any external network.

pub mod device;

use alloc::vec::Vec;
use spin::Mutex;
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::socket::{dhcpv4, tcp};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, IpEndpoint, Ipv4Address};
use crate::drivers::{pci, virtio};
use crate::timer;
use device::NetDevice;

/// Persistent network state, kept alive past `init()` so later kernel code
/// (e.g. the AI Core's Ollama client) can open TCP sockets against the same
/// interface. Everything here is kernel-internal; there is no syscall surface.
struct NetState {
    iface: Interface,
    device: NetDevice,
    sockets: SocketSet<'static>,
}

static NET_STATE: Mutex<Option<NetState>> = Mutex::new(None);

/// Ephemeral local port counter for outgoing TCP connections.
static NEXT_LOCAL_PORT: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(49152);

fn alloc_local_port() -> u16 {
    use core::sync::atomic::Ordering;
    let p = NEXT_LOCAL_PORT.fetch_add(1, Ordering::Relaxed);
    if p < 49152 { 49152 } else { p }
}

/// VirtIO PCI vendor ID.
const VIRTIO_VENDOR: u16 = 0x1AF4;
/// Legacy and modern VirtIO-net PCI device IDs.
const VIRTIO_NET_LEGACY: u16 = 0x1000;
const VIRTIO_NET_MODERN: u16 = 0x1041;

/// How long to drive the DHCP client before giving up (milliseconds).
const DHCP_TIMEOUT_MS: u64 = 15_000;

/// Discover and bring up the network device, then run the DHCP demo.
///
/// This must be called *after* interrupts are enabled, so the PIT-driven
/// millisecond clock (`timer::millis`) advances and smoltcp's retransmit
/// timers make progress.
pub fn init() {
    // ── Discover a VirtIO-net device on the PCI bus ──────────────────────────
    let mut dev = match pci::find(&[
        (VIRTIO_VENDOR, VIRTIO_NET_LEGACY),
        (VIRTIO_VENDOR, VIRTIO_NET_MODERN),
    ]) {
        Some(d) => d,
        None => {
            crate::kprintln!("[net]  no VirtIO-net device found");
            return;
        }
    };

    // Enable I/O space + bus-master (DMA) before re-reading BAR0.
    dev.enable_io_and_busmaster();
    dev.bar0 = pci::read32(dev.bus, dev.dev, dev.func, 0x10);
    crate::kprintln!(
        "[net]  PCI {:02x}:{:02x}.{} vendor={:#06x} device={:#06x} BAR0={:#010x}",
        dev.bus, dev.dev, dev.func, dev.vendor_id, dev.device_id, dev.bar0);

    if dev.bar0 & 1 == 0 {
        crate::kprintln!("[net]  BAR0 is MMIO — legacy I/O-port driver incompatible; skipping");
        return;
    }

    // ── Initialise the driver ────────────────────────────────────────────────
    let mac = match virtio::net::init(dev.io_base()) {
        Ok(mac) => mac,
        Err(e) => {
            crate::kprintln!("[net]  VirtIO-net init failed: {}", e);
            return;
        }
    };
    crate::kprintln!(
        "[net]  VirtIO-net up: MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

    // ── Build the smoltcp interface ──────────────────────────────────────────
    let mut nic = NetDevice;
    let mut config = Config::new(HardwareAddress::Ethernet(EthernetAddress(mac)));
    // Seed doesn't need to be secure, just varied across boots.
    config.random_seed = timer::ticks()
        ^ ((mac[5] as u64) << 8 | mac[4] as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);

    let now = Instant::from_millis(timer::millis() as i64);
    let mut iface = Interface::new(config, &mut nic, now);

    // ── DHCPv4 client socket ─────────────────────────────────────────────────
    let mut sockets = SocketSet::new(alloc::vec::Vec::new());
    let dhcp_handle = sockets.add(dhcpv4::Socket::new());

    crate::kprintln!("[net]  DHCP: discovering (DISCOVER → OFFER → REQUEST → ACK)...");

    // ── Bounded poll loop ────────────────────────────────────────────────────
    let start = timer::millis();
    let mut last_log = start;
    let mut bound = false;

    while timer::millis().wrapping_sub(start) < DHCP_TIMEOUT_MS {
        let ts = Instant::from_millis(timer::millis() as i64);
        iface.poll(ts, &mut nic, &mut sockets);

        match sockets.get_mut::<dhcpv4::Socket>(dhcp_handle).poll() {
            Some(dhcpv4::Event::Configured(cfg)) => {
                let (rx, tx) = virtio::net::stats();
                crate::kprintln!(
                    "[net]  DHCP bound: ip={} (frames rx={} tx={})", cfg.address, rx, tx);
                if let Some(router) = cfg.router {
                    crate::kprintln!("[net]  gateway: {}", router);
                }
                for (i, dns) in cfg.dns_servers.iter().enumerate() {
                    crate::kprintln!("[net]  dns[{}]: {}", i, dns);
                }

                iface.update_ip_addrs(|addrs| {
                    addrs.clear();
                    let _ = addrs.push(IpCidr::Ipv4(cfg.address));
                });
                if let Some(router) = cfg.router {
                    let _ = iface.routes_mut().add_default_ipv4_route(router);
                }
                bound = true;
                break;
            }
            Some(dhcpv4::Event::Deconfigured) => {
                crate::kprintln!("[net]  DHCP: configuration lost");
            }
            None => {}
        }

        // Periodic progress so a silent network is visible in the boot log.
        let nowms = timer::millis();
        if nowms.wrapping_sub(last_log) >= 3_000 {
            let (rx, tx) = virtio::net::stats();
            crate::kprintln!("[net]  DHCP: waiting... (frames rx={} tx={})", rx, tx);
            last_log = nowms;
        }

        // Brief pause between polls; the timer IRQ still advances the clock.
        for _ in 0..20_000 {
            core::hint::spin_loop();
        }
    }

    if !bound {
        let (rx, tx) = virtio::net::stats();
        crate::kprintln!(
            "[net]  DHCP timed out after {} ms (frames rx={} tx={}) — no DHCP server?",
            DHCP_TIMEOUT_MS, rx, tx);
    }

    // Keep the interface, device, and socket set alive past this function so
    // later kernel code (e.g. the AI Core's Ollama client) can open TCP
    // sockets against the same interface via `tcp_request`.
    *NET_STATE.lock() = Some(NetState { iface, device: nic, sockets });
}

/// Send `request` over a new TCP connection to `ip:port` and return whatever
/// bytes the peer sends back before closing (or before `timeout_ms` elapses).
///
/// Blocking: drives the interface's poll loop itself, matching the DHCP
/// bring-up above. Intended to be called from kernel-mode tasks (e.g.
/// `task_nexus_ai`) — there is no syscall wrapper since callers already run
/// in the kernel.
pub fn tcp_request(
    ip: Ipv4Address,
    port: u16,
    request: &[u8],
    timeout_ms: u64,
) -> Result<Vec<u8>, &'static str> {
    let mut guard = NET_STATE.lock();
    let state = guard.as_mut().ok_or("net: interface not initialised")?;

    let rx_buffer = tcp::SocketBuffer::new(alloc::vec![0u8; 4096]);
    let tx_buffer = tcp::SocketBuffer::new(alloc::vec![0u8; 4096]);
    let mut socket = tcp::Socket::new(rx_buffer, tx_buffer);

    let remote = IpEndpoint::new(IpAddress::Ipv4(ip), port);
    let local_port = alloc_local_port();
    socket
        .connect(state.iface.context(), remote, local_port)
        .map_err(|_| "net: tcp connect failed")?;

    let handle: SocketHandle = state.sockets.add(socket);

    let start = timer::millis();
    let mut sent = false;
    let mut response: Vec<u8> = Vec::new();

    let result = loop {
        let ts = Instant::from_millis(timer::millis() as i64);
        state.iface.poll(ts, &mut state.device, &mut state.sockets);

        let socket = state.sockets.get_mut::<tcp::Socket>(handle);

        if !sent && socket.can_send() {
            if socket.send_slice(request).is_ok() {
                sent = true;
            }
        }

        while socket.can_recv() {
            let mut buf = [0u8; 512];
            match socket.recv_slice(&mut buf) {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buf[..n]),
                Err(_) => break,
            }
        }

        if sent && !socket.may_recv() && !response.is_empty() {
            break Ok(response);
        }

        if timer::millis().wrapping_sub(start) > timeout_ms {
            break Err("net: tcp_request timed out");
        }

        if socket.state() == tcp::State::Closed && !sent {
            break Err("net: connection closed before request could be sent");
        }

        for _ in 0..2_000 {
            core::hint::spin_loop();
        }
    };

    state.sockets.remove(handle);
    result
}
