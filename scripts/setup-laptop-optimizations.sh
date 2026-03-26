#!/usr/bin/env bash
# =============================================================================
# setup-laptop-optimizations.sh — Pop!_OS Laptop Performance Tuning
# Intel i9-13900HX + NVIDIA RTX 4080 Laptop + 64GB RAM
# Ported from wlfogle/i9-13900hx-optimizations to Pop!_OS (nala/apt)
# Covers: NVIDIA perf mode, huge pages, CPU governor, network stack (BBR),
#         memory management, Ollama tuning, KVM/VFIO readiness
# Run as your normal user (sudo will be prompted as needed)
# =============================================================================
set -euo pipefail

CPU_CORES=$(nproc --all)
TOTAL_MEM_GB=$(free -g | awk '/^Mem:/{print $2}')

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║   Laptop Performance Optimization                    ║"
echo "║   i9-13900HX + RTX 4080 + 64GB — Pop!_OS            ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
echo "  CPU cores : ${CPU_CORES}"
echo "  RAM       : ${TOTAL_MEM_GB}GB"
echo ""

# ── Step 1: NVIDIA persistence + performance mode ─────────────────────────────
echo "[1/6] Configuring NVIDIA for maximum performance..."
sudo nala install -y nvidia-utils || true

# Enable persistence mode (keeps GPU initialized, reduces latency spikes)
sudo nvidia-smi -pm 1 2>/dev/null && echo "  NVIDIA persistence mode: ON" || true

# Create performance-mode service (runs at boot)
sudo tee /etc/systemd/system/nvidia-performance.service > /dev/null << 'EOF'
[Unit]
Description=NVIDIA RTX 4080 — Maximum Performance Mode
After=graphical.target

[Service]
Type=oneshot
RemainAfterExit=yes
# Persistence daemon
ExecStart=/usr/bin/nvidia-smi -pm 1
# Maximum performance state
ExecStart=/usr/bin/nvidia-smi --auto-boost-default=0
# Prefer maximum graphics clocks on AC
ExecStart=/usr/bin/nvidia-settings -a '[gpu:0]/GPUPowerMizerMode=1' || true

[Install]
WantedBy=graphical.target
EOF
sudo systemctl daemon-reload
sudo systemctl enable nvidia-performance.service
sudo systemctl start  nvidia-performance.service 2>/dev/null || true

# ── Step 2: Huge pages for Ollama / LLM inference ─────────────────────────────
echo "[2/6] Configuring huge pages for LLM inference..."

# Use 40% of RAM for 2MB huge pages
HUGEPAGES_2MB=$(( TOTAL_MEM_GB * 1024 * 40 / 100 / 2 ))

sudo tee /etc/sysctl.d/99-hugepages-llm.conf > /dev/null << EOF
# Huge pages for LLM / Ollama workloads
vm.nr_hugepages = ${HUGEPAGES_2MB}
vm.nr_overcommit_hugepages = 10
vm.hugetlb_shm_group = 0
kernel.shmmax = $(( TOTAL_MEM_GB * 1024 * 1024 * 1024 / 2 ))
kernel.shmall = $(( TOTAL_MEM_GB * 1024 * 1024 / 4 ))
EOF

# Enable transparent huge pages (always mode — best for model inference)
echo always | sudo tee /sys/kernel/mm/transparent_hugepage/enabled > /dev/null
echo always | sudo tee /sys/kernel/mm/transparent_hugepage/defrag  > /dev/null

# ── Step 3: Memory management for large models ────────────────────────────────
echo "[3/6] Tuning VM/memory management for 64GB + LLMs..."

sudo tee /etc/sysctl.d/97-memory-ai.conf > /dev/null << 'EOF'
# Memory tuning for large AI models
vm.swappiness              = 1
vm.dirty_ratio             = 15
vm.dirty_background_ratio  = 5
vm.dirty_expire_centisecs  = 3000
vm.dirty_writeback_centisecs = 500
vm.overcommit_memory       = 1
vm.overcommit_ratio        = 90
vm.min_free_kbytes         = 262144
vm.zone_reclaim_mode       = 0
vm.vfs_cache_pressure      = 50
vm.max_map_count           = 2147483647
# NUMA balancing
kernel.numa_balancing       = 1
EOF

# ── Step 4: Network stack — BBR + large buffers ───────────────────────────────
echo "[4/6] Tuning network stack (BBR, 128MB buffers, cake qdisc)..."

sudo tee /etc/sysctl.d/99-network-performance.conf > /dev/null << 'EOF'
# High-performance network stack
net.core.rmem_default          = 262144
net.core.rmem_max              = 134217728
net.core.wmem_default          = 262144
net.core.wmem_max              = 134217728
net.core.netdev_max_backlog    = 30000
net.core.netdev_budget         = 600
net.ipv4.tcp_rmem              = 4096 65536 134217728
net.ipv4.tcp_wmem              = 4096 65536 134217728
net.ipv4.tcp_congestion_control = bbr
net.core.default_qdisc         = cake
net.ipv4.tcp_fastopen          = 3
net.ipv4.tcp_slow_start_after_idle = 0
net.core.busy_read             = 50
net.core.busy_poll             = 50
# IP forwarding (for KVM/docker)
net.ipv4.ip_forward            = 1
net.ipv6.conf.all.forwarding   = 1
EOF

# Load BBR module
sudo modprobe tcp_bbr 2>/dev/null || true
echo tcp_bbr | sudo tee -a /etc/modules-load.d/bbr.conf > /dev/null 2>/dev/null || true

# Apply all sysctl settings now
sudo sysctl --system > /dev/null

# ── Step 5: CPU governor — performance on AC ──────────────────────────────────
echo "[5/6] Setting CPU governor and scheduler tuning..."

sudo nala install -y cpufrequtils 2>/dev/null || true

# Set performance governor for all cores
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    [[ -f "$cpu" ]] && echo performance | sudo tee "$cpu" > /dev/null
done

# Persist via cpufrequtils
sudo tee /etc/default/cpufrequtils > /dev/null << 'EOF'
GOVERNOR="performance"
EOF

# Scheduler tuning for AI/interactive workloads
sudo tee /etc/sysctl.d/98-scheduler.conf > /dev/null << 'EOF'
kernel.sched_min_granularity_ns      = 1000000
kernel.sched_wakeup_granularity_ns   = 2000000
kernel.sched_migration_cost_ns       = 500000
kernel.sched_latency_ns              = 6000000
kernel.sched_autogroup_enabled       = 1
EOF
sudo sysctl -p /etc/sysctl.d/98-scheduler.conf > /dev/null

# ── Step 6: Ollama tuning ─────────────────────────────────────────────────────
echo "[6/6] Optimizing Ollama for ${TOTAL_MEM_GB}GB RAM + RTX 4080..."

if command -v ollama &>/dev/null || [[ -f /usr/local/bin/ollama ]]; then
    sudo mkdir -p /etc/systemd/system/ollama.service.d

    sudo tee /etc/systemd/system/ollama.service.d/override.conf > /dev/null << EOF
[Service]
# RTX 4080 + 64GB RAM tuning
Environment="OLLAMA_NUM_PARALLEL=4"
Environment="OLLAMA_MAX_LOADED_MODELS=3"
Environment="OLLAMA_MAX_QUEUE=512"
Environment="OLLAMA_FLASH_ATTENTION=1"
Environment="OLLAMA_HOST=0.0.0.0:11434"
Environment="OLLAMA_KEEP_ALIVE=24h"
Environment="OLLAMA_ORIGINS=*"
Environment="GOMAXPROCS=${CPU_CORES}"
Environment="GOMEMLIMIT=$(( TOTAL_MEM_GB * 80 / 100 ))GiB"
# Process priority
Nice=-10
IOSchedulingClass=1
IOSchedulingPriority=4
LimitMEMLOCK=infinity
LimitNOFILE=1048576
CPUWeight=1000
EOF

    sudo systemctl daemon-reload
    sudo systemctl restart ollama 2>/dev/null || true
    echo "  Ollama service tuned."
else
    echo "  Ollama not installed — skipping. Install: curl -fsSL https://ollama.ai/install.sh | sh"
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║   Laptop Optimizations Applied                                   ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   ✓ NVIDIA RTX 4080 persistence + performance mode              ║"
printf "║   ✓ Huge pages: %-50s ║\n" "${HUGEPAGES_2MB} × 2MB pages"
echo "║   ✓ Transparent huge pages: always                               ║"
echo "║   ✓ vm.swappiness=1, overcommit tuned for 64GB                  ║"
echo "║   ✓ TCP BBR + cake qdisc + 128MB buffers                        ║"
echo "║   ✓ CPU governor: performance                                    ║"
echo "║   ✓ Ollama: parallel=4, flash attention, 24h keepalive          ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   Persistent across reboots — no reboot required for most.      ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
