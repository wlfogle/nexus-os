# RetroArch on Fire TV

Play retro games on Fire TV using ROMs from the laptop NFS share.

## Prerequisites
- Laptop NFS server running (see `docs/NFS.md`)
- ROMs at `/media/loufogle/Games/roms` exported to Tiamat
- Fire TV on same LAN (`192.168.12.x`)

## Install RetroArch
1. **Fire TV App Store**: Search "RetroArch" and install
2. Or **sideload** via Downloader app if not in your region

## Configure ROM Access via NFS
RetroArch supports NFS natively:
1. Open RetroArch → **Settings** → **Network**
2. Under NFS, add server: `192.168.12.172` (laptop IP)
3. Mount path: `/media/loufogle/Games/roms`
4. Go to **Load Content** → browse the NFS mount

Alternatively, if ROMs are mounted on Tiamat:
- NFS server: `192.168.12.242`
- Mount path: `/mnt/laptop/roms`

## Controller Setup
1. RetroArch → **Settings** → **Input** → **Port 1 Controls**
2. Fire TV remote works for basic navigation
3. For actual gaming: pair a Bluetooth controller (8BitDo, Xbox, PS4/PS5)
4. Map buttons under **Port 1 Controls** → **Set All Controls**

## Scan and Play
1. **Load Content** → **Scan Directory** → select your NFS ROM folder
2. RetroArch auto-detects consoles (NES, SNES, Genesis, GBA, etc.)
3. Scanned games appear in **Playlists** organized by system
4. Select a game → RetroArch downloads the correct core automatically

## Supported Systems
The retro collection includes NES, SNES, Genesis, GBA, N64, PS1, Arcade, and more. RetroArch handles all of them with downloadable cores.

## Tips
- Enable **Rewind** in Settings → Frame Throttle for save-state style rewind
- Set **Video** → **Aspect Ratio** to "Core Provided" for correct display
- Use **Quick Menu** (hold Start+Select) to save/load states mid-game
