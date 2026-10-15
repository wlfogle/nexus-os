#!/usr/bin/env python3
"""Generate a minimal, spec-conformant FDT for emulator testing.

Why this exists
---------------
QEMU's `raspi4b` machine does not synthesize a device tree the way `virt`
does -- it expects an externally supplied DTB, exactly as real Pi 4 firmware
provides one. Without a `-dtb`, the kernel sees no device tree, platform
detection returns `Unknown`, and it falls back to QEMU virt's UART base. On
a Pi that is simply the wrong device, so nothing is ever printed.

This emits the smallest device tree that still exercises the real detection
path: a root node whose `compatible` property carries the board strings
`arch::aarch64::platform::classify_compatible` matches on.

Scope and honesty
-----------------
This is a **test fixture, not a substitute for the real DTB**. It proves
platform classification and UART base selection, nothing more. It contains
no `/memory`, `/soc`, clocks or peripherals, so it cannot stand in for the
real `bcm2711-rpi-4-b.dtb` once the kernel starts deriving a memory map or
programming real peripherals. The upstream parser was already verified
host-side against the genuine Pi 4 blob (see platform.rs module docs); this
only covers what happens *after* that parse succeeds.

Usage
-----
    make-test-dtb.py rpi4 > /tmp/rpi4.dtb
    make-test-dtb.py virt > /tmp/virt.dtb

Then:
    qemu-system-aarch64 -M raspi4b -kernel Image -dtb /tmp/rpi4.dtb ...
"""

import struct
import sys

FDT_MAGIC = 0xD00DFEED
FDT_BEGIN_NODE = 0x00000001
FDT_END_NODE = 0x00000002
FDT_PROP = 0x00000003
FDT_END = 0x00000009

# Root `compatible` values, byte-for-byte as the real blobs carry them.
# The Pi 4 publishes two NUL-separated strings, most-specific first; QEMU
# virt publishes one. Both confirmed against real DTBs via `dtc` -- see the
# verification notes in platform.rs.
BOARDS = {
    "rpi4": b"raspberrypi,4-model-b\x00brcm,bcm2711\x00",
    "virt": b"linux,dummy-virt\x00",
}


def pad4(b: bytes) -> bytes:
    """Pad to a 4-byte boundary, as the structure block requires."""
    return b + b"\x00" * ((-len(b)) % 4)


def build(compatible: bytes) -> bytes:
    # Strings block. `#address-cells` and `#size-cells` are not optional
    # decoration: QEMU parses the DTB itself before the guest ever runs and
    # aborts with FDT_ERR_NOTFOUND if the root node lacks them, so a blob
    # carrying only `compatible` never reaches the kernel at all.
    name_compatible = b"compatible\x00"
    name_addr_cells = b"#address-cells\x00"
    name_size_cells = b"#size-cells\x00"
    strings = name_compatible + name_addr_cells + name_size_cells
    off_compatible = 0
    off_addr_cells = len(name_compatible)
    off_size_cells = off_addr_cells + len(name_addr_cells)

    def prop(nameoff: int, value: bytes) -> bytes:
        return (
            struct.pack(">I", FDT_PROP)
            + struct.pack(">I", len(value))
            + struct.pack(">I", nameoff)
            + pad4(value)
        )

    # Structure block: a single root node carrying three properties.
    # 2 address cells / 2 size cells is what both the real Pi 4 and QEMU
    # virt device trees use for 64-bit addressing.
    struct_block = b""
    struct_block += struct.pack(">I", FDT_BEGIN_NODE)
    struct_block += pad4(b"\x00")            # root node name is the empty string
    struct_block += prop(off_addr_cells, struct.pack(">I", 2))
    struct_block += prop(off_size_cells, struct.pack(">I", 2))
    struct_block += prop(off_compatible, compatible)
    struct_block += struct.pack(">I", FDT_END_NODE)
    struct_block += struct.pack(">I", FDT_END)

    header_size = 40
    # Memory reservation block is mandatory and 8-byte aligned; a single
    # all-zero entry terminates an empty list.
    rsvmap = struct.pack(">QQ", 0, 0)

    off_mem_rsvmap = header_size
    off_dt_struct = off_mem_rsvmap + len(rsvmap)
    off_dt_strings = off_dt_struct + len(struct_block)
    totalsize = off_dt_strings + len(strings)

    header = struct.pack(
        ">IIIIIIIIII",
        FDT_MAGIC,
        totalsize,
        off_dt_struct,
        off_dt_strings,
        off_mem_rsvmap,
        17,            # version
        16,            # last_comp_version
        0,             # boot_cpuid_phys
        len(strings),  # size_dt_strings
        len(struct_block),
    )
    return header + rsvmap + struct_block + strings


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in BOARDS:
        print(f"usage: {sys.argv[0]} {{{'|'.join(BOARDS)}}}", file=sys.stderr)
        return 2
    sys.stdout.buffer.write(build(BOARDS[sys.argv[1]]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
