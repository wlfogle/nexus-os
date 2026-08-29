//! NexusOS ACPI Table Discovery — Phase K5
//!
//! Locates the RSDP via Limine's `RsdpRequest` (no BIOS memory scan needed —
//! Limine hands us the address directly), walks the RSDT/XSDT to find the
//! Fixed ACPI Description Table (FADT) and Multiple APIC Description Table
//! (MADT), and exposes exactly the fields later phases need: FADT's PM1a
//! control block + ACPI reset register (for `reboot`/`shutdown`), and
//! MADT's Local APIC/IOAPIC addresses and enabled-CPU count (for SMP
//! bring-up). Every table's checksum is verified before use; a corrupt or
//! missing table is logged and skipped rather than treated as fatal — the
//! rest of the kernel already runs fine without ACPI (it did throughout
//! Phases 1-K4), so this degrades gracefully instead of panicking.
//!
//! # Address handling — the one subtlety here
//! Limine hands us the RSDP's location as an **already-virtual** pointer
//! (confirmed by this codebase's own precedent: `io::framebuffer::init`
//! uses `fb.address.as_ptr()` directly with no HHDM translation — Limine's
//! boot-protocol "address" fields are pre-translated, not raw physical
//! addresses). Everything *inside* the ACPI tables themselves (RSDP's own
//! `rsdt_address`/`xsdt_address`, every RSDT/XSDT entry, FADT's `dsdt`
//! field) are genuine ACPI-spec physical addresses that still need
//! `paging::phys_to_virt` before dereferencing. Mixing these up would
//! silently corrupt every read past the RSDP itself.
//!
//! Nothing here *acts* on any of this yet (no LAPIC/IOAPIC programming, no
//! AP bring-up, no ACPI power-management writes) — that's later increments
//! in this same phase, built on top of what this module discovers.

use spin::Mutex;
use crate::memory::paging;

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct RsdpV1 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct RsdpV2 {
    v1: RsdpV1,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct SdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: [u8; 4],
    creator_revision: u32,
}

/// Fields this driver actually needs from the FADT. The real table has
/// grown many optional fields across ACPI revisions (it's ~244 bytes in
/// ACPI 6.x); rather than modeling all of it, `parse_fadt` reads only
/// these specific offsets directly, which is both simpler and immune to
/// getting padding/alignment wrong for fields we never touch.
#[derive(Clone, Copy, Default, Debug)]
pub struct Fadt {
    pub pm1a_cnt_blk: u32,   // I/O port address
    pub pm1b_cnt_blk: u32,   // 0 if not present
    pub pm1_cnt_len:  u8,
    pub dsdt:         u32,   // physical address of the DSDT (for a future _S5 scan)
    pub reset_reg_addr:  u64, // meaningless unless reset_supported
    pub reset_reg_space: u8,  // Generic Address Structure address_space_id: 0=memory, 1=I/O
    pub reset_value:     u8,
    pub reset_supported:  bool,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Madt {
    pub local_apic_addr: u32,
    pub ioapic_addr:     u32,
    pub ioapic_gsi_base: u32,
    /// Count of enabled Processor Local APIC entries — i.e. usable CPUs,
    /// matching what Limine's own `SmpResponse::cpu_count` should report.
    pub cpu_count: usize,
}

struct AcpiInfo {
    fadt: Option<Fadt>,
    madt: Option<Madt>,
}

static ACPI: Mutex<Option<AcpiInfo>> = Mutex::new(None);

fn checksum_ok(bytes: &[u8]) -> bool {
    bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) == 0
}

/// Read an SDT header (and return the *virtual* pointer it can be found
/// at) from a raw ACPI-spec physical address.
unsafe fn read_sdt_header(phys: u64) -> (SdtHeader, u64) {
    let virt = paging::phys_to_virt(phys);
    let header = core::ptr::read_unaligned(virt as *const SdtHeader);
    (header, virt)
}

/// Discover ACPI tables. `rsdp_virt` is the address Limine's `RsdpRequest`
/// handed us, already virtual (see module docs) — not a physical address.
/// Returns `true` if at least the RSDP itself checked out; a missing or
/// corrupt FADT/MADT is logged but not fatal (callers check `fadt()`/
/// `madt()` for `None` and degrade gracefully).
pub fn init(rsdp_virt: u64) -> bool {
    if rsdp_virt == 0 {
        crate::kprintln!("[acpi] no RSDP provided by bootloader");
        return false;
    }

    let v1 = unsafe { core::ptr::read_unaligned(rsdp_virt as *const RsdpV1) };
    if &v1.signature != b"RSD PTR " {
        crate::kprintln!("[acpi] RSDP signature mismatch");
        return false;
    }
    if !checksum_ok(unsafe { core::slice::from_raw_parts(rsdp_virt as *const u8, 20) }) {
        crate::kprintln!("[acpi] RSDP v1 checksum failed");
        return false;
    }

    // Prefer the 64-bit XSDT (ACPI 2.0+) when present and its own extended
    // checksum is valid; fall back to the 32-bit RSDT otherwise. Both list
    // physical pointers to every other SDT, just at different pointer widths.
    let mut sdt_ptrs_phys = v1.rsdt_address as u64;
    let mut ptr_is_64bit = false;
    if v1.revision >= 2 {
        let v2 = unsafe { core::ptr::read_unaligned(rsdp_virt as *const RsdpV2) };
        let v2_len = v2.length as usize;
        if v2.xsdt_address != 0
            && v2_len >= 36
            && checksum_ok(unsafe { core::slice::from_raw_parts(rsdp_virt as *const u8, v2_len) })
        {
            sdt_ptrs_phys = v2.xsdt_address;
            ptr_is_64bit = true;
        }
    }

    let (root_header, root_virt) = unsafe { read_sdt_header(sdt_ptrs_phys) };
    let root_len = root_header.length as usize;
    let root_sig = root_header.signature;
    if root_len < 36 {
        crate::kprintln!("[acpi] root SDT has an implausible length ({})", root_len);
        return false;
    }
    if !checksum_ok(unsafe { core::slice::from_raw_parts(root_virt as *const u8, root_len) }) {
        crate::kprintln!("[acpi] root SDT checksum failed");
        return false;
    }

    let entry_size = if ptr_is_64bit { 8usize } else { 4usize };
    let entry_count = (root_len - 36) / entry_size;
    crate::kprintln!(
        "[acpi] root SDT: {} ({}-bit pointers, {} entries)",
        core::str::from_utf8(&root_sig).unwrap_or("????"),
        if ptr_is_64bit { 64 } else { 32 }, entry_count
    );

    let mut fadt = None;
    let mut madt = None;

    for i in 0..entry_count {
        let entry_addr = root_virt + 36 + (i * entry_size) as u64;
        let table_phys: u64 = if ptr_is_64bit {
            unsafe { core::ptr::read_unaligned(entry_addr as *const u64) }
        } else {
            unsafe { core::ptr::read_unaligned(entry_addr as *const u32) as u64 }
        };
        if table_phys == 0 { continue; }

        let (header, virt) = unsafe { read_sdt_header(table_phys) };
        let len = header.length as usize;
        let sig = header.signature;
        if len < 36 || !checksum_ok(unsafe { core::slice::from_raw_parts(virt as *const u8, len) }) {
            crate::kprintln!(
                "[acpi] skipping corrupt table {} (len={})",
                core::str::from_utf8(&sig).unwrap_or("????"), len
            );
            continue;
        }

        match &sig {
            b"FACP" => fadt = Some(parse_fadt(virt, len)),
            b"APIC" => madt = Some(parse_madt(virt, len)),
            _ => {}
        }
    }

    match fadt {
        Some(f) => crate::kprintln!(
            "[acpi] FADT: PM1a_CNT={:#06x} PM1b_CNT={:#06x} reset_supported={} DSDT={:#010x}",
            f.pm1a_cnt_blk, f.pm1b_cnt_blk, f.reset_supported, f.dsdt
        ),
        None => crate::kprintln!("[acpi] no FADT found"),
    }
    match madt {
        Some(m) => crate::kprintln!(
            "[acpi] MADT: local_apic={:#010x} ioapic={:#010x} gsi_base={} cpu_count={}",
            m.local_apic_addr, m.ioapic_addr, m.ioapic_gsi_base, m.cpu_count
        ),
        None => crate::kprintln!("[acpi] no MADT found"),
    }

    *ACPI.lock() = Some(AcpiInfo { fadt, madt });
    true
}

/// Parse the handful of FADT fields we need. Offsets are from the ACPI
/// 6.x spec, relative to the table's own base (i.e. including the 36-byte
/// SDT header already at offset 0):
///   40  DSDT (u32)
///   64  PM1a_CNT_BLK (u32)      68  PM1b_CNT_BLK (u32)
///   89  PM1_CNT_LEN (u8)
///   112 Flags (u32)             — bit 10 = RESET_REG_SUP
///   116 RESET_REG (Generic Address Structure, 12 bytes: the reset value's
///       own comments below cover mm bytes 116/120/128)
///   128 RESET_VALUE (u8)
fn parse_fadt(virt: u64, len: usize) -> Fadt {
    let r8  = |off: u64| unsafe { core::ptr::read_unaligned((virt + off) as *const u8) };
    let r32 = |off: u64| unsafe { core::ptr::read_unaligned((virt + off) as *const u32) };
    let r64 = |off: u64| unsafe { core::ptr::read_unaligned((virt + off) as *const u64) };

    let dsdt         = r32(40);
    let pm1a_cnt_blk = r32(64);
    let pm1b_cnt_blk = r32(68);
    let pm1_cnt_len  = r8(89);

    // ACPI 2.0+ reset fields; only present if the table is long enough.
    let mut reset_reg_addr  = 0u64;
    let mut reset_reg_space = 0u8;
    let mut reset_value     = 0u8;
    let mut reset_supported = false;
    if len > 129 {
        let flags = r32(112);
        reset_supported = flags & (1 << 10) != 0; // RESET_REG_SUP
        reset_reg_space = r8(116);  // GAS address_space_id: 0=SystemMemory, 1=SystemIO
        reset_reg_addr  = r64(120); // GAS address (bytes 120-127; bytes 117-119 are width/offset/access_size)
        reset_value     = r8(128);
    }

    Fadt {
        pm1a_cnt_blk, pm1b_cnt_blk, pm1_cnt_len, dsdt,
        reset_reg_addr, reset_reg_space, reset_value, reset_supported,
    }
}

/// Parse the MADT: Local APIC base address at offset 36, then a stream of
/// variable-length interrupt-controller-structure entries starting at
/// offset 44 (after the 4-byte address + 4-byte flags following the
/// 36-byte SDT header). Entry type 0 = Processor Local APIC (counted if
/// its Enabled flag bit is set); type 1 = I/O APIC.
fn parse_madt(virt: u64, len: usize) -> Madt {
    let local_apic_addr = unsafe { core::ptr::read_unaligned((virt + 36) as *const u32) };
    let mut madt = Madt { local_apic_addr, ..Default::default() };

    let mut off = 44u64;
    while off + 2 <= len as u64 {
        let entry_type = unsafe { core::ptr::read_unaligned((virt + off) as *const u8) };
        let entry_len  = unsafe { core::ptr::read_unaligned((virt + off + 1) as *const u8) } as u64;
        if entry_len < 2 || off + entry_len > len as u64 { break; }

        match entry_type {
            0 => {
                // Processor Local APIC: byte 4..8 = Flags (u32), bit 0 = Enabled.
                let flags = unsafe { core::ptr::read_unaligned((virt + off + 4) as *const u32) };
                if flags & 1 != 0 { madt.cpu_count += 1; }
            }
            1 => {
                // I/O APIC: byte 4..8 = address, byte 8..12 = GSI base.
                madt.ioapic_addr     = unsafe { core::ptr::read_unaligned((virt + off + 4) as *const u32) };
                madt.ioapic_gsi_base = unsafe { core::ptr::read_unaligned((virt + off + 8) as *const u32) };
            }
            _ => {}
        }
        off += entry_len;
    }
    madt
}

/// Copy of the discovered FADT, if any.
pub fn fadt() -> Option<Fadt> {
    ACPI.lock().as_ref()?.fadt
}

/// Copy of the discovered MADT, if any.
pub fn madt() -> Option<Madt> {
    ACPI.lock().as_ref()?.madt
}
