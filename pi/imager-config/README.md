# Ziggy — Raspberry Pi Imager Config

Two ways to use these files. **Option A is easiest.**

---

## Option A — Pi Imager + setup-sd.sh (recommended)

### Step 1: Flash with Raspberry Pi Imager

1. Install Pi Imager if needed:
   ```bash
   sudo nala install rpi-imager
   ```
   Or download from https://www.raspberrypi.com/software/

2. **Copy imager.json into Pi Imager's config dir** so it pre-fills the advanced settings:
   ```bash
   mkdir -p ~/.config/Raspberry\ Pi
   cp pi/imager-config/imager.json ~/.config/Raspberry\ Pi/imager.json
   ```
   > Edit `imager.json` first — fill in your actual `wifiPassword` for "stella" if you want WiFi.

3. Open rpi-imager:
   ```bash
   rpi-imager
   ```

4. Choose OS: **Raspberry Pi OS Lite (64-bit)** (Bookworm, no desktop)

5. Choose Storage: your SD card adapter

6. Click the **gear icon** (or Ctrl+Shift+X) — settings should already be pre-filled:
   - Hostname: `ziggy`
   - SSH: enabled, authorized key: (lou-laptop key)
   - Username: `pi`
   - WiFi: stella (if you filled in the password)
   - Timezone: America/New_York

7. Click **Save** → **Write** → wait for flash + verify

### Step 2: Inject firstrun.sh (while SD card is still mounted)

```bash
chmod +x pi/imager-config/setup-sd.sh
./pi/imager-config/setup-sd.sh
```

This copies `firstrun.sh` onto the boot partition so it runs automatically on first boot and:
- Sets hostname `ziggy` + static IP `192.168.12.20`
- Pre-installs Docker, git, python3
- Clones the homelab repo
- Runs `pi/setup-pi.sh` (AdGuard replica, wg-easy, Vaultwarden)
- Reboots once to apply static IP

### Step 3: Eject and boot Ziggy

1. Safely eject SD card
2. Plug **ethernet** into Ziggy (wired = more stable than WiFi for a server)
3. Insert SD card, power on
4. Wait ~3 minutes for firstrun to complete
5. From laptop:
   ```bash
   ssh ziggy           # uses the alias you already have
   # or
   ssh pi@192.168.12.20
   ```
6. Check firstrun log:
   ```bash
   cat /var/log/firstrun.log
   ```

---

## Option B — Manual (no setup-sd.sh)

If you just want to use Pi Imager's built-in advanced settings without the
firstrun script (you'll finish setup manually after SSH in):

1. Copy `imager.json` as above
2. Flash normally — Pi Imager will handle hostname, user, SSH key, WiFi, timezone
3. After first boot, SSH in and run setup manually:
   ```bash
   ssh ziggy
   git clone https://github.com/wlfogle/homelab-media-stack.git /opt/homelab-media-stack
   sudo bash /opt/homelab-media-stack/pi/setup-pi.sh
   ```

---

## Files in this directory

| File | Purpose |
|------|---------|
| `imager.json` | Pi Imager preferences — drop into `~/.config/Raspberry Pi/` |
| `firstrun.sh` | First-boot automation — runs on Ziggy, sets static IP, installs Docker + services |
| `setup-sd.sh` | Run on laptop post-flash — injects firstrun.sh into SD boot partition |
| `README.md` | This file |

---

## Default credentials

| | Value |
|--|--|
| Username | `pi` |
| SSH password (fallback) | `ziggypi` — **change after first login** with `passwd` |
| SSH key auth | lou-laptop key pre-installed — no password needed |

---

## Notes

- **Wired ethernet is strongly recommended** for Ziggy. It's a DNS server — WiFi is unreliable.
- Static IP `192.168.12.20` is set in `firstrun.sh` via `dhcpcd.conf`
- After everything is running, add a DHCP reservation in your router for `192.168.12.20` as a backup
- `ziggy.local` will resolve via mDNS (avahi) on the LAN once running
