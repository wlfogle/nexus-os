# RetroArch on Fire TV

Play retro games on Fire TV using ROMs from Tiamat.

## Prerequisites
- ROMs on Tiamat at `/mnt/hdd/media/roms/` (94GB, 12 systems — see `docs/GAMING.md`)
- Fire TV on same LAN (`192.168.12.x`)
- Bluetooth controller paired (8BitDo, Xbox, PS5)

## Install RetroArch
1. **Fire TV App Store**: Search "RetroArch" and install
2. Or **sideload** via Downloader app if not in your region

## Option A: RetroPie on Tiamat (CT-280)
RetroPie + EmulationStation running on Tiamat LXC.
- Access via VNC or Moonlight streaming from Fire TV
- All ROMs pre-loaded at `/home/pi/RetroPie/roms/`

## Option B: RetroArch Native + NFS (standalone on Fire TV)
RetroArch supports NFS natively:
1. Open RetroArch → **Settings** → **Network**
2. Under NFS, add server: `192.168.12.242` (Tiamat)
3. Mount path: `/mnt/hdd/media/roms`
4. Go to **Load Content** → browse the NFS mount

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
