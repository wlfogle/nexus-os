//! NexusOS USB HID Boot-Protocol Report Parser — Phase K4
//!
//! Thin layer on top of `drivers::xhci`: turns raw boot-protocol keyboard
//! (8-byte) and mouse (3-byte) reports into the exact same
//! `io::keyboard::push_translated` / `io::mouse::push_event` calls the PS/2
//! drivers use, so everything above those two modules can't tell (and
//! doesn't need to know) whether input came from PS/2 or USB.
//!
//! Report shapes are fixed by the USB HID 1.11 boot protocol (Appendix B):
//!   Keyboard: [modifiers, reserved, keycode0..keycode5]  (8 bytes)
//!   Mouse:    [buttons, dx, dy, ...]                     (>= 3 bytes)
//!
//! `poll()` is meant to be called periodically (see `task_usb_hid_poll` in
//! `main.rs`, which drives it once per timer tick from a dedicated kernel
//! thread) rather than tied to a hardware interrupt -- `drivers::xhci` is a
//! polling driver throughout, matching this codebase's other bus/block
//! drivers.

use spin::Mutex;
use crate::drivers::xhci::{self, HidKind};
use crate::io::{keyboard, mouse};

// --- USB HID keyboard usage IDs (0x04..=0x38) -> ASCII -----------------------
//
// Index = usage ID - 0x04. Mirrors io::keyboard's PS/2 scancode-set-1
// NORMAL/SHIFTED tables in spirit (0 = unmapped), just keyed by the USB HID
// Keyboard/Keypad usage page instead of a PS/2 scancode.

#[rustfmt::skip]
static NORMAL: [u8; 0x35] = [
    // 0x04-0x1D: a-z
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j',
    b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't',
    b'u', b'v', b'w', b'x', b'y', b'z',
    // 0x1E-0x27: 1-9, 0
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0',
    // 0x28-0x38
    10,    // Enter
    27,    // Escape
    8,     // Backspace
    9,     // Tab
    b' ', // Space
    b'-', b'=', b'[', b']', b'\\',
    0,     // Non-US # (unmapped)
    b';', b'\'', b'`', b',', b'.', b'/',
];

#[rustfmt::skip]
static SHIFTED: [u8; 0x35] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J',
    b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
    b'U', b'V', b'W', b'X', b'Y', b'Z',
    b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')',
    10, 27, 8, 9, b' ',
    b'_', b'+', b'{', b'}', b'|',
    0,
    b':', b'"', b'~', b'<', b'>', b'?',
];

// Modifier byte (report[0]) bits.
const MOD_LSHIFT: u8 = 1 << 1;
const MOD_RSHIFT: u8 = 1 << 5;
const MOD_LCTRL:  u8 = 1 << 0;
const MOD_RCTRL:  u8 = 1 << 4;

/// Last report's held keycodes, used to edge-detect presses: boot reports
/// list every *currently held* key on every report, not press/release
/// events, so pushing every code in every report would flood the input
/// buffer with the same character on every poll tick for as long as a key
/// stays down. Only keycodes that are new since the last report are pushed.
static LAST_KEYS: Mutex<[u8; 6]> = Mutex::new([0u8; 6]);

/// Call periodically (e.g. once per timer tick) to drain any pending USB
/// HID report and translate it into the shared keyboard/mouse input paths.
/// Non-blocking: returns immediately whether or not a report was available.
pub fn poll() {
    let mut buf = [0u8; 8];
    if let Some((n, kind)) = xhci::poll_hid_report(&mut buf) {
        match kind {
            HidKind::Keyboard => handle_keyboard_report(&buf[..n]),
            HidKind::Mouse    => handle_mouse_report(&buf[..n]),
            HidKind::Other    => {}
        }
    }
}

fn handle_keyboard_report(report: &[u8]) {
    if report.len() < 8 { return; }
    let modifiers = report[0];
    let shift = modifiers & (MOD_LSHIFT | MOD_RSHIFT) != 0;
    let ctrl  = modifiers & (MOD_LCTRL  | MOD_RCTRL)  != 0;

    let mut current = [0u8; 6];
    current.copy_from_slice(&report[2..8]);

    let mut last = LAST_KEYS.lock();
    for &usage in &current {
        if usage < 0x04 { continue; } // 0 = no key; 1-3 = reserved/error-rollover codes
        if last.contains(&usage) { continue; } // still held from the previous report -- not a new press
        let idx = (usage - 0x04) as usize;
        if idx >= NORMAL.len() { continue; }
        let mut ch = if shift { SHIFTED[idx] } else { NORMAL[idx] };
        if ch == 0 { continue; }
        if ctrl && ch.is_ascii_alphabetic() {
            ch = ch.to_ascii_uppercase() - b'@'; // 'A'-'@'=1 ... 'Z'-'@'=26, matching io::keyboard's PS/2 path
        }
        keyboard::push_translated(ch);
    }
    *last = current;
}

fn handle_mouse_report(report: &[u8]) {
    if report.len() < 3 { return; }
    let buttons = report[0];
    let dx = report[1] as i8 as i16;
    let dy = report[2] as i8 as i16;
    mouse::push_event(mouse::MouseEvent {
        dx,
        // USB HID boot mouse reports already use +Y = downward on screen
        // (per the HID boot protocol's de-facto implementation convention
        // across every mainstream OS's boot driver) -- unlike PS/2, whose
        // raw hardware packets report +Y = up and get flipped in
        // io::mouse::handle_irq. No flip needed here.
        dy,
        left:   buttons & 0x01 != 0,
        right:  buttons & 0x02 != 0,
        middle: buttons & 0x04 != 0,
    });
}
