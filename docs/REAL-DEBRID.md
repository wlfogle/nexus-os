# Real-Debrid + rdt-client
Use Real-Debrid as a high-speed download source alongside qBittorrent.
## Why Use Both
- `rdt-client`: great for cached public content from Real-Debrid
- `qBittorrent`: keep for private trackers and non-cached content
## rdt-client Deployment
Service exists in `media-stack/docker-compose.yml` as `rdt-client` on port `6500`.
Required env:
```bash
RDTCLIENT_PORT=6500
REAL_DEBRID_API_KEY=<your-token>
```
## Get Real-Debrid API Key
1. Log in to Real-Debrid.
2. Open API token page: `https://real-debrid.com/apitoken`
3. Copy token and set `REAL_DEBRID_API_KEY` in `.env`.
## Sonarr / Radarr Integration
In each app:
1. Settings -> Download Clients -> Add.
2. Choose custom/blackhole-compatible method recommended by current rdt-client docs.
3. Point host to CT-213 / service endpoint.
4. Keep qBittorrent configured as fallback.
## Folder Mapping
Ensure consistent paths between apps and downloader:
- Downloads root: `/downloads`
- Media roots:
  - Movies: `/movies`
  - TV: `/tv`
## Operational Flow
1. Sonarr/Radarr send job.
2. rdt-client resolves through Real-Debrid cache.
3. Completed files land in downloads.
4. Sonarr/Radarr import and move/hardlink into media libraries.
## Security Notes
- Do not commit API tokens to git.
- Keep token only in `.env` and secrets management.
