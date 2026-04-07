# Hardware Documentation

## Server — CyberPowerPC C Series ET8890-37125 (Tiamat @ 192.168.12.242 post-Proxmox)

> Specs confirmed via SSH + PowerShell query on 2026-03-13

| Component | Spec |
|-----------|------|
| **CPU** | AMD Ryzen 5 3600, 6-core / 12-thread, 3.6GHz base / 4.2GHz boost |
| **RAM** | 8GB DDR4 @ 3000MHz — single stick, **3 slots free** |
| **GPU** | XFX Radeon RX 580 Series, 4GB VRAM, Driver 31.0.21924.61 |
| **Motherboard** | Gigabyte B450M DS3H WIFI-CF (MicroATX, AM4 socket) |
| **Storage (OS)** | 240GB WD SSD |
| **Storage (Media)** | 2TB WD HDD |
| **Storage (External)** | 240GB Kingston SSD (USB) + 2GB USB flash drive |
| **PSU** | 450W |
| **Networking** | Gigabit Ethernet + 802.11AC Wi-Fi (onboard) |
| **OS** | Windows 10 Home 64-bit, Build 19045 |
| **Build date** | ~March 2020 |
| **Chassis** | Full tower, tempered glass side panel |
| **Service number** | 888-937-5582 |

## Recommended Upgrades

1. **RAM** ✅ UPGRADED: 32GB DDR4-3200 (was 8GB, upgraded 2026-03-26)
   - VM-901 can now run alongside the full media stack
   - FlareSolverr Cloudflare solving now possible (Chrome headless needs ~2GB)
   - B450M DS3H supports up to 128GB across 4 slots (2 slots used)
2. **OS**: Migrated to Proxmox VE ✅
3. **Storage**: 2TB WD HDD in use for media. 240GB WD SSD (sdb) passed through to VM-901 for games.
4. **Network**: Use wired Ethernet — avoid Wi-Fi for streaming stability

## GPU Passthrough Status

RX 580 (XFX) is **fully configured for VFIO passthrough** — no setup needed:
- `amd_iommu=on iommu=pt` in GRUB cmdline ✅
- IOMMU Group 2: `09:00.0` (GPU) + `09:00.1` (HDMI audio) — clean group ✅
- Both functions bound to `vfio-pci` driver ✅
- VFIO PCI IDs: `1002:67df,1002:aaf0` in `/etc/modprobe.d/vfio-pci.conf` ✅
- `vfio_iommu_type1 allow_unsafe_interrupts=1` set ✅

> With GPU passed through, Proxmox host is **headless** — manage via web UI or SSH only.

## Storage on Tiamat

| Device | Size | Use |
|--------|------|-----|
| sda (ST2000DM008 HDD) | 1.8TB | Proxmox LVM: root (96GB), media-hdd (1.5TB), all CT/VM disks |
| sdb (WDC WDS240G2G0A SSD) | 223GB | Passed through to VM-901 (Windows games storage); contains existing Windows install on sdb4 |

## Ziggy (Raspberry Pi 3B+)

| Component | Spec |
|-----------|------|
| **CPU** | ARM Cortex-A53, 4-core 1.4GHz (64-bit) |
| **RAM** | 1GB LPDDR2 |
| **Storage** | microSD (class 10 / A1 recommended, 16GB+) |
| **Networking** | 10/100 Ethernet + 802.11AC Wi-Fi (use wired) |
| **OS** | Raspberry Pi OS Lite 64-bit (Bookworm) |
| **Services** | AdGuard Home (replica), wg-easy (remote VPN), Vaultwarden |

> Vaultwarden requires HTTPS. Use Caddy as a reverse proxy on the Pi for automatic TLS.

## Laptop — Primary Admin / Dev Machine

| Component | Spec |
|-----------|------|
| **OS** | Pop!_OS 22.04 LTS (KDE Plasma 5.24.7) |
| **Kernel** | 6.17.9-76061709-generic (64-bit) |
| **CPU** | Intel Core i9-13900HX, 24-core (8P+16E) / 32-thread, 13th Gen |
| **RAM** | 62.5 GB |
| **GPU (integrated)** | Intel Arc / Intel Xe Graphics (Mesa) |
| **GPU (discrete)** | NVIDIA GeForce RTX 4080 (laptop) |
| **Graphics platform** | X11 |
| **Role** | Primary admin workstation, repo dev, Proxmox SSH, Android APK builds |

> NVIDIA RTX 4080 available for CUDA workloads — useful for AI/transcoding experiments.
> Android APK builds run here (Java 17 + Android SDK).

## Client Devices

| Device | Role |
|--------|------|
| 2x Fire TV + Fire TV Cube | Jellyfin + Plex, Homarr/Overseerr via Silk Browser, nzb360 sideload, TiamatsStack APK |
| Android (tablet/phone) | Jellyfin, Plex, nzb360, Bitwarden, WireGuard |
| Laptop (i9-13900HX) | Proxmox admin, Homarr, MediaStack Control, SSH, APK builds |

## Network Requirements

- All devices on the same LAN for local streaming
- Server should have a static LAN IP (set via router DHCP reservation or Proxmox static config)
- Gigabit router/switch recommended for smooth 1080p/4K local streaming
