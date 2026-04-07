# Gaming

## ROM Collection — Tiamat `/mnt/hdd/media/roms/`
All ROMs live on Tiamat's 2TB HDD (always available, no laptop dependency).

| System | Folder | ROMs | Size |
|--------|--------|------|------|
| Atari 2600 | `atari2600/` | 834 | 10MB |
| Gameboy Advance | `gba/` | 102 | 386MB |
| MAME/Arcade | `arcade/` | 1,047 | 85MB |
| Neo Geo | `neogeo/` | 517 | 2.8GB |
| Nintendo 64 | `n64/` | 49 | 702MB |
| NES | `nes/` | 2,123 | 224MB |
| Sega Master System | `mastersystem/` | 401 | 46MB |
| Sega Genesis | `megadrive/` | 956 | 634MB |
| SNES | `snes/` | 837 | 660MB |
| TurboGrafx-16 | `pcengine/` | 683 | 93MB |
| ZX Spectrum | `zxspectrum/` | 13 | 5MB |
| Nintendo Switch | `switch/` | 13 | 89GB |

**Total: 7,575 retro + 13 Switch = 94GB**

Source: retro from `6666 games in 1 Ultimate Classic Games Collection [Retro Legends]` ISO,
Switch NSPs from laptop `/media/loufogle/Games/roms/`.

## Gaming CT (CT-280 — RetroPie)
- RetroPie + EmulationStation on Debian LXC
- Bind mount: `/mnt/hdd/media/roms` → `/home/pi/RetroPie/roms`
- Access: VNC or `http://games.tiamat.local` via Traefik (EmulationStation web)
- Play from: laptop, Fire TV (Moonlight/VNC), any device on LAN

## Windows Gaming VM (VM-901)
- Windows 11 26H1 on Tiamat
- RX 580 GPU passthrough (VFIO)
- 240GB SSD passthrough (`/dev/sdb`) for game storage
- 300GB LVM OS disk
- PlayOn Home saves to `/mnt/hdd/media/playon` → Plex/Jellyfin
- With 32GB RAM, can now run alongside the full media stack

## Fire TV Gaming
- **Retro**: RetroArch from App Store, NFS ROMs from `192.168.12.242:/mnt/hdd/media/roms`
- **Web**: RetroArch Web at `http://games.tiamat.local` via Silk Browser
- **Switch/PC streaming**: Moonlight on Fire TV → Sunshine on laptop (RTX 4080)
- See `docs/RETRO-GAMING.md` for detailed Fire TV setup

## Laptop Gaming
- **RetroPie 4.8.11** installed at `/opt/retropie/` (35 systems, 25 libretro cores)
- **Ryubing** (Switch emulator) via snap
- **Sunshine** (planned) for game streaming to Fire TVs
