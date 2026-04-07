# Sonarr / Radarr Indexer Setup Guide

Configure Prowlarr as the central indexer manager, then sync to Sonarr and Radarr.
All indexers below are torrent-based and routed through the WireGuard VPN (CT-101 proxy).

---

## Step 1 — Connect Prowlarr to Sonarr & Radarr

1. Open Prowlarr → **Settings → Apps**
2. Add **Sonarr**: `http://192.168.12.214:8989` | API Key from Sonarr → Settings → General
3. Add **Radarr**: `http://192.168.12.215:7878` | API Key from Radarr → Settings → General
4. Click **Sync App Indexers** — Prowlarr will push all configured indexers automatically

---

## Step 2 — Add Indexers in Prowlarr

Go to **Indexers → Add Indexer** and search for each one below.

### Tier 1 — Most Reliable (add all of these)

#### EZTV
- **Type**: Torrent (public)
- **Best for**: TV shows — very comprehensive, updated frequently
- **URL**: https://eztv.re
- **Add in Prowlarr**: search "EZTV"

#### TorrentGalaxy (TGx)
- **Type**: Torrent (public)
- **Best for**: Movies and TV, excellent quality tagging
- **URL**: https://torrentgalaxy.to
- **Add in Prowlarr**: search "TorrentGalaxy"

#### 1337x
- **Type**: Torrent (public)
- **Best for**: Movies, TV, general content — large library
- **URL**: https://1337x.to
- **Add in Prowlarr**: search "1337x"

#### The Pirate Bay (TPB)
- **Type**: Torrent (public)
- **Best for**: Backup / fallback — massive library
- **Add in Prowlarr**: search "The Pirate Bay"

#### Nyaa (Anime)
- **Type**: Torrent (public)
- **Best for**: Anime — the definitive anime torrent source
- **URL**: https://nyaa.si
- **Add in Prowlarr**: search "Nyaa"

---

### Tier 2 — Good Secondary Sources

| Indexer         | Best for              | Notes                              |
|----------------|-----------------------|------------------------------------|
| RARBG (mirror) | Movies, TV            | RARBG shutdown 2023; mirrors exist |
| YTS             | Movies (x264/x265)    | Small file sizes, high quality     |
| Zooqle          | TV, Movies            | Good for older content             |
| LimeTorrents    | General               | Backup source                      |
| Kickass (KAT)   | General               | Mirror available                   |

---

## Step 3 — Radarr Quality Profiles

Go to **Radarr → Settings → Quality Profiles**

### Recommended: "HD-1080p"
- Allowed: Bluray-1080p, WEB-1080p, HDTV-1080p
- Cutoff: WEB-1080p
- **Disable**: SDTV, DVD, Bluray-480p (these are low quality)

### For space savings: "HD-720p"
- Allowed: Bluray-720p, WEB-720p, HDTV-720p
- Good choice for large libraries on the 2TB HDD

---

## Step 4 — Sonarr Quality Profiles

Go to **Sonarr → Settings → Quality Profiles**

### Recommended: "HD-1080p"
- Same as Radarr above
- Enable **Season Packs** if you prefer full seasons over individual episodes

### Anime profile
- Cutoff: WEB-1080p
- Preferred word: `[SubsPlease]` or `[Erai-raws]` (best English sub groups)

---

## Step 5 — Download Client (qBittorrent)

### In Prowlarr/Sonarr/Radarr → Settings → Download Clients

- **Host**: `192.168.12.212`
- **Port**: `8080`
- **Username**: `admin`
- **Password**: (set in qBittorrent WebUI on first run)
- **Category**: `sonarr` or `radarr` (keeps downloads organized)

### qBittorrent VPN kill-switch

qBittorrent in CT-212 is configured to proxy all traffic through CT-101 (TinyProxy → WireGuard):

```
Settings → Connection → Proxy Server
Type: HTTP
Host: 192.168.12.101
Port: 8888
```

> **Note**: If CT-101 goes down, qBittorrent will fail to connect — this is the kill-switch working correctly.

---

## Step 6 — Indexer Flags to Watch For

When browsing releases in Sonarr/Radarr, prefer releases with:

| Flag         | Meaning                              | Prefer? |
|-------------|--------------------------------------|---------|
| `PROPER`    | Re-release fixing encoding issues    | Yes     |
| `REPACK`    | Re-release from same group           | Yes     |
| `REMUX`     | Untouched Blu-ray stream             | Yes (if space allows) |
| `HDR`       | High dynamic range                   | Yes (Fire TV supports it) |
| `DV`        | Dolby Vision                         | Optional |
| `EXTENDED`  | Extended cut                         | Your choice |
| `CAM`       | Filmed in cinema — terrible quality  | **Never** |
| `TS`        | Telesync — poor quality              | **Never** |

---

## Troubleshooting

**Prowlarr shows "No Results"**
- Check indexer is reachable: Prowlarr → Indexers → Test
- Some indexers block VPN exit IPs — try a different VPN endpoint in CT-100

**Sonarr/Radarr not receiving indexers from Prowlarr**
- Verify API keys match (Sonarr/Radarr → Settings → General → API Key)
- Re-run "Sync App Indexers" in Prowlarr

**Downloads stuck in qBittorrent**
- Check CT-101 proxy is running: `pct exec 101 -- ps aux | grep tinyproxy`
- Check WireGuard tunnel: `pct exec 100 -- wg show`

**Releases grabbed but wrong quality**
- Lower cutoff in quality profile
- Check "Preferred Words" in Sonarr/Radarr profiles aren't filtering too aggressively
