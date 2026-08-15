//! Flexikey: Linux equivalent of Clevo's Flexikey(R) feature - lets any key
//! on the physical keyboard be remapped, turned into a key combo, a text
//! macro, a program launcher, or disabled entirely. Per Clevo's own
//! manuals this is purely a software feature (not a hardware protocol), so
//! it is implemented entirely at the Linux input layer via `evdev`
//! (grabbing the physical keyboard) and `uinput` (emitting the
//! remapped/macro output through a virtual keyboard) - exactly what the
//! ported Python `flexikey.py` did.
//!
//! Profiles are stored as JSON, in the same location and shape as the
//! Python version so a user's already-configured profiles carry over
//! unchanged: `~/.config/originpc-control-center/flexikey/` (`profiles.json`
//! index + one `<name>.json` per profile).
//!
//! Two deliberate fixes over the Python reference, both required for this
//! to be genuinely usable rather than just a port:
//!
//! 1. The Python daemon read the grabbed device with a plain blocking
//!    `read_loop()`; `stop()` could only take effect once another key was
//!    pressed, since the reading thread was parked in a blocking `read()`
//!    syscall with no way to interrupt it. Here the device is switched to
//!    non-blocking mode and the engine thread polls it, rechecking the
//!    stop flag every `POLL_INTERVAL` - so `stop_flexikey_engine` returns
//!    promptly instead of hanging indefinitely.
//! 2. The Python engine only ever matched `EV_KEY` events whose keystate
//!    was exactly "key down" before deciding whether to run a mapped
//!    action or re-emit the key; both key-up and autorepeat events were
//!    silently dropped in *both* branches. For passthrough (unmapped) keys
//!    this means the virtual device would see a press with no matching
//!    release, i.e. every ordinary, non-remapped key would appear stuck
//!    "held down" forever from the perspective of anything reading the
//!    virtual device - clearly broken for daily typing. Here, passthrough
//!    keys forward every event value (press/release/repeat) unchanged;
//!    only mapped keys are edge-triggered (action fires once, on press).

use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, Device, EventSummary, EventType, InputEvent, KeyCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Matches the real Flexikey feature's profile count, and the Python
/// implementation's `MAX_PROFILES`.
const MAX_PROFILES: usize = 12;
/// Highest standard `KEY_*` code registered on the virtual output device -
/// matches the Python implementation's `set(range(1, 249))` bound, which
/// covers every named key from `KEY_ESC` (1) through `KEY_MICMUTE` (248).
const MAX_REGISTERED_KEYCODE: u16 = 248;
/// How long a synthesized tap/combo holds keys down before releasing them,
/// matching the Python implementation's `time.sleep(0.01)`.
const SYNTH_PRESS_DURATION: Duration = Duration::from_millis(10);
/// How often the engine's read loop rechecks the stop flag while the
/// keyboard device has no pending events.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, Error)]
pub enum FlexikeyError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid profile JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown Flexikey profile: {0}")]
    UnknownProfile(String),
    #[error("maximum of {MAX_PROFILES} Flexikey profiles reached")]
    ProfileLimitReached,
    #[error("no candidate keyboard input device found")]
    NoKeyboardFound,
    #[error("unknown key code: {0}")]
    UnknownKey(String),
    #[error("no active Flexikey profile is set")]
    NoActiveProfile,
    #[error("active Flexikey profile has no mappings configured")]
    NoMappingsConfigured,
    #[error("invalid shell command: {0}")]
    InvalidCommand(String),
    #[error("Flexikey engine is already running")]
    AlreadyRunning,
    #[error("Flexikey engine is not running")]
    NotRunning,
    #[error("Flexikey engine thread panicked")]
    EngineThreadPanicked,
}

pub type Result<T> = std::result::Result<T, FlexikeyError>;

// ---------------------------------------------------------------------------
// Profile storage
// ---------------------------------------------------------------------------

/// The `profiles.json` index: which profiles exist and which is active.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfilesIndex {
    pub active_profile: Option<String>,
    #[serde(default)]
    pub profiles: Vec<String>,
}

/// A single profile's mappings, stored as `<name>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub mappings: HashMap<String, Action>,
}

/// What happens when a mapped key is pressed. Key names throughout (map
/// keys, `target`, `keys`) are raw evdev key names such as `"KEY_F13"` -
/// exactly the strings produced by `capture_next_key` and consumed by
/// `evdev::KeyCode`'s `FromStr`/`Debug` impls - matching the Python
/// implementation's use of `getattr(ecodes, key_name)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "remap")]
    Remap { target: String },
    #[serde(rename = "combo")]
    Combo { keys: Vec<String> },
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "launch")]
    Launch { command: String },
    #[serde(rename = "disabled")]
    Disabled,
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn config_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));
    base.join("originpc-control-center").join("flexikey")
}

fn profiles_index_path() -> PathBuf {
    config_dir().join("profiles.json")
}

fn ensure_config_dir() -> Result<()> {
    fs::create_dir_all(config_dir())?;
    Ok(())
}

/// Sanitizes a profile name into a safe filename, matching the Python
/// implementation's `''.join(c if c.isalnum() or c in '-_' else '_' ...)`.
fn profile_path(name: &str) -> PathBuf {
    let safe: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    config_dir().join(format!("{safe}.json"))
}

pub fn load_profiles_index() -> Result<ProfilesIndex> {
    ensure_config_dir()?;
    let path = profiles_index_path();
    if !path.exists() {
        return Ok(ProfilesIndex::default());
    }
    // A corrupt index is recreated from scratch rather than surfaced as a
    // hard error, matching the Python implementation's recovery behavior.
    match fs::read_to_string(&path) {
        Ok(contents) => Ok(serde_json::from_str(&contents).unwrap_or_default()),
        Err(_) => Ok(ProfilesIndex::default()),
    }
}

pub fn save_profiles_index(index: &ProfilesIndex) -> Result<()> {
    ensure_config_dir()?;
    let contents = serde_json::to_string_pretty(index)?;
    fs::write(profiles_index_path(), contents)?;
    Ok(())
}

pub fn load_profile(name: &str) -> Result<Profile> {
    let path = profile_path(name);
    if !path.exists() {
        return Ok(Profile {
            name: name.to_string(),
            mappings: HashMap::new(),
        });
    }
    let contents = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn save_profile(profile: &Profile) -> Result<()> {
    let mut index = load_profiles_index()?;
    if !index.profiles.iter().any(|p| p == &profile.name) {
        if index.profiles.len() >= MAX_PROFILES {
            return Err(FlexikeyError::ProfileLimitReached);
        }
        index.profiles.push(profile.name.clone());
        save_profiles_index(&index)?;
    }
    ensure_config_dir()?;
    let contents = serde_json::to_string_pretty(profile)?;
    fs::write(profile_path(&profile.name), contents)?;
    Ok(())
}

pub fn set_active_profile(name: &str) -> Result<()> {
    let mut index = load_profiles_index()?;
    if !index.profiles.iter().any(|p| p == name) {
        return Err(FlexikeyError::UnknownProfile(name.to_string()));
    }
    index.active_profile = Some(name.to_string());
    save_profiles_index(&index)
}

pub fn delete_profile(name: &str) -> Result<()> {
    let mut index = load_profiles_index()?;
    if let Some(pos) = index.profiles.iter().position(|p| p == name) {
        index.profiles.remove(pos);
        if index.active_profile.as_deref() == Some(name) {
            index.active_profile = None;
        }
        save_profiles_index(&index)?;
    }
    let path = profile_path(name);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Keyboard discovery
// ---------------------------------------------------------------------------

/// Finds the first evdev input device that looks like the built-in
/// keyboard: it must expose letter keys (to exclude pure mouse/power
/// button devices) and must not be our own virtual output device.
/// Mirrors the Python implementation's `find_candidate_keyboards()[0]`.
fn find_first_candidate_keyboard() -> Option<Device> {
    for (_, device) in evdev::enumerate() {
        let has_letters = device
            .supported_keys()
            .map(|keys| keys.contains(KeyCode::KEY_A) && keys.contains(KeyCode::KEY_Z))
            .unwrap_or(false);
        if !has_letters {
            continue;
        }
        let is_virtual = device
            .name()
            .map(|n| {
                let n = n.to_lowercase();
                n.contains("flexikey") || n.contains("uinput")
            })
            .unwrap_or(false);
        if is_virtual {
            continue;
        }
        return Some(device);
    }
    None
}

/// Blocks (the caller is expected to run this inside `spawn_blocking`)
/// until one key is pressed on the candidate keyboard, then returns its
/// evdev key name (e.g. `"KEY_F13"`). Does not grab the device, so the
/// keypress also reaches the desktop normally - matching the Python GUI's
/// `capture_key`.
pub fn capture_next_key() -> Result<String> {
    let mut device = find_first_candidate_keyboard().ok_or(FlexikeyError::NoKeyboardFound)?;
    loop {
        for event in device.fetch_events()? {
            if let EventSummary::Key(_, key_code, value) = event.destructure() {
                if value == 1 {
                    return Ok(format!("{key_code:?}"));
                }
            }
        }
    }
}

fn parse_key(name: &str) -> Result<KeyCode> {
    name.parse::<KeyCode>()
        .map_err(|_| FlexikeyError::UnknownKey(name.to_string()))
}

// ---------------------------------------------------------------------------
// Macro playback (uinput virtual device)
// ---------------------------------------------------------------------------

/// Emits remapped keys / macros through a virtual uinput device.
pub struct MacroPlayer {
    device: VirtualDevice,
}

impl MacroPlayer {
    /// Registers the full standard keyboard keymap on the virtual device so
    /// any remap target or macro key can be synthesized, matching the
    /// Python implementation's capability set.
    pub fn new() -> Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for code in 1..=MAX_REGISTERED_KEYCODE {
            keys.insert(KeyCode::new(code));
        }
        let device = VirtualDevice::builder()?
            .name("OriginPC Flexikey Virtual Keyboard")
            .with_keys(&keys)?
            .build()?;
        Ok(Self { device })
    }

    fn emit_key(&mut self, code: u16, value: i32) -> Result<()> {
        self.device
            .emit(&[InputEvent::new(EventType::KEY.0, code, value)])?;
        Ok(())
    }

    /// Forwards a raw key event exactly as received - used to keep
    /// non-remapped keys working normally (including releases and
    /// autorepeat) while the physical keyboard is grabbed.
    pub fn passthrough(&mut self, code: u16, value: i32) -> Result<()> {
        self.emit_key(code, value)
    }

    pub fn tap(&mut self, key: KeyCode) -> Result<()> {
        self.emit_key(key.code(), 1)?;
        thread::sleep(SYNTH_PRESS_DURATION);
        self.emit_key(key.code(), 0)
    }

    pub fn combo(&mut self, keys: &[KeyCode]) -> Result<()> {
        for &key in keys {
            self.emit_key(key.code(), 1)?;
        }
        thread::sleep(SYNTH_PRESS_DURATION);
        for &key in keys.iter().rev() {
            self.emit_key(key.code(), 0)?;
        }
        Ok(())
    }

    /// Minimal, dependency-free text typer: maps printable ASCII to
    /// `KEY_*` codes (with a shift-combo for uppercase letters and shifted
    /// symbols), mirroring the Python implementation's `text()`.
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        for ch in text.chars() {
            self.type_char(ch)?;
        }
        Ok(())
    }

    fn type_char(&mut self, ch: char) -> Result<()> {
        if ch == '\n' {
            return self.tap(KeyCode::KEY_ENTER);
        }
        if ch == ' ' {
            return self.tap(KeyCode::KEY_SPACE);
        }
        if ch.is_ascii_alphabetic() {
            let key = parse_key(&format!("KEY_{}", ch.to_ascii_uppercase()))?;
            return if ch.is_uppercase() {
                self.combo(&[KeyCode::KEY_LEFTSHIFT, key])
            } else {
                self.tap(key)
            };
        }
        if ch.is_ascii_digit() {
            let key = parse_key(&format!("KEY_{ch}"))?;
            return self.tap(key);
        }
        if let Some(&(_, name)) = SHIFTED_SYMBOLS.iter().find(|(c, _)| *c == ch) {
            let key = parse_key(name)?;
            return self.combo(&[KeyCode::KEY_LEFTSHIFT, key]);
        }
        if let Some(&(_, name)) = SIMPLE_SYMBOLS.iter().find(|(c, _)| *c == ch) {
            let key = parse_key(name)?;
            return self.tap(key);
        }
        eprintln!("flexikey: no key mapping for character {ch:?}, skipping");
        Ok(())
    }
}

const SHIFTED_SYMBOLS: &[(char, &str)] = &[
    ('!', "KEY_1"),
    ('@', "KEY_2"),
    ('#', "KEY_3"),
    ('$', "KEY_4"),
    ('%', "KEY_5"),
    ('^', "KEY_6"),
    ('&', "KEY_7"),
    ('*', "KEY_8"),
    ('(', "KEY_9"),
    (')', "KEY_0"),
    ('_', "KEY_MINUS"),
    ('+', "KEY_EQUAL"),
    ('{', "KEY_LEFTBRACE"),
    ('}', "KEY_RIGHTBRACE"),
    ('|', "KEY_BACKSLASH"),
    (':', "KEY_SEMICOLON"),
    ('"', "KEY_APOSTROPHE"),
    ('<', "KEY_COMMA"),
    ('>', "KEY_DOT"),
    ('?', "KEY_SLASH"),
    ('~', "KEY_GRAVE"),
];

const SIMPLE_SYMBOLS: &[(char, &str)] = &[
    ('-', "KEY_MINUS"),
    ('=', "KEY_EQUAL"),
    ('[', "KEY_LEFTBRACE"),
    (']', "KEY_RIGHTBRACE"),
    ('\\', "KEY_BACKSLASH"),
    (';', "KEY_SEMICOLON"),
    ('\'', "KEY_APOSTROPHE"),
    (',', "KEY_COMMA"),
    ('.', "KEY_DOT"),
    ('/', "KEY_SLASH"),
    ('`', "KEY_GRAVE"),
];

// ---------------------------------------------------------------------------
// Engine: grabs the physical keyboard and applies the active profile
// ---------------------------------------------------------------------------

struct RunningEngine {
    stop_flag: Arc<AtomicBool>,
    join_handle: JoinHandle<()>,
}

/// Grabs the physical keyboard and applies the active profile's mappings,
/// re-emitting everything else transparently through a virtual output
/// device. Safe to share across threads/commands: `start`/`stop` are
/// idempotency-checked and guarded by a mutex.
pub struct FlexikeyEngine {
    running: Mutex<Option<RunningEngine>>,
}

impl Default for FlexikeyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FlexikeyEngine {
    pub fn new() -> Self {
        Self {
            running: Mutex::new(None),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.lock().unwrap().is_some()
    }

    /// Loads the active profile's mappings, grabs the physical keyboard,
    /// and starts a background thread applying remaps/macros until
    /// `stop()` is called. Returns an error immediately (without spawning
    /// anything) if there is no active profile, no mappings, no candidate
    /// keyboard device, or the virtual output device can't be created - so
    /// failures are surfaced to the caller instead of silently doing
    /// nothing in the background.
    pub fn start(&self) -> Result<()> {
        let mut guard = self.running.lock().unwrap();
        if guard.is_some() {
            return Err(FlexikeyError::AlreadyRunning);
        }

        let mappings = load_active_mappings()?;
        let mut device = find_first_candidate_keyboard().ok_or(FlexikeyError::NoKeyboardFound)?;
        device.grab()?;
        let player = MacroPlayer::new()?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_stop_flag = Arc::clone(&stop_flag);
        let join_handle = thread::Builder::new()
            .name("flexikey-engine".to_string())
            .spawn(move || run_engine_loop(device, player, mappings, thread_stop_flag))
            .map_err(FlexikeyError::Io)?;

        *guard = Some(RunningEngine {
            stop_flag,
            join_handle,
        });
        Ok(())
    }

    /// Signals the engine thread to stop, waits for it to actually release
    /// the keyboard, and returns. Bounded by `POLL_INTERVAL`, not by
    /// waiting for another keypress the way the Python daemon's blocking
    /// `read_loop()` was.
    pub fn stop(&self) -> Result<()> {
        let running = self.running.lock().unwrap().take();
        match running {
            Some(engine) => {
                engine.stop_flag.store(true, Ordering::SeqCst);
                engine
                    .join_handle
                    .join()
                    .map_err(|_| FlexikeyError::EngineThreadPanicked)
            }
            None => Err(FlexikeyError::NotRunning),
        }
    }
}

fn load_active_mappings() -> Result<HashMap<String, Action>> {
    let index = load_profiles_index()?;
    let active = index.active_profile.ok_or(FlexikeyError::NoActiveProfile)?;
    let profile = load_profile(&active)?;
    if profile.mappings.is_empty() {
        return Err(FlexikeyError::NoMappingsConfigured);
    }
    Ok(profile.mappings)
}

fn run_engine_loop(
    mut device: Device,
    mut player: MacroPlayer,
    mappings: HashMap<String, Action>,
    stop_flag: Arc<AtomicBool>,
) {
    if let Err(e) = device.set_nonblocking(true) {
        eprintln!("flexikey: failed to set keyboard device non-blocking, engine exiting: {e}");
        let _ = device.ungrab();
        return;
    }

    while !stop_flag.load(Ordering::SeqCst) {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    if let EventSummary::Key(_, key_code, value) = event.destructure() {
                        let key_name = format!("{key_code:?}");
                        if let Some(action) = mappings.get(&key_name) {
                            // Mapped keys are edge-triggered: the action
                            // fires once on press, and the physical
                            // release is swallowed (matches the Python
                            // implementation's macro-key semantics).
                            if value == 1 {
                                dispatch_action(&mut player, action);
                            }
                        } else if let Err(e) = player.passthrough(key_code.code(), value) {
                            eprintln!("flexikey: failed to passthrough key event: {e}");
                        }
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(POLL_INTERVAL);
            }
            Err(e) => {
                eprintln!("flexikey: keyboard device read error, engine stopping: {e}");
                break;
            }
        }
    }

    if let Err(e) = device.ungrab() {
        eprintln!("flexikey: failed to ungrab keyboard device on stop: {e}");
    }
}

fn dispatch_action(player: &mut MacroPlayer, action: &Action) {
    let result = match action {
        Action::Remap { target } => parse_key(target).and_then(|key| player.tap(key)),
        Action::Combo { keys } => keys
            .iter()
            .map(|k| parse_key(k))
            .collect::<Result<Vec<_>>>()
            .and_then(|codes| player.combo(&codes)),
        Action::Text { text } => player.type_text(text),
        Action::Launch { command } => launch_command(command),
        Action::Disabled => Ok(()),
    };
    if let Err(e) = result {
        eprintln!("flexikey: error executing action {action:?}: {e}");
    }
}

/// Launches a program in a new process group (detached from the engine's
/// own process group, analogous to the Python implementation's
/// `start_new_session=True`) so it survives the engine stopping.
fn launch_command(command: &str) -> Result<()> {
    let parts =
        shlex::split(command).ok_or_else(|| FlexikeyError::InvalidCommand(command.to_string()))?;
    let (program, args) = parts
        .split_first()
        .ok_or_else(|| FlexikeyError::InvalidCommand(command.to_string()))?;
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map(|_child| ())
        .map_err(FlexikeyError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_path_sanitizes_unsafe_characters() {
        // '.' and '/' are neither alphanumeric nor '-'/'_', so both are
        // replaced - this also means sanitized names can never escape
        // `config_dir()` via `..` or `/`.
        let path = profile_path("../etc/passwd");
        assert_eq!(path.file_name().unwrap(), "___etc_passwd.json");

        let path = profile_path("My Profile 1");
        assert_eq!(path.file_name().unwrap(), "My_Profile_1.json");
    }

    #[test]
    fn action_round_trips_through_json_matching_python_shape() {
        let remap: Action = serde_json::from_str(r#"{"type":"remap","target":"KEY_A"}"#).unwrap();
        assert!(matches!(remap, Action::Remap { target } if target == "KEY_A"));

        let combo: Action = serde_json::from_str(
            r#"{"type":"combo","keys":["KEY_LEFTCTRL","KEY_LEFTSHIFT","KEY_ESC"]}"#,
        )
        .unwrap();
        assert!(matches!(combo, Action::Combo { keys } if keys.len() == 3));

        let disabled: Action = serde_json::from_str(r#"{"type":"disabled"}"#).unwrap();
        assert!(matches!(disabled, Action::Disabled));
        assert_eq!(
            serde_json::to_string(&disabled).unwrap(),
            r#"{"type":"disabled"}"#
        );
    }

    #[test]
    fn parse_key_rejects_unknown_names() {
        assert!(matches!(
            parse_key("NOT_A_REAL_KEY"),
            Err(FlexikeyError::UnknownKey(_))
        ));
        assert!(parse_key("KEY_A").is_ok());
    }

    #[test]
    fn profiles_index_defaults_are_empty() {
        let index = ProfilesIndex::default();
        assert!(index.active_profile.is_none());
        assert!(index.profiles.is_empty());
    }
}
