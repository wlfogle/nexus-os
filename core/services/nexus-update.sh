#!/usr/bin/env bash
# =============================================================================
# NexusOS Auto-Update
# Weekly system maintenance: package updates, container pulls, cleanup.
# Runs as a systemd timer (nexus-update.timer).
# =============================================================================
set -euo pipefail

readonly LOG_DIR="/var/log/nexus-os"
readonly LOG_FILE="${LOG_DIR}/update.log"
readonly LOCK_FILE="/var/run/nexus-update.lock"
readonly MAX_LOG_SIZE=$((10 * 1024 * 1024))  # 10MB rotation threshold

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log() { printf "[%s] %s\n" "$(date '+%Y-%m-%d %H:%M:%S')" "$*" | tee -a "$LOG_FILE"; }
warn() { log "WARN: $*"; }
die() { log "FATAL: $*"; cleanup; exit 1; }

cleanup() {
    rm -f "$LOCK_FILE"
}
trap cleanup EXIT

acquire_lock() {
    if [[ -f "$LOCK_FILE" ]]; then
        local pid
        pid="$(cat "$LOCK_FILE" 2>/dev/null || true)"
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            die "Another update is running (PID $pid)"
        fi
        warn "Stale lock file found — removing"
        rm -f "$LOCK_FILE"
    fi
    echo $$ > "$LOCK_FILE"
}

rotate_log() {
    if [[ -f "$LOG_FILE" ]] && (( $(stat -c%s "$LOG_FILE" 2>/dev/null || echo 0) > MAX_LOG_SIZE )); then
        mv "$LOG_FILE" "${LOG_FILE}.1"
        log "Log rotated"
    fi
}

# ---------------------------------------------------------------------------
# Update Functions
# ---------------------------------------------------------------------------
update_nala() {
    log "--- Updating system packages (nala) ---"
    if command -v nala &>/dev/null; then
        nala update 2>&1 | tee -a "$LOG_FILE"
        nala upgrade -y 2>&1 | tee -a "$LOG_FILE"
        log "Nala update complete"
    elif command -v apt-get &>/dev/null; then
        warn "nala not found, falling back to apt-get"
        apt-get update -qq 2>&1 | tee -a "$LOG_FILE"
        DEBIAN_FRONTEND=noninteractive apt-get upgrade -y -qq 2>&1 | tee -a "$LOG_FILE"
        log "apt-get update complete"
    else
        warn "No supported package manager found"
    fi
}

update_flatpak() {
    log "--- Updating Flatpak packages ---"
    if command -v flatpak &>/dev/null; then
        local count
        count="$(flatpak list --app 2>/dev/null | wc -l || echo 0)"
        if (( count > 0 )); then
            flatpak update -y --noninteractive 2>&1 | tee -a "$LOG_FILE"
            log "Flatpak update complete ($count apps)"
        else
            log "No Flatpak apps installed — skipping"
        fi
    else
        log "Flatpak not installed — skipping"
    fi
}

update_snap() {
    log "--- Updating Snap packages ---"
    if command -v snap &>/dev/null; then
        local count
        count="$(snap list 2>/dev/null | tail -n +2 | wc -l || echo 0)"
        if (( count > 0 )); then
            snap refresh 2>&1 | tee -a "$LOG_FILE"
            log "Snap update complete ($count snaps)"
        else
            log "No Snaps installed — skipping"
        fi
    else
        log "Snap not installed — skipping"
    fi
}

update_docker() {
    log "--- Updating Docker containers ---"
    if command -v docker &>/dev/null && docker info &>/dev/null; then
        # Pull latest images for running containers
        local images
        images="$(docker ps --format '{{.Image}}' 2>/dev/null | sort -u || true)"
        if [[ -n "$images" ]]; then
            local pulled=0
            while IFS= read -r img; do
                log "Pulling: $img"
                if docker pull "$img" 2>&1 | tee -a "$LOG_FILE"; then
                    (( pulled++ ))
                else
                    warn "Failed to pull: $img"
                fi
            done <<< "$images"
            log "Docker pull complete ($pulled images updated)"
        else
            log "No running Docker containers — skipping pulls"
        fi

        # Also update docker-compose stacks if any compose files exist
        if command -v docker-compose &>/dev/null || docker compose version &>/dev/null 2>&1; then
            local compose_dir="/opt/nexus-os/stacks"
            if [[ -d "$compose_dir" ]]; then
                for f in "$compose_dir"/*/docker-compose.yml "$compose_dir"/*/compose.yml; do
                    if [[ -f "$f" ]]; then
                        local stack_dir
                        stack_dir="$(dirname "$f")"
                        log "Updating stack: $stack_dir"
                        (cd "$stack_dir" && docker compose pull 2>&1 | tee -a "$LOG_FILE" && docker compose up -d 2>&1 | tee -a "$LOG_FILE") || warn "Stack update failed: $stack_dir"
                    fi
                done
            fi
        fi
    else
        log "Docker not available — skipping"
    fi
}

update_ollama() {
    log "--- Updating Ollama models ---"
    if command -v ollama &>/dev/null; then
        local models
        models="$(ollama list 2>/dev/null | tail -n +2 | awk '{print $1}' || true)"
        if [[ -n "$models" ]]; then
            while IFS= read -r model; do
                log "Pulling latest: $model"
                ollama pull "$model" 2>&1 | tee -a "$LOG_FILE" || warn "Failed to update model: $model"
            done <<< "$models"
            log "Ollama model update complete"
        else
            log "No Ollama models installed — skipping"
        fi
    else
        log "Ollama not installed — skipping"
    fi
}

cleanup_system() {
    log "--- Running system cleanup ---"

    # Clean package cache
    if command -v nala &>/dev/null; then
        nala autoremove -y 2>&1 | tee -a "$LOG_FILE"
        nala clean 2>&1 | tee -a "$LOG_FILE"
    elif command -v apt-get &>/dev/null; then
        apt-get autoremove -y -qq 2>&1 | tee -a "$LOG_FILE"
        apt-get clean -qq 2>&1 | tee -a "$LOG_FILE"
    fi

    # Clean old journal entries (keep 7 days)
    if command -v journalctl &>/dev/null; then
        journalctl --vacuum-time=7d 2>&1 | tee -a "$LOG_FILE"
    fi

    # Prune Docker (dangling images, stopped containers, unused networks)
    if command -v docker &>/dev/null && docker info &>/dev/null; then
        docker system prune -f 2>&1 | tee -a "$LOG_FILE"
        log "Docker prune complete"
    fi

    # Clean /tmp files older than 7 days
    find /tmp -type f -atime +7 -delete 2>/dev/null || true

    log "System cleanup complete"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    [[ $EUID -eq 0 ]] || die "Must be run as root"
    mkdir -p "$LOG_DIR"
    rotate_log
    acquire_lock

    log "=========================================="
    log "  NexusOS Auto-Update Starting"
    log "=========================================="

    local start_time
    start_time="$(date +%s)"

    update_nala
    update_flatpak
    update_snap
    update_docker
    update_ollama
    cleanup_system

    local end_time elapsed
    end_time="$(date +%s)"
    elapsed=$(( end_time - start_time ))

    log "=========================================="
    log "  NexusOS Auto-Update Complete"
    log "  Duration: $(( elapsed / 60 ))m $(( elapsed % 60 ))s"
    log "=========================================="
}

main "$@"
