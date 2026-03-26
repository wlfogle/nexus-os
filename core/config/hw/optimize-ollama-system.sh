#!/bin/bash
# Ollama and LLM System Optimization Script
# Optimized for Intel i9-13900HX with 64GB RAM

set -e

echo "🧠 Ollama & LLM Performance Optimization"
echo "========================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# System detection
CPU_CORES=$(nproc --all)
TOTAL_MEMORY_GB=$(free -g | awk '/^Mem:/{print $2}')
AVAILABLE_MEMORY_GB=$(free -g | awk '/^Mem:/{print $7}')

log_info "System: ${CPU_CORES} cores, ${TOTAL_MEMORY_GB}GB total RAM, ${AVAILABLE_MEMORY_GB}GB available"

# Optimize huge pages for large models
optimize_hugepages() {
    log_info "Configuring huge pages for LLM performance..."
    
    # Calculate optimal huge page allocation (use ~40% of RAM for huge pages)
    HUGEPAGE_SIZE_GB=$((TOTAL_MEMORY_GB * 40 / 100))
    HUGEPAGES_2MB=$((HUGEPAGE_SIZE_GB * 1024 / 2))  # 2MB pages
    HUGEPAGES_1GB=$((HUGEPAGE_SIZE_GB / 1))  # 1GB pages
    
    # Enable transparent huge pages (always mode for better performance)
    echo always | sudo tee /sys/kernel/mm/transparent_hugepage/enabled > /dev/null
    echo always | sudo tee /sys/kernel/mm/transparent_hugepage/defrag > /dev/null
    echo madvise | sudo tee /sys/kernel/mm/transparent_hugepage/shmem_enabled > /dev/null
    
    # Configure huge pages
    echo $HUGEPAGES_2MB | sudo tee /proc/sys/vm/nr_hugepages > /dev/null
    echo 10 | sudo tee /proc/sys/vm/nr_overcommit_hugepages > /dev/null
    
    # Make changes persistent
    cat > /tmp/hugepage_config << EOF
# Huge page optimizations for LLM workloads
vm.nr_hugepages = $HUGEPAGES_2MB
vm.nr_overcommit_hugepages = 10
vm.hugetlb_shm_group = 0
kernel.shmmax = $((TOTAL_MEMORY_GB * 1024 * 1024 * 1024 / 2))
kernel.shmall = $((TOTAL_MEMORY_GB * 1024 * 1024 / 4))
EOF
    
    sudo mv /tmp/hugepage_config /etc/sysctl.d/99-hugepages-llm.conf
    
    log_success "Configured ${HUGEPAGE_SIZE_GB}GB of huge pages for LLM workloads"
}

# Optimize CPU governor and scheduling
optimize_cpu() {
    log_info "Optimizing CPU governor and scheduling for AI workloads..."
    
    # Set performance governor for all cores
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            echo performance | sudo tee "$cpu" > /dev/null
        fi
    done
    
    # Disable CPU idle states for consistent performance
    for state in /sys/devices/system/cpu/cpu*/cpuidle/state*/disable; do
        if [ -f "$state" ]; then
            echo 1 | sudo tee "$state" > /dev/null 2>&1 || true
        fi
    done
    
    # Optimize scheduler for AI workloads
    cat > /tmp/cpu_ai_config << EOF
# CPU optimizations for AI/LLM workloads
kernel.sched_min_granularity_ns = 1000000
kernel.sched_wakeup_granularity_ns = 2000000
kernel.sched_migration_cost_ns = 500000
kernel.sched_latency_ns = 6000000
kernel.sched_nr_migrate = 8
kernel.sched_rr_timeslice_ms = 25
EOF
    
    sudo mv /tmp/cpu_ai_config /etc/sysctl.d/98-cpu-ai-optimization.conf
    
    log_success "CPU optimized for sustained AI workloads"
}

# Memory optimizations for large models
optimize_memory() {
    log_info "Configuring memory management for large AI models..."
    
    cat > /tmp/memory_ai_config << EOF
# Memory optimizations for LLM workloads
vm.swappiness = 1
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5
vm.dirty_expire_centisecs = 3000
vm.dirty_writeback_centisecs = 500
vm.overcommit_memory = 1
vm.overcommit_ratio = 90
vm.min_free_kbytes = 262144
vm.zone_reclaim_mode = 0
vm.vfs_cache_pressure = 50

# NUMA balancing for AI workloads
kernel.numa_balancing = 1
kernel.numa_balancing_scan_delay_ms = 1000
kernel.numa_balancing_scan_period_min_ms = 1000
kernel.numa_balancing_scan_period_max_ms = 60000
kernel.numa_balancing_scan_size_mb = 256

# Large memory allocation support
vm.max_map_count = 2147483647
EOF
    
    sudo mv /tmp/memory_ai_config /etc/sysctl.d/97-memory-ai-optimization.conf
    
    log_success "Memory management optimized for large AI models"
}

# Network optimizations for model serving
optimize_network() {
    log_info "Optimizing network stack for AI model serving..."
    
    cat > /tmp/network_ai_config << EOF
# Network optimizations for AI model serving
net.core.rmem_default = 262144
net.core.rmem_max = 134217728
net.core.wmem_default = 262144
net.core.wmem_max = 134217728
net.core.netdev_max_backlog = 30000
net.core.netdev_budget = 600
net.ipv4.tcp_rmem = 4096 87380 134217728
net.ipv4.tcp_wmem = 4096 65536 134217728
net.ipv4.tcp_congestion_control = bbr
net.ipv4.tcp_slow_start_after_idle = 0
net.core.busy_read = 50
net.core.busy_poll = 50
EOF
    
    sudo mv /tmp/network_ai_config /etc/sysctl.d/96-network-ai-optimization.conf
    
    log_success "Network stack optimized for model serving"
}

# Install and configure Ollama with optimizations
install_ollama() {
    log_info "Installing and configuring Ollama with performance optimizations..."
    
    if ! command -v ollama &> /dev/null; then
        curl -fsSL https://ollama.ai/install.sh | sh
        log_success "Ollama installed"
    else
        log_info "Ollama already installed, updating..."
        ollama update || true
    fi
    
    # Create optimized Ollama configuration
    sudo mkdir -p /etc/systemd/system/ollama.service.d
    
    cat > /tmp/ollama_override.conf << EOF
[Service]
# Performance optimizations for LLM inference
Environment="OLLAMA_NUM_PARALLEL=4"
Environment="OLLAMA_MAX_LOADED_MODELS=3"
Environment="OLLAMA_MAX_QUEUE=512"
Environment="OLLAMA_FLASH_ATTENTION=1"
Environment="OLLAMA_HOST=0.0.0.0:11434"
Environment="OLLAMA_KEEP_ALIVE=24h"
Environment="OLLAMA_ORIGINS=*"

# Memory optimizations
Environment="OLLAMA_MAX_VRAM=0"
Environment="GOMAXPROCS=${CPU_CORES}"
Environment="GOMEMLIMIT=$((TOTAL_MEMORY_GB * 80 / 100))GiB"

# CPU affinity for better performance
CPUAffinity=0-$((CPU_CORES-1))

# Process priority
Nice=-10
IOSchedulingClass=1
IOSchedulingPriority=4

# Memory locking for better performance
LimitMEMLOCK=infinity
LimitNOFILE=1048576

# Use huge pages if available
Environment="MALLOC_MMAP_THRESHOLD_=131072"
Environment="MALLOC_TRIM_THRESHOLD_=131072"

# Enable all CPU cores for model inference
CPUQuota=
CPUWeight=1000
EOF
    
    sudo mv /tmp/ollama_override.conf /etc/systemd/system/ollama.service.d/override.conf
    
    # Reload and restart Ollama
    sudo systemctl daemon-reload
    sudo systemctl enable ollama
    sudo systemctl restart ollama
    
    log_success "Ollama configured with performance optimizations"
}

# Create LLM performance monitoring script
create_monitoring() {
    log_info "Creating LLM performance monitoring script..."
    
    cat > "$HOME/monitor-llm-performance.sh" << 'EOF'
#!/bin/bash
echo "🧠 LLM Performance Monitor"
echo "=========================="
echo "Timestamp: $(date)"
echo ""

echo "=== System Resources ==="
echo "Memory Usage:"
free -h | grep -E "(Mem|Swap)"
echo ""

echo "CPU Frequency:"
cat /proc/cpuinfo | grep MHz | head -4
echo ""

echo "CPU Temperature:"
sensors 2>/dev/null | grep -E "(Core|temp)" | head -8 || echo "sensors not available"
echo ""

echo "=== Huge Pages Status ==="
echo "Huge Pages Total: $(cat /proc/meminfo | grep HugePages_Total | awk '{print $2}')"
echo "Huge Pages Free: $(cat /proc/meminfo | grep HugePages_Free | awk '{print $2}')"
echo "Huge Page Size: $(cat /proc/meminfo | grep Hugepagesize | awk '{print $2 $3}')"
echo ""

echo "=== Memory Allocation ==="
echo "Available Memory: $(cat /proc/meminfo | grep MemAvailable | awk '{print $2/1024/1024 " GB"}')"
echo "Cache Memory: $(cat /proc/meminfo | grep ^Cached | awk '{print $2/1024/1024 " GB"}')"
echo "Buffer Memory: $(cat /proc/meminfo | grep Buffers | awk '{print $2/1024/1024 " GB"}')"
echo ""

echo "=== Ollama Status ==="
if systemctl is-active ollama >/dev/null 2>&1; then
    echo "Ollama Service: Running"
    echo "Ollama Process:"
    ps aux | grep ollama | grep -v grep | head -3
    echo ""
    
    # Show loaded models if ollama is responding
    timeout 5 ollama list 2>/dev/null && echo "" || echo "Ollama not responding to commands"
else
    echo "Ollama Service: Not running"
fi

echo "=== GPU Status (if available) ==="
nvidia-smi 2>/dev/null | head -20 || echo "NVIDIA GPU not available"
echo ""

echo "=== Network Connections ==="
ss -tuln | grep :11434 || echo "Ollama port not listening"
echo ""

echo "=== Recent System Load ==="
uptime
echo ""
iostat -x 1 1 2>/dev/null | tail -10 || echo "iostat not available"
EOF
    
    chmod +x "$HOME/monitor-llm-performance.sh"
    
    log_success "LLM monitoring script created: ~/monitor-llm-performance.sh"
}

# Create model management helper
create_model_helper() {
    log_info "Creating model management helper script..."
    
    cat > "$HOME/manage-llm-models.sh" << 'EOF'
#!/bin/bash
# LLM Model Management Helper

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }

echo "🤖 LLM Model Management Helper"
echo "=============================="

# Check available memory
AVAILABLE_GB=$(free -g | awk '/^Mem:/{print $7}')
log_info "Available memory: ${AVAILABLE_GB}GB"

case "$1" in
    "install-recommended")
        echo "Installing recommended models based on your 64GB RAM:"
        echo ""
        
        if [ "$AVAILABLE_GB" -gt 40 ]; then
            log_info "Installing large models (40GB+ RAM available)..."
            ollama pull llama3.1:70b-instruct-q4_0  # ~40GB
            ollama pull codellama:34b-instruct-q4_0  # ~20GB
            ollama pull mistral-nemo:12b-instruct-2407  # ~7GB
        elif [ "$AVAILABLE_GB" -gt 25 ]; then
            log_info "Installing medium models (25-40GB RAM available)..."
            ollama pull llama3.1:8b-instruct-q6_K  # ~6GB
            ollama pull codellama:13b-instruct-q4_0  # ~8GB
            ollama pull mistral:7b-instruct-q4_0  # ~4GB
        else
            log_info "Installing smaller models (<25GB RAM available)..."
            ollama pull llama3.1:8b-instruct-q4_0  # ~5GB
            ollama pull codellama:7b-instruct-q4_0  # ~4GB
            ollama pull mistral:7b-instruct-q4_0  # ~4GB
        fi
        ;;
    
    "list-sizes")
        echo "Model size estimates:"
        echo "llama3.1:70b-instruct-q4_0  ~40GB (best quality, needs 50GB+ RAM)"
        echo "llama3.1:8b-instruct-q6_K   ~6GB (good quality)"
        echo "llama3.1:8b-instruct-q4_0   ~5GB (balanced)"
        echo "codellama:34b-instruct-q4_0  ~20GB (large coding model)"
        echo "codellama:13b-instruct-q4_0  ~8GB (medium coding model)"
        echo "codellama:7b-instruct-q4_0   ~4GB (small coding model)"
        echo "mistral-nemo:12b-instruct    ~7GB (good general model)"
        echo "mistral:7b-instruct-q4_0     ~4GB (fast inference)"
        ;;
    
    "benchmark")
        log_info "Running LLM inference benchmark..."
        echo "Testing with small prompt on available models..."
        
        for model in $(ollama list | tail -n +2 | awk '{print $1}' | grep -v '^$'); do
            if [ "$model" != "NAME" ]; then
                echo ""
                log_info "Benchmarking $model..."
                time timeout 30 ollama run "$model" "Write a hello world program in Python" 2>/dev/null || echo "Model failed or timed out"
            fi
        done
        ;;
    
    *)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  install-recommended  Install models optimized for your RAM"
        echo "  list-sizes          Show model memory requirements"
        echo "  benchmark           Test inference speed of installed models"
        echo ""
        echo "Examples:"
        echo "  $0 install-recommended"
        echo "  $0 benchmark"
        ;;
esac
EOF
    
    chmod +x "$HOME/manage-llm-models.sh"
    
    log_success "Model management script created: ~/manage-llm-models.sh"
}

# Apply all optimizations
apply_optimizations() {
    log_info "Applying all system optimizations..."
    
    # Apply sysctl settings
    sudo sysctl -p /etc/sysctl.d/99-hugepages-llm.conf 2>/dev/null || true
    sudo sysctl -p /etc/sysctl.d/98-cpu-ai-optimization.conf 2>/dev/null || true
    sudo sysctl -p /etc/sysctl.d/97-memory-ai-optimization.conf 2>/dev/null || true
    sudo sysctl -p /etc/sysctl.d/96-network-ai-optimization.conf 2>/dev/null || true
    
    log_success "System optimizations applied"
}

# Install additional AI/ML tools
install_ai_tools() {
    log_info "Installing additional AI/ML development tools..."
    
    # Install Python AI/ML ecosystem
    sudo pacman -S --needed --noconfirm \
        python-pip \
        python-numpy \
        python-scipy \
        python-pandas \
        python-matplotlib \
        python-scikit-learn \
        python-pytorch-cuda \
        python-tensorflow \
        jupyter-notebook \
        git-lfs \
        htop \
        iotop \
        nvtop \
        stress-ng
    
    # Install additional monitoring tools
    if ! command -v bpytop &> /dev/null; then
        pip install --user bpytop gpustat
    fi
    
    log_success "AI/ML tools installed"
}

# Main execution
main() {
    echo "🚀 Optimizing system for Ollama and LLM workloads..."
    echo "Detected: Intel i9-13900HX with ${CPU_CORES} cores and ${TOTAL_MEMORY_GB}GB RAM"
    echo ""
    
    read -p "Proceed with full system optimization for LLM performance? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Optimization cancelled"
        exit 0
    fi
    
    optimize_hugepages
    optimize_cpu
    optimize_memory
    optimize_network
    install_ollama
    install_ai_tools
    create_monitoring
    create_model_helper
    apply_optimizations
    
    log_success "🎉 LLM system optimization complete!"
    echo ""
    echo "==========================================="
    echo "🎯 Next Steps:"
    echo "1. Reboot to apply all kernel optimizations"
    echo "2. Run ~/manage-llm-models.sh install-recommended"
    echo "3. Use ~/monitor-llm-performance.sh to monitor performance"
    echo "4. Test with: ollama run llama3.1:8b-instruct"
    echo ""
    echo "🔧 Your system is now optimized for:"
    echo "  • 🧠 Large Language Model inference (up to 70B parameters)"
    echo "  • 🚀 Multi-model serving with Ollama"
    echo "  • 💾 Efficient memory usage with huge pages"
    echo "  • ⚡ Maximum CPU performance for AI workloads"
    echo "  • 🌐 High-performance model serving"
    echo "==========================================="
}

main "$@"
