#!/usr/bin/env bash
# =============================================================================
# NexusOS Health Monitor
# Hourly system health checks: disk, services, temperatures, docker, memory.
# Runs as a systemd timer (nexus-health.timer).
# Output: /var/log/nexus-os/health.log
# =============================================================================
set -euo pipefail

readonly LOG_DIR="/var/log/nexus-os"
readonly LOG_FILE="${LOG_DIR}/health.log"
readonly MAX_LOG_SIZE=$((5 * 1024 * 1024))  # 5MB rotation threshold
readonly DISK_WARN_PERCENT=85
readonly DISK_CRIT_PERCENT=95
readonly MEM_WARN_PERCENT=90
readonly GPU_TEMP_WARN=80
readonly GPU_TEMP_CRIT=90

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log() { printf "[%s] %s\n" "$(date '+%Y-%m-%d %H:%M:%S')" "$*" | tee -a "$LOG_FILE"; }
warn() { log "WARN: $*"; }
crit() { log "CRITICAL: $*"; }

rotate_log() {
    if [[ -f "$LOG_FILE" ]] && (( $(stat -c%s "$LOG_FILE" 2>/dev/null || echo 0) > MAX_LOG_SIZE )); then
        mv "$LOG_FILE" "${LOG_FILE}.1"
        log "Log rotated"
    fi
}

# ---------------------------------------------------------------------------
# Health Checks
# ---------------------------------------------------------------------------
check_disk() {
    log "--- Disk Usage ---"
    local issues=0
    while IFS= read -r line; do
        local mount pct
        mount="$(echo "$line" | awk '{print $6}')"
        pct="$(echo "$line" | awk '{print $5}' | tr -d '%')"

        if (( pct >= DISK_CRIT_PERCENT )); then
            crit "Disk $mount at ${pct}%% — critically full"
            (( issues++ ))
        elif (( pct >= DISK_WARN_PERCENT )); then
            warn "Disk $mount at ${pct}%% — high usage"
            (( issues++ ))
        else
            log "  $mount: ${pct}%% used"
        fi
    done < <(df -h --output=source,size,used,avail,pcent,target -x tmpfs -x devtmpfs -x squashfs 2>/dev/null | tail -n +2)

    if (( issues == 0 )); then
        log "  All disks OK"
    fi
}

check_memory() {
    log "--- Memory Usage ---"

    local total_kb used_kb avail_kb
    total_kb="$(grep MemTotal /proc/meminfo | awk '{print $2}')"
    avail_kb="$(grep MemAvailable /proc/meminfo | awk '{print $2}')"
    used_kb=$(( total_kb - avail_kb ))

    local pct=$(( (used_kb * 100) / total_kb ))
    local total_gb=$(( total_kb / 1024 / 1024 ))
    local avail_gb=$(( avail_kb / 1024 / 1024 ))

    if (( pct >= MEM_WARN_PERCENT )); then
        warn "Memory at ${pct}%% (${avail_gb}GB free of ${total_gb}GB)"
    else
        log "  Memory: ${pct}%% used (${avail_gb}GB free of ${total_gb}GB)"
    fi

    # Swap usage
    local swap_total swap_used
    swap_total="$(grep SwapTotal /proc/meminfo | awk '{print $2}')"
    swap_used="$(( swap_total - $(grep SwapFree /proc/meminfo | awk '{print $2}') ))"
    if (( swap_total > 0 )); then
        local swap_pct=$(( (swap_used * 100) / swap_total ))
        log "  Swap: ${swap_pct}%% used ($(( swap_used / 1024 ))MB / $(( swap_total / 1024 ))MB)"
    else
        log "  Swap: not configured"
    fi
}

check_gpu() {
    log "--- GPU Status ---"
    if ! command -v nvidia-smi &>/dev/null; then
        log "  nvidia-smi not available — skipping GPU checks"
        return
    fi

    local gpu_data
    gpu_data="$(nvidia-smi --query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total,fan.speed,power.draw --format=csv,noheader,nounits 2>/dev/null || true)"

    if [[ -z "$gpu_data" ]]; then
        warn "nvidia-smi returned no data"
        return
    fi

    while IFS=',' read -r name temp util mem_used mem_total fan power; do
        # Trim whitespace using bash parameter expansion (no xargs)
        name="${name#"${name%%[![:space:]]*}"}" ; name="${name%"${name##*[![:space:]]}"}"
        temp="${temp#"${temp%%[![:space:]]*}"}" ; temp="${temp%"${temp##*[![:space:]]}"}"
        util="${util#"${util%%[![:space:]]*}"}" ; util="${util%"${util##*[![:space:]]}"}"
        mem_used="${mem_used#"${mem_used%%[![:space:]]*}"}" ; mem_used="${mem_used%"${mem_used##*[![:space:]]}"}"
        mem_total="${mem_total#"${mem_total%%[![:space:]]*}"}" ; mem_total="${mem_total%"${mem_total##*[![:space:]]}"}"
        fan="${fan#"${fan%%[![:space:]]*}"}" ; fan="${fan%"${fan##*[![:space:]]}"}"
        power="${power#"${power%%[![:space:]]*}"}" ; power="${power%"${power##*[![:space:]]}"}"

        if [[ "$temp" =~ ^[0-9]+$ ]]; then
            if (( temp >= GPU_TEMP_CRIT )); then
                crit "GPU $name temperature: ${temp}°C — OVERHEATING"
            elif (( temp >= GPU_TEMP_WARN )); then
                warn "GPU $name temperature: ${temp}°C — running hot"
            else
                log "  GPU: $name | ${temp}°C | ${util}%% util | ${mem_used}/${mem_total}MB VRAM | Fan: ${fan}%% | ${power}W"
            fi
        else
            log "  GPU: $name | Temp: N/A | ${util}%% util | ${mem_used}/${mem_total}MB VRAM"
        fi
    done <<< "$gpu_data"
}

check_cpu_temp() {
    log "--- CPU Temperature ---"

    local temp_file=""
    # Try common thermal zone locations
    for zone in /sys/class/thermal/thermal_zone*/temp; do
        if [[ -f "$zone" ]]; then
            local zone_type
            zone_type="$(cat "$(dirname "$zone")/type" 2>/dev/null || echo 'unknown')"
            if [[ "$zone_type" == *"x86_pkg"* ]] || [[ "$zone_type" == *"coretemp"* ]] || [[ "$zone_type" == *"cpu"* ]]; then
                temp_file="$zone"
                break
            fi
            # Use first available if no CPU-specific one found
            [[ -z "$temp_file" ]] && temp_file="$zone"
        fi
    done

    if [[ -n "$temp_file" ]]; then
        local temp_raw temp_c
        temp_raw="$(cat "$temp_file" 2>/dev/null || echo 0)"
        temp_c=$(( temp_raw / 1000 ))
        if (( temp_c > 90 )); then
            crit "CPU temperature: ${temp_c}°C — OVERHEATING"
        elif (( temp_c > 80 )); then
            warn "CPU temperature: ${temp_c}°C — running hot"
        else
            log "  CPU temperature: ${temp_c}°C"
        fi
    else
        log "  CPU temperature sensor not found"
    fi
}

check_services() {
    log "--- NexusOS Services ---"

    local services=(
        "ollama:Ollama LLM"
        "nexus-stella:Stella AI Guardian"
        "nexus-maxjr:MaxJr Performance"
        "nexus-orchestrator:AI Orchestrator"
        "docker:Docker Engine"
        "ufw:Firewall"
        "fail2ban:Fail2Ban"
        "ssh:SSH Server"
    )

    local failed=0
    for entry in "${services[@]}"; do
        local svc="${entry%%:*}"
        local label="${entry##*:}"
        local status
        status="$(systemctl is-active "$svc" 2>/dev/null || echo 'not-found')"

        case "$status" in
            active)
                log "  [OK]   $label ($svc)"
                ;;
            inactive)
                log "  [OFF]  $label ($svc)"
                ;;
            failed)
                warn "$label ($svc) has FAILED"
                (( failed++ ))
                ;;
            not-found)
                log "  [N/A]  $label ($svc) — not installed"
                ;;
            *)
                warn "$label ($svc) status: $status"
                ;;
        esac
    done

    if (( failed > 0 )); then
        warn "$failed service(s) in failed state"
    fi
}

check_docker() {
    log "--- Docker Containers ---"
    if ! command -v docker &>/dev/null || ! docker info &>/dev/null 2>&1; then
        log "  Docker not available — skipping"
        return
    fi

    local total running stopped unhealthy
    total="$(docker ps -a --format '{{.ID}}' 2>/dev/null | wc -l || echo 0)"
    running="$(docker ps --format '{{.ID}}' 2>/dev/null | wc -l || echo 0)"
    stopped=$(( total - running ))

    log "  Containers: $running running, $stopped stopped, $total total"

    # Check for unhealthy containers
    unhealthy="$(docker ps --filter health=unhealthy --format '{{.Names}}' 2>/dev/null || true)"
    if [[ -n "$unhealthy" ]]; then
        while IFS= read -r name; do
            warn "Unhealthy container: $name"
        done <<< "$unhealthy"
    fi

    # Check for restarting containers (crash loops)
    local restarting
    restarting="$(docker ps --filter status=restarting --format '{{.Names}}' 2>/dev/null || true)"
    if [[ -n "$restarting" ]]; then
        while IFS= read -r name; do
            warn "Container in restart loop: $name"
        done <<< "$restarting"
    fi
}

check_load() {
    log "--- System Load ---"
    local load1 load5 load15 ncpu
    read -r load1 load5 load15 _ < /proc/loadavg
    ncpu="$(nproc 2>/dev/null || echo 1)"
    log "  Load average: $load1 / $load5 / $load15 ($ncpu CPUs)"

    # Warn if 1-min load exceeds 2x CPU count
    local load1_int="${load1%%.*}"
    if (( load1_int > ncpu * 2 )); then
        warn "System load ($load1) exceeds 2x CPU count ($ncpu)"
    fi
}

check_uptime() {
    log "--- Uptime ---"
    local up_secs
    up_secs="$(awk '{print int($1)}' /proc/uptime)"
    local days=$(( up_secs / 86400 ))
    local hours=$(( (up_secs % 86400) / 3600 ))
    local mins=$(( (up_secs % 3600) / 60 ))
    log "  System uptime: ${days}d ${hours}h ${mins}m"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    mkdir -p "$LOG_DIR"
    rotate_log

    log "=========================================="
    log "  NexusOS Health Check"
    log "=========================================="

    check_uptime
    check_load
    check_memory
    check_disk
    check_cpu_temp
    check_gpu
    check_services
    check_docker

    log "=========================================="
    log "  Health check complete"
    log "=========================================="
}

main "$@"
