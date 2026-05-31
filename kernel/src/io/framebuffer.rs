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
    addr:        *mut u8,
    width:       u64,
    height:      u64,
    pitch:       u64,   // bytes per row
    bpp:         u16,   // bits per pixel
    bytes_pp:    u64,   // bytes per pixel = bpp/8
    col:         u64,   // cursor column in pixels
    row:         u64,   // cursor row in pixels
    // RGB component values (not pre-packed — we pack on write using mask shifts)
    fg_r: u8,  fg_g: u8,  fg_b: u8,
    bg_r: u8,  bg_g: u8,  bg_b: u8,
    // Limine-reported pixel layout
    red_shift:   u8,
    green_shift: u8,
    blue_shift:  u8,
}

unsafe impl Send for FbConsole {}

impl FbConsole {
    // Each 8x8 font glyph is rendered at SCALE×SCALE pixels per dot.
    // At 1280x800, SCALE=2 gives 16x16 character cells (80 cols × 50 rows).
    const SCALE:  u64 = 2;
    const CHAR_W: u64 = 8 * Self::SCALE;
    const CHAR_H: u64 = 8 * Self::SCALE;

    const fn uninit() -> Self {
        Self {
            addr:        core::ptr::null_mut(),
            width:       0,
            height:      0,
            pitch:       0,
            bpp:         0,
            bytes_pp:    4,
            col:         0,
            row:         0,
            fg_r: 0xEA, fg_g: 0xEA, fg_b: 0xEA,  // light grey
            bg_r: 0x1A, bg_g: 0x1A, bg_b: 0x2E,  // dark blue-black
            // Default XRGB — Limine will override on init()
            red_shift:   16,
            green_shift: 8,
            blue_shift:  0,
        }
    }

    /// Pack R, G, B bytes into a pixel word using the actual framebuffer layout.
    #[inline]
    fn pack(&self, r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << self.red_shift)
            | ((g as u32) << self.green_shift)
            | ((b as u32) << self.blue_shift)
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
                let fg = self.pack(self.fg_r, self.fg_g, self.fg_b);
                let bg = self.pack(self.bg_r, self.bg_g, self.bg_b);
                for (gy, &row_bits) in glyph.iter().enumerate() {
                    for gx in 0..8u64 {
                        let bit = (row_bits >> gx) & 1; // LSB = leftmost pixel
                        let colour = if bit != 0 { fg } else { bg };
                        // Draw SCALE×SCALE block per font pixel
                        for sy in 0..Self::SCALE {
                            for sx in 0..Self::SCALE {
                                let px = self.col + gx * Self::SCALE + sx;
                                let py = self.row + gy as u64 * Self::SCALE + sy;
                                self.put_pixel(px, py, colour);
                            }
                        }
                    }
                }
                self.col += Self::CHAR_W;
            }
            _ => {}
        }
    }

    #[inline]
    fn put_pixel(&mut self, x: u64, y: u64, colour: u32) {
        if self.addr.is_null() { return; }
        let offset = (y * self.pitch + x * self.bytes_pp) as usize;
        unsafe {
            // Write only as many bytes as bpp dictates
            match self.bytes_pp {
                4 => {
                    let ptr = self.addr.add(offset) as *mut u32;
                    core::ptr::write_volatile(ptr, colour);
                }
                3 => {
                    let ptr = self.addr.add(offset);
                    core::ptr::write_volatile(ptr,             (colour & 0xFF) as u8);
                    core::ptr::write_volatile(ptr.add(1), ((colour >> 8)  & 0xFF) as u8);
                    core::ptr::write_volatile(ptr.add(2), ((colour >> 16) & 0xFF) as u8);
                }
                _ => {}
            }
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

    // Log actual fb parameters to serial for debugging
    crate::kprintln!(
        "[fb]   {}x{} bpp={} pitch={} R<<{} G<<{} B<<{}",
        fb.width, fb.height, fb.bpp, fb.pitch,
        fb.red_mask_shift, fb.green_mask_shift, fb.blue_mask_shift
    );

    let bpp        = fb.bpp;
    let bytes_pp   = (bpp as u64) / 8;

    let mut c = CONSOLE.lock();
    c.addr        = fb.address.as_ptr().unwrap_or(core::ptr::null_mut());
    c.width       = fb.width;
    c.height      = fb.height;
    c.pitch       = fb.pitch;
    c.bpp         = bpp;
    c.bytes_pp    = bytes_pp;
    c.col         = 0;
    c.row         = 0;
    c.red_shift   = fb.red_mask_shift;
    c.green_shift = fb.green_mask_shift;
    c.blue_shift  = fb.blue_mask_shift;

    // Clear to background colour
    if !c.addr.is_null() && (bpp == 32 || bpp == 24) {
        let (bgr, bgg, bgb) = (c.bg_r, c.bg_g, c.bg_b);
        let bg = c.pack(bgr, bgg, bgb);
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
