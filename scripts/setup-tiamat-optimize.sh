#!/usr/bin/env bash
# =============================================================================
# setup-tiamat-optimize.sh — Tiamat Performance Tuning
# AMD Ryzen 5 3600 (6c/12t) | ~7.7GB RAM | 1.8TB Seagate HDD
# Role: Proxmox media server (Plex/Jellyfin transcoding) + Ollama CPU inference
# =============================================================================
set -euo pipefail

CPU_CORES=$(nproc --all)
TOTAL_MEM_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
TOTAL_MEM_MB=$((TOTAL_MEM_KB / 1024))

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║   Tiamat System Optimization                         ║"
echo "║   Ryzen 5 3600 — Media Server + Ollama              ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
printf "  CPU : AMD Ryzen 5 3600 (%s threads)\n" "${CPU_CORES}"
printf "  RAM : %sMB\n" "${TOTAL_MEM_MB}"
echo ""

# ── Step 1: CPU governor — performance for transcoding + inference ─────────────
echo "[1/7] Setting CPU governor to performance..."
apt-get install -y cpufrequtils linux-cpupower 2>/dev/null || apt-get install -y cpufrequtils 2>/dev/null || true

for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    [[ -f "$cpu" ]] && echo performance > "$cpu"
done

# Persist
cat > /etc/default/cpufrequtils << 'EOF'
GOVERNOR="performance"
EOF

# Keep Core Performance Boost (turbo) ON — critical for transcode bursts
[[ -f /sys/devices/system/cpu/cpufreq/boost ]] && echo 1 > /sys/devices/system/cpu/cpufreq/boost || true

echo "  CPU governor: performance, turbo: on"

# ── Step 2: KSM — stretch 7.7GB RAM across LXC containers ─────────────────────
echo "[2/7] Enabling KSM (Kernel Same-page Merging) for RAM efficiency..."

# Enable KSM
echo 1 > /sys/kernel/mm/ksm/run
echo 1000 > /sys/kernel/mm/ksm/sleep_millisecs   # scan every 1s
echo 1000 > /sys/kernel/mm/ksm/pages_to_scan     # 1000 pages per scan

# Persist via rc.local
cat > /etc/systemd/system/ksm-config.service << 'EOF'
[Unit]
Description=KSM Configuration for Tiamat RAM Efficiency
After=multi-user.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/bin/bash -c 'echo 1 > /sys/kernel/mm/ksm/run; echo 1000 > /sys/kernel/mm/ksm/sleep_millisecs; echo 1000 > /sys/kernel/mm/ksm/pages_to_scan'

[Install]
WantedBy=multi-user.target
EOF
systemctl daemon-reload
systemctl enable ksm-config.service
echo "  KSM: enabled — will merge duplicate pages across containers"

# ── Step 3: Huge pages for Ollama CPU inference ────────────────────────────────
echo "[3/7] Configuring huge pages for Ollama LLM inference..."

# With 7.7GB total: reserve 1.5GB as 2MB huge pages for Ollama (750 pages)
# Leave enough for Proxmox host + containers
HUGEPAGES=750

echo ${HUGEPAGES} > /proc/sys/vm/nr_hugepages
echo always > /sys/kernel/mm/transparent_hugepage/enabled
echo defer+madvise > /sys/kernel/mm/transparent_hugepage/defrag

cat > /etc/sysctl.d/99-hugepages-tiamat.conf << EOF
vm.nr_hugepages = ${HUGEPAGES}
vm.nr_overcommit_hugepages = 64
EOF

echo "  Huge pages: ${HUGEPAGES} × 2MB = 1.5GB reserved for Ollama"

# ── Step 4: Memory / VM tuning for media server ────────────────────────────────
echo "[4/7] Tuning VM/memory for media streaming + LLM..."

cat > /etc/sysctl.d/97-memory-tiamat.conf << 'EOF'
# Tiamat memory tuning — media server + Ollama on 7.7GB
vm.swappiness              = 10         # Low swap — prefer RAM
vm.dirty_ratio             = 20         # More dirty pages before writeback (HDD)
vm.dirty_background_ratio  = 10
vm.dirty_expire_centisecs  = 6000       # HDD: longer expire OK
vm.dirty_writeback_centisecs = 1000
vm.vfs_cache_pressure      = 50         # Keep inode/dentry cache longer
vm.min_free_kbytes         = 131072     # Keep 128MB free always
vm.zone_reclaim_mode       = 0
vm.max_map_count           = 1048576    # Needed for Ollama large models
# KSM tuning
kernel.numa_balancing      = 0          # Single socket — disable NUMA balancing
EOF

sysctl -p /etc/sysctl.d/97-memory-tiamat.conf > /dev/null

# ── Step 5: I/O scheduler — HDD optimized for media reads ─────────────────────
echo "[5/7] Optimizing I/O scheduler for Seagate HDD media storage..."

for disk in /sys/block/sd*; do
    dev=$(basename "$disk")
    rotational=$(cat "$disk/queue/rotational" 2>/dev/null || echo "0")
    if [[ "$rotational" == "1" ]]; then
        # Spinning disk — use bfq (best for media: fairness + read-ahead)
        echo bfq > "$disk/queue/scheduler" 2>/dev/null || echo mq-deadline > "$disk/queue/scheduler" 2>/dev/null || true
        echo 2048 > "$disk/queue/read_ahead_kb" 2>/dev/null || true   # 2MB read-ahead for sequential media
        echo 256  > "$disk/queue/nr_requests"   2>/dev/null || true
        echo "  ${dev}: bfq scheduler, 2MB read-ahead (spinning media drive)"
    else
        # SSD/NVMe
        echo none > "$disk/queue/scheduler" 2>/dev/null || true
        echo 512  > "$disk/queue/read_ahead_kb" 2>/dev/null || true
        echo 64   > "$disk/queue/nr_requests"   2>/dev/null || true
        echo "  ${dev}: none scheduler (SSD)"
    fi
done

# Persist via udev
cat > /etc/udev/rules.d/60-io-scheduler.rules << 'EOF'
# Spinning disks — bfq for media server
ACTION=="add|change", KERNEL=="sd[a-z]", ATTR{queue/rotational}=="1", ATTR{queue/scheduler}="bfq", ATTR{queue/read_ahead_kb}="2048"
# SSDs/NVMe — no scheduler
ACTION=="add|change", KERNEL=="sd[a-z]", ATTR{queue/rotational}=="0", ATTR{queue/scheduler}="none"
ACTION=="add|change", KERNEL=="nvme*",   ATTR{queue/scheduler}="none"
EOF

# ── Step 6: Network tuning for streaming multiple clients ──────────────────────
echo "[6/7] Tuning network for media streaming (BBR + large buffers)..."

modprobe tcp_bbr 2>/dev/null || true
echo tcp_bbr >> /etc/modules 2>/dev/null || true

cat > /etc/sysctl.d/99-network-tiamat.conf << 'EOF'
# Network tuning for Plex/Jellyfin streaming to multiple clients
net.core.rmem_max              = 134217728
net.core.wmem_max              = 134217728
net.core.rmem_default          = 262144
net.core.wmem_default          = 262144
net.ipv4.tcp_rmem              = 4096 65536 134217728
net.ipv4.tcp_wmem              = 4096 65536 134217728
net.core.netdev_max_backlog    = 10000
net.ipv4.tcp_congestion_control = bbr
net.core.default_qdisc         = cake
net.ipv4.tcp_fastopen          = 3
net.ipv4.tcp_slow_start_after_idle = 0
net.ipv4.ip_forward            = 1
# Increase connection tracking for many simultaneous streams
net.netfilter.nf_conntrack_max = 131072
EOF

sysctl -p /etc/sysctl.d/99-network-tiamat.conf > /dev/null 2>&1 || true

# ── Step 7: Ollama systemd service tuning ─────────────────────────────────────
echo "[7/7] Configuring Ollama for Ryzen 5 3600 CPU inference (AVX2)..."

if systemctl list-unit-files | grep -q ollama; then
    mkdir -p /etc/systemd/system/ollama.service.d
    cat > /etc/systemd/system/ollama.service.d/override.conf << EOF
[Service]
# Ryzen 5 3600 — 6c/12t, AVX2, ~7GB RAM available
Environment="OLLAMA_NUM_PARALLEL=2"
Environment="OLLAMA_MAX_LOADED_MODELS=1"
Environment="OLLAMA_MAX_QUEUE=64"
Environment="OLLAMA_FLASH_ATTENTION=1"
Environment="OLLAMA_HOST=0.0.0.0:11434"
Environment="OLLAMA_KEEP_ALIVE=10m"
Environment="OLLAMA_ORIGINS=*"
# Limit Ollama to 4GB so media server has headroom
Environment="GOMEMLIMIT=4GiB"
Environment="GOMAXPROCS=${CPU_CORES}"
# CPU affinity — all cores, but low priority vs media transcoding
CPUWeight=512
Nice=10
IOSchedulingClass=3
IOSchedulingPriority=7
LimitMEMLOCK=infinity
LimitNOFILE=65536
EOF
    systemctl daemon-reload
    systemctl restart ollama 2>/dev/null && echo "  Ollama restarted with new config." || echo "  Ollama not running — config staged for next start."
else
    echo "  Ollama not installed. Config will be applied when installed."
    echo "  Install: curl -fsSL https://ollama.ai/install.sh | sh"
fi

# ── File descriptor limits ─────────────────────────────────────────────────────
cat > /etc/security/limits.d/99-tiamat.conf << 'EOF'
# File descriptor limits for media server + Ollama
*  soft  nofile  524288
*  hard  nofile  524288
*  soft  nproc   65536
*  hard  nproc   65536
root soft nofile  524288
root hard nofile  524288
EOF

# ── Disable unused Proxmox services to free RAM ────────────────────────────────
echo "  Disabling unused Proxmox enterprise services..."
systemctl disable --now pve-ha-lrm pve-ha-crm corosync 2>/dev/null || true  # HA cluster — not needed solo
# Keep: pveproxy, pvedaemon, pvestatd, pvescheduler, pvefw-logger

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║   Tiamat Optimization COMPLETE                                   ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   ✓ CPU governor: performance (turbo boost ON)                  ║"
printf "║   ✓ KSM: enabled — scanning %s pages/cycle                 ║\n" "1000"
printf "║   ✓ Huge pages: %-50s ║\n" "${HUGEPAGES} × 2MB = 1.5GB for Ollama"
echo "║   ✓ vm.swappiness=10, dirty ratios tuned for HDD               ║"
echo "║   ✓ HDD I/O: bfq scheduler + 2MB read-ahead                   ║"
echo "║   ✓ Network: BBR + cake + 128MB buffers                        ║"
echo "║   ✓ Ollama: 4GB limit, Nice=10 (media gets priority)           ║"
echo "║   ✓ File descriptors: 524288                                    ║"
echo "║   ✓ HA cluster services disabled (not needed solo)             ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   Recommended models for 4GB Ollama limit:                      ║"
echo "║   • ollama pull llama3.2:3b     (2GB — fast responses)         ║"
echo "║   • ollama pull mistral:7b-q4   (4GB — best quality at limit)  ║"
echo "║   • ollama pull qwen2.5:3b      (2GB — good coding help)       ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
