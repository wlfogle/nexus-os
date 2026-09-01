//! AArch64 Platform Detection — Bahamut Increment B1
//!
//! Distinguishes QEMU's `virt` machine from real Raspberry Pi 4 (BCM2711)
//! hardware at boot, the same "discover, don't guess" principle used for
//! x86_64 ACPI table parsing (see `crate::acpi`). Every later Bahamut driver
//! increment (B2 real UART0, B3 GIC-400, B4 EMMC2) needs to know which base
//! addresses to program, so this has to land first.
//!
//! # Method
//! Limine hands us a pointer to the flattened device tree (FDT/DTB) via
//! `DtbRequest`. We walk its structure block (Devicetree Specification v0.4
//! §5.4) only as far as the root node's own `compatible` property -- a
//! hand-rolled parser (matching this codebase's existing precedent of not
//! pulling in a crate for ACPI/etc), since that's all platform detection
//! needs and the format is a simple, fixed, versioned binary layout.
//!
//! # Verification methodology
//! Before this ever ran as kernel code, the exact same parsing algorithm
//! (`parse_root_compatible`/`classify_compatible` below, operating on a
//! plain `&[u8]` with no unsafe code) was compiled as a small host-side Rust
//! program and run against two *real* device tree blobs, replacing every
//! assumption with a verified fact:
//!   - `qemu-system-aarch64 -M virt -machine dumpdtb=...` (QEMU's actual
//!     generated DTB for the `virt` board): root `compatible` is exactly
//!     `"linux,dummy-virt"` (a single string, confirmed via `dtc`).
//!   - The real `bcm2711-rpi-4-b.dtb` shipped by the upstream
//!     `raspberrypi/firmware` repository (the literal file real Pi4 GPU
//!     firmware loads and passes to the OS at boot): root `compatible` is
//!     `"raspberrypi,4-model-b\0brcm,bcm2711"` (two NUL-separated strings,
//!     confirmed via `dtc`).
//! The host test also confirmed graceful `None` handling (no panics) against
//! garbage, truncated, and empty input, since a panic here would crash boot
//! before any diagnostic output could even be printed.
//!
//! Note: QEMU's `raspi4b` machine does *not* synthesize its own DTB the way
//! `virt` does (confirmed: `-machine raspi4b,dumpdtb=...` reports "this
//! machine doesn't have an FDT" on QEMU versions with the newer, clearer
//! error message, and simply hangs on others) -- real Pi4 boot (and QEMU's
//! `raspi4b` emulation of it) expects an externally-supplied DTB file,
//! matching real hardware exactly rather than a QEMU-synthesized
//! approximation. This module doesn't care how the DTB got to Limine, only
//! that Limine hands us a pointer to one.

use core::sync::atomic::{AtomicU8, Ordering};

/// Detected hardware platform.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Platform {
    /// QEMU's `virt` board (root `compatible` = `"linux,dummy-virt"`).
    QemuVirt,
    /// Real Raspberry Pi 4 / BCM2711 (root `compatible` includes
    /// `"raspberrypi,4-model-b"`).
    RaspberryPi4,
    /// No DTB, or one reporting a board this kernel doesn't recognise yet.
    Unknown,
}

const P_UNINIT: u8 = 0;
const P_QEMU_VIRT: u8 = 1;
const P_RPI4: u8 = 2;
const P_UNKNOWN: u8 = 3;

/// Cached result of `detect()`, so later driver increments (B2+) can query
/// `current()` cheaply from anywhere without re-parsing the DTB.
static DETECTED: AtomicU8 = AtomicU8::new(P_UNINIT);

fn encode(p: Platform) -> u8 {
    match p {
        Platform::QemuVirt => P_QEMU_VIRT,
        Platform::RaspberryPi4 => P_RPI4,
        Platform::Unknown => P_UNKNOWN,
    }
}

fn decode(v: u8) -> Platform {
    match v {
        P_QEMU_VIRT => Platform::QemuVirt,
        P_RPI4 => Platform::RaspberryPi4,
        _ => Platform::Unknown,
    }
}

/// Detect the platform from Limine's `DtbRequest` pointer (already a usable
/// pointer per the Limine protocol, same convention as `KernelFileRequest`'s
/// `base` pointer elsewhere in this codebase). Caches the result: safe and
/// cheap to call more than once. Logs the outcome either way.
pub fn detect(dtb_ptr: u64) -> Platform {
    let result = parse_root_compatible(dtb_ptr).unwrap_or(Platform::Unknown);
    DETECTED.store(encode(result), Ordering::Relaxed);
    match result {
        Platform::QemuVirt => crate::kprintln!("[platform] detected: QEMU virt"),
        Platform::RaspberryPi4 => crate::kprintln!("[platform] detected: real Raspberry Pi 4 (BCM2711)"),
        Platform::Unknown => crate::kprintln!("[platform] detected: unknown (no/unrecognised DTB) -- falling back to QEMU virt addresses"),
    }
    result
}

/// The most recently detected platform. Returns `Platform::Unknown` if
/// `detect()` has not run yet this boot.
#[allow(dead_code)] // consumed starting with Increment B2's UART bring-up
pub fn current() -> Platform {
    decode(DETECTED.load(Ordering::Relaxed))
}

// ─── FDT structure-block tokens (Devicetree Specification v0.4 §5.4.1) ───────

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4;
const FDT_END: u32 = 0x9;
const FDT_MAGIC: u32 = 0xd00d_feed;

/// Sanity bound on `totalsize` before we ever construct a slice covering
/// the whole blob -- real DTBs are a few tens of KiB; this is generously
/// large while still rejecting a garbage/corrupt pointer's header.
const MAX_DTB_SIZE: usize = 16 * 1024 * 1024;

#[inline]
fn align4(x: usize) -> usize {
    (x + 3) & !3
}

fn be32(dtb: &[u8], off: usize) -> Option<u32> {
    let b = dtb.get(off..off + 4)?;
    Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn cstr_at(dtb: &[u8], off: usize, limit: usize) -> Option<&str> {
    if off >= limit {
        return None;
    }
    let slice = &dtb[off..limit];
    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    core::str::from_utf8(&slice[..end]).ok()
}

/// The `compatible` property is one or more NUL-separated strings,
/// most-specific first (matches how the Linux kernel itself resolves
/// `compatible`). A match on *any* of them is sufficient.
fn classify_compatible(dtb: &[u8], value_off: usize, len: usize) -> Platform {
    let value = match dtb.get(value_off..value_off + len) {
        Some(v) => v,
        None => return Platform::Unknown,
    };
    let mut start = 0usize;
    while start < len {
        let end = value[start..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| start + p)
            .unwrap_or(len);
        if let Ok(s) = core::str::from_utf8(&value[start..end]) {
            crate::kprintln!("[platform] DTB root compatible: \"{}\"", s);
            if s == "raspberrypi,4-model-b" {
                return Platform::RaspberryPi4;
            }
            if s == "linux,dummy-virt" || s == "qemu,virt" {
                return Platform::QemuVirt;
            }
        }
        start = end + 1;
    }
    Platform::Unknown
}

/// Walk the DTB structure block only far enough to read the root node's own
/// `compatible` property (name `""`) -- does not recurse into children,
/// since the root node's own properties always precede its first child
/// node in the structure block per the spec.
///
/// # Safety-relevant notes
/// `dtb_ptr` is trusted (Limine-provided), matching how every other
/// Limine-response-derived pointer is treated in this codebase (e.g. ACPI
/// table parsing). The only two raw-pointer reads are: (1) the fixed-size
/// 16-byte header, to learn `totalsize`, and (2) constructing a `&[u8]`
/// slice of exactly that many bytes for everything else -- which is 100%
/// safe Rust from that point on, and the exact same algorithm already
/// verified host-side against real DTB files (see module docs).
fn parse_root_compatible(dtb_ptr: u64) -> Option<Platform> {
    if dtb_ptr == 0 {
        return None;
    }

    let header: &[u8] = unsafe { core::slice::from_raw_parts(dtb_ptr as *const u8, 16) };
    let magic = be32(header, 0)?;
    if magic != FDT_MAGIC {
        crate::kprintln!("[platform] DTB magic mismatch ({:#010x}) -- no valid device tree", magic);
        return None;
    }
    let totalsize = be32(header, 4)? as usize;
    if totalsize == 0 || totalsize > MAX_DTB_SIZE {
        crate::kprintln!("[platform] DTB totalsize implausible ({} bytes)", totalsize);
        return None;
    }
    let off_dt_struct = be32(header, 8)? as usize;
    let off_dt_strings = be32(header, 12)? as usize;
    if off_dt_struct >= totalsize || off_dt_strings >= totalsize {
        crate::kprintln!("[platform] DTB header offsets out of range");
        return None;
    }

    let dtb: &[u8] = unsafe { core::slice::from_raw_parts(dtb_ptr as *const u8, totalsize) };

    let mut pos = off_dt_struct;
    // Find the root node's FDT_BEGIN_NODE.
    loop {
        let tok = be32(dtb, pos)?;
        pos += 4;
        match tok {
            FDT_BEGIN_NODE => {
                // Skip the NUL-terminated name (root's is empty), pad to 4.
                while pos < totalsize && dtb[pos] != 0 {
                    pos += 1;
                }
                pos += 1;
                pos = align4(pos);
                break;
            }
            FDT_NOP => continue,
            _ => return None, // malformed: expected the root FDT_BEGIN_NODE first
        }
    }

    // Scan the root node's immediate properties for "compatible".
    loop {
        if pos + 4 > totalsize {
            return None;
        }
        let tok = be32(dtb, pos)?;
        pos += 4;
        match tok {
            FDT_PROP => {
                if pos + 8 > totalsize {
                    return None;
                }
                let len = be32(dtb, pos)? as usize;
                let nameoff = be32(dtb, pos + 4)? as usize;
                pos += 8;
                if pos + len > totalsize {
                    return None;
                }
                let name = cstr_at(dtb, off_dt_strings + nameoff, totalsize);
                if name == Some("compatible") {
                    return Some(classify_compatible(dtb, pos, len));
                }
                pos += align4(len);
            }
            FDT_NOP => continue,
            FDT_BEGIN_NODE | FDT_END_NODE | FDT_END => {
                // Root node has no "compatible" property, or DTB ended.
                return Some(Platform::Unknown);
            }
            _ => return None,
        }
    }
}
