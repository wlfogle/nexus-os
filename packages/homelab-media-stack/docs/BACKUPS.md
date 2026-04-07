# Backups
## Current State
`scripts/backup.sh` runs daily at **03:00** via `/etc/cron.d/vzdump` on Tiamat.

What it does:
1. **vzdump snapshot** of all 17 running containers → `/mnt/hdd/backups/lxc/` (zstd compressed)
2. **Jellyseerr config pull** from CT-242 Docker volume → `/mnt/hdd/backups/appdata/jellyseerr-config-DATE.tar.gz`
3. **appdata tar** of `/opt/appdata/` (host-side container configs) → `/mnt/hdd/backups/appdata/appdata-DATE.tar.gz`
4. **Rotation** — keeps 7 days of each

Containers backed up: 100, 101, 102, 103, 104, 105, 106, 107, 210, 212, 214, 215, 230, 231, 240, 242, 900
(VMs excluded — too large for nightly; vzdump those manually before major changes)

Log: `/var/log/homelab-backup.log`

## Planned — Restic (not yet deployed)
Restic will supplement vzdump with a deduplicated, prunable appdata backup.
Destination: `/mnt/hdd/backups/restic`

Install when ready:
```bash
apt install -y restic
export RESTIC_REPOSITORY=/mnt/hdd/backups/restic
export RESTIC_PASSWORD='change-this'
restic init
```
Schedule at 03:30 (30 min after vzdump finishes):
```bash
30 3 * * * root restic -r /mnt/hdd/backups/restic --password-file /root/.restic-pass backup /opt/appdata >> /var/log/restic-backup.log 2>&1
```
Retention:
```bash
restic forget --prune --keep-daily 7 --keep-weekly 4 --keep-monthly 3
```
## Restore
Vzdump restore (from Proxmox UI or CLI):
```bash
# List available backups
ls /mnt/hdd/backups/lxc/
# Restore a container
qmrestore /mnt/hdd/backups/lxc/vzdump-lxc-242-*.tar.zst 242 --storage local-lvm
```
Restic restore (once deployed):
```bash
restic snapshots
restic restore latest --target /tmp/restic-restore --include /opt/appdata/sonarr
```
## Notes
- Test restore monthly — a backup never tested is not a backup.
- Move restic password to a root-only file: `echo 'yourpassword' > /root/.restic-pass && chmod 600 /root/.restic-pass`
