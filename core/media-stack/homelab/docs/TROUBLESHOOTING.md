# Troubleshooting

## Seerr Login (2026-04-05)
Use local login: click **"Sign in with Seerr"** on the login page.
- Email: `seerr@local` | Password: `seerr`
- Jellyfin login returns 500 when Jellyfin is actively streaming (DbUpdateConcurrencyException). Use local login instead.
- Note: Prowlarr/Radarr SQLite DB locks periodically under heavy search load — restart the service if it stops responding.
  Fix: `pct exec <CT_ID> -- systemctl restart <service>` then `sync && echo 3 > /proc/sys/vm/drop_caches` on Proxmox host if still stuck.

Troubleshooting for the Tiamat Proxmox media stack (192.168.12.242).
All commands run via SSH: `ssh root@192.168.12.242`

## Container Won't Start
```bash
pct start <CTID>
pct config <CTID>       # check config
journalctl -u pve* -n 50  # Proxmox logs
```

## Service Not Responding
Check if the container is running and the service is up:
```bash
pct list                                  # all container status
pct exec <CTID> -- systemctl status <svc>  # service inside CT
pct exec <CTID> -- ss -tlnp               # listening ports
```

Common services:
- Sonarr (CT-214): `pct exec 214 -- systemctl status sonarr`
- Radarr (CT-215): `pct exec 215 -- systemctl status radarr`
- Prowlarr (CT-210): `pct exec 210 -- systemctl status prowlarr`
- qBittorrent (CT-212): `pct exec 212 -- systemctl status qbittorrent-nox`
- Jellyfin (CT-231): `pct exec 231 -- systemctl status jellyfin`

## VPN / Kill-Switch Issues
The download pipeline: qBit → TinyProxy (CT-101:8888) → WireGuard (CT-100) → internet

```bash
# 1. Check WireGuard tunnel
pct exec 100 -- wg show

# 2. Check TinyProxy is running
pct exec 101 -- ps aux | grep tinyproxy

# 3. Test VPN exit IP (should NOT be your home IP)
pct exec 101 -- curl -s https://api.ipify.org

# 4. Test proxy from qBit container
pct exec 212 -- curl -s -x http://192.168.12.101:8888 https://api.ipify.org

# 5. If NAT is broken on CT-100 (traffic not forwarding):
pct exec 100 -- sh -c 'echo 1 > /proc/sys/net/ipv4/ip_forward && iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o eth0 -j MASQUERADE'
pct exec 100 -- /etc/init.d/iptables save
```

## Prowlarr: No Search Results
- Check indexers: `http://192.168.12.210:9696` → Indexers → Test All
- Cloudflare-blocked indexers (1337x, EZTV, TorrentGalaxy) need FlareSolverr
- FlareSolverr status: `curl -s http://192.168.12.102:8191`
- Re-sync to Sonarr/Radarr: Prowlarr → Settings → Apps → Sync

## Downloads Stuck in qBittorrent
- Proxy down: check CT-101 TinyProxy (see VPN section above)
- Wrong category: verify `sonarr` → `/data/torrents/tv`, `radarr` → `/data/torrents/movies`
- Disk full: `df -h /mnt/hdd` on Proxmox host

## Jellyfin/Plex Missing New Content
- Content lands in `/mnt/hdd/media/movies` or `/mnt/hdd/media/tv`
- Jellyfin: Dashboard → Scheduled Tasks → Scan All Libraries
- Plex: Settings → Libraries → Scan
- Check bind mounts: `pct config 231 | grep mp`

## Jellyseerr Requests Not Processing
- Verify Sonarr/Radarr connections: Jellyseerr → Settings → Services
- Check API keys match (Sonarr: `9e2127824e7446f6a2ddc5da67cfe693`, Radarr: `cc7485c9f5a64f78bfd226ffe23e2991`)

## Container Logs
```bash
# Live logs from inside a container
pct exec <CTID> -- journalctl -f -u <service>

# Proxmox task log
cat /var/log/pve/tasks/active
```

## TiVo ARP Poisoning (Fixed)
Static ARP entries added in `/etc/network/interfaces` for .231, .215, .230 to prevent TiVo (00:11:d9:b8:80:a7) from claiming media stack IPs.

## FlareSolverr Docker inside LXC (2026-04-06)
Chrome hangs at "Testing web browser installation" inside Docker-in-LXC without SYS_ADMIN cap.
Correct docker run command:
```
pct exec 102 -- docker rm -f flaresolverr
pct exec 102 -- docker run -d --name flaresolverr --restart unless-stopped \
  -p 8191:8191 -e LOG_LEVEL=info \
  --shm-size=2g --cap-add=SYS_ADMIN \
  ghcr.io/flaresolverr/flaresolverr:latest
```

## Radarr wrong IP in Prowlarr + Traefik (2026-04-06)
Radarr moved from 192.168.12.215 to 192.168.12.225 (HDHR conflict).
Both Traefik dynamic/download-stack.yml and Prowlarr Applications must use .225.
Already fixed via API. If it recurs:
```
# Traefik (CT-103)
sed -i "s|192.168.12.215:7878|192.168.12.225:7878|g" /etc/traefik/dynamic/download-stack.yml
# Prowlarr — update via Settings > Apps > Radarr > baseUrl to http://192.168.12.225:7878
```
