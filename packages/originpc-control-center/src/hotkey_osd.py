#!/usr/bin/env python3
"""
OriginPC Hotkey OSD - on-screen display for Fn-key actions
=============================================================
Linux equivalent of Clevo's "Fn hotkeys and OSD" companion app. Listens on
every evdev input device for the key events the clevo-hotkeys kernel module
reports (keyboard-backlight up/down/cycle/toggle, touchpad toggle, rfkill)
as well as the standard already-handled brightness/volume/mute keys, and
shows a small themed popup describing the action - the same UX pattern as
the Windows OSD overlay.

This is a pure userspace consumer of standard Linux input events; it does
not talk to any hardware protocol directly.
"""

import sys
import os
import glob
import logging

try:
    import evdev
    from evdev import ecodes, InputDevice, categorize
    EVDEV_AVAILABLE = True
except ImportError as e:
    EVDEV_AVAILABLE = False
    _EVDEV_IMPORT_ERROR = e

from PyQt5.QtWidgets import QApplication, QLabel
from PyQt5.QtCore import Qt, QTimer, QThread, pyqtSignal
from PyQt5.QtGui import QFont

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - hotkey-osd - %(levelname)s - %(message)s',
)
log = logging.getLogger('hotkey-osd')

# Keycode -> (label, icon glyph) shown in the OSD.
OSD_MESSAGES = {
    'KEY_KBDILLUMUP': ("Keyboard Backlight +", "\u2b06"),
    'KEY_KBDILLUMDOWN': ("Keyboard Backlight -", "\u2b07"),
    'KEY_KBDILLUMTOGGLE': ("Keyboard Backlight Toggled", "\u2728"),
    'KEY_LIGHTS_TOGGLE': ("Keyboard Effect Cycled", "\U0001f308"),
    'KEY_F21': ("Touchpad Toggled", "\U0001f5b1"),
    'KEY_RFKILL': ("Wireless Toggled", "\U0001f4f6"),
    'KEY_PROG1': ("Battery Gauge", "\U0001f50b"),
    'KEY_BRIGHTNESSUP': ("Screen Brightness +", "\u2600"),
    'KEY_BRIGHTNESSDOWN': ("Screen Brightness -", "\U0001f505"),
    'KEY_VOLUMEUP': ("Volume +", "\U0001f50a"),
    'KEY_VOLUMEDOWN': ("Volume -", "\U0001f509"),
    'KEY_MUTE': ("Muted", "\U0001f507"),
}


class OSDPopup(QLabel):
    """Frameless, click-through, auto-hiding overlay label."""

    def __init__(self):
        super().__init__()
        self.setWindowFlags(
            Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool | Qt.X11BypassWindowManagerHint
        )
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setAttribute(Qt.WA_ShowWithoutActivating)
        self.setFont(QFont("Sans Serif", 16))
        self.setStyleSheet(
            "QLabel {"
            "  background-color: rgba(30, 30, 30, 220);"
            "  color: white;"
            "  border-radius: 12px;"
            "  padding: 18px 28px;"
            "}"
        )
        self.setAlignment(Qt.AlignCenter)
        self.hide_timer = QTimer(self)
        self.hide_timer.setSingleShot(True)
        self.hide_timer.timeout.connect(self.hide)

    def show_message(self, text):
        self.setText(text)
        self.adjustSize()
        screen = QApplication.primaryScreen().geometry()
        x = screen.center().x() - self.width() // 2
        y = int(screen.height() * 0.15)
        self.move(x, y)
        self.show()
        self.hide_timer.start(1500)


class HotkeyListener(QThread):
    """Background thread reading all keyboard-capable evdev devices."""
    hotkey_event = pyqtSignal(str)

    def __init__(self):
        super().__init__()
        self._running = True

    def run(self):
        if not EVDEV_AVAILABLE:
            log.error(f"python3-evdev is required for the hotkey OSD: {_EVDEV_IMPORT_ERROR}")
            return

        devices = []
        for path in evdev.list_devices():
            try:
                dev = InputDevice(path)
            except (OSError, PermissionError):
                continue
            caps = dev.capabilities().get(ecodes.EV_KEY, [])
            interesting = any(getattr(ecodes, name, None) in caps for name in
                               ['KEY_KBDILLUMUP', 'KEY_KBDILLUMDOWN', 'KEY_KBDILLUMTOGGLE',
                                'KEY_LIGHTS_TOGGLE', 'KEY_F21', 'KEY_RFKILL', 'KEY_PROG1',
                                'KEY_BRIGHTNESSUP', 'KEY_BRIGHTNESSDOWN',
                                'KEY_VOLUMEUP', 'KEY_VOLUMEDOWN', 'KEY_MUTE'])
            if interesting:
                devices.append(dev)
                log.info(f"Listening on {dev.path} ({dev.name})")
            else:
                dev.close()

        if not devices:
            log.warning("No input device exposing known hotkey codes was found; "
                        "is the clevo-hotkeys module loaded?")
            return

        from selectors import DefaultSelector, EVENT_READ
        selector = DefaultSelector()
        for dev in devices:
            selector.register(dev, EVENT_READ)

        while self._running:
            for key, _ in selector.select(timeout=1.0):
                dev = key.fileobj
                try:
                    for event in dev.read():
                        if event.type != ecodes.EV_KEY:
                            continue
                        key_event = categorize(event)
                        if key_event.keystate != key_event.key_down:
                            continue
                        name = key_event.keycode if isinstance(key_event.keycode, str) else key_event.keycode[0]
                        if name in OSD_MESSAGES:
                            self.hotkey_event.emit(name)
                except (OSError, BlockingIOError):
                    continue

    def stop(self):
        self._running = False


def main():
    app = QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(False)

    osd = OSDPopup()
    listener = HotkeyListener()

    def on_hotkey(name):
        label, icon = OSD_MESSAGES.get(name, (name, ""))
        osd.show_message(f"{icon}  {label}")

    listener.hotkey_event.connect(on_hotkey)
    listener.start()

    exit_code = app.exec_()
    listener.stop()
    listener.wait(2000)
    sys.exit(exit_code)


if __name__ == '__main__':
    main()
