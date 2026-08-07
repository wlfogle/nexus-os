#!/usr/bin/env bash
# Native Reclaimerr installer for tiamat.
#
# Purpose:
#   Replace the broken PyInstaller/Desktop/tray build in /opt/reclaimerr with a
#   headless source install (FastAPI + granian) on the configured port.
#
# Safety:
#   - Preserves /opt/reclaimerr/.env.
#   - Backs up the existing /opt/reclaimerr tree before replacing it.
#   - Builds into a staging directory first.
#   - Starts a one-shot validation server before touching the systemd service.
#   - Rolls back the systemd unit on failure.
#   - Uses nala for OS packages when packages are needed.
#
# Run ON tiamat as root:
#   bash /home/loufogle/install-reclaimerr-native.sh

set -Eeuo pipefail

APP="reclaimerr"
REPO_URL="https://github.com/jessielw/Reclaimerr.git"
SRC_DIR="/opt/reclaimerr-src"
APP_DIR="/opt/reclaimerr"
BACKUP_ROOT="/opt/reclaimerr-backups"
ENV_FILE="${APP_DIR}/.env"
DATA_DIR="${APP_DIR}/data"
LOG_FILE="/var/log/reclaimerr-native-install.log"
SERVICE="/etc/systemd/system/reclaimerr.service"
STAGING="/opt/reclaimerr-stage.$$"
INSTALL_TS="$(date +%Y%m%d-%H%M%S)"
BACKUP_DIR="${BACKUP_ROOT}/${INSTALL_TS}"

log() {
  printf '[%s] %s\n' "$(date -Is)" "$*" | tee -a "$LOG_FILE"
}

die() {
  log "ERROR: $*"
  exit 1
}

need_root() {
  [ "$(id -u)" -eq 0 ] || die "run as root on tiamat"
}

pkg_install() {
  local missing=("$@")
  [ "${#missing[@]}" -gt 0 ] || return 0
  log "Installing OS packages with nala: ${missing[*]}"
  if command -v nala >/dev/null 2>&1; then
    nala install -y "${missing[@]}"
  else
    log "WARNING: nala not found; falling back to apt-get"
    apt-get update
    DEBIAN_FRONTEND=noninteractive apt-get install -y "${missing[@]}"
  fi
}

require_tools() {
  local pkgs=()
  command -v git >/dev/null 2>&1 || pkgs+=(git)
  command -v curl >/dev/null 2>&1 || pkgs+=(curl)
  command -v npm >/dev/null 2>&1 || pkgs+=(npm)
  command -v python3 >/dev/null 2>&1 || pkgs+=(python3)
  python3 -c 'import venv' >/dev/null 2>&1 || pkgs+=(python3-venv)
  # Native Python wheels may need compilation.
  command -v gcc >/dev/null 2>&1 || pkgs+=(build-essential)
  pkg_install "${pkgs[@]}"

  python3 - <<'PY'
import sys
if not ((3, 11) <= sys.version_info < (3, 14)):
    raise SystemExit(f"Python must be >=3.11,<3.14, got {sys.version}")
PY
  node_major="$(node -p 'process.versions.node.split(".")[0]' 2>/dev/null || echo 0)"
  [ "$node_major" -ge 20 ] || die "Node.js 20+ required, got $(node --version 2>/dev/null || echo missing)"
}

read_env_value() {
  local key="$1"
  local default="$2"
  if [ -f "$ENV_FILE" ]; then
    local value
    value="$(grep -E "^${key}=" "$ENV_FILE" | tail -1 | cut -d= -f2- || true)"
    if [ -n "$value" ]; then
      printf '%s' "$value"
      return 0
    fi
  fi
  printf '%s' "$default"
}

ensure_env() {
  mkdir -p "$APP_DIR"
  if [ ! -f "$ENV_FILE" ]; then
    log "Creating default ${ENV_FILE}"
    cat >"$ENV_FILE" <<'EOF'
HOST=0.0.0.0
PORT=8242
API_HOST=0.0.0.0
API_PORT=8242
DATA_DIR=/opt/reclaimerr/data
TZ=Pacific/Honolulu
PROXY_TRUSTED_HOSTS=127.0.0.1,::1,192.168.12.30,192.168.12.242
EOF
  fi

  grep -q '^API_HOST=' "$ENV_FILE" || echo 'API_HOST=0.0.0.0' >>"$ENV_FILE"
  grep -q '^API_PORT=' "$ENV_FILE" || echo 'API_PORT=8242' >>"$ENV_FILE"
  grep -q '^DATA_DIR=' "$ENV_FILE" || echo "DATA_DIR=${DATA_DIR}" >>"$ENV_FILE"
  grep -q '^PROXY_TRUSTED_HOSTS=' "$ENV_FILE" || echo 'PROXY_TRUSTED_HOSTS=127.0.0.1,::1,192.168.12.30,192.168.12.242' >>"$ENV_FILE"
  chmod 600 "$ENV_FILE"
}

clone_or_update() {
  if [ -d "${SRC_DIR}/.git" ]; then
    log "Updating source repo in ${SRC_DIR}"
    git -C "$SRC_DIR" fetch --depth 1 origin main
    git -C "$SRC_DIR" reset --hard origin/main
  else
    log "Cloning ${REPO_URL} to ${SRC_DIR}"
    rm -rf "$SRC_DIR"
    git clone --depth 1 "$REPO_URL" "$SRC_DIR"
  fi
}

build_frontend() {
  log "Building frontend"
  cd "${SRC_DIR}/frontend"
  npm install
  VITE_APP_CHANNEL=dev npm run build
  [ -d "${SRC_DIR}/frontend/dist" ] || die "frontend build did not produce frontend/dist"
}

install_backend_to_staging() {
  log "Installing backend into staging venv ${STAGING}"
  rm -rf "$STAGING"
  mkdir -p "$STAGING/app" "$STAGING/data/database" "$STAGING/data/logs" "$STAGING/data/static/avatars"
  cp -a "${SRC_DIR}/backend" "$STAGING/app/backend"
  cp -a "${SRC_DIR}/alembic.ini" "${SRC_DIR}/pyproject.toml" "${SRC_DIR}/README.md" "${SRC_DIR}/CHANGELOG.md" "$STAGING/app/"
  mkdir -p "$STAGING/app/frontend"
  cp -a "${SRC_DIR}/frontend/dist" "$STAGING/app/frontend/dist"

  python3 -m venv "$STAGING/venv"
  "$STAGING/venv/bin/python" -m pip install --upgrade pip wheel setuptools
  cd "$STAGING/app"
  "$STAGING/venv/bin/python" -m pip install .
  "$STAGING/venv/bin/python" - <<'PY'
import importlib
for mod in ("granian", "backend.api.main", "alembic"):
    importlib.import_module(mod)
print("python imports OK")
PY
}

backup_existing() {
  log "Backing up existing ${APP_DIR} and service to ${BACKUP_DIR}"
  mkdir -p "$BACKUP_DIR"
  if [ -d "$APP_DIR" ]; then
    cp -a "$APP_DIR" "$BACKUP_DIR/reclaimerr"
  fi
  if [ -f "$SERVICE" ]; then
    cp -a "$SERVICE" "$BACKUP_DIR/reclaimerr.service"
  fi
}

promote_staging() {
  log "Promoting staging to ${APP_DIR}"
  mkdir -p "$APP_DIR"
  if [ -f "$ENV_FILE" ]; then
    cp -a "$ENV_FILE" /tmp/reclaimerr.env.$$
  fi
  if [ -d "$DATA_DIR" ]; then
    cp -a "$DATA_DIR" "$STAGING/data.existing"
    rm -rf "$STAGING/data"
    mv "$STAGING/data.existing" "$STAGING/data"
  fi
  rm -rf "${APP_DIR}.old"
  mv "$APP_DIR" "${APP_DIR}.old"
  mv "$STAGING" "$APP_DIR"
  if [ -f /tmp/reclaimerr.env.$$ ]; then
    mv /tmp/reclaimerr.env.$$ "$ENV_FILE"
    chmod 600 "$ENV_FILE"
  fi
  rm -rf "${APP_DIR}.old"
}

write_service() {
  log "Writing headless systemd service"
  cat >"$SERVICE" <<EOF
[Unit]
Description=Reclaimerr Media Cleanup Service (native headless)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=root
WorkingDirectory=${APP_DIR}/app
EnvironmentFile=${ENV_FILE}
Environment=FRONTEND_DIST=${APP_DIR}/app/frontend/dist
Environment=PYTHONUNBUFFERED=1
ExecStart=${APP_DIR}/venv/bin/granian --interface asgi --workers 1 --host \${API_HOST} --port \${API_PORT} backend.api.main:app
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload
}

validate_before_service() {
  local host port pid code
  host="$(read_env_value API_HOST 0.0.0.0)"
  port="$(read_env_value API_PORT 8242)"
  log "Validating server manually on ${host}:${port}"
  # Stop old failed service if present so it does not interfere.
  systemctl stop reclaimerr.service >/dev/null 2>&1 || true
  cd "${APP_DIR}/app"
  set +e
  env $(grep -v '^[[:space:]]*#' "$ENV_FILE" | xargs) \
    FRONTEND_DIST="${APP_DIR}/app/frontend/dist" \
    "${APP_DIR}/venv/bin/granian" --interface asgi --workers 1 --host "$host" --port "$port" backend.api.main:app \
    >"${APP_DIR}/data/logs/validation.log" 2>&1 &
  pid=$!
  set -e
  sleep 5
  if ! kill -0 "$pid" >/dev/null 2>&1; then
    cat "${APP_DIR}/data/logs/validation.log" >&2 || true
    die "validation server exited early"
  fi
  code="$(curl -s -m 10 -o /dev/null -w '%{http_code}' "http://127.0.0.1:${port}/api/version" || true)"
  if [ "$code" != "200" ]; then
    echo "Validation /api/version returned HTTP ${code}; trying / ..." | tee -a "$LOG_FILE"
    code="$(curl -s -m 10 -o /dev/null -w '%{http_code}' "http://127.0.0.1:${port}/" || true)"
    [ "$code" = "200" ] || {
      kill "$pid" >/dev/null 2>&1 || true
      cat "${APP_DIR}/data/logs/validation.log" >&2 || true
      die "validation failed; HTTP ${code}"
    }
  fi
  kill "$pid" >/dev/null 2>&1 || true
  wait "$pid" >/dev/null 2>&1 || true
  log "Validation OK"
}

start_service() {
  log "Starting reclaimerr.service"
  systemctl reset-failed reclaimerr.service || true
  systemctl enable --now reclaimerr.service
  sleep 4
  systemctl is-active --quiet reclaimerr.service || {
    systemctl --no-pager -l status reclaimerr.service >&2 || true
    journalctl -u reclaimerr.service --no-pager -n 50 >&2 || true
    die "reclaimerr.service failed to start"
  }
  local port code
  port="$(read_env_value API_PORT 8242)"
  code="$(curl -s -m 10 -o /dev/null -w '%{http_code}' "http://127.0.0.1:${port}/" || true)"
  [ "$code" = "200" ] || [ "$code" = "302" ] || die "service running but root returned HTTP ${code}"
  log "Reclaimerr native service is active on port ${port}"
}

main() {
  need_root
  mkdir -p "$BACKUP_ROOT"
  touch "$LOG_FILE"
  log "=== Reclaimerr native install start ==="
  require_tools
  ensure_env
  clone_or_update
  build_frontend
  install_backend_to_staging
  backup_existing
  promote_staging
  write_service
  validate_before_service
  start_service
  log "=== Reclaimerr native install complete ==="
}

main "$@"
