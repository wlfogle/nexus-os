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
use x86_64::instructions::port::Port;
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
    /// Legacy ISA IRQ → GSI overrides (Interrupt Source Override entries,
    /// MADT type 2, bus==0/ISA only). Index = ISA IRQ number (0-15). `None`
    /// means no override — the GSI equals the ISA IRQ number, the common
    /// case. Real firmware very commonly overrides IRQ0 (the PIT) to a
    /// different GSI (e.g. GSI 2), so this can never be assumed identity
    /// without actually checking — see `isa_irq_to_gsi`.
    pub iso: [Option<u32>; 16],
    /// MPS INTI flags paired with each `iso` entry: bits 0-1 = polarity
    /// (00/01 = active-high, 11 = active-low), bits 2-3 = trigger mode
    /// (00/01 = edge, 11 = level). `0` ("conforms to bus spec") means the
    /// ISA default of active-high/edge. See `isa_irq_polarity_trigger`.
    pub iso_flags: [u16; 16],
}

impl Madt {
    /// Resolve a legacy ISA IRQ number to its actual Global System
    /// Interrupt, honoring any Interrupt Source Override instead of
    /// assuming identity (`gsi == isa_irq`), which is only the default in
    /// the *absence* of an override.
    pub fn isa_irq_to_gsi(&self, isa_irq: u8) -> u32 {
        self.iso.get(isa_irq as usize).copied().flatten().unwrap_or(isa_irq as u32)
    }

    /// Polarity/trigger mode to use when programming this ISA IRQ's I/O
    /// APIC redirection entry: `(active_low, level_triggered)`. Defaults to
    /// `(false, false)` (active-high, edge-triggered — the ISA/8259
    /// default) when there's no override or its flags say "conforms to bus
    /// spec", matching every override this driver has ever needed to
    /// actually honor (IRQ0/1/12).
    pub fn isa_irq_polarity_trigger(&self, isa_irq: u8) -> (bool, bool) {
        let flags = self.iso_flags.get(isa_irq as usize).copied().unwrap_or(0);
        let polarity = flags & 0x3;
        let trigger = (flags >> 2) & 0x3;
        (polarity == 3, trigger == 3)
    }
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
            2 => {
                // Interrupt Source Override: byte 2 = bus (0=ISA), byte 3 =
                // source IRQ, bytes 4..8 = GSI, bytes 8..10 = flags. Only
                // ISA overrides matter to this driver (everything it routes
                // — timer/keyboard/mouse — is an ISA IRQ).
                let bus = unsafe { core::ptr::read_unaligned((virt + off + 2) as *const u8) };
                if bus == 0 {
                    let irq = unsafe { core::ptr::read_unaligned((virt + off + 3) as *const u8) } as usize;
                    if irq < 16 {
                        madt.iso[irq] = Some(unsafe {
                            core::ptr::read_unaligned((virt + off + 4) as *const u32)
                        });
                        madt.iso_flags[irq] = unsafe {
                            core::ptr::read_unaligned((virt + off + 8) as *const u16)
                        };
                    }
                }
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

// ─── Power management (Phase K5, increment 2) ────────────────────────────

/// Try the FADT's ACPI Reset Register. Only System I/O (space 1) and
/// System Memory (space 0) address spaces are implemented — the two
/// actually used in practice; anything else is treated as unsupported.
/// Returns `Err` (never panics) if unsupported or if the write somehow
/// didn't reset the machine, so `reboot()` can fall back to something
/// universal instead of hanging.
fn try_acpi_reset() -> Result<(), &'static str> {
    let f = fadt().ok_or("no FADT")?;
    if !f.reset_supported {
        return Err("ACPI reset not supported by firmware");
    }
    match f.reset_reg_space {
        1 => unsafe { // System I/O
            let mut port: Port<u8> = Port::new(f.reset_reg_addr as u16);
            port.write(f.reset_value);
        },
        0 => unsafe { // System Memory
            let virt = paging::phys_to_virt(f.reset_reg_addr);
            core::ptr::write_volatile(virt as *mut u8, f.reset_value);
        },
        _ => return Err("unsupported reset register address space"),
    }
    // A working reset happens immediately; reaching this line means it
    // didn't, so report failure rather than silently continuing.
    Err("ACPI reset register write did not reset the machine")
}

/// Universal fallback: pulse the 8042 keyboard controller's reset line
/// (command 0xFE = "pulse output port bit 0", wired to the CPU's RESET pin
/// on essentially every real x86 system and every emulator). This is the
/// same technique used by every minimal OS's reboot path when ACPI reset
/// is unavailable or unsupported — it needs no ACPI support at all.
fn reset_via_8042() -> ! {
    unsafe {
        let mut status: Port<u8> = Port::new(0x64);
        let mut spins = 0u32;
        // Wait for the controller's input buffer to be clear (bit 1) before
        // writing a command, bounded so a wedged controller can't hang us
        // forever here (the 8042 pulse is the fallback of last resort, but
        // it can't be allowed to itself hang forever if it isn't working).
        while status.read() & 0x02 != 0 && spins < 100_000 { spins += 1; }
        crate::kprintln!("[acpi] reboot: 8042 input buffer clear after {} spins, pulsing reset", spins);
        let mut cmd: Port<u8> = Port::new(0x64);
        cmd.write(0xFEu8);
    }
    // If the pulse didn't reset the machine, there is nothing else safe
    // left to try — halt rather than fall through into undefined behavior.
    crate::kprintln!("[acpi] reboot: 8042 pulse did not reset the machine");
    loop { unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); } }
}

/// Reboot the machine: try the ACPI Reset Register first, then
/// unconditionally fall back to the 8042 controller pulse. Never returns.
pub fn reboot() -> ! {
    crate::kprintln!("[acpi] reboot: trying ACPI reset register...");
    if let Err(e) = try_acpi_reset() {
        crate::kprintln!("[acpi] reboot: ACPI reset unavailable ({}), falling back to 8042 pulse", e);
    }
    reset_via_8042();
}

/// Scan a DSDT for the `_S5_` object's Package and extract SLP_TYPa/
/// SLP_TYPb. This is deliberately **not** a general AML interpreter — it
/// only understands the specific, extremely common encoding real-world
/// DSDTs use for this one object (a `PackageOp` immediately following the
/// name, whose first two elements are each either a bare ZeroOp/OneOp
/// byte or a `BytePrefix` + one literal byte). This is the same minimal
/// technique essentially every hobby/minimal OS uses for ACPI shutdown
/// (see e.g. the OSDev wiki's "Shutdown" page) rather than implementing
/// full AML parsing for one two-integer object. Returns `None` — not a
/// guess — if the encoding doesn't match what this recognizes.
fn find_s5_sleep_type(dsdt_virt: u64, dsdt_len: usize) -> Option<(u8, u8)> {
    let bytes = unsafe { core::slice::from_raw_parts(dsdt_virt as *const u8, dsdt_len) };
    let pos = bytes.windows(4).position(|w| w == b"_S5_")?;
    let mut i = pos + 4;

    if bytes.get(i) != Some(&0x12) { return None; } // PackageOp
    i += 1;

    // PkgLength (ACPI spec §20.2.4): top 2 bits of the lead byte give how
    // many extra length bytes follow (0-3). We only need to skip past it
    // correctly to reach NumElements + the element list, not compute the
    // actual length.
    let lead = *bytes.get(i)?;
    let extra_bytes = (lead >> 6) as usize;
    i += 1 + extra_bytes;
    i += 1; // NumElements

    let mut values = [0u8; 2];
    for slot in values.iter_mut() {
        let op = *bytes.get(i)?;
        match op {
            0x00 | 0x01 => { *slot = op; i += 1; }         // ZeroOp / OneOp: value is the opcode itself
            0x0A        => { *slot = *bytes.get(i + 1)?; i += 2; } // BytePrefix + literal byte
            _ => return None, // an encoding we don't recognize -- bail out rather than guess
        }
    }
    Some((values[0], values[1]))
}

/// Attempt ACPI S5 (soft power-off): locate `_S5`'s SLP_TYPa/b in the DSDT,
/// then write `SLP_TYP | SLP_EN` to PM1a_CNT_BLK (and PM1b_CNT_BLK too, if
/// present, using SLP_TYPb). Unlike `reboot()`, there is no universal
/// fallback for power-off if this fails, so this returns `Err` instead of
/// `!` — the caller (the shell) reports the failure and the machine
/// simply keeps running, which is the only safe default.
pub fn shutdown() -> Result<(), &'static str> {
    let f = fadt().ok_or("no FADT (ACPI not available)")?;
    if f.pm1a_cnt_blk == 0 { return Err("no PM1a_CNT_BLK in FADT"); }
    if f.dsdt == 0 { return Err("no DSDT pointer in FADT"); }

    let dsdt_virt = paging::phys_to_virt(f.dsdt as u64);
    let dsdt_len = unsafe { core::ptr::read_unaligned((dsdt_virt + 4) as *const u32) } as usize;
    if !(36..=(1 << 20)).contains(&dsdt_len) {
        return Err("DSDT has an implausible length");
    }

    let (slp_typa, slp_typb) = find_s5_sleep_type(dsdt_virt, dsdt_len)
        .ok_or("could not locate/parse the _S5 package in the DSDT")?;

    const SLP_EN: u16 = 1 << 13;
    unsafe {
        let mut port: Port<u16> = Port::new(f.pm1a_cnt_blk as u16);
        port.write(((slp_typa as u16) << 10) | SLP_EN);

        if f.pm1b_cnt_blk != 0 {
            let mut port_b: Port<u16> = Port::new(f.pm1b_cnt_blk as u16);
            port_b.write(((slp_typb as u16) << 10) | SLP_EN);
        }
    }

    // The power-off is not necessarily synchronous with the write above --
    // confirmed against QEMU, whose ACPI PM1 control handler queues the
    // actual shutdown for the next main-loop iteration rather than cutting
    // power on the writing instruction itself. Spin briefly to give that a
    // window to land instead of declaring failure the instant we're still
    // executing; only report Err once a generous bound has elapsed.
    for _ in 0..50_000_000u64 {
        core::hint::spin_loop();
    }
    Err("PM1 control write did not power off the machine")
}
