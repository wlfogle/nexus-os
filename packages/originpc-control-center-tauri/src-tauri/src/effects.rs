//! Lighting effect animation loops for `start_effect`/`stop_effect`.
//!
//! Each effect computes a full-keyboard color frame from elapsed time and
//! hands it to `RgbController::apply_frame`, which only writes keys whose
//! color actually changed since the previous frame (see `hw::rgb`'s
//! diff-based `send_raw`) - this module just needs to produce frames, not
//! worry about redundant hidraw writes itself.
//!
//! Ported from the animation math in the Python app's `advanced_wave_effect`
//! / `breathing_effect` (`enhanced-professional-control-center.py`), adapted
//! to run as a single cancellable Tokio task instead of a persistent Python
//! thread polling a stop `Event`. The keymap assigns protocol indices
//! row-major (high nibble = row, low nibble = column - see
//! `hw::keymap::KEYBOARD_MAP`), so per-key (row, column) positions are
//! derived from the index instead of needing a separate spatial layout
//! table.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use clevo_hw::keymap::KEYBOARD_MAP;
use clevo_hw::{Color, RgbController};

/// Selectable lighting effects, per the frozen `start_effect` contract.
#[derive(Debug, Clone, Copy)]
pub enum Effect {
    Wave,
    Breathing,
    Rainbow,
}

impl Effect {
    pub fn parse(name: &str) -> Result<Self, String> {
        match name {
            "wave" => Ok(Effect::Wave),
            "breathing" => Ok(Effect::Breathing),
            "rainbow" => Ok(Effect::Rainbow),
            other => Err(format!(
                "unknown effect '{other}' (expected \"wave\", \"breathing\" or \"rainbow\")"
            )),
        }
    }
}

/// Runs `effect` at ~30 FPS until the surrounding Tokio task is aborted
/// (from `stop_effect`, or a fresh `start_effect` call replacing it).
pub async fn run(rgb: Arc<RgbController>, effect: Effect, speed: f64) {
    // Guard against zero/negative/NaN speed values from the frontend
    // instead of trusting them, since they'd otherwise freeze or spin the
    // animation.
    let speed = if speed.is_finite() { speed.clamp(0.1, 10.0) } else { 1.0 };
    let start = Instant::now();
    let mut ticker = tokio::time::interval(Duration::from_millis(33));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;
        let elapsed = start.elapsed().as_secs_f64();
        let frame = frame_for(effect, elapsed, speed);
        let rgb = rgb.clone();
        match tokio::task::spawn_blocking(move || rgb.apply_frame(&frame)).await {
            Ok(Ok(())) => {}
            // Device write failed (e.g. unplugged mid-effect) or the
            // blocking task panicked - stop the loop rather than spin
            // forever against a dead device.
            Ok(Err(_)) | Err(_) => break,
        }
    }
}

fn frame_for(effect: Effect, elapsed: f64, speed: f64) -> HashMap<&'static str, Color> {
    match effect {
        Effect::Wave => wave_frame(elapsed, speed),
        Effect::Breathing => breathing_frame(elapsed, speed),
        Effect::Rainbow => rainbow_frame(elapsed, speed),
    }
}

/// (key name, row, column) for every mapped key, derived once from the
/// protocol index.
fn key_positions() -> &'static [(&'static str, f64, f64)] {
    static POSITIONS: OnceLock<Vec<(&'static str, f64, f64)>> = OnceLock::new();
    POSITIONS.get_or_init(|| {
        KEYBOARD_MAP
            .iter()
            .map(|(&name, &index)| (name, (index >> 4) as f64, (index & 0x0F) as f64))
            .collect()
    })
}

/// Diagonal color wave sweeping across the keyboard, ported from
/// `advanced_wave_effect(wave_type='diagonal')`.
fn wave_frame(elapsed: f64, speed: f64) -> HashMap<&'static str, Color> {
    let positions = key_positions();
    let max_position = positions
        .iter()
        .fold(0.0_f64, |max, &(_, row, col)| max.max(row + col));
    let wave_position = (elapsed * speed * 4.0).rem_euclid(max_position + 6.0);

    positions
        .iter()
        .map(|&(name, row, col)| {
            let position = row + col;
            let distance = (position - wave_position).abs();
            let color = if distance <= 3.0 {
                let brightness = 1.0 - (distance / 3.0);
                let hue =
                    (wave_position * 20.0 + position * 30.0 + elapsed * 50.0).rem_euclid(360.0);
                hsv_to_rgb(hue, 1.0, brightness)
            } else {
                Color::OFF
            };
            (name, color)
        })
        .collect()
}

/// Full-saturation rainbow gradient scrolling across the keyboard columns.
fn rainbow_frame(elapsed: f64, speed: f64) -> HashMap<&'static str, Color> {
    key_positions()
        .iter()
        .map(|&(name, _row, col)| {
            let hue = (col * 18.0 + elapsed * speed * 60.0).rem_euclid(360.0);
            (name, hsv_to_rgb(hue, 1.0, 1.0))
        })
        .collect()
}

/// The reference app's default keyboard color (255, 102, 0 - see
/// `enhanced-professional-control-center.py`'s `current_color` fallback),
/// pulsed via a triangle wave ported from `breathing_effect`'s 0..100..0
/// brightness ramp. The Python version had no `speed` parameter; here it
/// scales the ramp's period.
const BREATHING_BASE: Color = Color::new(255, 102, 0);

fn breathing_frame(elapsed: f64, speed: f64) -> HashMap<&'static str, Color> {
    let period = 2.0 / speed;
    let phase = (elapsed % period) / period;
    let brightness = if phase < 0.5 { phase * 2.0 } else { (1.0 - phase) * 2.0 };
    let color = Color::new(
        (BREATHING_BASE.r as f64 * brightness).round() as u8,
        (BREATHING_BASE.g as f64 * brightness).round() as u8,
        (BREATHING_BASE.b as f64 * brightness).round() as u8,
    );
    key_positions()
        .iter()
        .map(|&(name, _, _)| (name, color))
        .collect()
}

/// Ported from the Python app's `RGBKeyboardController.hsv_to_rgb` static method.
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Color {
    let h = h / 360.0;
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match (i as i64).rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Color::new(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_all_contract_effects() {
        assert!(matches!(Effect::parse("wave"), Ok(Effect::Wave)));
        assert!(matches!(Effect::parse("breathing"), Ok(Effect::Breathing)));
        assert!(matches!(Effect::parse("rainbow"), Ok(Effect::Rainbow)));
    }

    #[test]
    fn parse_rejects_unknown_effect() {
        assert!(Effect::parse("sparkle").is_err());
    }

    #[test]
    fn hsv_to_rgb_matches_known_points() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), Color::new(255, 0, 0));
        assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), Color::new(0, 255, 0));
        assert_eq!(hsv_to_rgb(0.0, 0.0, 0.0), Color::OFF);
    }

    #[test]
    fn wave_frame_covers_every_mapped_key() {
        let frame = wave_frame(0.0, 1.0);
        assert_eq!(frame.len(), KEYBOARD_MAP.len());
    }

    #[test]
    fn breathing_frame_starts_dark_and_covers_every_key() {
        let frame = breathing_frame(0.0, 1.0);
        assert_eq!(frame.len(), KEYBOARD_MAP.len());
        assert_eq!(frame["esc"], Color::OFF);
    }
}
