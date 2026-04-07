# Proxmox VE 9 — Installation Guide (Tiamat)

## Base network
- Proxmox host: `192.168.12.242`
- Gateway: `192.168.12.1`
- Web UI: `https://192.168.12.242:8006`

## Prepare 2TB HDD (wipes old Steam data)
```bash
lsblk
parted /dev/sdb --script mklabel gpt mkpart primary ext4 0% 100%
mkfs.ext4 -L hdd /dev/sdb1
mkdir -p /mnt/hdd
echo "LABEL=hdd /mnt/hdd ext4 defaults,nofail 0 2" >> /etc/fstab
mount -a
mkdir -p /mnt/hdd/{torrents/{movies,tv,music,books},media/{movies,tv,music,books},backups}
```

## Key deployment CTs
| CT | Hostname | IP | Purpose |
|---|---|---|---|
| 100 | wireguard | 192.168.12.100 | WireGuard server |
| 101 | wg-proxy | 192.168.12.101 | WireGuard client + TinyProxy |
| 102 | flaresolverr | 192.168.12.102 | FlareSolverr |
| 210 | prowlarr | 192.168.12.210 | Indexer manager |
| 212 | qbittorrent | 192.168.12.212 | Downloader |
| 214 | sonarr | 192.168.12.214 | TV automation |
| 215 | radarr | 192.168.12.215 | Movie automation |
| 230 | plex | 192.168.12.230 | Media server |
| 231 | jellyfin | 192.168.12.231 | Media server |
| 900 | ziggy | DHCP | Ollama runtime |

## WireGuard/TinyProxy
```bash
pct exec 100 -- sh -lc "cd /opt/homelab-media-stack/infrastructure/wireguard-server && sh setup-wg-server.sh"
pct exec 101 -- sh -lc "cd /opt/homelab-media-stack/infrastructure/wireguard-server && sh setup-gluetun-client.sh"
```

Set qBittorrent and Prowlarr proxy to `192.168.12.101:8888`.

## CT-900 Ollama + model share
- Proxmox mounts laptop model export at `/mnt/laptop-models`
- CT-900 bind-mounts `/mnt/laptop-models`
- Ollama service override sets `OLLAMA_MODELS=/mnt/laptop-models`

Verify:
```bash
pct exec 900 -- ollama list
pct exec 900 -- systemctl status ollama --no-pager
```
