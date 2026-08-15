// Mirrors `hw/src/keymap.rs`'s `KEYBOARD_MAP` and `KEY_GROUPS`. This is a
// plain UI-only data file (there is no "list keys/groups" command in
// CONTRACT.md), used to populate the RGB Control tab's group buttons and
// per-key picker. The actual protocol index lookup and validation always
// happens in Rust (`RgbController::set_key_color`/`set_group_color`) - if a
// name here were ever wrong, the backend would reject it with an
// `UnknownKey`/`UnknownGroup` error rather than silently misbehaving.
// If `hw/src/keymap.rs` ever changes, update this file to match.

export interface KeyGroupOption {
  id: string;
  label: string;
}

export const KEY_GROUPS: KeyGroupOption[] = [
  { id: "all_keys", label: "All Keys" },
  { id: "wasd", label: "WASD" },
  { id: "arrow_keys", label: "Arrow Keys" },
  { id: "function_keys", label: "Function Keys (F1-F12)" },
  { id: "numbers", label: "Number Row" },
  { id: "letters", label: "Letters" },
  { id: "keypad", label: "Numeric Keypad" },
  { id: "modifiers", label: "Modifiers" },
  { id: "spacebar", label: "Spacebar" },
  { id: "navigation", label: "Navigation Cluster" },
];

// Every key name accepted by `set_key_color`, in physical-layout order.
export const KEY_NAMES: string[] = [
  "esc",
  "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12",
  "prtsc", "scroll", "pause",
  "home", "ins", "pgup", "pgdn", "del", "end",
  "`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "backspace",
  "numlock", "kp_divide", "kp_multiply", "kp_minus",
  "tab", "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]", "\\",
  "kp_7", "kp_8", "kp_9", "kp_plus",
  "caps", "a", "s", "d", "f", "g", "h", "j", "k", "l", ";", "'", "enter",
  "kp_4", "kp_5", "kp_6",
  "lshift", "z", "x", "c", "v", "b", "n", "m", ",", ".", "/", "rshift", "up",
  "kp_1", "kp_2", "kp_3", "kp_enter",
  "lctrl", "fn", "super", "lalt", "space_left", "space_center", "space", "space_far_right",
  "ralt", "menu", "rctrl", "left", "down", "right",
  "kp_0", "kp_period",
];
