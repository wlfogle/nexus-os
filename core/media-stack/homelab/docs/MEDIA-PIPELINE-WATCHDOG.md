# Media Pipeline Watchdog
This watchdog makes the request-to-download pipeline self-healing for Sonarr, Radarr, Readarr, Lidarr, and future compatible apps.

## What it does
- Ensures every configured app has a qBittorrent download client
- Removes poisoned releases automatically
- Removes stale queue items older than the configured threshold
- Allows the app to re-search clean releases automatically
- Works from a single JSON config so new apps can be added without code changes

## Poisoned release patterns
- `.exe`
- `executable file`
- `password protected`
- `rar password`
- `contains executable`

## Files
- Script: `scripts/media-pipeline-watchdog.py`
- Sample config: `config/media-pipeline-watchdog.sample.json`
- systemd service: `infrastructure/watchdogs/media-pipeline-watchdog.service`
- systemd timer: `infrastructure/watchdogs/media-pipeline-watchdog.timer`

## Deployment on Tiamat
Copy the script to `/usr/local/bin/media-pipeline-watchdog.py`, copy a real config to `/etc/media-pipeline-watchdog.json`, install the service/timer units, then run:

```bash
systemctl daemon-reload
systemctl enable --now media-pipeline-watchdog.timer
systemctl start media-pipeline-watchdog.service
```

## Adding future apps
Add another object to `apps` in the config with:
- `name`
- `url`
- `api_version`
- `api_key`
- `media_kind`

Supported `media_kind` values map to qBit categories:
- `tv`
- `movie`
- `book`
- `music`
- `adult`
