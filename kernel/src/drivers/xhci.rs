//! NexusOS xHCI (USB 3.x Host Controller) Driver — Phase K4
//!
//! Drives a PCI-class xHCI controller (class 0x0C / subclass 0x03 /
//! prog-if 0x30) through its memory-mapped register set. Scope is
//! deliberately narrow, matching this codebase's other block/bus drivers
//! (`ahci.rs`, `nvme.rs`): a single root hub, boot-protocol HID devices only
//! (see `drivers::usb_hid`), no external hubs, no mass storage, no
//! isochronous transfers, and polling instead of MSI/MSI-X interrupts — the
//! event ring is drained by spinning, exactly like AHCI/NVMe poll their
//! command-completion bits rather than taking an IRQ.
//!
//! # Register layout (xHCI spec)
//! - **Capability Registers** (MMIO base): fixed-size header describing the
//!   controller (`CAPLENGTH`, `HCSPARAMS1/2`, `HCCPARAMS1`, doorbell/runtime
//!   offsets).
//! - **Operational Registers** (base + CAPLENGTH): `USBCMD`, `USBSTS`,
//!   `CRCR` (command ring), `DCBAAP` (device context array), `CONFIG`,
//!   `PORTSC[n]` (one per root-hub port).
//! - **Runtime Registers** (base + RTSOFF): per-interrupter event-ring
//!   registers (`ERSTSZ`/`ERSTBA`/`ERDP`) — only interrupter 0 is used here.
//! - **Doorbell Array** (base + DBOFF): ring a slot's doorbell to signal new
//!   command/transfer-ring work.
//!
//! # Memory layout
//! Every structure here (Device Context Base Address Array, Command Ring,
//! Event Ring segment + its segment table, per-device Input/Output
//! Contexts) fits in a single 4 KiB frame — allocated and zeroed the same
//! way `ahci.rs::alloc_zeroed_frame` does. No multi-page contiguous DMA
//! allocation is needed at this scale (a handful of slots, 256 TRBs per
//! ring is already generous for boot-protocol HID traffic).
//!
//! # Bring-up sequence
//! 1. BIOS→OS handoff via the USB Legacy Support extended capability, if
//!    present (some firmware keeps emulating a PS/2 device through the
//!    controller until asked to stop).
//! 2. Reset (`USBCMD.HCRST`), wait for `USBSTS.CNR` to clear.
//! 3. Program `DCBAAP`, `CRCR`, `CONFIG.MaxSlotsEn`, and the primary
//!    interrupter's `ERSTSZ`/`ERSTBA`/`ERDP`.
//! 4. Set `USBCMD.RS` (run) and confirm with a No-Op command, whose
//!    completion event proves the command ring, event ring, and doorbell
//!    path all work end to end before any device enumeration is attempted.

use spin::Mutex;
use core::sync::atomic::{fence, Ordering};
use crate::memory::{physical, paging};
use crate::drivers;

// ─── PCI class code ───────────────────────────────────────────────────────────

const CLASS_SERIAL_BUS: u8 = 0x0C;
const SUBCLASS_USB:     u8 = 0x03;
const PROGIF_XHCI:      u8 = 0x30;

// ─── Capability register offsets (from MMIO base) ────────────────────────────

const CAP_CAPLENGTH:  u64 = 0x00; // u8
const CAP_HCSPARAMS1: u64 = 0x04;
const CAP_HCCPARAMS1: u64 = 0x10;
const CAP_DBOFF:      u64 = 0x14;
const CAP_RTSOFF:     u64 = 0x18;

// ─── Operational register offsets (from operational base = MMIO + CAPLENGTH) ─

const OP_USBCMD:  u64 = 0x00;
const OP_USBSTS:  u64 = 0x04;
const OP_CRCR:    u64 = 0x18;
const OP_DCBAAP:  u64 = 0x30;
const OP_CONFIG:  u64 = 0x38;
const OP_PORTSC0: u64 = 0x400; // PORTSC for port 1; port n is at + (n-1)*0x10

const USBCMD_RS:    u32 = 1 << 0; // Run/Stop
const USBCMD_HCRST: u32 = 1 << 1; // Host Controller Reset

const USBSTS_HCH: u32 = 1 << 0;  // Host Controller Halted
const USBSTS_CNR: u32 = 1 << 11; // Controller Not Ready

const CRCR_RCS: u64 = 1 << 0; // Ring Cycle State

// PORTSC bits we touch. Bits 17/18/19/20/21/22/23 (CSC/PEC/WRC/OCC/PRC/PLC/
// CEC) are RW1C "write 1 to clear" status-change bits: reading PORTSC and
// writing the same value back would silently clear any of those that
// happen to be set, discarding a status change we haven't looked at yet.
// Every write below explicitly zeroes that whole mask except the exact
// change bit (if any) we mean to acknowledge, so unrelated pending changes
// are never lost.
const PORTSC_CCS: u32 = 1 << 0;  // Current Connect Status (RO)
const PORTSC_PED: u32 = 1 << 1;  // Port Enabled (RW — write 1 to disable, not used here)
const PORTSC_PR:  u32 = 1 << 4;  // Port Reset (RW1S)
const PORTSC_PRC: u32 = 1 << 21; // Port Reset Change (RW1C)
const PORTSC_RW1C_MASK: u32 =
    (1 << 17) | (1 << 18) | (1 << 19) | (1 << 20) | (1 << 21) | (1 << 22) | (1 << 23);
const PORTSC_SPEED_SHIFT: u32 = 10; // Port Speed ID (RO), bits 13:10

const TRB_TYPE_ENABLE_SLOT_CMD:    u32 = 9;
const TRB_TYPE_ADDRESS_DEVICE_CMD: u32 = 11;

// Port/Slot-Context "Protocol Speed ID" values in the default (no
// vendor-specific PSI table) assignment xHCI controllers use — the same
// numeric encoding is shared verbatim between PORTSC's speed field and the
// Slot Context's Speed field, so no translation is needed between them.
const SPEED_FULL:  u8 = 1;
const SPEED_LOW:   u8 = 2;
const SPEED_HIGH:  u8 = 3;
const SPEED_SUPER: u8 = 4;

const EP_TYPE_CONTROL: u32 = 4;

// ─── Runtime register offsets (from runtime base = MMIO + RTSOFF) ───────────

const RT_IR0: u64 = 0x20; // Interrupter Register Set 0
#[allow(dead_code)] // used once we switch from polling to interrupt-driven events
const IR_IMAN:   u64 = 0x00;
const IR_ERSTSZ: u64 = 0x08;
const IR_ERSTBA: u64 = 0x10;
const IR_ERDP:   u64 = 0x18;

const ERDP_EHB: u64 = 1 << 3; // Event Handler Busy — write 1 to clear

// ─── TRB (Transfer Request Block): 16 bytes, shared by every ring ───────────

const TRB_CYCLE: u32 = 1 << 0;

const TRB_TYPE_NOOP_CMD:        u32 = 23;
const TRB_TYPE_LINK:            u32 = 6;
const TRB_TYPE_CMD_COMPLETION:  u32 = 33;
#[allow(dead_code)] // used once device transfer rings (the next increment) land
const TRB_TYPE_TRANSFER_EVENT:  u32 = 32;
#[allow(dead_code)] // used once port enumeration (the next increment) lands
const TRB_TYPE_PORT_STS_CHANGE: u32 = 34;

const COMPLETION_SUCCESS: u8 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct Trb {
    parameter: u64,
    status:    u32,
    control:   u32,
}

impl Trb {
    #[inline]
    fn trb_type(&self) -> u32 {
        (self.control >> 10) & 0x3F
    }

    #[inline]
    fn cycle(&self) -> bool {
        self.control & TRB_CYCLE != 0
    }

    #[inline]
    fn completion_code(&self) -> u8 {
        (self.status >> 24) as u8
    }

    /// Slot ID a Command Completion / Transfer event refers to. Unused until
    /// the next increment (port/device enumeration needs it to know which
    /// slot a completion event belongs to); kept alongside the other TRB
    /// field accessors now since they're all part of the same format.
    #[inline]
    #[allow(dead_code)]
    fn slot_id(&self) -> u8 {
        (self.control >> 24) as u8
    }
}

const RING_TRBS: usize = 256; // one 4 KiB page / 16 bytes per TRB

/// A single producer-consumer TRB ring: used for the Command Ring, and
/// reused as-is for every per-endpoint Transfer Ring (EP0's control-transfer
/// ring, later real HID interrupt-IN rings) — both share the exact same
/// "flat array of TRBs terminated by a Link TRB" shape. Only the one event
/// ring segment is structurally different (no Link TRB — the controller
/// wraps it itself via `ERDP`/`ERSTSZ`), so it gets its own constructor and
/// is only ever consumed via `peek_event`, never `enqueue_command`.
struct Ring {
    virt:  u64,
    phys:  u64,
    index: usize,
    cycle: bool, // producer cycle state (for rings we write TRBs into)
}

impl Ring {
    fn new() -> Self {
        let phys = alloc_zeroed_frame();
        let virt = paging::phys_to_virt(phys);
        // Slot RING_TRBS-1 = Link TRB back to slot 0, cycle bit matching the
        // ring's initial producer cycle state (1) so the controller can
        // follow it the first time it wraps.
        let link = Trb {
            parameter: phys,
            status: 0,
            control: (TRB_TYPE_LINK << 10) | TRB_CYCLE | (1 << 1), // Toggle Cycle bit
        };
        unsafe {
            let last = (virt as *mut Trb).add(RING_TRBS - 1);
            core::ptr::write_volatile(last, link);
        }
        Self { virt, phys, index: 0, cycle: true }
    }

    fn new_event_ring_segment() -> Self {
        let phys = alloc_zeroed_frame();
        let virt = paging::phys_to_virt(phys);
        Self { virt, phys, index: 0, cycle: true }
    }

    /// Write `trb` (with the correct cycle bit already applied by the
    /// caller's fields other than `control`'s cycle bit, which this
    /// function sets) into the next command-ring slot, advancing past the
    /// Link TRB automatically. Returns the physical address the TRB was
    /// written at, so the caller can match it against a later completion
    /// event's Parameter field.
    fn enqueue_command(&mut self, mut trb: Trb) -> u64 {
        if self.index == RING_TRBS - 1 {
            // Toggle our producer cycle state to match the Link TRB we
            // already wrote (it always carries the pre-toggle cycle value
            // and the Toggle Cycle bit), then wrap to slot 0.
            self.cycle = !self.cycle;
            self.index = 0;
        }
        trb.control = (trb.control & !TRB_CYCLE) | (self.cycle as u32);
        let addr = self.phys + (self.index * core::mem::size_of::<Trb>()) as u64;
        unsafe {
            let slot = (self.virt as *mut Trb).add(self.index);
            core::ptr::write_volatile(slot, trb);
        }
        self.index += 1;
        addr
    }

    /// Non-destructively peek the event-ring slot at `index` and return it
    /// only if its cycle bit matches `expected_cycle` (i.e. the controller
    /// has actually produced it, not stale/zeroed memory from before the
    /// ring wrapped).
    fn peek_event(&self, index: usize, expected_cycle: bool) -> Option<Trb> {
        let trb = unsafe { core::ptr::read_volatile((self.virt as *const Trb).add(index)) };
        if trb.cycle() == expected_cycle { Some(trb) } else { None }
    }
}

// ─── MMIO helpers (same volatile-pointer style as ahci.rs) ──────────────────

#[inline]
unsafe fn r32(addr: u64) -> u32 { core::ptr::read_volatile(addr as *const u32) }
#[inline]
unsafe fn w32(addr: u64, v: u32) { core::ptr::write_volatile(addr as *mut u32, v); }
#[inline]
#[allow(dead_code)] // used once port/device register reads (the next increment) land
unsafe fn r64(addr: u64) -> u64 { core::ptr::read_volatile(addr as *const u64) }
#[inline]
unsafe fn w64(addr: u64, v: u64) { core::ptr::write_volatile(addr as *mut u64, v); }

/// Allocate a single physical frame and zero it through the HHDM.
fn alloc_zeroed_frame() -> u64 {
    let phys = physical::alloc_frame();
    unsafe { core::ptr::write_bytes(paging::phys_to_virt(phys) as *mut u8, 0, 4096); }
    phys
}

/// Write one dword of a Slot/Endpoint/Input-Control context at `dword_index`
/// (0-7 within that 32-byte context, regardless of whether the controller's
/// actual context size is 32 or 64 bytes — the extra bytes in 64-byte mode
/// are trailing padding after the same 8-dword layout, never additional
/// meaningful fields).
#[inline]
unsafe fn write_dword(ctx_base: u64, dword_index: usize, value: u32) {
    core::ptr::write_volatile((ctx_base as *mut u32).add(dword_index), value);
}

/// Max packet size (bytes) for EP0 given a device's Protocol Speed ID, used
/// before any descriptor has actually been read from the device. This is
/// the standard USB-spec default per speed class, not a guess: Low Speed
/// devices are only ever 8, and High/Super Speed devices are fixed at
/// 64/512 by spec — only Full Speed varies in principle (8/16/32/64), and 8
/// is the always-safe conservative choice there.
fn max_packet_size_for_speed(speed: u8) -> u16 {
    match speed {
        SPEED_LOW   => 8,
        SPEED_FULL  => 8,
        SPEED_HIGH  => 64,
        SPEED_SUPER => 512,
        _           => 8,
    }
}

// ─── Driver state ─────────────────────────────────────────────────────────────

/// A single addressed USB device. This driver only tracks one at a time
/// (single root hub, first device found — see the module-level scope note);
/// multi-device support is future work, not a limitation of this struct's
/// shape specifically.
struct UsbDevice {
    #[allow(dead_code)] // used once control transfers (the next increment) ring the device doorbell
    slot_id:  u8,
    #[allow(dead_code)] // reported in boot logs today; will drive re-enumeration on unplug later
    port:     u8,
    #[allow(dead_code)] // needed once usb_hid parses reports that vary by speed/timing
    speed:    u8,
    /// EP0's control-transfer ring — kept alive here so its producer
    /// index/cycle state survives between separate control transfers
    /// (SET_CONFIGURATION, then later SET_PROTOCOL/SET_IDLE, etc.).
    #[allow(dead_code)] // used once the control-transfer helper (the next increment) lands
    ep0_ring: Ring,
}

pub struct Xhci {
    op_base:      u64, // Operational Registers virtual base
    rt_base:      u64, // Runtime Registers virtual base
    db_base:      u64, // Doorbell Array virtual base
    max_slots:    u8,
    max_ports:    u8,
    context_size: usize, // 32 or 64 bytes, from HCCPARAMS1.CSZ
    dcbaa_virt:   u64,
    cmd_ring:     Ring,
    evt_ring:     Ring,
    evt_index:    usize, // next event-ring slot we expect the controller to produce
    evt_cycle:    bool,  // consumer cycle state for the event ring
    device:       Option<UsbDevice>,
}

static XHCI: Mutex<Option<Xhci>> = Mutex::new(None);

/// Detect and bring up the first xHCI controller: reset, program the
/// command/event rings, and start it (`USBCMD.RS`), verified with a No-Op
/// command round-trip through the event ring. Device/port enumeration is
/// added on top of this in a later increment — this function's job is only
/// to prove the controller itself is alive and the ring plumbing works.
pub fn init() -> bool {
    let dev = match drivers::find_by_class(CLASS_SERIAL_BUS, SUBCLASS_USB, PROGIF_XHCI) {
        Some(d) => d,
        None => return false,
    };

    drivers::enable_mem_and_busmaster(&dev);

    let mmio_phys = drivers::read_bar_addr(&dev, 0);
    if mmio_phys == 0 {
        crate::kprintln!("[xhci] controller found but BAR0 is unmapped");
        return false;
    }
    // 64 KiB is comfortably more than capability + operational + runtime +
    // doorbell registers for any controller with a modest port count.
    let mmio = drivers::map_mmio(mmio_phys, 0x10000);

    crate::kprintln!(
        "[xhci] PCI {:02x}:{:02x}.{} vendor={:#06x} device={:#06x} BAR0={:#012x}",
        dev.bus, dev.dev, dev.func, dev.vendor_id, dev.device_id, mmio_phys
    );

    match bringup(mmio) {
        Ok(mut x) => {
            crate::kprintln!(
                "[xhci] online: max_slots={} max_ports={} context_size={}B",
                x.max_slots, x.max_ports, x.context_size
            );
            match x.scan_and_reset_port() {
                Some(port) => match x.enable_slot() {
                    Ok(slot) => {
                        crate::kprintln!("[xhci] port {} -> slot {} enabled", port, slot);
                        let speed = x.port_speed(port);
                        match x.address_device(slot, port, speed) {
                            Ok(()) => crate::kprintln!(
                                "[xhci] slot {} addressed (speed_id={})", slot, speed),
                            Err(e) => crate::kprintln!("[xhci] address_device failed: {}", e),
                        }
                    }
                    Err(e) => crate::kprintln!("[xhci] enable_slot failed: {}", e),
                },
                None => crate::kprintln!("[xhci] no connected USB devices found"),
            }
            *XHCI.lock() = Some(x);
            true
        }
        Err(e) => {
            crate::kprintln!("[xhci] init failed: {}", e);
            false
        }
    }
}

fn bringup(mmio: u64) -> Result<Xhci, &'static str> {
    let cap_length = unsafe { core::ptr::read_volatile((mmio + CAP_CAPLENGTH) as *const u8) } as u64;
    let hcsparams1 = unsafe { r32(mmio + CAP_HCSPARAMS1) };
    let hccparams1 = unsafe { r32(mmio + CAP_HCCPARAMS1) };
    let dboff  = unsafe { r32(mmio + CAP_DBOFF) }  as u64 & !0x3;
    let rtsoff = unsafe { r32(mmio + CAP_RTSOFF) } as u64 & !0x1F;

    let max_slots = (hcsparams1 & 0xFF) as u8;
    let max_ports = ((hcsparams1 >> 24) & 0xFF) as u8;
    let context_size = if hccparams1 & (1 << 2) != 0 { 64 } else { 32 };

    let op_base = mmio + cap_length;
    let rt_base = mmio + rtsoff;
    let db_base = mmio + dboff;

    // ── Reset ────────────────────────────────────────────────────────────
    // Stop the controller first if it's already running (firmware may have
    // left it active), then reset, then wait for CNR to clear before
    // touching any other operational register — the spec requires this.
    unsafe {
        let cmd = r32(op_base + OP_USBCMD);
        if cmd & USBCMD_RS != 0 {
            w32(op_base + OP_USBCMD, cmd & !USBCMD_RS);
            wait_for(|| r32(op_base + OP_USBSTS) & USBSTS_HCH != 0, "controller did not halt")?;
        }
        w32(op_base + OP_USBCMD, USBCMD_HCRST);
        wait_for(|| r32(op_base + OP_USBCMD) & USBCMD_HCRST == 0, "reset did not complete")?;
        wait_for(|| r32(op_base + OP_USBSTS) & USBSTS_CNR == 0, "controller not ready after reset")?;
    }

    // ── Device Context Base Address Array ───────────────────────────────
    // One 64-bit entry per slot (index 0 is the scratchpad pointer, which
    // we leave null — no scratchpad buffers needed for boot-protocol HID).
    let dcbaa_phys = alloc_zeroed_frame();
    let dcbaa_virt = paging::phys_to_virt(dcbaa_phys);
    unsafe { w64(op_base + OP_DCBAAP, dcbaa_phys); }

    // ── Command ring ───────────────────────────────────
    let cmd_ring = Ring::new();
    unsafe { w64(op_base + OP_CRCR, cmd_ring.phys | CRCR_RCS); }

    // ── Event ring (one segment) + its segment table ────────────────────
    let evt_ring = Ring::new_event_ring_segment();
    let erst_phys = alloc_zeroed_frame();
    let erst_virt = paging::phys_to_virt(erst_phys);
    unsafe {
        // ERST entry: { ring segment base address (u64), segment size (u32), reserved (u32) }
        core::ptr::write_volatile(erst_virt as *mut u64, evt_ring.phys);
        core::ptr::write_volatile((erst_virt + 8) as *mut u32, RING_TRBS as u32);
        core::ptr::write_volatile((erst_virt + 12) as *mut u32, 0);

        w32(rt_base + RT_IR0 + IR_ERSTSZ, 1); // one segment
        w64(rt_base + RT_IR0 + IR_ERDP, evt_ring.phys);
        w64(rt_base + RT_IR0 + IR_ERSTBA, erst_phys); // must be written after ERSTSZ/ERDP
    }

    // ── Enable slots, then run ───────────────────────────────────────────
    unsafe {
        let config = r32(op_base + OP_CONFIG);
        w32(op_base + OP_CONFIG, (config & !0xFF) | max_slots as u32);
        let cmd = r32(op_base + OP_USBCMD);
        w32(op_base + OP_USBCMD, cmd | USBCMD_RS);
        wait_for(|| r32(op_base + OP_USBSTS) & USBSTS_HCH == 0, "controller did not start")?;
    }

    let mut x = Xhci {
        op_base, rt_base, db_base, max_slots, max_ports, context_size,
        dcbaa_virt, cmd_ring, evt_ring, evt_index: 0, evt_cycle: true,
        device: None,
    };

    // ── Smoke test: issue a No-Op command, confirm its completion event ──
    let noop = Trb { parameter: 0, status: 0, control: TRB_TYPE_NOOP_CMD << 10 };
    let noop_addr = x.cmd_ring.enqueue_command(noop);
    unsafe { w32(x.db_base, 0); } // ring doorbell 0 (command doorbell), target field 0

    let event = x.poll_command_completion(noop_addr)
        .map_err(|_| "No-Op smoke test: timed out or never matched")?;
    if event.completion_code() != COMPLETION_SUCCESS {
        return Err("No-Op smoke test: command completed with a non-success code");
    }

    Ok(x)
}

impl Xhci {
    /// Scan root-hub ports for a connected device, reset the first one
    /// found, and return its port number (1-based) once the reset
    /// completes and the port reports Enabled. Returns `None` if no port
    /// currently has a device connected — a normal, expected state, not
    /// an error.
    fn scan_and_reset_port(&self) -> Option<u8> {
        for port in 1..=self.max_ports {
            let addr = self.op_base + OP_PORTSC0 + (port as u64 - 1) * 0x10;
            let val = unsafe { r32(addr) };
            if val & PORTSC_CCS == 0 {
                continue; // nothing plugged into this port
            }
            crate::kprintln!("[xhci] port {} has a device connected (PORTSC={:#010x})", port, val);

            // Request a reset. Clearing the RW1C mask (rather than leaving
            // whatever was in `val`) means we don't re-arm/clear a change
            // bit we haven't handled yet, while still setting PR.
            unsafe { w32(addr, (val & !PORTSC_RW1C_MASK) | PORTSC_PR); }

            if wait_for(|| unsafe { r32(addr) } & PORTSC_PRC != 0, "port reset timed out").is_err() {
                crate::kprintln!("[xhci] port {} reset timed out", port);
                continue;
            }

            // Acknowledge PRC (write 1 to clear just that bit) and check
            // whether the port actually came up enabled.
            let after = unsafe { r32(addr) };
            unsafe { w32(addr, (after & !PORTSC_RW1C_MASK) | PORTSC_PRC); }

            if after & PORTSC_PED == 0 {
                crate::kprintln!("[xhci] port {} did not enable after reset", port);
                continue;
            }

            crate::kprintln!("[xhci] port {} reset complete, enabled", port);
            return Some(port);
        }
        None
    }

    /// Root-hub port speed (Protocol Speed ID, PORTSC bits 13:10).
    fn port_speed(&self, port: u8) -> u8 {
        let addr = self.op_base + OP_PORTSC0 + (port as u64 - 1) * 0x10;
        let val = unsafe { r32(addr) };
        ((val >> PORTSC_SPEED_SHIFT) & 0xF) as u8
    }

    /// Issue an Enable Slot command and return the Slot ID the controller
    /// assigned, confirmed via its Command Completion event.
    fn enable_slot(&mut self) -> Result<u8, &'static str> {
        let trb = Trb { parameter: 0, status: 0, control: TRB_TYPE_ENABLE_SLOT_CMD << 10 };
        let cmd_addr = self.cmd_ring.enqueue_command(trb);
        unsafe { w32(self.db_base, 0); } // command doorbell

        let event = self.poll_command_completion(cmd_addr)?;
        if event.completion_code() != COMPLETION_SUCCESS {
            return Err("Enable Slot: command completed with a non-success code");
        }
        Ok(event.slot_id())
    }

    /// Build the Input Context for a freshly-enabled slot (Input Control
    /// Context with Slot+EP0 added, a Slot Context for a device directly
    /// attached to the root hub with no upstream hub, and an EP0 Context
    /// pointing at a fresh control-transfer ring), point the slot's DCBAA
    /// entry at a fresh Output Device Context, and issue Address Device.
    /// On success, stores the addressed device (including its EP0 ring, so
    /// later control transfers can keep enqueueing onto it) in `self.device`.
    fn address_device(&mut self, slot_id: u8, port: u8, speed: u8) -> Result<(), &'static str> {
        let ctx_sz = self.context_size as u64;

        // Input Context layout: [Input Control Context][Slot Context][EP0 Context],
        // each `ctx_sz` bytes apart, all in one page (32B*3=96, 64B*3=192 —
        // comfortably under 4 KiB either way).
        let input_phys = alloc_zeroed_frame();
        let input_virt = paging::phys_to_virt(input_phys);

        // EP0's own control-transfer ring — a separate allocation from the
        // Input Context, which the controller only reads once and never
        // touches again after this command completes.
        let ep0_ring = Ring::new();

        unsafe {
            // Input Control Context (context index 0): Add Context flags
            // A0 (Slot) and A1 (EP0) — both required for Address Device.
            // Drop Context flags (DW0) stay zero: nothing to drop on a
            // brand new slot.
            write_dword(input_virt, 1, 0b11);

            // Slot Context (context index 1).
            let slot_off = input_virt + ctx_sz;
            const ROUTE_STRING: u32 = 0;   // directly attached to the root hub — no external hubs in this driver
            const CONTEXT_ENTRIES: u32 = 1; // only EP0 (DCI 1) is valid so far
            write_dword(slot_off, 0,
                ROUTE_STRING | ((speed as u32) << 20) | (CONTEXT_ENTRIES << 27));
            write_dword(slot_off, 1, (port as u32) << 16); // Root Hub Port Number
            write_dword(slot_off, 2, 0); // Interrupter Target = 0 (our only interpreter)
            write_dword(slot_off, 3, 0); // USB Device Address / Slot State: output-only, controller fills in

            // EP0 Context (context index 2).
            let ep0_off = input_virt + ctx_sz * 2;
            let max_packet = max_packet_size_for_speed(speed) as u32;
            const CERR: u32 = 3; // Error Count: retry up to 3 times before reporting a transaction error
            write_dword(ep0_off, 1, (CERR << 1) | (EP_TYPE_CONTROL << 3) | (max_packet << 16));
            let deq_lo = (ep0_ring.phys as u32 & !0xF) | 1; // bits 4-31 = TR Dequeue Pointer, bit0 DCS=1 (initial cycle state)
            let deq_hi = (ep0_ring.phys >> 32) as u32;
            write_dword(ep0_off, 2, deq_lo);
            write_dword(ep0_off, 3, deq_hi);
            write_dword(ep0_off, 4, 8); // Average TRB Length: spec-recommended default for control endpoints
        }

        // Point this slot's DCBAA entry at a fresh Output Device Context
        // (Slot Context + EP0 Context only, no Input Control Context) —
        // the controller copies the validated Input Context fields into
        // this on a successful Address Device.
        let dev_ctx_phys = alloc_zeroed_frame();
        unsafe {
            let dcbaa_entry = (self.dcbaa_virt as *mut u64).add(slot_id as usize);
            core::ptr::write_volatile(dcbaa_entry, dev_ctx_phys);
        }

        let trb = Trb {
            parameter: input_phys,
            status: 0,
            // BSR (bit 9) = 0: issue a real SET_ADDRESS to the device now,
            // rather than only validating the Input Context.
            control: (TRB_TYPE_ADDRESS_DEVICE_CMD << 10) | ((slot_id as u32) << 24),
        };
        let cmd_addr = self.cmd_ring.enqueue_command(trb);
        unsafe { w32(self.db_base, 0); } // command doorbell

        let event = self.poll_command_completion(cmd_addr)?;
        if event.completion_code() != COMPLETION_SUCCESS {
            return Err("Address Device: command completed with a non-success code");
        }

        self.device = Some(UsbDevice { slot_id, port, speed, ep0_ring });
        Ok(())
    }

    /// Poll the event ring until a Command Completion Event matching
    /// `expected_addr` appears. The event ring is a single stream shared by
    /// every kind of event — in particular, `scan_and_reset_port`'s port
    /// reset generates a Port Status Change Event that can (and in
    /// practice does) land ahead of a subsequent command's completion
    /// event. Treating "the next event" as automatically being the one we
    /// asked for is wrong; this skips anything that isn't our match
    /// (logging what it was) instead of misinterpreting it.
    fn poll_command_completion(&mut self, expected_addr: u64) -> Result<Trb, &'static str> {
        for _ in 0..32 {
            let event = self.poll_event(50_000_000)
                .ok_or("timed out waiting for a command completion event")?;
            match event.trb_type() {
                TRB_TYPE_CMD_COMPLETION if event.parameter == expected_addr => return Ok(event),
                TRB_TYPE_CMD_COMPLETION => {
                    crate::kprintln!("[xhci] ignoring a command completion for a different command");
                }
                TRB_TYPE_PORT_STS_CHANGE => {
                    crate::kprintln!("[xhci] port status change event noted, still waiting for command completion");
                }
                other => {
                    crate::kprintln!("[xhci] ignoring unexpected event type {} while waiting for command completion", other);
                }
            }
        }
        Err("gave up waiting for the matching command completion event after skipping 32 unrelated events")
    }

    /// Poll the event ring for the next TRB the controller has produced,
    /// spinning up to `max_spins` times. Advances the consumer index/cycle
    /// state and updates `ERDP` (clearing the Event Handler Busy bit) once
    /// a TRB is consumed, so repeated calls drain the ring in order.
    fn poll_event(&mut self, max_spins: u64) -> Option<Trb> {
        for _ in 0..max_spins {
            fence(Ordering::SeqCst);
            if let Some(trb) = self.evt_ring.peek_event(self.evt_index, self.evt_cycle) {
                self.evt_index += 1;
                if self.evt_index == RING_TRBS {
                    self.evt_index = 0;
                    self.evt_cycle = !self.evt_cycle;
                }
                let erdp_addr = self.evt_ring.phys
                    + (self.evt_index * core::mem::size_of::<Trb>()) as u64;
                unsafe {
                    w64(self.rt_base + RT_IR0 + IR_ERDP, erdp_addr | ERDP_EHB);
                }
                return Some(trb);
            }
            core::hint::spin_loop();
        }
        None
    }
}

/// Spin until `cond` is true, treating a very large bounded loop as "never
/// happening" rather than blocking forever on a wedged controller.
fn wait_for(mut cond: impl FnMut() -> bool, timeout_msg: &'static str) -> Result<(), &'static str> {
    for _ in 0..100_000_000u64 {
        if cond() { return Ok(()); }
        core::hint::spin_loop();
    }
    Err(timeout_msg)
}

/// Whether an xHCI controller was successfully brought up.
pub fn is_present() -> bool {
    XHCI.lock().is_some()
}
