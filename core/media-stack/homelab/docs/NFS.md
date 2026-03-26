# Laptop NFS Shares -> Tiamat
This mounts selected laptop storage into Tiamat so media/services can use existing data without copying.
## Planned Shares
- `/media/loufogle/Data/Calibre Library` -> `/mnt/laptop/calibre`
- `/media/loufogle/Data/Cookbooks` -> `/mnt/laptop/cookbooks`
- `/media/loufogle/SystemBackup/Videos` -> `/mnt/laptop/videos`
- `/media/loufogle/ISOs1` -> `/mnt/laptop/isos`
- `/media/loufogle/Games/roms` -> `/mnt/laptop/roms`
- Optional IPTV file: `/media/loufogle/Data/Downloads/lou.m3u`
## Laptop Setup
Install NFS server:
```bash
sudo apt update
sudo apt install -y nfs-kernel-server
```
Edit `/etc/exports`:
```bash
/media/loufogle/Data/Calibre\040Library 192.168.12.242(rw,sync,no_subtree_check,no_root_squash)
/media/loufogle/Data/Cookbooks 192.168.12.242(rw,sync,no_subtree_check,no_root_squash)
/media/loufogle/SystemBackup/Videos 192.168.12.242(rw,sync,no_subtree_check,no_root_squash)
/media/loufogle/ISOs1 192.168.12.242(rw,sync,no_subtree_check,no_root_squash)
/media/loufogle/Games/roms 192.168.12.242(rw,sync,no_subtree_check,no_root_squash)
```
Apply:
```bash
sudo exportfs -ra
sudo systemctl enable --now nfs-kernel-server
```
## Tiamat Setup
Install NFS client tools:
```bash
apt update
apt install -y nfs-common
```
Create mountpoints:
```bash
mkdir -p /mnt/laptop/{calibre,cookbooks,videos,isos,roms}
```
Add to `/etc/fstab`:
```bash
192.168.12.172:/media/loufogle/Data/Calibre\040Library /mnt/laptop/calibre nfs defaults,_netdev 0 0
192.168.12.172:/media/loufogle/Data/Cookbooks /mnt/laptop/cookbooks nfs defaults,_netdev 0 0
192.168.12.172:/media/loufogle/SystemBackup/Videos /mnt/laptop/videos nfs defaults,_netdev 0 0
192.168.12.172:/media/loufogle/ISOs1 /mnt/laptop/isos nfs defaults,_netdev 0 0
192.168.12.172:/media/loufogle/Games/roms /mnt/laptop/roms nfs defaults,_netdev 0 0
```
Mount:
```bash
mount -a
```
Verify:
```bash
df -h | grep /mnt/laptop
```
## Container Usage
- Calibre-Web CT: bind mount `/mnt/laptop/calibre` and `/mnt/laptop/cookbooks`
- Plex/Jellyfin: add `/mnt/laptop/videos` as Home Videos library
- TVHeadend/Jellyfin Live TV: use `lou.m3u` via NFS path
- Proxmox ISO imports: copy from `/mnt/laptop/isos`
## Notes
- NFS mounts depend on laptop availability.
- If laptop is off, dependent services should be tolerant of missing mounts.
