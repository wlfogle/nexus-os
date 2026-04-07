#!/bin/bash
# ============================================================
# Phase 5 — Deploy Media Management Services (CT-240–CT-250)
# Run on Proxmox host (Tiamat) as root
# Creates individual LXCs, installs Docker, starts services
#
# ⚠ RAM WARNING: Tiamat has 8GB total. This creates 11 CTs.
#   Use --skip to selectively deploy. Priority order:
#   1. bazarr (subtitles) + overseerr/jellyseerr (requests)
#   2. tautulli (analytics) + recyclarr/kometa (quality)
#   3. Everything else
# ============================================================
set -e

TEMPLATE="local:vztmpl/debian-12-standard_12.12-1_amd64.tar.zst"
STORAGE="local-lvm"
TZ="${TZ:-America/New_York}"
PUID="${PUID:-1000}"
PGID="${PGID:-1000}"

# Skip list — add CT IDs to skip (e.g., SKIP="243 246 247")
SKIP="${SKIP:-}"

# ── Helper Functions ─────────────────────────────────────────

should_skip() {
  local ctid="$1"
  for s in $SKIP; do
    [ "$s" = "$ctid" ] && return 0
  done
  return 1
}

create_ct() {
  local ctid="$1" hostname="$2" ram="$3" disk="$4"
  local extra_conf="${5:-}"

  if should_skip "$ctid"; then
    echo "==> Skipping CT-${ctid} (${hostname}) — in SKIP list"
    return 1
  fi

  if pct status "$ctid" &>/dev/null; then
    echo "==> CT-${ctid} already exists, skipping creation"
    return 0
  fi

  echo "==> Creating CT-${ctid} (${hostname})..."
  pct create "$ctid" "$TEMPLATE" \
    --hostname "$hostname" \
    --memory "$ram" --cores 1 \
    --net0 name=eth0,bridge=vmbr0,ip=dhcp,type=veth \
    --storage "$STORAGE" --rootfs "${STORAGE}:${disk}" \
    --unprivileged 1 --features nesting=1 \
    --onboot 1

  # Add bind mount for services that need media access
  if [ -n "$extra_conf" ]; then
    echo "$extra_conf" >> "/etc/pve/lxc/${ctid}.conf"
  fi

  return 0
}

setup_docker() {
  local ctid="$1"
  echo "    Installing Docker in CT-${ctid}..."
  pct start "$ctid" 2>/dev/null || true
  sleep 5
  pct exec "$ctid" -- bash -c "
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq >/dev/null 2>&1
    apt-get install -y -qq curl ca-certificates >/dev/null 2>&1
    curl -fsSL https://get.docker.com | sh >/dev/null 2>&1
  "
}

run_container() {
  local ctid="$1" name="$2" image="$3" ports="$4" envs="$5" volumes="$6"

  local port_args=""
  if [ -n "$ports" ]; then
    for p in $ports; do
      port_args="$port_args -p $p"
    done
  fi

  local env_args="-e PUID=$PUID -e PGID=$PGID -e TZ=$TZ"
  if [ -n "$envs" ]; then
    for e in $envs; do
      env_args="$env_args -e $e"
    done
  fi

  local vol_args=""
  if [ -n "$volumes" ]; then
    for v in $volumes; do
      vol_args="$vol_args -v $v"
    done
  fi

  echo "    Starting ${name}..."
  pct exec "$ctid" -- bash -c "
    mkdir -p /opt/appdata/${name}
    docker run -d \
      --name ${name} \
      --restart unless-stopped \
      ${port_args} \
      ${env_args} \
      -v /opt/appdata/${name}:/config \
      ${vol_args} \
      ${image}
  "
}

# ── CT-240: Bazarr (subtitles) ──────────────────────────────
echo ""
echo "═══ Phase 5: Media Management Services ═══"
echo ""

if create_ct 240 bazarr 512 8 "mp0: /mnt/hdd/media,mp=/data/media"; then
  setup_docker 240
  run_container 240 bazarr "lscr.io/linuxserver/bazarr:latest" \
    "6767:6767" \
    "" \
    "/data/media/movies:/movies /data/media/tv:/tv"
fi

# ── CT-241: Overseerr (Plex requests) ───────────────────────
if create_ct 241 overseerr 512 8; then
  setup_docker 241
  echo "    Starting overseerr..."
  pct exec 241 -- bash -c "
    mkdir -p /opt/appdata/overseerr
    docker run -d \
      --name overseerr \
      --restart unless-stopped \
      -p 5055:5055 \
      -e LOG_LEVEL=debug \
      -e TZ=$TZ \
      -e PORT=5055 \
      -v /opt/appdata/overseerr:/app/config \
      sctx/overseerr:latest
  "
fi

# ── CT-242: Jellyseerr (Jellyfin requests) ──────────────────
if create_ct 242 jellyseerr 512 8; then
  setup_docker 242
  echo "    Starting jellyseerr..."
  pct exec 242 -- bash -c "
    mkdir -p /opt/appdata/jellyseerr
    docker run -d \
      --name jellyseerr \
      --restart unless-stopped \
      -p 5055:5055 \
      -e LOG_LEVEL=debug \
      -e TZ=$TZ \
      -v /opt/appdata/jellyseerr:/app/config \
      fallenbagel/jellyseerr:latest
  "
fi

# ── CT-243: Ombi (requests — legacy fallback) ───────────────
if create_ct 243 ombi 512 8; then
  setup_docker 243
  run_container 243 ombi "lscr.io/linuxserver/ombi:latest" \
    "3579:3579" "" ""
fi

# ── CT-244: Tautulli (Plex analytics) ───────────────────────
if create_ct 244 tautulli 512 8; then
  setup_docker 244
  run_container 244 tautulli "ghcr.io/tautulli/tautulli:latest" \
    "8181:8181" "" ""
fi

# ── CT-245: Kometa (Plex Meta Manager) ──────────────────────
if create_ct 245 kometa 512 8; then
  setup_docker 245
  echo "    Starting kometa..."
  pct exec 245 -- bash -c "
    mkdir -p /opt/appdata/kometa
    docker run -d \
      --name kometa \
      --restart unless-stopped \
      -e TZ=$TZ \
      -e KOMETA_RUN=true \
      -e KOMETA_TIMES=03:00 \
      -v /opt/appdata/kometa:/config \
      kometateam/kometa:latest
  "
fi

# ── CT-246: Gaps (collection finder) ────────────────────────
if create_ct 246 gaps 256 8; then
  setup_docker 246
  echo "    Starting gaps..."
  pct exec 246 -- bash -c "
    mkdir -p /opt/appdata/gaps
    docker run -d \
      --name gaps \
      --restart unless-stopped \
      -p 8484:8484 \
      -e TZ=$TZ \
      -v /opt/appdata/gaps:/usr/data \
      housewrecker/gaps:latest
  "
fi

# ── CT-247: Janitorr (media cleanup) ───────────────────────
if create_ct 247 janitorr 256 8; then
  setup_docker 247
  echo "    Starting janitorr..."
  pct exec 247 -- bash -c "
    mkdir -p /opt/appdata/janitorr
    docker run -d \
      --name janitorr \
      --restart unless-stopped \
      -e TZ=$TZ \
      -v /opt/appdata/janitorr:/config \
      schizo99/janitorr:latest
  "
fi

# ── CT-248: Decluttarr (queue cleanup) ──────────────────────
if create_ct 248 decluttarr 256 8; then
  setup_docker 248
  echo "    Starting decluttarr..."
  pct exec 248 -- bash -c "
    mkdir -p /opt/appdata/decluttarr
    docker run -d \
      --name decluttarr \
      --restart unless-stopped \
      -e TZ=$TZ \
      -e PUID=$PUID \
      -e PGID=$PGID \
      -v /opt/appdata/decluttarr:/config \
      ghcr.io/manimatter/decluttarr:latest
  "
fi

# ── CT-249: Watchlistarr ────────────────────────────────────
if create_ct 249 watchlistarr 256 8; then
  setup_docker 249
  echo "    Starting watchlistarr..."
  pct exec 249 -- bash -c "
    mkdir -p /opt/appdata/watchlistarr
    docker run -d \
      --name watchlistarr \
      --restart unless-stopped \
      -e TZ=$TZ \
      -v /opt/appdata/watchlistarr:/config \
      nylonee/watchlistarr:latest
  "
fi

# ── CT-250: Traktarr (Trakt.tv sync) ───────────────────────
if create_ct 250 traktarr 256 8; then
  setup_docker 250
  echo "    Starting traktarr..."
  pct exec 250 -- bash -c "
    mkdir -p /opt/appdata/traktarr
    docker run -d \
      --name traktarr \
      --restart unless-stopped \
      -e TZ=$TZ \
      -v /opt/appdata/traktarr:/config \
      cloudb0x/traktarr:latest
  "
fi

echo ""
echo "═══════════════════════════════════════════"
echo "  Phase 5 Deployment Complete"
echo "═══════════════════════════════════════════"
echo ""
echo "SERVICES DEPLOYED:"
echo "  CT-240  Bazarr         :6767    Subtitles"
echo "  CT-241  Overseerr      :5055    Plex request management"
echo "  CT-242  Jellyseerr     :5055    Jellyfin request management"
echo "  CT-243  Ombi           :3579    Request management (legacy)"
echo "  CT-244  Tautulli       :8181    Plex analytics"
echo "  CT-245  Kometa         (cron)   Plex Meta Manager"
echo "  CT-246  Gaps           :8484    Collection finder"
echo "  CT-247  Janitorr       (cron)   Media cleanup"
echo "  CT-248  Decluttarr     (cron)   Queue cleanup"
echo "  CT-249  Watchlistarr   (cron)   Watchlist sync"
echo "  CT-250  Traktarr       (cron)   Trakt.tv sync"
echo ""
echo "NEXT STEPS:"
echo "  1. Configure Overseerr → connect to Plex (192.168.12.230:32400)"
echo "     + Sonarr (192.168.12.214:8989) + Radarr (192.168.12.215:7878)"
echo "  2. Configure Jellyseerr → connect to Jellyfin (192.168.12.231:8096)"
echo "     + Sonarr + Radarr"
echo "  3. Configure Bazarr subtitle providers"
echo "  4. Configure Tautulli → Plex connection"
echo "  5. Configure Kometa with Plex/TMDb API keys"
echo ""
echo "SKIP CONTAINERS: Rerun with SKIP=\"243 246\" to skip specific CTs"
echo ""
echo "⚠ RAM: With all 11 CTs running + existing stack, monitor with:"
echo "   ssh root@192.168.12.242 'free -h; pct list'"
echo ""
