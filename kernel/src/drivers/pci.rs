//! NexusOS PCI Bus Scanner
//!
//! Uses the legacy PCI I/O mechanism (ports 0xCF8/0xCFC) to enumerate
//! PCI devices and locate their BARs.

use x86_64::instructions::port::Port;

const PCI_ADDR: u16 = 0xCF8;
const PCI_DATA: u16 = 0xCFC;

/// Build a PCI configuration address dword.
#[inline]
fn cfg_addr(bus: u8, dev: u8, func: u8, reg: u8) -> u32 {
    0x8000_0000
        | ((bus  as u32) << 16)
        | ((dev  as u32) << 11)
        | ((func as u32) <<  8)
        | ((reg  as u32) &  0xFC)
}

/// Read a 32-bit dword from PCI configuration space.
pub fn read32(bus: u8, dev: u8, func: u8, reg: u8) -> u32 {
    unsafe {
        Port::<u32>::new(PCI_ADDR).write(cfg_addr(bus, dev, func, reg));
        Port::<u32>::new(PCI_DATA).read()
    }
}

/// Read a 16-bit word from PCI configuration space.
pub fn read16(bus: u8, dev: u8, func: u8, reg: u8) -> u16 {
    let dword = read32(bus, dev, func, reg & !3);
    (dword >> ((reg & 2) * 8)) as u16
}

/// Write a 32-bit dword to PCI configuration space.
pub fn write32(bus: u8, dev: u8, func: u8, reg: u8, val: u32) {
    unsafe {
        Port::<u32>::new(PCI_ADDR).write(cfg_addr(bus, dev, func, reg));
        Port::<u32>::new(PCI_DATA).write(val);
    }
}

/// Minimal description of a found PCI device.
#[derive(Clone, Copy, Debug)]
pub struct PciDevice {
    pub bus:  u8,
    pub dev:  u8,
    pub func: u8,
    pub vendor_id:    u16,
    pub device_id:    u16,
    pub subsystem_id: u16,
    /// BAR0 raw value (bit 0 = 1 → I/O space; I/O base = bar0 & !3)
    pub bar0: u32,
}

impl PciDevice {
    /// I/O base address from BAR0 (valid only when bit 0 = 1).
    pub fn io_base(&self) -> u16 {
        (self.bar0 & !3) as u16
    }

    /// Enable I/O space + bus-master bits in the command register.
    pub fn enable_io_and_busmaster(&self) {
        let cmd = read16(self.bus, self.dev, self.func, 0x04);
        write32(self.bus, self.dev, self.func, 0x04,
                (cmd | 0x0005) as u32);  // I/O enable (bit 0) + bus master (bit 2)
    }
}

/// Scan the PCI bus for a device matching any of the given (vendor, device) pairs.
/// Returns the first match found, or `None`.
pub fn find(pairs: &[(u16, u16)]) -> Option<PciDevice> {
    for bus in 0u8..=255 {
        for dev in 0u8..32 {
            // Check function 0 first; skip if vendor is 0xFFFF (empty slot).
            let hdr = read32(bus, dev, 0, 0);
            let vid = (hdr & 0xFFFF) as u16;
            if vid == 0xFFFF { continue; }

            // Determine how many functions to check.
            let header_type = read16(bus, dev, 0, 0x0E) as u8;
            let max_func: u8 = if header_type & 0x80 != 0 { 8 } else { 1 };

            for func in 0..max_func {
                let vd = read32(bus, dev, func, 0);
                let vendor = (vd & 0xFFFF) as u16;
                let device = ((vd >> 16) & 0xFFFF) as u16;
                if vendor == 0xFFFF { continue; }

                for &(v, d) in pairs {
                    if vendor == v && device == d {
                        let ss = read32(bus, dev, func, 0x2C);
                        let subsystem_id = ((ss >> 16) & 0xFFFF) as u16;
                        let bar0 = read32(bus, dev, func, 0x10);
                        return Some(PciDevice {
                            bus, dev, func,
                            vendor_id: vendor,
                            device_id: device,
                            subsystem_id,
                            bar0,
                        });
                    }
                }
            }
        }
    }
    None
}
