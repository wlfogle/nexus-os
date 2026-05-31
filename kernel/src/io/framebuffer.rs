//! NexusOS Framebuffer Text Console (laptop only)
//!
//! Renders ASCII text to the Limine framebuffer using an embedded 8×8
//! bitmap font.  Output is also always mirrored to COM1 serial.

use core::fmt;
use spin::Mutex;
use limine::FramebufferResponse;

// ─── 8×8 bitmap font (printable ASCII 0x20..=0x7e) ──────────────────────────
// Each character is 8 bytes, one bit per pixel (MSB = leftmost pixel).
// This is a minimal but complete font covering all printable characters.
include!("font8x8.rs");

// ─── Console state ───────────────────────────────────────────────────────────

struct FbConsole {
    addr:       *mut u8,
    width:      u64,
    height:     u64,
    pitch:      u64,
    bpp:        u16,        // bits per pixel — we only support 32
    col:        u64,        // current cursor column (pixels)
    row:        u64,        // current cursor row (pixels)
    fg:         u32,        // foreground colour (0x00RRGGBB)
    bg:         u32,        // background colour
}

unsafe impl Send for FbConsole {}

impl FbConsole {
    const CHAR_W: u64 = 8;
    const CHAR_H: u64 = 8;

    const fn uninit() -> Self {
        Self {
            addr:   core::ptr::null_mut(),
            width:  0,
            height: 0,
            pitch:  0,
            bpp:    0,
            col:    0,
            row:    0,
            fg:     0x00EAEAEA,  // light grey
            bg:     0x001A1A2E,  // dark blue-black
        }
    }

    fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.col = 0;
                self.row += Self::CHAR_H;
                if self.row + Self::CHAR_H > self.height {
                    self.scroll();
                }
            }
            '\r' => {
                self.col = 0;
            }
            c if (c as u32) >= 0x20 && (c as u32) <= 0x7e => {
                if self.col + Self::CHAR_W > self.width {
                    self.col = 0;
                    self.row += Self::CHAR_H;
                    if self.row + Self::CHAR_H > self.height {
                        self.scroll();
                    }
                }
                let glyph_idx = (c as usize) - 0x20;
                let glyph = &FONT8X8[glyph_idx];
                for (gy, &row_bits) in glyph.iter().enumerate() {
                    for gx in 0..8u64 {
                        let bit = (row_bits >> (7 - gx)) & 1;
                        let colour = if bit != 0 { self.fg } else { self.bg };
                        let px = self.col + gx;
                        let py = self.row + gy as u64;
                        self.put_pixel(px, py, colour);
                    }
                }
                self.col += Self::CHAR_W;
            }
            _ => {}
        }
    }

    #[inline]
    fn put_pixel(&mut self, x: u64, y: u64, colour: u32) {
        if self.addr.is_null() || self.bpp != 32 { return; }
        let offset = (y * self.pitch + x * 4) as usize;
        unsafe {
            let ptr = self.addr.add(offset) as *mut u32;
            core::ptr::write_volatile(ptr, colour);
        }
    }

    fn scroll(&mut self) {
        // Move all rows up by CHAR_H pixels
        let row_bytes = (self.pitch * Self::CHAR_H) as usize;
        let total = (self.pitch * self.height) as usize;
        unsafe {
            core::ptr::copy(
                self.addr.add(row_bytes),
                self.addr,
                total - row_bytes,
            );
            // Clear the last row
            core::ptr::write_bytes(
                self.addr.add(total - row_bytes),
                0,
                row_bytes,
            );
        }
        // Keep row at last line
        self.row = self.height - Self::CHAR_H;
    }
}

impl fmt::Write for FbConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            self.write_char(ch);
        }
        Ok(())
    }
}

// ─── Singleton ───────────────────────────────────────────────────────────────

static CONSOLE: Mutex<FbConsole> = Mutex::new(FbConsole::uninit());

/// Initialise the framebuffer console from a Limine framebuffer response.
pub fn init(fb_resp: &FramebufferResponse) {
    let fbs = fb_resp.framebuffers();
    if fbs.is_empty() { return; }
    // NonNullPtr<Framebuffer> implements Deref<Target = Framebuffer>
    let fb = &*fbs[0];

    let mut c = CONSOLE.lock();
    c.addr   = fb.address.as_ptr().unwrap_or(core::ptr::null_mut());
    c.width  = fb.width;
    c.height = fb.height;
    c.pitch  = fb.pitch;
    c.bpp    = fb.bpp;
    c.col    = 0;
    c.row    = 0;

    // Clear to background colour
    if !c.addr.is_null() && c.bpp == 32 {
        let bg = c.bg;
        for y in 0..c.height {
            for x in 0..c.width {
                c.put_pixel(x, y, bg);
            }
        }
    }
}

/// Internal: write formatted text to framebuffer.
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    CONSOLE.lock().write_fmt(args).ok();
}
