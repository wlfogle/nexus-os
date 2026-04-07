# 🚀 Quick Start Guide

## TL;DR - Just Install It

```bash
# 1. List disks and choose yours
lsblk -d -o NAME,SIZE,TYPE,MODEL

# 2. Set target disk
export TARGET_DISK=/dev/nvme0n1  # CHANGE THIS!

# 3. Run installer
sudo ./install.sh

# 4. Type YES when prompted
# 5. Wait 15-30 minutes
# 6. Reboot
```

---

## Customize Installation

```bash
# Set your preferences
export TARGET_DISK=/dev/sda
export USERNAME="yourname"
export USER_PASSWORD="yourpassword"
export HOSTNAME="yourcomputer"

# Run with your settings
sudo -E ./install.sh
```

---

## What You Get

After reboot, you'll have:

- ✅ Kubuntu 24.04 with ZFS
- ✅ KDE Desktop ready to use
- ✅ NVIDIA drivers (if GPU detected)
- ✅ Steam + gaming tools
- ✅ Docker installed
- ✅ Ollama AI service
- ✅ Development tools (Rust, Node.js, Git)
- ✅ WireGuard VPN configured

---

## After First Boot

### Start Media Stack
```bash
cd ~/media-stack
docker compose up -d
```

### Access Services
- Jellyfin: http://localhost:8096
- qBittorrent: http://localhost:8080
- Radarr: http://localhost:7878
- Sonarr: http://localhost:8989

### Pull AI Models
```bash
ollama pull codellama
ollama pull llama2
```

---

## Troubleshooting

**Check installation log:**
```bash
cat install.log
```

**Resume failed installation:**
```bash
sudo ./install.sh  # Will continue from where it failed
```

**Common fixes:**
- No internet? Check WiFi/ethernet
- Not UEFI? Enable in BIOS
- Disk too small? Need 32GB minimum

---

## Need Help?

See full README.md for detailed documentation.
