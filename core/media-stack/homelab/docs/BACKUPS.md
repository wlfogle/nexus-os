# Backups (Restic Strategy)
Use Restic to protect app configs and critical media-stack state.
## Scope
Primary backup source:
- `/opt/appdata` (container configs, service state)
Optional:
- selected docs/scripts from repo
Destination:
- `/mnt/hdd/backups/restic`
## Initial Setup
Install restic on Proxmox host:
```bash
apt update
apt install -y restic
```
Initialize repo:
```bash
export RESTIC_REPOSITORY=/mnt/hdd/backups/restic
export RESTIC_PASSWORD='change-this'
restic init
```
## Backup Script
Create `/usr/local/bin/restic-backup.sh`:
```bash
#!/bin/bash
set -euo pipefail
export RESTIC_REPOSITORY=/mnt/hdd/backups/restic
export RESTIC_PASSWORD='change-this'
restic backup /opt/appdata
restic forget --prune --keep-daily 7 --keep-weekly 4 --keep-monthly 3
```
Make executable:
```bash
chmod +x /usr/local/bin/restic-backup.sh
```
## Schedule
Cron daily at 03:30:
```bash
30 3 * * * /usr/local/bin/restic-backup.sh >> /var/log/restic-backup.log 2>&1
```
## Restore Examples
List snapshots:
```bash
restic snapshots
```
Restore latest snapshot:
```bash
restic restore latest --target /tmp/restic-restore
```
Restore single path:
```bash
restic restore latest --target /tmp/restic-restore --include /opt/appdata/sonarr
```
## Hardening Notes
- Move `RESTIC_PASSWORD` into a root-only file or secret store.
- Test restore monthly (backup is useless without restore validation).
