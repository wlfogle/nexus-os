#!/usr/bin/env python3
"""
OriginPC Flexikey - key remap / macro engine
=============================================
Linux equivalent of Clevo's Flexikey(R) application. Per Clevo's own manuals,
Flexikey is purely a software feature: it lets a single key launch multiple
key combinations, launch programs, type text macros, or be disabled - all
implemented by intercepting key presses, not by any special hardware
protocol. This module replicates that entirely at the Linux input layer
using evdev (to read/grab the physical keyboard) and uinput (to emit the
remapped/macro output), so it works transparently under any desktop
environment and window manager, on both X11 and Wayland.

Usage:
  flexikey.py --daemon         Run the remap/macro engine (systemd --user unit)
  flexikey.py --gui            Open the profile/macro configuration UI
  flexikey.py --list-devices   List candidate keyboard input devices
  flexikey.py --capture        Interactively print key names as they are pressed

Profiles are stored as JSON under ~/.config/originpc-control-center/flexikey/
(one file per profile; "active_profile" recorded in profiles.json). Up to 12
profiles are supported, matching the real Flexikey's profile count.
"""

import sys
import os
import json
import time
import glob
import shlex
import signal
import logging
import subprocess
import threading
from pathlib import Path

try:
    import evdev
    from evdev import ecodes, UInput, InputDevice, categorize
    EVDEV_AVAILABLE = True
except ImportError as e:
    EVDEV_AVAILABLE = False
    _EVDEV_IMPORT_ERROR = e

CONFIG_DIR = Path(os.environ.get('XDG_CONFIG_HOME', Path.home() / '.config')) / 'originpc-control-center' / 'flexikey'
STATE_DIR = Path(os.environ.get('XDG_STATE_HOME', Path.home() / '.local' / 'state')) / 'originpc-control-center'
PROFILES_INDEX = CONFIG_DIR / 'profiles.json'
MAX_PROFILES = 12

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - flexikey - %(levelname)s - %(message)s',
)
log = logging.getLogger('flexikey')


# ---------------------------------------------------------------------------
# Profile storage
# ---------------------------------------------------------------------------

def ensure_dirs():
    CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    STATE_DIR.mkdir(parents=True, exist_ok=True)


def load_profiles_index():
    ensure_dirs()
    if PROFILES_INDEX.exists():
        try:
            with open(PROFILES_INDEX, 'r') as f:
                return json.load(f)
        except (json.JSONDecodeError, OSError) as e:
            log.warning(f"Could not read profiles index, recreating: {e}")
    return {'active_profile': None, 'profiles': []}


def save_profiles_index(index):
    ensure_dirs()
    with open(PROFILES_INDEX, 'w') as f:
        json.dump(index, f, indent=2)


def profile_path(name):
    safe = ''.join(c if c.isalnum() or c in '-_' else '_' for c in name)
    return CONFIG_DIR / f"{safe}.json"


def load_profile(name):
    path = profile_path(name)
    if not path.exists():
        return {'name': name, 'mappings': {}}
    with open(path, 'r') as f:
        return json.load(f)


def save_profile(profile):
    index = load_profiles_index()
    if profile['name'] not in index['profiles']:
        if len(index['profiles']) >= MAX_PROFILES:
            raise ValueError(f"Maximum of {MAX_PROFILES} Flexikey profiles reached")
        index['profiles'].append(profile['name'])
        save_profiles_index(index)
    with open(profile_path(profile['name']), 'w') as f:
        json.dump(profile, f, indent=2)


def set_active_profile(name):
    index = load_profiles_index()
    if name not in index['profiles']:
        raise ValueError(f"Unknown profile: {name}")
    index['active_profile'] = name
    save_profiles_index(index)


def delete_profile(name):
    index = load_profiles_index()
    if name in index['profiles']:
        index['profiles'].remove(name)
        if index.get('active_profile') == name:
            index['active_profile'] = None
        save_profiles_index(index)
    path = profile_path(name)
    if path.exists():
        path.unlink()


# ---------------------------------------------------------------------------
# Mapping actions
# ---------------------------------------------------------------------------
# A mapping entry looks like:
#   {"type": "remap", "target": "KEY_A"}
#   {"type": "combo", "keys": ["KEY_LEFTCTRL", "KEY_LEFTSHIFT", "KEY_ESC"]}
#   {"type": "text", "text": "hello world"}
#   {"type": "launch", "command": "gnome-terminal"}
#   {"type": "disabled"}

def find_candidate_keyboards():
    """Return evdev input devices that look like the built-in keyboard."""
    if not EVDEV_AVAILABLE:
        return []
    candidates = []
    for path in evdev.list_devices():
        try:
            dev = InputDevice(path)
        except (OSError, PermissionError):
            continue
        caps = dev.capabilities().get(ecodes.EV_KEY, [])
        # A real keyboard exposes letter keys; exclude pure mouse/power buttons.
        has_letters = ecodes.KEY_A in caps and ecodes.KEY_Z in caps
        is_virtual = 'flexikey' in dev.name.lower() or 'uinput' in dev.name.lower()
        if has_letters and not is_virtual:
            candidates.append(dev)
        else:
            dev.close()
    return candidates


class MacroPlayer:
    """Emits remapped keys / macros through a virtual uinput device."""

    def __init__(self):
        # Register the full standard keyboard keymap plus a few extras so
        # any remap target or macro key can be synthesized.
        capabilities = {ecodes.EV_KEY: sorted(set(evdev.ecodes.keys.keys()) & set(range(1, 249)))}
        self.ui = UInput(capabilities, name='OriginPC Flexikey Virtual Keyboard')

    def close(self):
        self.ui.close()

    def _code(self, key_name):
        code = getattr(ecodes, key_name, None)
        if code is None:
            raise ValueError(f"Unknown key code: {key_name}")
        return code

    def tap(self, key_name):
        code = self._code(key_name)
        self.ui.write(ecodes.EV_KEY, code, 1)
        self.ui.syn()
        time.sleep(0.01)
        self.ui.write(ecodes.EV_KEY, code, 0)
        self.ui.syn()

    def combo(self, key_names):
        codes = [self._code(k) for k in key_names]
        for code in codes:
            self.ui.write(ecodes.EV_KEY, code, 1)
            self.ui.syn()
        time.sleep(0.01)
        for code in reversed(codes):
            self.ui.write(ecodes.EV_KEY, code, 0)
            self.ui.syn()

    def text(self, text):
        # Minimal, dependency-free text typer: map printable ASCII to
        # KEY_* codes (with shift for uppercase/symbols).
        shift_map = {
            '!': '1', '@': '2', '#': '3', '$': '4', '%': '5', '^': '6',
            '&': '7', '*': '8', '(': '9', ')': '0', '_': 'MINUS', '+': 'EQUAL',
            '{': 'LEFTBRACE', '}': 'RIGHTBRACE', '|': 'BACKSLASH', ':': 'SEMICOLON',
            '"': 'APOSTROPHE', '<': 'COMMA', '>': 'DOT', '?': 'SLASH', '~': 'GRAVE',
        }
        for ch in text:
            if ch == '\n':
                self.tap('KEY_ENTER')
                continue
            if ch == ' ':
                self.tap('KEY_SPACE')
                continue
            if ch.isalpha():
                key = f'KEY_{ch.upper()}'
                if ch.isupper():
                    self.combo(['KEY_LEFTSHIFT', key])
                else:
                    self.tap(key)
                continue
            if ch.isdigit():
                self.tap(f'KEY_{ch}')
                continue
            if ch in shift_map:
                base = shift_map[ch]
                key = base if base.startswith('KEY_') else f'KEY_{base}'
                self.combo(['KEY_LEFTSHIFT', key])
                continue
            simple = {'-': 'KEY_MINUS', '=': 'KEY_EQUAL', '[': 'KEY_LEFTBRACE',
                      ']': 'KEY_RIGHTBRACE', '\\': 'KEY_BACKSLASH', ';': 'KEY_SEMICOLON',
                      "'": 'KEY_APOSTROPHE', ',': 'KEY_COMMA', '.': 'KEY_DOT',
                      '/': 'KEY_SLASH', '`': 'KEY_GRAVE'}
            if ch in simple:
                self.tap(simple[ch])
                continue
            log.warning(f"Flexikey: no key mapping for character {ch!r}, skipping")


class FlexikeyEngine:
    """Grabs the physical keyboard and applies the active profile's mappings."""

    def __init__(self):
        self.running = False
        self.device = None
        self.player = None
        self.mappings = {}

    def _load_active_mappings(self):
        index = load_profiles_index()
        active = index.get('active_profile')
        if not active:
            log.info("Flexikey: no active profile, running as pure passthrough")
            self.mappings = {}
            return
        profile = load_profile(active)
        self.mappings = profile.get('mappings', {})
        log.info(f"Flexikey: loaded profile '{active}' with {len(self.mappings)} mapping(s)")

    def _handle_action(self, action):
        try:
            kind = action.get('type')
            if kind == 'remap':
                self.player.tap(action['target'])
            elif kind == 'combo':
                self.player.combo(action['keys'])
            elif kind == 'text':
                self.player.text(action['text'])
            elif kind == 'launch':
                subprocess.Popen(shlex.split(action['command']),
                                  stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                  start_new_session=True)
            elif kind == 'disabled':
                pass  # swallow the key entirely
            else:
                log.warning(f"Flexikey: unknown action type {kind!r}")
        except Exception as e:
            log.error(f"Flexikey: error executing action {action}: {e}")

    def run(self):
        if not EVDEV_AVAILABLE:
            log.error(f"python3-evdev is required for Flexikey: {_EVDEV_IMPORT_ERROR}")
            sys.exit(1)

        self._load_active_mappings()
        if not self.mappings:
            log.info("Flexikey: no mappings configured, exiting (nothing to do)")
            return

        candidates = find_candidate_keyboards()
        if not candidates:
            log.error("Flexikey: no candidate keyboard input device found")
            sys.exit(1)
        self.device = candidates[0]
        log.info(f"Flexikey: grabbing {self.device.path} ({self.device.name})")

        self.player = MacroPlayer()
        self.device.grab()
        self.running = True

        signal.signal(signal.SIGTERM, lambda *_: self.stop())
        signal.signal(signal.SIGINT, lambda *_: self.stop())

        try:
            for event in self.device.read_loop():
                if not self.running:
                    break
                if event.type != ecodes.EV_KEY:
                    continue
                key_event = categorize(event)
                if key_event.keystate != key_event.key_down:
                    continue
                key_name = key_event.keycode if isinstance(key_event.keycode, str) else key_event.keycode[0]
                action = self.mappings.get(key_name)
                if action:
                    self._handle_action(action)
                else:
                    # Not remapped: re-emit the original key unchanged so the
                    # keyboard keeps working normally while grabbed.
                    self.player.ui.write(ecodes.EV_KEY, event.code, event.value)
                    self.player.ui.syn()
        finally:
            self.stop()

    def stop(self):
        self.running = False
        try:
            if self.device:
                self.device.ungrab()
                self.device.close()
        except Exception:
            pass
        try:
            if self.player:
                self.player.close()
        except Exception:
            pass
        log.info("Flexikey: stopped, keyboard released")


# ---------------------------------------------------------------------------
# CLI helpers
# ---------------------------------------------------------------------------

def cmd_list_devices():
    if not EVDEV_AVAILABLE:
        print(f"python3-evdev not available: {_EVDEV_IMPORT_ERROR}")
        return
    for dev in find_candidate_keyboards():
        print(f"{dev.path}\t{dev.name}")
        dev.close()


def cmd_capture():
    if not EVDEV_AVAILABLE:
        print(f"python3-evdev not available: {_EVDEV_IMPORT_ERROR}")
        return
    candidates = find_candidate_keyboards()
    if not candidates:
        print("No candidate keyboard device found")
        return
    dev = candidates[0]
    print(f"Reading from {dev.path} ({dev.name}) - press Ctrl+C to stop")
    try:
        for event in dev.read_loop():
            if event.type == ecodes.EV_KEY:
                key_event = categorize(event)
                if key_event.keystate == key_event.key_down:
                    name = key_event.keycode if isinstance(key_event.keycode, str) else key_event.keycode[0]
                    print(name)
    except KeyboardInterrupt:
        pass
    finally:
        dev.close()


def run_gui():
    from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout,
                                  QHBoxLayout, QListWidget, QPushButton, QComboBox,
                                  QLineEdit, QLabel, QMessageBox, QInputDialog, QFormLayout)

    class FlexikeyWindow(QMainWindow):
        def __init__(self):
            super().__init__()
            self.setWindowTitle("OriginPC Flexikey")
            self.resize(560, 420)
            central = QWidget()
            self.setCentralWidget(central)
            root = QHBoxLayout(central)

            # Profiles column
            profile_col = QVBoxLayout()
            profile_col.addWidget(QLabel("Profiles (max 12)"))
            self.profile_list = QListWidget()
            self.profile_list.currentTextChanged.connect(self.on_profile_selected)
            profile_col.addWidget(self.profile_list)
            profile_btns = QHBoxLayout()
            add_btn = QPushButton("New")
            add_btn.clicked.connect(self.new_profile)
            del_btn = QPushButton("Delete")
            del_btn.clicked.connect(self.delete_current_profile)
            activate_btn = QPushButton("Set Active")
            activate_btn.clicked.connect(self.activate_current_profile)
            profile_btns.addWidget(add_btn)
            profile_btns.addWidget(del_btn)
            profile_btns.addWidget(activate_btn)
            profile_col.addLayout(profile_btns)
            root.addLayout(profile_col, 1)

            # Mappings column
            map_col = QVBoxLayout()
            map_col.addWidget(QLabel("Mappings for selected profile"))
            self.mapping_list = QListWidget()
            map_col.addWidget(self.mapping_list)

            form = QFormLayout()
            self.source_key = QLineEdit()
            self.source_key.setPlaceholderText("e.g. KEY_F13 (press capture to detect)")
            capture_btn = QPushButton("Capture next key")
            capture_btn.clicked.connect(self.capture_key)
            source_row = QHBoxLayout()
            source_row.addWidget(self.source_key)
            source_row.addWidget(capture_btn)
            form.addRow("Source key:", source_row)

            self.action_type = QComboBox()
            self.action_type.addItems(["text", "combo", "launch", "remap", "disabled"])
            form.addRow("Action:", self.action_type)

            self.action_value = QLineEdit()
            self.action_value.setPlaceholderText(
                "text: literal text | combo/remap: KEY_A,KEY_B | launch: shell command")
            form.addRow("Value:", self.action_value)

            map_col.addLayout(form)
            add_map_btn = QPushButton("Add / Update Mapping")
            add_map_btn.clicked.connect(self.add_mapping)
            remove_map_btn = QPushButton("Remove Selected Mapping")
            remove_map_btn.clicked.connect(self.remove_mapping)
            btn_row = QHBoxLayout()
            btn_row.addWidget(add_map_btn)
            btn_row.addWidget(remove_map_btn)
            map_col.addLayout(btn_row)
            root.addLayout(map_col, 2)

            self.current_profile = None
            self.reload_profiles()

        def reload_profiles(self):
            index = load_profiles_index()
            self.profile_list.clear()
            self.profile_list.addItems(index['profiles'])
            active = index.get('active_profile')
            self.setWindowTitle(f"OriginPC Flexikey - active: {active or 'none'}")

        def on_profile_selected(self, name):
            if not name:
                self.current_profile = None
                self.mapping_list.clear()
                return
            self.current_profile = load_profile(name)
            self.mapping_list.clear()
            for key, action in self.current_profile.get('mappings', {}).items():
                self.mapping_list.addItem(f"{key} -> {action}")

        def new_profile(self):
            name, ok = QInputDialog.getText(self, "New Profile", "Profile name:")
            if ok and name:
                try:
                    save_profile({'name': name, 'mappings': {}})
                    self.reload_profiles()
                except ValueError as e:
                    QMessageBox.warning(self, "Flexikey", str(e))

        def delete_current_profile(self):
            item = self.profile_list.currentItem()
            if item:
                delete_profile(item.text())
                self.reload_profiles()

        def activate_current_profile(self):
            item = self.profile_list.currentItem()
            if item:
                set_active_profile(item.text())
                self.reload_profiles()
                QMessageBox.information(self, "Flexikey",
                    f"'{item.text()}' is now active. Restart the Flexikey service:\n"
                    "systemctl --user restart originpc-flexikey")

        def capture_key(self):
            if not EVDEV_AVAILABLE:
                QMessageBox.warning(self, "Flexikey", "python3-evdev not available")
                return
            candidates = find_candidate_keyboards()
            if not candidates:
                QMessageBox.warning(self, "Flexikey", "No keyboard device found")
                return
            dev = candidates[0]
            QMessageBox.information(self, "Flexikey", "Press the key you want to remap now.")
            try:
                for event in dev.read_loop():
                    if event.type == ecodes.EV_KEY:
                        key_event = categorize(event)
                        if key_event.keystate == key_event.key_down:
                            name = key_event.keycode if isinstance(key_event.keycode, str) else key_event.keycode[0]
                            self.source_key.setText(name)
                            break
            finally:
                dev.close()

        def add_mapping(self):
            if not self.current_profile:
                QMessageBox.warning(self, "Flexikey", "Select or create a profile first")
                return
            key = self.source_key.text().strip()
            if not key:
                QMessageBox.warning(self, "Flexikey", "Set a source key first")
                return
            kind = self.action_type.currentText()
            value = self.action_value.text().strip()
            if kind == 'text':
                action = {'type': 'text', 'text': value}
            elif kind == 'combo':
                action = {'type': 'combo', 'keys': [k.strip() for k in value.split(',') if k.strip()]}
            elif kind == 'remap':
                action = {'type': 'remap', 'target': value}
            elif kind == 'launch':
                action = {'type': 'launch', 'command': value}
            else:
                action = {'type': 'disabled'}
            self.current_profile.setdefault('mappings', {})[key] = action
            save_profile(self.current_profile)
            self.on_profile_selected(self.current_profile['name'])

        def remove_mapping(self):
            item = self.mapping_list.currentItem()
            if not item or not self.current_profile:
                return
            key = item.text().split(' -> ')[0]
            self.current_profile.get('mappings', {}).pop(key, None)
            save_profile(self.current_profile)
            self.on_profile_selected(self.current_profile['name'])

    app = QApplication(sys.argv)
    window = FlexikeyWindow()
    window.show()
    sys.exit(app.exec_())


def main():
    ensure_dirs()
    if '--daemon' in sys.argv:
        FlexikeyEngine().run()
    elif '--gui' in sys.argv:
        run_gui()
    elif '--list-devices' in sys.argv:
        cmd_list_devices()
    elif '--capture' in sys.argv:
        cmd_capture()
    else:
        print(__doc__)


if __name__ == '__main__':
    main()
