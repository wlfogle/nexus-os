# Tiamat Warp Agent — Fix Instructions

Run all commands on Tiamat (192.168.12.242) as root.

## Issue 1: CT-217 Readarr — Port 8787 Unreachable from Host

Readarr is running inside the container (localhost:8787 returns HTTP 200), but the Proxmox host gets "Connection refused." The .NET runtime is likely binding to IPv6-only despite `BindAddress=*`.

**Fix:**

```bash
# 1. Check what Readarr is actually binding to
pct exec 217 -- ss -tlnp | grep 8787

# 2. Force IPv4 bind in config
pct exec 217 -- systemctl stop readarr
pct exec 217 -- sed -i 's|<BindAddress>\*</BindAddress>|<BindAddress>0.0.0.0</BindAddress>|' /var/lib/readarr/config.xml
pct exec 217 -- systemctl start readarr
sleep 5

# 3. Verify from host
curl -s --max-time 5 http://192.168.12.217:8787/api/v1/system/status \
  -H "X-Api-Key: 19566aa7fb90487ebd2c643ad8c6595d" | python3 -m json.tool
```

If that still fails, check if the issue is the veth bridge:

```bash
# From inside the container, test binding explicitly
pct exec 217 -- ss -tlnp | grep 8787
# If it shows [::]:8787 only (no 0.0.0.0:8787), the config change didn't take
pct exec 217 -- cat /var/lib/readarr/config.xml | grep BindAddress
```

---

## Issue 2: Configure Readarr — Root Folder + Download Client

Once port 8787 is reachable:

```bash
READARR_KEY=19566aa7fb90487ebd2c643ad8c6595d
READARR=http://192.168.12.217:8787

# Add root folder for books
curl -s -X POST "$READARR/api/v1/rootfolder" \
  -H "X-Api-Key: $READARR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"path":"/data/media/books","name":"Books","defaultQualityProfileId":1,"defaultMetadataProfileId":1,"isCalibreLibrary":false}'

# Add qBittorrent as download client
# NOTE: Replace the qBittorrent password if it was changed from the default
curl -s -X POST "$READARR/api/v1/downloadclient" \
  -H "X-Api-Key: $READARR_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"qBittorrent",
    "implementation":"QBittorrent",
    "configContract":"QBittorrentSettings",
    "enable":true,
    "protocol":"torrent",
    "priority":1,
    "fields":[
      {"name":"host","value":"192.168.12.212"},
      {"name":"port","value":8080},
      {"name":"username","value":"admin"},
      {"name":"password","value":"adminadmin"},
      {"name":"bookCategory","value":"readarr"},
      {"name":"useSsl","value":false}
    ]
  }'
```

---

## Issue 3: Connect Readarr to Prowlarr

```bash
PROWLARR_KEY=6719026a4a5042a99897597122fa4495

# Add Readarr as a Prowlarr application
curl -s -X POST "http://192.168.12.210:9696/api/v1/applications" \
  -H "X-Api-Key: $PROWLARR_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"Readarr",
    "implementation":"Readarr",
    "configContract":"ReadarrSettings",
    "syncLevel":"fullSync",
    "tags":[],
    "fields":[
      {"name":"prowlarrUrl","value":"http://192.168.12.210:9696"},
      {"name":"baseUrl","value":"http://192.168.12.217:8787"},
      {"name":"apiKey","value":"19566aa7fb90487ebd2c643ad8c6595d"},
      {"name":"syncCategories","value":[7000,7010,7020,7030,7040,7050,7060]}
    ]
  }'

# Trigger index sync
curl -s -X POST "http://192.168.12.210:9696/api/v1/command" \
  -H "X-Api-Key: $PROWLARR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name":"AppIndexerSync"}'
```

---

## Issue 4: FlareSolverr Can't Solve Cloudflare

FlareSolverr v3.3.21 times out on all Cloudflare-protected sites (1337x, EZTV, TorrentGalaxy). The indexers are added to Prowlarr and enabled, but won't return results until this is fixed.

### Option A — Upgrade FlareSolverr to Latest (try first)

```bash
pct exec 102 -- bash -c "
  systemctl stop flaresolverr 2>/dev/null
  pkill -f flaresolverr 2>/dev/null

  # Check current version
  /opt/flaresolverr/flaresolverr --version 2>/dev/null || echo 'no version flag'

  # Re-download latest
  curl -sL https://github.com/FlareSolverr/FlareSolverr/releases/latest/download/flaresolverr_linux_x64.tar.gz -o /tmp/fs.tar.gz
  tar -xzf /tmp/fs.tar.gz -C /opt/flaresolverr --strip-components=1
  rm /tmp/fs.tar.gz

  # Restart
  systemctl start flaresolverr 2>/dev/null || /opt/flaresolverr/flaresolverr &
  sleep 5
"

# Test
curl -s --max-time 90 -X POST http://192.168.12.102:8191/v1 \
  -H "Content-Type: application/json" \
  -d '{"cmd":"request.get","url":"https://1337x.to","maxTimeout":60000}' \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['status'], d.get('message','ok')[:100])"
```

### Option B — Switch to FlareSolverr Docker (if native upgrade fails)

```bash
# Install Docker in CT-102 if not present
pct exec 102 -- bash -c "
  which docker || (apt-get update -qq && apt-get install -y -qq curl && curl -fsSL https://get.docker.com | sh)
"

# Stop native FlareSolverr, run Docker version
pct exec 102 -- bash -c "
  pkill -f flaresolverr 2>/dev/null
  systemctl disable flaresolverr 2>/dev/null
  docker run -d --name flaresolverr --restart unless-stopped \
    -p 8191:8191 \
    -e LOG_LEVEL=info \
    ghcr.io/flaresolverr/flaresolverr:latest
"
```

### Option C — Jackett Fallback

If Cloudflare is unbeatable, deploy CT-211 Jackett as a secondary indexer manager and add the same sites there.

---

## Issue 5: Deploy CT-218 Lidarr (Music)

```bash
TEMPLATE="local:vztmpl/debian-12-standard_12.12-1_amd64.tar.zst"

pct create 218 "$TEMPLATE" \
  --hostname lidarr \
  --memory 1024 --cores 2 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.218/24,gw=192.168.12.1,type=veth \
  --storage local-lvm --rootfs local-lvm:8 \
  --unprivileged 1 --features nesting=1 \
  --swap 512 --onboot 1

echo "mp0: /mnt/hdd,mp=/data" >> /etc/pve/lxc/218.conf

pct start 218
sleep 5

pct exec 218 -- bash -c "
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq curl sqlite3 mediainfo libchromaprint-tools >/dev/null 2>&1

  useradd -r -s /usr/sbin/nologin -d /var/lib/lidarr lidarr 2>/dev/null || true
  mkdir -p /var/lib/lidarr /opt/Lidarr

  curl -sL 'https://lidarr.servarr.com/v1/update/master/updatefile?os=linux&runtime=netcore&arch=x64' -o /tmp/lidarr.tar.gz
  tar -xzf /tmp/lidarr.tar.gz -C /opt/Lidarr --strip-components=1
  rm /tmp/lidarr.tar.gz

  chown -R lidarr:lidarr /opt/Lidarr /var/lib/lidarr

  cat > /etc/systemd/system/lidarr.service <<EOF
[Unit]
Description=Lidarr Daemon
After=syslog.target network.target
[Service]
User=lidarr
Group=lidarr
Type=simple
ExecStart=/opt/Lidarr/Lidarr -nobrowser -data=/var/lib/lidarr
TimeoutStopSec=20
KillMode=process
Restart=on-failure
[Install]
WantedBy=multi-user.target
EOF

  systemctl daemon-reload
  systemctl enable --now lidarr
"

sleep 10

# Fix bind address (same IPv6 issue as Readarr)
pct exec 218 -- systemctl stop lidarr
pct exec 218 -- sed -i 's|<BindAddress>\*</BindAddress>|<BindAddress>0.0.0.0</BindAddress>|' /var/lib/lidarr/config.xml
pct exec 218 -- systemctl start lidarr
sleep 5

# Verify
curl -s --max-time 5 http://192.168.12.218:8686 -o /dev/null -w "HTTP:%{http_code}\n"
```

### Configure Lidarr

```bash
LIDARR_KEY=$(pct exec 218 -- grep -oP '(?<=<ApiKey>)[^<]+' /var/lib/lidarr/config.xml)
LIDARR=http://192.168.12.218:8686

# Add root folder
curl -s -X POST "$LIDARR/api/v1/rootfolder" \
  -H "X-Api-Key: $LIDARR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"path":"/data/media/music","name":"Music","defaultQualityProfileId":1,"defaultMetadataProfileId":1}'

# Add qBittorrent
curl -s -X POST "$LIDARR/api/v1/downloadclient" \
  -H "X-Api-Key: $LIDARR_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"qBittorrent",
    "implementation":"QBittorrent",
    "configContract":"QBittorrentSettings",
    "enable":true,
    "protocol":"torrent",
    "priority":1,
    "fields":[
      {"name":"host","value":"192.168.12.212"},
      {"name":"port","value":8080},
      {"name":"username","value":"admin"},
      {"name":"password","value":"adminadmin"},
      {"name":"musicCategory","value":"lidarr"},
      {"name":"useSsl","value":false}
    ]
  }'

# Add to Prowlarr
PROWLARR_KEY=6719026a4a5042a99897597122fa4495
curl -s -X POST "http://192.168.12.210:9696/api/v1/applications" \
  -H "X-Api-Key: $PROWLARR_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\":\"Lidarr\",
    \"implementation\":\"Lidarr\",
    \"configContract\":\"LidarrSettings\",
    \"syncLevel\":\"fullSync\",
    \"tags\":[],
    \"fields\":[
      {\"name\":\"prowlarrUrl\",\"value\":\"http://192.168.12.210:9696\"},
      {\"name\":\"baseUrl\",\"value\":\"http://192.168.12.218:8686\"},
      {\"name\":\"apiKey\",\"value\":\"$LIDARR_KEY\"},
      {\"name\":\"syncCategories\",\"value\":[3000,3010,3020,3030,3040]}
    ]
  }"

# Sync indexers
curl -s -X POST "http://192.168.12.210:9696/api/v1/command" \
  -H "X-Api-Key: $PROWLARR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name":"AppIndexerSync"}'
```

---

## Issue 6: Deploy CT-232 Audiobookshelf

```bash
TEMPLATE="local:vztmpl/debian-12-standard_12.12-1_amd64.tar.zst"

pct create 232 "$TEMPLATE" \
  --hostname audiobookshelf \
  --memory 1024 --cores 2 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.232/24,gw=192.168.12.1,type=veth \
  --storage local-lvm --rootfs local-lvm:8 \
  --unprivileged 1 --features nesting=1 \
  --swap 512 --onboot 1

echo "mp0: /mnt/hdd/media/audiobooks,mp=/audiobooks" >> /etc/pve/lxc/232.conf

pct start 232
sleep 5

pct exec 232 -- bash -c "
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq curl ca-certificates >/dev/null 2>&1

  # Install Docker
  curl -fsSL https://get.docker.com | sh >/dev/null 2>&1

  mkdir -p /opt/appdata/audiobookshelf/config /opt/appdata/audiobookshelf/metadata

  docker run -d \
    --name audiobookshelf \
    --restart unless-stopped \
    -p 13378:80 \
    -v /opt/appdata/audiobookshelf/config:/config \
    -v /opt/appdata/audiobookshelf/metadata:/metadata \
    -v /audiobooks:/audiobooks \
    ghcr.io/advplyr/audiobookshelf:latest
"

sleep 10
curl -s --max-time 5 http://192.168.12.232:13378 -o /dev/null -w "Audiobookshelf HTTP:%{http_code}\n"
```

---

## Issue 7: Deploy CT-233 Calibre-Web

Calibre-Web needs the laptop's Calibre library via NFS.

### Step 1: Set up NFS mount from laptop

```bash
# On Tiamat host, add NFS mount if not already present
mkdir -p /mnt/laptop/calibre
grep -q 'laptop/calibre' /etc/fstab || echo "192.168.12.172:/home/loufogle/Calibre Library /mnt/laptop/calibre nfs4 ro,soft,timeo=50,_netdev 0 0" >> /etc/fstab
mount /mnt/laptop/calibre 2>/dev/null || echo "NFS mount failed — verify laptop NFS export is configured"
```

> **NOTE:** Adjust the NFS export path to match where the Calibre library actually lives on the laptop. The laptop must have an NFS export configured for this path.

### Step 2: Create and deploy container

```bash
TEMPLATE="local:vztmpl/debian-12-standard_12.12-1_amd64.tar.zst"

pct create 233 "$TEMPLATE" \
  --hostname calibre-web \
  --memory 1024 --cores 1 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.233/24,gw=192.168.12.1,type=veth \
  --storage local-lvm --rootfs local-lvm:8 \
  --unprivileged 1 --features nesting=1 \
  --swap 512 --onboot 1

# Bind mount both the HDD books dir and the NFS calibre library
cat >> /etc/pve/lxc/233.conf <<EOF
mp0: /mnt/hdd/media/books,mp=/books
mp1: /mnt/laptop/calibre,mp=/calibre
EOF

pct start 233
sleep 5

pct exec 233 -- bash -c "
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq curl ca-certificates >/dev/null 2>&1

  # Install Docker
  curl -fsSL https://get.docker.com | sh >/dev/null 2>&1

  mkdir -p /opt/appdata/calibre-web

  docker run -d \
    --name calibre-web \
    --restart unless-stopped \
    -p 8083:8083 \
    -e PUID=1000 \
    -e PGID=1000 \
    -e TZ=America/New_York \
    -v /opt/appdata/calibre-web:/config \
    -v /calibre:/books:ro \
    lscr.io/linuxserver/calibre-web:latest
"

sleep 10
curl -s --max-time 5 http://192.168.12.233:8083 -o /dev/null -w "Calibre-Web HTTP:%{http_code}\n"
```

### Calibre-Web First Login
- URL: `http://192.168.12.233:8083`
- Default credentials: `admin` / `admin123`
- Set database location to `/books/metadata.db`

---

## Issue 8: Add qBittorrent Download Categories

```bash
# Add categories for new services
# Open qBit UI or use the API:
QBIT=http://192.168.12.212:8080

# Login (get SID cookie)
SID=$(curl -s -c - "$QBIT/api/v2/auth/login" -d "username=admin&password=adminadmin" | grep -oP 'SID\s+\K\S+')

# Add readarr category
curl -s -b "SID=$SID" "$QBIT/api/v2/torrents/createCategory" \
  -d "category=readarr&savePath=/data/torrents/books"

# Add lidarr category
curl -s -b "SID=$SID" "$QBIT/api/v2/torrents/createCategory" \
  -d "category=lidarr&savePath=/data/torrents/music"

echo "Categories added"
```

---

## Verification Checklist

After completing all fixes, run this to verify everything:

```bash
echo "=== Service Status ==="
for svc in "217:8787:Readarr" "218:8686:Lidarr" "232:13378:Audiobookshelf" "233:8083:Calibre-Web"; do
  CT=$(echo $svc | cut -d: -f1)
  PORT=$(echo $svc | cut -d: -f2)
  NAME=$(echo $svc | cut -d: -f3)
  IP="192.168.12.$CT"
  HTTP=$(curl -s --max-time 5 "http://$IP:$PORT" -o /dev/null -w "%{http_code}")
  echo "  CT-$CT $NAME ($IP:$PORT) = HTTP $HTTP"
done

echo ""
echo "=== Prowlarr Apps ==="
PROWLARR_KEY=6719026a4a5042a99897597122fa4495
curl -s "http://192.168.12.210:9696/api/v1/applications" \
  -H "X-Api-Key: $PROWLARR_KEY" \
  | python3 -c "import sys,json; [print('  '+a['name']+' ('+a['implementation']+')') for a in json.load(sys.stdin)]"

echo ""
echo "=== Prowlarr Indexers ==="
curl -s "http://192.168.12.210:9696/api/v1/indexer" \
  -H "X-Api-Key: $PROWLARR_KEY" \
  | python3 -c "import sys,json; [print('  '+str(i['id'])+': '+i['name']+' enabled='+str(i['enable'])) for i in json.load(sys.stdin)]"

echo ""
echo "=== FlareSolverr ==="
curl -s --max-time 5 -X POST http://192.168.12.102:8191/v1 \
  -H "Content-Type: application/json" \
  -d '{"cmd":"sessions.list"}' \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print('  Status:', d.get('status'), 'Version:', d.get('version'))"

echo ""
echo "=== Container List ==="
pct list
```
