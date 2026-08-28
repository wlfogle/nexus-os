//! Keyboard map for the OriginPC EON17-X (Clevo, ITE 8910 controller).
//!
//! Ported verbatim from `scripts/gaming/clevo-kbd-rgb` and the Python
//! `enhanced-professional-control-center.py` keyboard map, both already
//! validated on real hardware. Do not "fix" indices here without physically
//! re-testing on the device - these were derived from a real key-scan.

use std::collections::HashMap;

use once_cell::sync::Lazy;

/// Maps a lowercase key name (e.g. "a", "f1", "kp_plus") to its protocol index.
pub static KEYBOARD_MAP: Lazy<HashMap<&'static str, u8>> = Lazy::new(|| {
    HashMap::from([
        ("esc", 0x00),
        ("f1", 0x01), ("f2", 0x02), ("f3", 0x03), ("f4", 0x04),
        ("f5", 0x05), ("f6", 0x06), ("f7", 0x07), ("f8", 0x08),
        ("f9", 0x09), ("f10", 0x0A), ("f11", 0x0B), ("f12", 0x0C),
        ("prtsc", 0x0D), ("scroll", 0x0E), ("pause", 0x0F),
        ("home", 0x10), ("ins", 0x11), ("pgup", 0x12), ("pgdn", 0x13), ("del", 0x14), ("end", 0x15),
        ("`", 0x20), ("1", 0x21), ("2", 0x22), ("3", 0x23), ("4", 0x24), ("5", 0x25),
        ("6", 0x26), ("7", 0x27), ("8", 0x28), ("9", 0x29), ("0", 0x2A),
        ("-", 0x2B), ("=", 0x2D), ("backspace", 0x2E),
        ("numlock", 0x30), ("kp_divide", 0x31), ("kp_multiply", 0x32), ("kp_minus", 0x33),
        ("tab", 0x40), ("q", 0x42), ("w", 0x43), ("e", 0x44), ("r", 0x45),
        ("t", 0x46), ("y", 0x47), ("u", 0x48), ("i", 0x49), ("o", 0x4A),
        ("p", 0x4B), ("[", 0x4C), ("]", 0x4D), ("\\", 0x4E),
        ("kp_7", 0x50), ("kp_8", 0x51), ("kp_9", 0x52), ("kp_plus", 0x53),
        ("caps", 0x60), ("a", 0x62), ("s", 0x63), ("d", 0x64),
        ("f", 0x65), ("g", 0x66), ("h", 0x67), ("j", 0x68), ("k", 0x69),
        ("l", 0x6A), (";", 0x6B), ("'", 0x6C), ("enter", 0x6E),
        ("kp_4", 0x70), ("kp_5", 0x71), ("kp_6", 0x72),
        ("lshift", 0x80), ("z", 0x83), ("x", 0x84), ("c", 0x85),
        ("v", 0x86), ("b", 0x87), ("n", 0x88), ("m", 0x89), (",", 0x8A),
        (".", 0x8B), ("/", 0x8C), ("rshift", 0x8D), ("up", 0x8F),
        ("kp_1", 0x90), ("kp_2", 0x91), ("kp_3", 0x92), ("kp_enter", 0x93),
        ("lctrl", 0xA0), ("fn", 0xA2), ("super", 0xA3), ("lalt", 0xA4),
        ("space_left", 0xA5), ("space_center", 0xA6), ("space", 0xA8), ("space_far_right", 0xA9),
        ("ralt", 0xAA), ("menu", 0xAB), ("rctrl", 0xAC),
        ("left", 0xAE), ("down", 0xAF), ("right", 0xB0),
        ("kp_0", 0xB1), ("kp_period", 0xB2),
    ])
});

/// Named groups of keys used by the "Quick Key Groups" UI and by effects.
pub static KEY_GROUPS: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    HashMap::from([
        ("wasd", vec!["w", "a", "s", "d"]),
        ("arrow_keys", vec!["up", "left", "down", "right"]),
        ("function_keys", (1..=12).map(|i| Box::leak(format!("f{i}").into_boxed_str()) as &'static str).collect()),
        ("numbers", {
            let mut v = vec!["`"];
            v.extend((0..10).map(|i| Box::leak(i.to_string().into_boxed_str()) as &'static str));
            v.extend(["-", "="]);
            v
        }),
        ("letters", "qwertyuiopasdfghjklzxcvbnm".chars().map(|c| Box::leak(c.to_string().into_boxed_str()) as &'static str).collect()),
        ("keypad", vec!["numlock", "kp_divide", "kp_multiply", "kp_minus", "kp_7", "kp_8", "kp_9",
                        "kp_plus", "kp_4", "kp_5", "kp_6", "kp_1", "kp_2", "kp_3", "kp_enter", "kp_0", "kp_period"]),
        ("modifiers", vec!["lshift", "rshift", "lctrl", "rctrl", "lalt", "ralt", "fn", "super", "menu", "caps"]),
        ("spacebar", vec!["space_left", "space_center", "space", "space_far_right"]),
        ("navigation", vec!["ins", "home", "pgup", "del", "end", "pgdn"]),
        ("all_keys", KEYBOARD_MAP.keys().copied().collect()),
    ])
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_map_has_no_duplicate_indices() {
        // Every protocol index must map to exactly one key name, otherwise
        // two keys would silently fight over the same physical LED.
        let mut indices: Vec<u8> = KEYBOARD_MAP.values().copied().collect();
        let unique_count = indices.len();
        indices.sort_unstable();
        indices.dedup();
        assert_eq!(
            indices.len(),
            unique_count,
            "KEYBOARD_MAP contains duplicate protocol indices"
        );
    }

    #[test]
    fn keyboard_map_contains_well_known_keys() {
        for key in ["esc", "a", "z", "space", "kp_plus", "enter", "fn", "f12"] {
            assert!(KEYBOARD_MAP.contains_key(key), "missing expected key '{key}'");
        }
    }

    #[test]
    fn key_groups_only_reference_known_keys() {
        for (group, keys) in KEY_GROUPS.iter() {
            for key in keys {
                assert!(
                    KEYBOARD_MAP.contains_key(key),
                    "group '{group}' references unknown key '{key}'"
                );
            }
        }
    }

    #[test]
    fn all_keys_group_covers_every_mapped_key() {
        assert_eq!(KEY_GROUPS["all_keys"].len(), KEYBOARD_MAP.len());
    }
}
