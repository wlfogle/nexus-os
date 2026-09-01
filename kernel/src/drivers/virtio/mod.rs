//! NexusOS VirtIO Common Layer
//!
//! Implements the VirtIO legacy (pre-1.0) I/O port interface.
//! The I/O base comes from PCI BAR0.

pub mod blk;
pub mod net;
pub mod gpu;

use x86_64::instructions::port::Port;

// ─── VirtIO legacy register offsets (from BAR0) ──────────────────────────────

pub const REG_DEVICE_FEATURES: u16 = 0x00; // R  u32
pub const REG_DRIVER_FEATURES: u16 = 0x04; // W  u32
pub const REG_QUEUE_ADDRESS:   u16 = 0x08; // W  u32  (physical page number)
pub const REG_QUEUE_SIZE:      u16 = 0x0C; // R  u16
pub const REG_QUEUE_SELECT:    u16 = 0x0E; // W  u16
pub const REG_QUEUE_NOTIFY:    u16 = 0x10; // W  u16
pub const REG_DEVICE_STATUS:   u16 = 0x12; // RW u8
pub const REG_ISR_STATUS:      u16 = 0x13; // R  u8  (read clears)
// Device-specific config starts at offset 0x14 in legacy mode (MSI-X disabled).
// blk config:
pub const REG_BLK_CAPACITY:    u16 = 0x14; // R  u64 (lo u32 at 0x14, hi u32 at 0x18)
// net config: 6-byte MAC at 0x14 (valid when VIRTIO_NET_F_MAC negotiated),
// link status u16 at 0x1A (valid when VIRTIO_NET_F_STATUS negotiated).
pub const REG_NET_MAC:         u16 = 0x14; // R  [u8; 6]
pub const REG_NET_STATUS:      u16 = 0x1A; // R  u16

// ─── Device status bits ───────────────────────────────────────────────────────

pub const STATUS_RESET:       u8 = 0x00;
pub const STATUS_ACKNOWLEDGE: u8 = 0x01;
pub const STATUS_DRIVER:      u8 = 0x02;
pub const STATUS_DRIVER_OK:   u8 = 0x04;
pub const STATUS_FAILED:      u8 = 0x80;

// ─── VirtIO feature bits (blk) ────────────────────────────────────────────────

pub const VIRTIO_BLK_F_RO:     u32 = 1 << 5;  // disk is read-only
pub const VIRTIO_BLK_F_FLUSH:  u32 = 1 << 9;  // supports flush

// ─── VirtIO feature bits (net) + transport ───────────────────────────────────

pub const VIRTIO_NET_F_MAC:        u32 = 1 << 5;  // device provides a MAC in config
pub const VIRTIO_NET_F_MRG_RXBUF:  u32 = 1 << 15; // mergeable RX buffers (12-byte hdr)
pub const VIRTIO_NET_F_STATUS:     u32 = 1 << 16; // config status field is valid
pub const VIRTIO_RING_F_EVENT_IDX: u32 = 1 << 29; // used/avail event index in ring

/// Read the 6-byte MAC address from a VirtIO-net device's legacy config space.
pub fn net_mac(base: u16) -> [u8; 6] {
    let mut mac = [0u8; 6];
    for (i, b) in mac.iter_mut().enumerate() {
        *b = read8(base, REG_NET_MAC + i as u16);
    }
    mac
}

// ─── I/O helpers ─────────────────────────────────────────────────────────────

#[inline] pub fn read8(base: u16, reg: u16)  -> u8  { unsafe { Port::<u8>::new(base + reg).read()  } }
#[inline] pub fn read16(base: u16, reg: u16) -> u16 { unsafe { Port::<u16>::new(base + reg).read() } }
#[inline] pub fn read32(base: u16, reg: u16) -> u32 { unsafe { Port::<u32>::new(base + reg).read() } }

#[inline] pub fn write8(base: u16, reg: u16, v: u8)  { unsafe { Port::<u8>::new(base + reg).write(v)  } }
#[inline] pub fn write16(base: u16, reg: u16, v: u16) { unsafe { Port::<u16>::new(base + reg).write(v) } }
#[inline] pub fn write32(base: u16, reg: u16, v: u32) { unsafe { Port::<u32>::new(base + reg).write(v) } }

/// Read the device status register.
pub fn status(base: u16) -> u8  { read8(base, REG_DEVICE_STATUS) }

/// Write the device status register.
pub fn set_status(base: u16, s: u8) { write8(base, REG_DEVICE_STATUS, s) }

/// Reset the device and ack it.
pub fn reset_and_ack(base: u16) {
    set_status(base, STATUS_RESET);      // reset
    set_status(base, STATUS_ACKNOWLEDGE); // guest OS acknowledged
    set_status(base, STATUS_ACKNOWLEDGE | STATUS_DRIVER); // driver loaded
}

/// Read the disk capacity in sectors (512 B each).
pub fn blk_capacity(base: u16) -> u64 {
    let lo = read32(base, REG_BLK_CAPACITY) as u64;
    let hi = read32(base, REG_BLK_CAPACITY + 4) as u64;
    lo | (hi << 32)
}
