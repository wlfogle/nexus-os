#!/usr/bin/env python3
"""Generate app icons without any image library.

Draws a simple mark -- stacked "shelves" over a dark rounded field -- directly
into an RGBA buffer and writes real PNGs using zlib + struct. Avoids adding
Pillow or an ImageMagick dependency just to produce a few icons.
"""

from __future__ import annotations

import os
import struct
import zlib

OUT = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")

BG = (17, 21, 28, 255)        # near-black slate
SHELF = (94, 176, 214, 255)   # muted cyan
ACCENT = (233, 178, 92, 255)  # warm amber


def rounded(x: int, y: int, w: int, h: int, r: int) -> bool:
    """Is (x, y) inside a w*h rounded rectangle with corner radius r?"""
    if x < r and y < r:
        return (r - x) ** 2 + (r - y) ** 2 <= r * r
    if x >= w - r and y < r:
        return (x - (w - r - 1)) ** 2 + (r - y) ** 2 <= r * r
    if x < r and y >= h - r:
        return (r - x) ** 2 + (y - (h - r - 1)) ** 2 <= r * r
    if x >= w - r and y >= h - r:
        return (x - (w - r - 1)) ** 2 + (y - (h - r - 1)) ** 2 <= r * r
    return True


def render(size: int) -> bytes:
    """Return raw RGBA rows for one icon."""
    s = size
    radius = max(2, s // 6)
    px = bytearray(s * s * 4)

    def put(x: int, y: int, c: tuple[int, int, int, int]) -> None:
        i = (y * s + x) * 4
        px[i:i + 4] = bytes(c)

    # background field
    for y in range(s):
        for x in range(s):
            put(x, y, BG if rounded(x, y, s, s, radius) else (0, 0, 0, 0))

    # three shelves of "books": horizontal bars with a gap, plus an amber spine
    margin = max(1, s // 6)
    inner = s - margin * 2
    bar_h = max(1, inner // 9)
    gap = max(1, inner // 9)

    top = margin + max(0, (inner - (bar_h * 3 + gap * 2)) // 2)
    for row in range(3):
        y0 = top + row * (bar_h + gap)
        # each shelf is slightly shorter than the one above it
        width = inner - row * (inner // 7)
        for y in range(y0, min(y0 + bar_h, s)):
            for x in range(margin, min(margin + width, s)):
                if rounded(x, y, s, s, radius):
                    put(x, y, SHELF)
        # amber spine marking the "current" volume on the middle shelf
        if row == 1:
            spine_w = max(1, bar_h)
            for y in range(y0, min(y0 + bar_h, s)):
                for x in range(margin + width - spine_w, min(margin + width, s)):
                    if 0 <= x < s and rounded(x, y, s, s, radius):
                        put(x, y, ACCENT)

    # PNG wants a filter byte at the start of every scanline
    raw = bytearray()
    for y in range(s):
        raw.append(0)
        raw += px[y * s * 4:(y + 1) * s * 4]
    return bytes(raw)


def chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png(path: str, size: int) -> None:
    raw = render(size)
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)  # 8-bit RGBA
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    with open(path, "wb") as fh:
        fh.write(png)
    print(f"{os.path.basename(path):<20} {size}x{size}  {len(png):>7} bytes")


def main() -> int:
    os.makedirs(OUT, exist_ok=True)
    targets = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
    }
    for name, size in targets.items():
        write_png(os.path.join(OUT, name), size)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
