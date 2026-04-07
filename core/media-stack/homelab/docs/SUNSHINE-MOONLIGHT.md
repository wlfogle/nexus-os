# Sunshine + Moonlight Game Streaming

Stream games from the laptop's RTX 4080 to any device — Fire TV, phone, tablet, or another laptop — over LAN or remotely via Tailscale.

---

## Architecture

```
[Laptop — Sunshine host]
  RTX 4080 + RTX 4080 encoder
  pop-os  |  LAN: 192.168.12.172 or 192.168.12.204
           |  Tailscale: 100.66.87.38
           |
     ┌─────┴────────────────────────────┐
     │           Moonlight clients      │
     ├── Fire TV      (LAN or Tailscale)│
     ├── Android phone/tablet           │
     ├── Another laptop/PC              │
     └── (any NVIDIA GameStream client) │
```

The controller plugs into the **client** device, not the host. Moonlight forwards inputs over the network.

---

## Host — Laptop (Sunshine)

### What's installed
| Item | Detail |
|------|--------|
| Binary | `~/.local/bin/Sunshine.AppImage` |
| Service | `sunshine-appimage` (user systemd) |
| Web UI | `https://localhost:47990` |
| Default credentials | `sunshine` / `sunshine` — **change after first login** |
| Dependency | `libstdc++6` from PPA `ppa:ubuntu-toolchain-r/test` |

### Configured apps
| App name in Moonlight | What launches |
|-----------------------|---------------|
| Desktop | Full Pop!_OS desktop |
| Switch (Ryubing) | `/snap/bin/ryubing-emulator` |
| Steam Big Picture | Steam in Big Picture mode |

### Service management
```bash
systemctl --user status sunshine-appimage
systemctl --user restart sunshine-appimage
systemctl --user stop sunshine-appimage
journalctl --user -u sunshine-appimage -f   # live logs
```

### Firewall ports (UFW — already configured)
| Port | Proto | Purpose |
|------|-------|---------|
| 47984 | TCP | RTSP stream setup |
| 47989 | TCP | HTTP |
| 47990 | TCP | HTTPS web UI |
| 48010 | TCP | RTSP stream |
| 47998 | UDP | Video stream |
| 47999 | UDP | Control |
| 48000 | UDP | Audio |
| 48002 | UDP | Audio (alt) |

Rules allow `192.168.12.0/24` (LAN) and `100.64.0.0/10` (Tailscale).

### Change password
Go to `https://localhost:47990` → top-right menu → **Change Password**.

### Add or edit apps
Go to `https://localhost:47990` → **Applications** → **Add New** or edit existing.

To add a ROM-specific Ryubing launch (optional):
- **App name**: e.g. `Switch — Zelda TotK`
- **Command**: `/snap/bin/ryubing-emulator /mnt/tiamat-roms/switch/TotK.nsp`
- Requires NFS mount of Tiamat ROMs — see `docs/NFS.md`

---

## Client Setup — Fire TV

### Install Moonlight
1. Open Fire TV app store → search **Moonlight Game Streaming** → Install
   - If not in store, sideload via **Downloader** app from `https://moonlight-stream.org/apk`
2. Open Moonlight → tap **Add Host manually**
3. Enter the laptop IP:
   - LAN (home): `192.168.12.172` or `192.168.12.204`
   - Remote (Tailscale): `100.66.87.38`
4. A PIN appears on screen — go to `https://localhost:47990` on the laptop → **Pin** → enter it

### Controllers on Fire TV
Connect a controller to the Fire TV **before** launching Moonlight:

**Bluetooth (recommended — no cables):**
1. Fire TV Settings → Controllers & Bluetooth Devices → Other Bluetooth Devices → Add Bluetooth Devices
2. Put controller in pairing mode:
   - **Xbox Wireless**: hold Xbox button + Share button until light flashes
   - **PS4 DualShock 4**: hold Share + PS button until light bar flashes
   - **PS5 DualSense**: hold Create + PS button until light flashes
   - **8BitDo**: press pairing button per device instructions
   - **Amazon Luna**: hold Luna button until light pulses
3. Select the controller from the list

**USB (wired):**
- Use a **USB OTG adapter** for your Fire TV model:
  - Fire TV Stick (1st–4th gen): **Micro-USB OTG** adapter → USB-A controller cable
  - Fire TV Stick 4K Max / Cube: **USB-C OTG** adapter → USB-A controller cable
- Plug in, controller is recognized automatically

### Streaming tips for Fire TV
- Set resolution/FPS in Moonlight settings to match your TV (4K 30fps or 1080p 60fps)
- If input feels laggy: Moonlight Settings → **Optimize game settings** → ON
- For Switch games: launch **Switch (Ryubing)** from Moonlight, not Desktop

---

## Client Setup — Android Phone / Tablet

### Install Moonlight
1. Install **Moonlight Game Streaming** from Play Store (free)
2. Open → tap **+** → enter host IP:
   - LAN: `192.168.12.172`
   - Tailscale: `100.66.87.38`
3. Enter PIN shown → type it in the Sunshine web UI at `https://localhost:47990`

### Controllers on Android
**Bluetooth:**
- Android Settings → Bluetooth → pair Xbox / PS4 / PS5 / 8BitDo controller
- Works the same as any Bluetooth device

**USB-C OTG:**
- USB-C to USB-A OTG cable → wired controller

**Touch overlay (no controller):**
- Moonlight has an on-screen gamepad overlay — enable in Moonlight Settings → **On-Screen Controls**

---

## Client Setup — Another Laptop / PC

### Install Moonlight
- **Linux**: `flatpak install flathub com.moonlight_stream.Moonlight` or download AppImage from moonlight-stream.org
- **Windows**: Download installer from moonlight-stream.org
- **macOS**: Available on App Store

### Connect
1. Open Moonlight → **+** → enter IP (LAN or Tailscale)
2. Enter PIN in Sunshine web UI
3. Controller: plug into **any USB port** on the client laptop, or use Bluetooth

---

## Remote Access (outside home network)

Tailscale is already installed and connected on the laptop (`100.66.87.38`).

To stream remotely:
1. Ensure Tailscale is running on the **client** device:
   - Install Tailscale from tailscale.com / Play Store / App Store
   - Sign in with the same Tailscale account
2. In Moonlight, use the Tailscale IP: `100.66.87.38`
3. Stream quality degrades with WAN latency — recommend 720p 60fps for remote play

> **Tip**: For best remote experience, set Moonlight bitrate to 15–20 Mbps and resolution to 1080p 60fps. The RTX 4080 NVENC encoder handles this with minimal CPU overhead.

---

## Switch ROM Access from Any Client

ROMs live on Tiamat (`/mnt/hdd/media/roms/switch/`). To make them available to Ryubing while streaming:

1. Mount Tiamat's NFS share on the laptop (if not already):
   ```bash
   sudo mount 192.168.12.242:/mnt/hdd/media/roms /mnt/tiamat-roms
   ```
2. In Ryubing: File → Open → navigate to `/mnt/tiamat-roms/switch/`
3. The game list persists across sessions once loaded

Alternatively, copy Switch NSPs locally to `/home/loufogle/Games/roms/switch/` for offline access.

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| Moonlight can't find host | Check Sunshine is running: `systemctl --user status sunshine-appimage` |
| PIN expired | Restart Sunshine: `systemctl --user restart sunshine-appimage` then re-pair |
| Black screen on connect | Sunshine needs an active display — check laptop lid/display is not off |
| Audio not working | In Moonlight settings → **Audio** → select the correct output |
| Controller not working in game | Ensure udev rule exists: `cat /etc/udev/rules.d/85-sunshine.rules` |
| Controller not recognized | Re-plug or re-pair controller; check `systemctl --user restart sunshine-appimage` |
| Remote play stutters | Lower bitrate/resolution in Moonlight settings; check Tailscale connection quality |
| GLIBCXX error on reinstall | Add PPA: `sudo add-apt-repository ppa:ubuntu-toolchain-r/test && sudo nala install libstdc++6` |

---

## Re-installing Sunshine (reference)

```bash
# Download latest AppImage
wget -O ~/.local/bin/Sunshine.AppImage \
  https://github.com/LizardByte/Sunshine/releases/latest/download/sunshine.AppImage
chmod +x ~/.local/bin/Sunshine.AppImage

# Set credentials
~/.local/bin/Sunshine.AppImage --creds sunshine sunshine

# Reload and restart service
systemctl --user daemon-reload
systemctl --user restart sunshine-appimage
```
