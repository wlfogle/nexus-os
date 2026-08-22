//! RGB keyboard protocol for the ITE 8910 controller (USB `048d:8910`).
//!
//! Protocol (validated on real hardware, unchanged from the Python/CLI
//! implementations): a 16-byte feature report
//! `[0xCC, 0x01, key_index, r, g, b, 0x00 * 10]` written to `/dev/hidrawN`.
//!
//! Unlike the ported Python app - which opened and closed the device for
//! *every single key write* - this holds one persistent handle behind a
//! mutex, reconnecting on write failure. This is a deliberate reliability
//! and performance fix, not just a language port.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

use thiserror::Error;

use crate::keymap::{KEYBOARD_MAP, KEY_GROUPS};

/// Candidate hidraw device paths, tried in order. The real device is
/// usually hidraw0, but other hidraw nodes may enumerate first depending on
/// USB topology, so all are probed the same way the Python app did.
const CANDIDATE_DEVICES: &[&str] = &["/dev/hidraw0", "/dev/hidraw1", "/dev/hidraw2", "/dev/hidraw3"];

#[derive(Debug, Error)]
pub enum RgbError {
    #[error("no writable RGB hidraw device found (checked {0:?})")]
    DeviceNotFound(Vec<&'static str>),
    #[error("unknown key name: {0}")]
    UnknownKey(String),
    #[error("unknown key group: {0}")]
    UnknownGroup(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, RgbError>;

/// A single RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const OFF: Color = Color::new(0, 0, 0);
}

/// Holds the open device handle plus the last-written color per key, so
/// effects and repeated calls can skip writes that would be no-ops (the
/// "diff-based effects" optimization).
struct DeviceState {
    file: File,
    path: &'static str,
    last_written: HashMap<u8, Color>,
}

/// Thread-safe RGB controller. Cheap to clone (wrap in `Arc`) since the
/// actual state lives behind the mutex.
pub struct RgbController {
    state: Mutex<Option<DeviceState>>,
}

impl Default for RgbController {
    fn default() -> Self {
        Self::new()
    }
}

impl RgbController {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }

    /// True if a writable RGB device is currently reachable (opening it if
    /// not already open). Used by the UI's connection-status indicator.
    pub fn is_connected(&self) -> bool {
        self.with_device(|_| Ok(())).is_ok()
    }

    /// Runs `f` with a valid, open device handle, opening/reopening it as
    /// needed. On write errors elsewhere, callers should call
    /// `reset_connection()` so the next call reopens from scratch.
    fn with_device<T>(&self, f: impl FnOnce(&mut DeviceState) -> Result<T>) -> Result<T> {
        let mut guard = self.state.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Self::open_device()?);
        }
        let state = guard.as_mut().unwrap();
        match f(state) {
            Ok(v) => Ok(v),
            Err(e) => {
                // Drop the handle so the next call retries discovery -
                // covers the device disappearing/reappearing (unplug,
                // suspend/resume) instead of wedging forever.
                *guard = None;
                Err(e)
            }
        }
    }

    fn open_device() -> Result<DeviceState> {
        for &path in CANDIDATE_DEVICES {
            if let Ok(file) = OpenOptions::new().write(true).open(path) {
                return Ok(DeviceState {
                    file,
                    path,
                    last_written: HashMap::new(),
                });
            }
        }
        Err(RgbError::DeviceNotFound(CANDIDATE_DEVICES.to_vec()))
    }

    /// Path of the currently-open device, if any (for diagnostics/UI).
    pub fn current_device_path(&self) -> Option<&'static str> {
        self.state.lock().unwrap().as_ref().map(|s| s.path)
    }

    fn send_raw(state: &mut DeviceState, key_index: u8, color: Color, force: bool) -> Result<()> {
        if !force && state.last_written.get(&key_index) == Some(&color) {
            return Ok(()); // diff-based skip: already this color
        }
        let mut packet = [0u8; 16];
        packet[0] = 0xCC;
        packet[1] = 0x01;
        packet[2] = key_index;
        packet[3] = color.r;
        packet[4] = color.g;
        packet[5] = color.b;
        state.file.write_all(&packet)?;
        state.last_written.insert(key_index, color);
        Ok(())
    }

    /// Set a single named key's color.
    pub fn set_key_color(&self, key_name: &str, color: Color) -> Result<()> {
        let key_index = *KEYBOARD_MAP
            .get(key_name.to_lowercase().as_str())
            .ok_or_else(|| RgbError::UnknownKey(key_name.to_string()))?;
        self.with_device(|state| Self::send_raw(state, key_index, color, false))
    }

    /// Set every key in a named group to the same color.
    pub fn set_group_color(&self, group: &str, color: Color) -> Result<()> {
        let keys = KEY_GROUPS
            .get(group)
            .ok_or_else(|| RgbError::UnknownGroup(group.to_string()))?;
        self.with_device(|state| {
            for &key in keys {
                if let Some(&idx) = KEYBOARD_MAP.get(key) {
                    Self::send_raw(state, idx, color, false)?;
                }
            }
            Ok(())
        })
    }

    /// Clear (turn off) every mapped key. Always forces a write even if a
    /// key is already believed to be off, since this is also used as a
    /// recovery path after firmware-side state gets out of sync (e.g. the
    /// `kp_plus` cyan-residue issue seen on this hardware after lid close).
    pub fn clear_all_keys(&self) -> Result<()> {
        self.with_device(|state| {
            let indices: Vec<u8> = KEYBOARD_MAP.values().copied().collect();
            for idx in indices {
                Self::send_raw(state, idx, Color::OFF, true)?;
            }
            Ok(())
        })
    }

    /// Apply a full-keyboard frame in one call, writing only keys whose
    /// color actually changed since the last frame (the "diff-based
    /// effects" optimization) - used by wave/breathing/rainbow effects
    /// running at animation frame rates.
    pub fn apply_frame(&self, frame: &HashMap<&'static str, Color>) -> Result<()> {
        self.with_device(|state| {
            for (&key, &color) in frame {
                if let Some(&idx) = KEYBOARD_MAP.get(key) {
                    Self::send_raw(state, idx, color, false)?;
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_off_is_all_zero() {
        assert_eq!(Color::OFF, Color::new(0, 0, 0));
    }

    #[test]
    fn set_key_color_rejects_unknown_key() {
        let controller = RgbController::new();
        let err = controller.set_key_color("not_a_real_key", Color::new(1, 2, 3));
        assert!(matches!(err, Err(RgbError::UnknownKey(_))));
    }

    #[test]
    fn set_group_color_rejects_unknown_group() {
        let controller = RgbController::new();
        let err = controller.set_group_color("not_a_real_group", Color::new(1, 2, 3));
        assert!(matches!(err, Err(RgbError::UnknownGroup(_))));
    }
}
