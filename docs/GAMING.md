# Gaming

## ROM Collection
- **Location** (laptop): `/media/loufogle/Games/roms` (89GB, Switch NSPs + retro)
- **Retro collection**: `/media/loufogle/Data/Downloads/6666 games in 1 Ultimate Classic Games Collection [Retro Legends]`
- **NFS export**: laptop → Tiamat at `/mnt/laptop/roms` (see `docs/NFS.md`)

## Windows Gaming VM (VM-901)
- Windows 11 26H1 on Tiamat
- RX 580 GPU passthrough (VFIO)
- 240GB SSD passthrough (`/dev/sdb`) for game storage
- 300GB LVM OS disk
- PlayOn Home saves to `/mnt/hdd/media/playon` → Plex/Jellyfin
- With 32GB RAM, can now run alongside the full media stack

## Fire TV Retro Gaming
See `docs/RETRO-GAMING.md` for RetroArch setup on Fire TV with NFS ROM access.

## Future: Emulation Frontend CT
Planned Proxmox container for web-based emulation (EmulationStation-Web or RetroArch Web):
- Mount `/mnt/laptop/roms` via NFS bind
- Expose via Traefik at `games.tiamat.local`
- Accessible from Fire TV Silk Browser or TiamatsStack app
