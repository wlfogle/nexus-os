# AnyDesk — Remote Graphical Control
Full-desktop remote access across all homelab machines.
Works through T-Mobile CGNAT — no port forwarding required.

---

## Architecture

```
Anywhere in the world
        │
        ▼
  AnyDesk Relay Servers  (AnyDesk's cloud, punches through CGNAT)
        │
        ├──▶  Tiamat (Proxmox + XFCE)   192.168.12.242
        ├──▶  Ziggy  (Pi 3B+ + XFCE)    192.168.12.20
        └──▶  Laptop (Pop!_OS)           192.168.12.x
```

Each machine has a permanent **AnyDesk ID** (9-digit number).
Connect by entering the ID + unattended password — no IP needed.

---

## Setup by Machine

### Tiamat (Proxmox VE — x86_64)

Run on the Proxmox host **after** Proxmox is installed:

```bash
ssh root@192.168.12.242
git clone https://github.com/wlfogle/homelab-media-stack.git /opt/homelab-media-stack
bash /opt/homelab-media-stack/scripts/setup-anydesk.sh
```

The script:
- Installs XFCE4 + lightdm (lightweight desktop, ~200MB)
- Adds AnyDesk apt repo and installs AnyDesk
- Enables AnyDesk as a systemd service (starts on boot)
- Prompts you to set an unattended access password
- Prints your Tiamat AnyDesk ID

**Note:** XFCE runs alongside Proxmox. The Proxmox web UI at `https://192.168.12.242:8006`
is unaffected. Once connected via AnyDesk, open Firefox on the XFCE desktop
and navigate to `https://localhost:8006` for full Proxmox management.

---

### Ziggy (Raspberry Pi 3B+ — ARM64)

Run on Ziggy after first boot:

```bash
ssh pi@192.168.12.20
sudo bash /opt/homelab-media-stack/pi/setup-anydesk.sh
```

The script:
- Installs XFCE4 + lightdm
- Downloads AnyDesk ARM64 .deb directly (no apt repo for ARM)
- Falls back to armhf if arm64 build is unavailable
- Enables AnyDesk service on boot
- Prompts for password, prints Ziggy AnyDesk ID

**Note:** Pi 3B+ has 1GB RAM. XFCE uses ~150MB at idle, AnyDesk ~50MB.
Avoid opening too many browser tabs while connected.

---

### Laptop (Pop!_OS — x86_64)

Already has a desktop. Just install AnyDesk:

```bash
wget -qO - https://keys.anydesk.com/repos/DEB-GPG-KEY \
    | gpg --dearmor \
    | sudo tee /usr/share/keyrings/anydesk-keyring.gpg > /dev/null

echo "deb [signed-by=/usr/share/keyrings/anydesk-keyring.gpg] http://deb.anydesk.com/ all main" \
    | sudo tee /etc/apt/sources.list.d/anydesk-stable.list

sudo nala update && sudo nala install -y anydesk
```

Launch AnyDesk from the applications menu or run `anydesk`.

---

## Android Phone

1. Open **Google Play Store**
2. Search **AnyDesk Remote Desktop**
3. Install → Open
4. Tap the address bar at the top
5. Enter a machine's AnyDesk ID (e.g. `123 456 789`)
6. Tap **Connect**
7. Enter the unattended password when prompted
8. You now have full graphical control of that machine on your phone

**Gestures:**
- Single tap = mouse click
- Two-finger scroll = scroll wheel
- Pinch = zoom
- Hold + drag = drag
- Tap keyboard icon = open on-screen keyboard

---

## Fire TV / Android TV

AnyDesk is available in the **Amazon Appstore**:

1. On Fire TV: Settings → My Fire TV → Developer Options → Apps from Unknown Sources → ON
2. Open Amazon Appstore → search **AnyDesk**
3. Install → Open
4. Navigate with D-pad to the address bar
5. Enter the machine AnyDesk ID with the on-screen keyboard
6. Press **Connect**, enter password
7. Control the remote desktop with the Fire TV remote D-pad

**Tip:** Pin AnyDesk to your Fire TV home screen for quick access.

---

## Connecting: Step-by-Step

```
1. Open AnyDesk on any device (phone, laptop, Fire TV)
2. In the "Remote Desk" field, type the target machine's ID:
      Tiamat:  <printed by setup-anydesk.sh>
      Ziggy:   <printed by pi/setup-anydesk.sh>
3. Click / tap Connect
4. Select "Log in with password" (unattended access)
5. Enter the password you set during setup
6. Full desktop appears — you have complete graphical control
```

---

## Finding AnyDesk IDs After Setup

If you need to retrieve a machine's ID later:

```bash
# On Tiamat or Ziggy
anydesk --get-id

# Or check the AnyDesk window if you're already connected
```

---

## What You Can Do Once Connected

| Machine | What you control |
|---------|-----------------|
| **Tiamat** | Full Proxmox host desktop — open `https://localhost:8006` in Firefox for Proxmox UI. Manage LXCs, VMs, storage. Run terminal commands. |
| **Ziggy** | Full Pi desktop — manage Docker containers, edit configs, run scripts, monitor system. |
| **Laptop** | Full Pop!_OS desktop — access local files, tools, browser. |

---

## Security Tips

- Use a **strong, unique password** for each machine's unattended access
- In AnyDesk Settings → Security: enable **Two-Factor Authentication**
- In AnyDesk Settings → Security: set **Allow access only if** → `AnyDesk window is shown` to require approval for new connections
- Whitelist your own AnyDesk IDs under Settings → Security → **Allowlist**

---

## Autostart Check

Verify AnyDesk survives a reboot:

```bash
# On Tiamat or Ziggy
systemctl status anydesk
# Should show: active (running)

# Re-enable if needed
systemctl enable anydesk
systemctl start anydesk
```

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| "Not ready" on connection | `systemctl restart anydesk` on target machine |
| Black screen after connect | XFCE/lightdm not running — `systemctl restart lightdm` |
| ARM64 deb download fails | Check https://anydesk.com/en/downloads/raspberry-pi for latest version, update `ANYDESK_VERSION` in `pi/setup-anydesk.sh` |
| Connection refused | AnyDesk service not running — `systemctl start anydesk` |
| Forgot unattended password | `echo "newpassword" \| anydesk --set-password` |
