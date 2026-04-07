#!/bin/bash
# ============================================================
# Fix CT-244 (tautulli) — Docker install incomplete at creation
# Run on Proxmox host (Tiamat) as root
# Safe to re-run: idempotent
# ============================================================
set -e

CTID=244
TZ="${TZ:-America/New_York}"

echo "==> Checking CT-${CTID} (tautulli)..."

# Ensure CT is running
if ! pct status $CTID 2>/dev/null | grep -q running; then
  echo "    Starting CT-${CTID}..."
  pct start $CTID
  sleep 6
fi

# Install Docker if missing
if ! pct exec $CTID -- which docker &>/dev/null; then
  echo "    Docker not found — installing..."
  pct exec $CTID -- bash -c "
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq >/dev/null 2>&1
    apt-get install -y -qq curl ca-certificates >/dev/null 2>&1
    curl -fsSL https://get.docker.com | sh >/dev/null 2>&1
  "
  echo "    Docker installed."
else
  echo "    Docker already present."
fi

# Start Tautulli if not already running
if ! pct exec $CTID -- docker ps --filter name=tautulli --filter status=running | grep -q tautulli; then
  echo "    Starting Tautulli..."
  pct exec $CTID -- bash -c "
    mkdir -p /opt/appdata/tautulli
    docker rm -f tautulli 2>/dev/null || true
    docker run -d \
      --name tautulli \
      --restart unless-stopped \
      -p 8181:8181 \
      -e PUID=1000 -e PGID=1000 -e TZ=${TZ} \
      -v /opt/appdata/tautulli:/config \
      ghcr.io/tautulli/tautulli:latest
  "
  echo "    Tautulli started."
else
  echo "    Tautulli already running — nothing to do."
fi

IP=$(pct exec $CTID -- hostname -I | awk '{print $1}')
echo ""
echo "CT-244 (tautulli) is healthy at http://${IP}:8181"
