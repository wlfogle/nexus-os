#!/usr/bin/env bash

set -o pipefail

INTERVAL="${1:-5}"

SONARR_KEY="${SONARR_KEY:-9e2127824e7446f6a2ddc5da67cfe693}"
RADARR_KEY="${RADARR_KEY:-cc7485c9f5a64f78bfd226ffe23e2991}"
READARR_KEY="${READARR_KEY:-19566aa7fb90487ebd2c643ad8c6595d}"
PROWLARR_KEY="${PROWLARR_KEY:-6719026a4a5042a99897597122fa4495}"

http_code() {
  curl -s -o /dev/null -w '%{http_code}' --max-time 3 "$1" 2>/dev/null || printf '000'
}

json_value() {
  python3 -c "$1" 2>/dev/null
}

while true; do
  clear
  printf 'Media Stack Monitor  %s\n\n' "$(date '+%Y-%m-%d %H:%M:%S')"

  QBIT_CODE="$(http_code 'http://192.168.12.212:8080')"
  PROWLARR_CODE="$(http_code 'http://192.168.12.210:9696')"
  FLARE_CODE="$(http_code 'http://192.168.12.102:8191')"
  SONARR_CODE="$(http_code 'http://192.168.12.214:8989')"
  RADARR_CODE="$(http_code 'http://192.168.12.225:7878')"
  READARR_CODE="$(http_code 'http://192.168.12.217:8787')"
  LIDARR_CODE="$(http_code 'http://192.168.12.218:8686')"
  JELLYFIN_CODE="$(http_code 'http://192.168.12.231:8096')"

  VPN_IP="$(curl -s --max-time 4 -x http://192.168.12.101:8888 https://icanhazip.com 2>/dev/null | tr -d '\r')"
  [ -z "$VPN_IP" ] && VPN_IP="DOWN"

  printf 'Services\n'
  printf '  qBittorrent : %s\n' "$QBIT_CODE"
  printf '  Prowlarr    : %s\n' "$PROWLARR_CODE"
  printf '  FlareSolverr: %s\n' "$FLARE_CODE"
  printf '  Sonarr      : %s\n' "$SONARR_CODE"
  printf '  Radarr      : %s\n' "$RADARR_CODE"
  printf '  Readarr     : %s\n' "$READARR_CODE"
  printf '  Lidarr      : %s\n' "$LIDARR_CODE"
  printf '  Jellyfin    : %s\n' "$JELLYFIN_CODE"
  printf '  VPN Proxy   : %s\n\n' "$VPN_IP"

  printf 'Queues\n'

  SONARR_QUEUE="$(curl -s --max-time 4 'http://192.168.12.214:8989/api/v3/queue?page=1&pageSize=50' -H "X-Api-Key: $SONARR_KEY" | json_value 'import sys,json; d=json.load(sys.stdin); print(d.get("totalRecords", "?"))' || printf '?')"
  RADARR_QUEUE="$(curl -s --max-time 4 'http://192.168.12.225:7878/api/v3/queue?page=1&pageSize=50' -H "X-Api-Key: $RADARR_KEY" | json_value 'import sys,json; d=json.load(sys.stdin); print(d.get("totalRecords", "?"))' || printf '?')"
  READARR_QUEUE="$(curl -s --max-time 4 'http://192.168.12.217:8787/api/v1/queue?page=1&pageSize=50' -H "X-Api-Key: $READARR_KEY" | json_value 'import sys,json; d=json.load(sys.stdin); print(d.get("totalRecords", "?"))' || printf '?')"

  printf '  Sonarr queue : %s\n' "$SONARR_QUEUE"
  printf '  Radarr queue : %s\n' "$RADARR_QUEUE"
  printf '  Readarr queue: %s\n\n' "$READARR_QUEUE"

  printf 'Prowlarr health\n'
  curl -s --max-time 4 'http://192.168.12.210:9696/api/v1/health' -H "X-Api-Key: $PROWLARR_KEY" | \
    python3 -c 'import sys,json
d=json.load(sys.stdin)
if not d:
    print("  OK")
else:
    for item in d[:8]:
        print(f"  {item.get(\"type\",\"?\")}: {item.get(\"message\",\"?\")}")' 2>/dev/null || printf '  unavailable\n'
  printf '\n'

  printf 'Recent Sonarr queue items\n'
  curl -s --max-time 4 'http://192.168.12.214:8989/api/v3/queue?page=1&pageSize=5' -H "X-Api-Key: $SONARR_KEY" | \
    python3 -c 'import sys,json
d=json.load(sys.stdin)
recs=d.get("records",[])
if not recs:
    print("  none")
else:
    for r in recs:
        print(f"  {r.get(\"title\",\"?\")[:70]} | {r.get(\"trackedDownloadState\",\"?\")} | {r.get(\"trackedDownloadStatus\",\"?\")}")' 2>/dev/null || printf '  unavailable\n'
  printf '\n'

  printf 'Recent Radarr queue items\n'
  curl -s --max-time 4 'http://192.168.12.225:7878/api/v3/queue?page=1&pageSize=5' -H "X-Api-Key: $RADARR_KEY" | \
    python3 -c 'import sys,json
d=json.load(sys.stdin)
recs=d.get("records",[])
if not recs:
    print("  none")
else:
    for r in recs:
        print(f"  {r.get(\"title\",\"?\")[:70]} | {r.get(\"trackedDownloadState\",\"?\")} | {r.get(\"trackedDownloadStatus\",\"?\")}")' 2>/dev/null || printf '  unavailable\n'

  sleep "$INTERVAL"
done
