//! `clevo-hw`: shared, GUI-independent hardware layer for the OriginPC
//! EON17-X. Used by the Tauri backend (`src-tauri`), the standalone
//! `lid-monitor` binary, and the Flexikey key-remap engine, so the RGB
//! protocol and keymap are implemented exactly once.
//!
//! See `CONTRACT.md` at the package root for the frozen command/event
//! contract the Tauri backend and frontend build against.

pub mod flexikey;
pub mod keymap;
pub mod power;
pub mod rgb;
pub mod sensors;

pub use power::{PowerInfo, PowerProfile, PowerReader};
pub use rgb::{Color, RgbController, RgbError};
pub use sensors::{SensorReader, SensorSnapshot, TemperatureReading};
