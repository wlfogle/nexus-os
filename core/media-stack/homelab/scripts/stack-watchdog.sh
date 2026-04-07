#!/bin/bash
# =============================================================================
# stack-watchdog.sh — timeout-safe self-healing watchdog for Tiamat media stack
# =============================================================================
LOG=/var/log/stack-watchdog.log
exec >> "$LOG" 2>&1
echo "--- watchdog run $(date) ---"

ct_running() { pct status "$1" 2>/dev/null | grep -q running; }
http_ok() {
  CODE=$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 "$1" 2>/dev/null)
  [ "$CODE" = "200" ] || [ "$CODE" = "301" ] || [ "$CODE" = "302" ] || [ "$CODE" = "307" ] || [ "$CODE" = "401" ]
}
run_pct() {
  local secs=$1
  shift
  timeout "$secs" pct exec "$@" 2>/dev/null
}
ensure_ct_running() {
  local id=$1
  local name=$2
  if ! ct_running "$id"; then
    echo "[CRIT] CT-$id ($name) not running — starting"
    timeout 20 pct start "$id" 2>/dev/null || true
    sleep 10
  fi
}

# CT-100 WireGuard server
ensure_ct_running 100 wireguard
run_pct 20 100 -- sh -c '
  echo 1 > /proc/sys/net/ipv4/ip_forward
  iptables -t nat -C POSTROUTING -s 10.0.0.0/24 -o eth0 -j MASQUERADE 2>/dev/null || \
    iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o eth0 -j MASQUERADE
  wg show wg0 >/dev/null 2>&1 || { wg-quick down wg0 2>/dev/null; sleep 1; wg-quick up wg0; }
'

# CT-101 WireGuard client + TinyProxy
ensure_ct_running 101 wg-proxy
run_pct 20 101 -- sh -c '
  wg show wg0 >/dev/null 2>&1 || { wg-quick down wg0 2>/dev/null; sleep 1; wg-quick up wg0; }
  pgrep tinyproxy >/dev/null || { rc-service tinyproxy restart 2>/dev/null || tinyproxy; sleep 2; }
'
PROXY_OK=$(run_pct 12 101 -- sh -c 'timeout 8 curl -s -x http://127.0.0.1:8888 https://icanhazip.com 2>/dev/null' | tr -d '[:space:]')
[ -n "$PROXY_OK" ] && echo "[OK] proxy $PROXY_OK" || echo "[WARN] proxy unavailable"

# CT-102 FlareSolverr
ensure_ct_running 102 flaresolverr
if ! http_ok "http://192.168.12.102:8191"; then
  echo "[WARN] FlareSolverr down"
  run_pct 20 102 -- systemctl restart flaresolverr || true
fi

# CT-212 qBittorrent
ensure_ct_running 212 qbittorrent
if ! http_ok "http://192.168.12.212:8080"; then
  echo "[WARN] qBit down — fixing"
  run_pct 25 212 -- bash -lc '
    pkill -f qbittorrent-nox 2>/dev/null || true
    find /var/lib/qbittorrent -name "*.lock" -delete 2>/dev/null || true
    systemctl restart qbittorrent.service 2>/dev/null || true
    sleep 8
    systemctl is-active qbittorrent.service >/dev/null 2>&1 || \
      runuser -u qbittorrent -- /usr/bin/qbittorrent-nox -d --webui-port=8080 --profile=/var/lib/qbittorrent 2>/dev/null || true
  '
fi

# CT-210 Prowlarr
ensure_ct_running 210 prowlarr
if ! http_ok "http://192.168.12.210:9696"; then
  echo "[WARN] Prowlarr down — restarting"
  run_pct 20 210 -- systemctl restart prowlarr || true
fi

# Arr apps
for PAIR in "214 8989 sonarr sonarr" "215 8989 radarr radarr" "217 8787 readarr readarr" "218 8686 lidarr lidarr"; do
  set -- $PAIR
  CT=$1
  PORT=$2
  SVC=$3
  NAME=$4
  IP="192.168.12.$CT"
  [ "$CT" = "215" ] && IP="192.168.12.225"
  ensure_ct_running "$CT" "$NAME"
  if ! http_ok "http://$IP:$PORT"; then
    echo "[WARN] $NAME down — restarting"
    run_pct 20 "$CT" -- systemctl restart "$SVC" || true
  fi
done

# Jellyfin
ensure_ct_running 231 jellyfin
if ! http_ok "http://192.168.12.231:8096"; then
  echo "[WARN] Jellyfin down — restarting"
  run_pct 25 231 -- systemctl restart jellyfin || true
fi

# Jellyseerr
ensure_ct_running 242 jellyseerr
if ! http_ok "http://192.168.12.151:5055"; then
  echo "[WARN] Jellyseerr down — restarting"
  run_pct 20 242 -- bash -lc 'systemctl restart jellyseerr 2>/dev/null || docker restart jellyseerr 2>/dev/null || true'
fi

echo "--- watchdog done $(date) ---"
