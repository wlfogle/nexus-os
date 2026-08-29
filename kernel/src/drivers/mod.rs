//! NexusOS Device Drivers
//!
//! Phase 5 (v0.5.1): VirtIO-blk disk driver via PCI BAR0 legacy interface.
//! Phase 6 (storage): NVMe and AHCI (SATA) block drivers for real disks.

#[cfg(target_arch = "x86_64")]
pub mod pci;
#[cfg(target_arch = "x86_64")]
pub mod virtio;
#[cfg(target_arch = "x86_64")]
pub mod nvme;
#[cfg(target_arch = "x86_64")]
pub mod ahci;
#[cfg(target_arch = "x86_64")]
pub mod blockdev;
#[cfg(target_arch = "x86_64")]
pub mod xhci;
#[cfg(target_arch = "x86_64")]
pub mod usb_hid;

#[cfg(target_arch = "x86_64")]
use pci::PciDevice;

/// Scan the PCI bus for the first device matching a (class, subclass, prog-if)
/// triple.  Used by class-defined controllers (NVMe, AHCI) which are not tied
/// to a fixed vendor/device ID.
///
/// The PCI class code lives in configuration register 0x08:
///   bits 31:24 = base class, 23:16 = subclass, 15:8 = prog-if, 7:0 = revision.
#[cfg(target_arch = "x86_64")]
pub fn find_by_class(class: u8, subclass: u8, prog_if: u8) -> Option<PciDevice> {
    for bus in 0u8..=255 {
        for dev in 0u8..32 {
            // Skip empty slots (function 0 vendor == 0xFFFF).
            let hdr = pci::read32(bus, dev, 0, 0);
            if (hdr & 0xFFFF) as u16 == 0xFFFF {
                continue;
            }

            // Multi-function devices have bit 7 set in the header type byte.
            let header_type = pci::read16(bus, dev, 0, 0x0E) as u8;
            let max_func: u8 = if header_type & 0x80 != 0 { 8 } else { 1 };

            for func in 0..max_func {
                let vd = pci::read32(bus, dev, func, 0);
                let vendor = (vd & 0xFFFF) as u16;
                if vendor == 0xFFFF {
                    continue;
                }
                let device = ((vd >> 16) & 0xFFFF) as u16;

                let class_reg = pci::read32(bus, dev, func, 0x08);
                let base = ((class_reg >> 24) & 0xFF) as u8;
                let sub  = ((class_reg >> 16) & 0xFF) as u8;
                let pif  = ((class_reg >> 8)  & 0xFF) as u8;

                if base == class && sub == subclass && pif == prog_if {
                    let ss = pci::read32(bus, dev, func, 0x2C);
                    let subsystem_id = ((ss >> 16) & 0xFFFF) as u16;
                    let bar0 = pci::read32(bus, dev, func, 0x10);
                    return Some(PciDevice {
                        bus,
                        dev,
                        func,
                        vendor_id: vendor,
                        device_id: device,
                        subsystem_id,
                        bar0,
                    });
                }
            }
        }
    }
    None
}

/// Enable PCI memory space + bus-master for an MMIO device (command reg 0x04).
/// VirtIO's helper only enables *I/O* space; NVMe/AHCI need *memory* space.
#[cfg(target_arch = "x86_64")]
pub fn enable_mem_and_busmaster(d: &PciDevice) {
    let cmd = pci::read16(d.bus, d.dev, d.func, 0x04);
    // bit 1 = memory space enable, bit 2 = bus master enable
    pci::write32(d.bus, d.dev, d.func, 0x04, (cmd | 0x0006) as u32);
}

/// Read a 64-bit BAR pair starting at `bar_index` (0..=5), returning the
/// physical base address with the low flag/type bits masked off.  Handles both
/// 32-bit and 64-bit (type == 0b10) memory BARs.
#[cfg(target_arch = "x86_64")]
pub fn read_bar_addr(d: &PciDevice, bar_index: u8) -> u64 {
    let off = 0x10 + bar_index * 4;
    let low = pci::read32(d.bus, d.dev, d.func, off);
    // bit0 == 1 → I/O space (not expected for these controllers)
    if low & 1 != 0 {
        return (low & !0x3) as u64;
    }
    let is_64bit = (low >> 1) & 0x3 == 0x2;
    let base_lo = (low & !0xF) as u64;
    if is_64bit {
        let high = pci::read32(d.bus, d.dev, d.func, off + 4) as u64;
        base_lo | (high << 32)
    } else {
        base_lo
    }
}

/// Map an MMIO physical region and return a kernel virtual address to access
/// it.
///
/// Always maps explicitly, regardless of address: MMIO BARs frequently land in
/// the PCI hole just below 4 GiB (common on both QEMU's q35 machine type and
/// real PC/laptop chipsets), which is *not* backed by real RAM and therefore
/// not covered by Limine's HHDM mapping despite being < 4 GiB — a previous
/// version of this function assumed anything below 4 GiB was HHDM-covered and
/// skipped mapping it, which page-faulted the first time a device's BAR (e.g.
/// AHCI's ABAR) landed in that hole. `map_page`'s existing-mapping check makes
/// this safe to call unconditionally: addresses genuinely already covered by
/// HHDM (real RAM) hit the idempotent "already mapped to the same frame"
/// return path and are a no-op; addresses in an unmapped MMIO hole get a fresh
/// uncacheable mapping created.
#[cfg(target_arch = "x86_64")]
pub fn map_mmio(phys: u64, size: usize) -> u64 {
    use crate::memory::paging;
    let virt = paging::phys_to_virt(phys);
    let page_flags = paging::flags::PRESENT
        | paging::flags::WRITABLE
        | paging::flags::NO_CACHE
        | paging::flags::NO_EXECUTE;
    let start = phys & !0xFFF;
    let end = (phys + size as u64 + 0xFFF) & !0xFFF;
    let mut p = start;
    while p < end {
        paging::map_page(paging::phys_to_virt(p), p, page_flags);
        p += 0x1000;
    }
    virt
}
