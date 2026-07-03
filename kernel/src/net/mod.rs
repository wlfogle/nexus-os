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

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::socket::dhcpv4;
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpCidr};
use crate::drivers::{pci, virtio};
use crate::timer;
use device::NetDevice;

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
}
