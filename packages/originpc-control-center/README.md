# OriginPC Control Center (Linux)

A Linux replacement for the Windows "Control Center" suite that ships with
Clevo-based laptops (tested on an OriginPC EON17-X, USB ID `048d:8910`,
ITE 8910 keyboard controller). Ports and fixes a previously working PyQt5
implementation (`originpc-enhanced-control-5.1.0`) and adds two features the
original build was missing: Flexikey-style key remapping/macros, and
Fn-hotkey capture with an on-screen display.

## Features

- **RGB keyboard control** - per-key, group, and whole-keyboard color;
  effects; brightness. Protocol: raw `0xCC 0x01 <key_index> <R> <G> <B>`
  writes (+ padding to 16 bytes) to `/dev/hidraw0`. Also see the simpler
  CLI equivalent at `../../scripts/gaming/clevo-kbd-rgb`.
- **System monitoring** - CPU/GPU/NVMe temperatures, fan speeds, battery,
  load, via `psutil` + `sensors`.
- **Lid monitor** - clears RGB state on lid close (fixes the known
  `kp_plus` cyan-residue bug), runs as a `systemd --user` service.
- **Flexikey** (new) - key remapping and macros (text, key combos, launch
  commands), profile-based (up to 12 profiles), implemented with
  `evdev`/`uinput` - no vendor protocol involved, matching how Clevo's own
  Flexikey works (it's a pure software feature, per Clevo's manuals).
- **Fn hotkeys + OSD** (new) - captures Fn-key events (keyboard backlight
  up/down/cycle/toggle, touchpad toggle, rfkill) via a small new kernel
  module and shows an on-screen popup, matching the Windows OSD behavior.
- **GPU Overclocking** - **not implemented**. See below.

## Why a new Flexikey and a new kernel module were needed

The real Microsoft Store "Control Center 3.0" (CLEVO CO.) is a launcher hub
for several separate apps: LEDs Keyboard Setting, Fan Speed Setting,
Flexikey, Fn hotkeys and OSD, and GPU Overclocking. The archived
`enhanced-control-5.1.0` build only replicated the first two. Flexikey and
Fn-hotkeys/OSD are added by this package; GPU Overclocking is documented as
out of scope (see below).

### Flexikey
Clevo's own manuals describe Flexikey as a hotkey configuration app that
lets a key launch key combinations, programs, or text macros, or be
disabled - a software-only feature. `src/flexikey.py` replicates this by
grabbing the physical keyboard's evdev device and re-emitting
remapped/macro output through a virtual `uinput` device.

### Fn hotkeys + OSD
Diagnosed directly on this machine:
- DMI: `sys_vendor=OriginPC`, `product_name=EON17-X`; ACPI exposes
  `CLV0001:00` / `CLV0002:00`.
- A System76 DKMS module (`system76_acpi`, builds `clevo_acpi.ko`) is loaded
  but **not bound** to either device - its DMI allowlist only covers
  System76 model names. It is untouched by this package.
- `/sys/bus/wmi/devices/` exposes the classic Clevo hotkey WMI GUID trio
  (`ABBC0F6B`/`ABBC0F6C`/`ABBC0F6D-8EA1-11D1-00A0-C90629100000`) used by the
  long-established community `clevo-wmi`/`clevo-xsm-wmi`/`tuxedo-keyboard`
  drivers, and **no driver claims them** on this machine (the only bound WMI
  driver, `nvidia-wmi-ec-backlight`, owns a different GUID).
- `kernel/clevo-hotkeys/clevo-hotkeys.c` is a small new GPL DKMS module that
  claims that GUID and reports events through a standard `sparse_keymap`
  input device. It only ever issues the documented, read-only "get event"
  WMBB query (function `0x01`) in response to a firmware notification - it
  never writes to EC/ACPI state.
- `src/hotkey_osd.py` listens on that input device (plus the
  already-standard brightness/volume keys) and shows a themed popup.

## GPU Overclocking - not implemented (research findings)

Clevo's own manuals list "CPU/Memory Overclocking Support" and "GPU
Overclocking" as separate sections that only appear on certain
desktop-replacement chassis - not a universal Control Center feature, and
not documented as present on the EON17-X. No community Linux driver
(`clevo-xsm-wmi`, `tuxedo-drivers`) implements EC *write* access for this;
the existing ecosystem is explicitly read-only for fan/EC telemetry (one
open GitHub issue on `clevo-xsm-wmi` notes "only offers read support, no
write support" for the relevant EC registers). Implementing real overclock
control would mean guessing at undocumented EC register writes on
unfamiliar firmware, which risks putting the EC into a bad state - this is
deliberately not attempted here. If pursued later as its own task, it needs
its own research pass: a DSDT dump plus tooling like `RLECViewer` to
identify the actual EC offsets Clevo's Windows tool writes, and testing on
a chassis confirmed to expose the feature.

## Install

```bash
./install.sh
```

Idempotent; re-run after pulling updates. Requires `sudo` only for the udev
rule and the `clevo-hotkeys` DKMS module.

Dependencies (already present on a standard Pop!_OS desktop): `python3`,
`python3-pyqt5`, `python3-psutil`, `python3-evdev`, `dkms`,
`linux-headers-$(uname -r)` (for building the kernel module).

## Usage

```bash
originpc-control-center                              # main GUI
originpc-flexikey                                     # configure remaps/macros
systemctl --user enable --now originpc-lid-monitor
systemctl --user enable --now originpc-hotkey-osd
systemctl --user enable --now originpc-flexikey       # after configuring a profile
```

CLI equivalents for scripting (unchanged, still useful):
```bash
sudo ../../scripts/gaming/clevo-kbd-rgb color ff0000
sudo ../../scripts/gaming/clevo-kbd-rgb preset gaming
```

## Troubleshooting

- **RGB does nothing**: check `ls -la /dev/hidraw0` - should be
  `crw-rw-rw-`. If not, re-run `install.sh` or manually:
  `sudo udevadm control --reload-rules && sudo udevadm trigger`.
- **Fn hotkeys do nothing**: check the module bound:
  `ls -la /sys/bus/wmi/devices/ABBC0F6B-8EA1-11D1-00A0-C90629100000/driver`
  should point at `clevo_hotkeys`. Check `dmesg | grep -i clevo` for probe
  errors. Check `dkms status` for `clevo-hotkeys`.
- **Flexikey remap not applying**: confirm a profile is active
  (`originpc-flexikey` GUI shows it in the window title) and the service is
  running: `systemctl --user status originpc-flexikey`. If a bad mapping
  locks out a key you need, stop the service:
  `systemctl --user stop originpc-flexikey`.
- **`kp_plus` cyan residue persists**: run `src/gentle-rgb-clear.py` or
  `src/originpc-rgb-fix.py` directly, or ensure `originpc-lid-monitor` is
  enabled (it clears this automatically on every lid close).

## Known issues carried over from the archived build

- Hardware-level RGB persistence across some lid/suspend events may still
  require the aggressive clearing routines in `lid-monitor-daemon.py`
  (unchanged from the archived implementation, since this behavior is
  firmware-side, not a bug in this package).
