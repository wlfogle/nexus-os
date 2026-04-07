#!/bin/bash
# ============================================================
# Phase 6 — Monitoring & Tools + remaining Phase 5 containers
# Run on Proxmox host (Tiamat) as root
#
# Creates:
#   CT-245  kometa     1024MB  Plex Meta Manager
#   CT-275  homarr      512MB  Unified dashboard :7575
#   CT-276  homepage    512MB  Static dashboard  :3000
#   CT-277  recyclarr   512MB  *arr quality sync
#
# Usage:
#   bash /opt/homelab-media-stack/scripts/deploy-phase6.sh
#   SKIP="276 277" bash .../deploy-phase6.sh   # skip specific CTs
# ============================================================
set -e

TEMPLATE="local:vztmpl/debian-12-standard_12.12-1_amd64.tar.zst"
STORAGE="local-lvm"
TZ="${TZ:-America/New_York}"
PUID="${PUID:-1000}"
PGID="${PGID:-1000}"

SKIP="${SKIP:-}"

# ── Helpers ──────────────────────────────────────────────────

should_skip() {
  local ctid="$1"
  for s in $SKIP; do [ "$s" = "$ctid" ] && return 0; done
  return 1
}

create_ct() {
  local ctid="$1" hostname="$2" ram="$3" disk="${4:-8}"
  should_skip "$ctid" && { echo "==> Skipping CT-${ctid} (${hostname})"; return 1; }
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
}

setup_docker() {
  local ctid="$1"
  echo "    Installing Docker in CT-${ctid}..."
  pct start "$ctid" 2>/dev/null || true
  sleep 6
  pct exec "$ctid" -- bash -c "
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq >/dev/null 2>&1
    apt-get install -y -qq curl ca-certificates >/dev/null 2>&1
    curl -fsSL https://get.docker.com | sh >/dev/null 2>&1
  "
  echo "    Docker installed in CT-${ctid}."
}

# ── CT-245: Kometa (Plex Meta Manager) ───────────────────────
echo ""
echo "═══ Phase 6: Monitoring & Tools ═══"
echo ""

if create_ct 245 kometa 1024 8; then
  setup_docker 245
  echo "    Starting kometa..."
  pct exec 245 -- bash -c "
    mkdir -p /opt/appdata/kometa
    docker run -d \
      --name kometa \
      --restart unless-stopped \
      -e TZ=${TZ} \
      -e KOMETA_RUN=true \
      -e KOMETA_TIMES=03:00 \
      -v /opt/appdata/kometa:/config \
      kometateam/kometa:latest
  "
  echo "    Kometa running (cron @ 03:00)."
fi

# ── CT-275: Homarr (unified dashboard) ───────────────────────
if create_ct 275 homarr 512 8; then
  setup_docker 275
  echo "    Starting homarr..."
  pct exec 275 -- bash -c "
    mkdir -p /opt/appdata/homarr/configs \
              /opt/appdata/homarr/icons \
              /opt/appdata/homarr/data
    docker run -d \
      --name homarr \
      --restart unless-stopped \
      -p 7575:7575 \
      -e TZ=${TZ} \
      -v /opt/appdata/homarr/configs:/app/data/configs \
      -v /opt/appdata/homarr/icons:/app/public/icons \
      -v /opt/appdata/homarr/data:/data \
      ghcr.io/ajnart/homarr:latest
  "
  echo "    Homarr running on :7575."
fi

# ── CT-276: Homepage (static dashboard) ──────────────────────
if create_ct 276 homepage 512 8; then
  setup_docker 276
  echo "    Starting homepage..."
  pct exec 276 -- bash -c "
    mkdir -p /opt/appdata/homepage
    docker run -d \
      --name homepage \
      --restart unless-stopped \
      -p 3000:3000 \
      -e PUID=${PUID} -e PGID=${PGID} -e TZ=${TZ} \
      -v /opt/appdata/homepage:/app/config \
      ghcr.io/gethomepage/homepage:latest
  "
  echo "    Homepage running on :3000."
fi

# ── CT-277: Recyclarr (*arr quality sync) ────────────────────
if create_ct 277 recyclarr 512 8; then
  setup_docker 277
  echo "    Starting recyclarr..."
  pct exec 277 -- bash -c "
    mkdir -p /opt/appdata/recyclarr
    docker run -d \
      --name recyclarr \
      --restart unless-stopped \
      -e TZ=${TZ} \
      -v /opt/appdata/recyclarr:/config \
      ghcr.io/recyclarr/recyclarr:latest
  "
  echo "    Recyclarr running (sync on startup)."
fi

# ── Summary ──────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════"
echo "  Phase 6 Deployment Complete"
echo "═══════════════════════════════════════════"
echo ""
echo "SERVICES DEPLOYED:"
echo "  CT-245  Kometa      (cron 03:00)  Plex Meta Manager"
echo "  CT-275  Homarr      :7575         Unified dashboard"
echo "  CT-276  Homepage    :3000         Static dashboard"
echo "  CT-277  Recyclarr   (cron)        *arr quality sync"
echo ""
echo "NEXT STEPS:"
echo "  Homarr   — http://\$(pct exec 275 -- hostname -I | awk '{print \$1}'):7575"
echo "  Homepage — http://\$(pct exec 276 -- hostname -I | awk '{print \$1}'):3000"
echo "  Kometa   — edit /opt/appdata/kometa/config.yml (add Plex URL + token)"
echo "  Recyclarr— edit /opt/appdata/recyclarr/recyclarr.yml (add Sonarr/Radarr keys)"
echo ""
echo "SKIP CONTAINERS: SKIP=\"275 276\" bash \$0"
echo ""
