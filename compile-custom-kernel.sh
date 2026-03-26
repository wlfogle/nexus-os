#!/bin/bash
# Custom Kernel Compilation Script for i9-13900HX
# Optimized for virtualization, AI programming, and self-hosting

set -e

echo "🔥 Custom Kernel Compilation for Intel i9-13900HX"
echo "=================================================="

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

# System specifications detected - Auto-detect for accuracy
CPU_CORES=$(nproc --all)
CPU_THREADS=$(lscpu | awk '/^Thread/ {print $4}')
MAKE_JOBS=$((CPU_CORES + 4))  # Use more jobs than cores for optimal compilation

# Memory-aware compilation adjustment
TOTAL_MEMORY_GB=$(free -g | awk '/^Mem:/{print $2}')
if [ "$TOTAL_MEMORY_GB" -ge 32 ]; then
    MAKE_JOBS=$((CPU_CORES * 2))  # With 64GB RAM, we can be more aggressive
fi
KERNEL_BUILD_DIR="$HOME/kernel-build"
KERNEL_VERSION="6.15"

log_info "System detected: Intel i9-13900HX with ${CPU_CORES} cores, ${CPU_THREADS} threads"
log_info "Using ${MAKE_JOBS} parallel jobs for compilation"

# Install build dependencies
install_dependencies() {
    log_info "Installing kernel build dependencies..."
    
    sudo pacman -S --needed --noconfirm \
        base-devel \
        bc \
        cpio \
        gettext \
        libelf \
        pahole \
        perl \
        python \
        rsync \
        tar \
        xz \
        zstd \
        git \
        xmlto \
        kmod \
        inetutils \
        bison \
        flex \
        coreutils \
        util-linux \
        arch-install-scripts
    
    log_success "Dependencies installed"
}

# Download kernel source
download_kernel() {
    log_info "Downloading latest stable kernel source..."
    
    mkdir -p "$KERNEL_BUILD_DIR"
    cd "$KERNEL_BUILD_DIR"
    
    # Get latest stable kernel
    if [ ! -d "linux-stable" ]; then
        git clone --depth=1 https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git linux-stable
    else
        cd linux-stable
        git pull
        cd ..
    fi
    
    log_success "Kernel source ready"
}

# Create optimized kernel configuration
create_custom_config() {
    log_info "Creating custom kernel configuration for i9-13900HX..."
    
    cd "$KERNEL_BUILD_DIR/linux-stable"
    
    # Start with current running kernel config
    if [ -f "/proc/config.gz" ]; then
        zcat /proc/config.gz > .config
        log_info "Using current kernel config as base"
    elif [ -f "/boot/config-$(uname -r)" ]; then
        cp /boot/config-$(uname -r) .config
        log_info "Using boot config as base"
    else
        make defconfig
        log_info "Using default config as base"
    fi
    
    # Create custom configuration optimized for gaming, virtualization, coding, and AI
    cat >> .config << 'EOF'

# ===== GAMING OPTIMIZATIONS =====
# Low-latency CPU scheduler for gaming
CONFIG_PREEMPT=y
CONFIG_PREEMPT_COUNT=y
CONFIG_PREEMPTION=y
CONFIG_PREEMPT_DYNAMIC=y
# CONFIG_PREEMPT_NONE is not set
# CONFIG_PREEMPT_VOLUNTARY is not set

# High-resolution timers for smooth gaming
CONFIG_HIGH_RES_TIMERS=y
CONFIG_GENERIC_CLOCKEVENTS=y
CONFIG_GENERIC_CLOCKEVENTS_BROADCAST=y

# Tickless system for better gaming performance
CONFIG_NO_HZ=y
CONFIG_NO_HZ_IDLE=y
CONFIG_NO_HZ_FULL=y

# ===== CPU OPTIMIZATIONS FOR i9-13900HX =====
CONFIG_PROCESSOR_SELECT=y
CONFIG_CPU_SUP_INTEL=y
CONFIG_X86_INTEL_PSTATE=y
CONFIG_X86_P4_CLOCKMOD=y

# Performance CPU Governor (for better performance)
CONFIG_CPU_FREQ_DEFAULT_GOV_PERFORMANCE=y
CONFIG_CPU_FREQ_GOV_PERFORMANCE=y
CONFIG_CPU_FREQ_GOV_ONDEMAND=y
CONFIG_CPU_FREQ_GOV_CONSERVATIVE=y
CONFIG_CPU_FREQ_GOV_SCHEDUTIL=y

# Intel Turbo Boost and Advanced CPU Features
CONFIG_X86_INTEL_PSTATE=y
CONFIG_X86_INTEL_LPSS=y
CONFIG_X86_INTEL_TURBO=y

# Enable advanced Intel features for 13th gen
CONFIG_X86_INTEL_UMIP=y
CONFIG_X86_INTEL_MPX=y

# Virtualization optimizations (for your VMs)
CONFIG_VIRTUALIZATION=y
CONFIG_KVM=y
CONFIG_KVM_INTEL=y
CONFIG_VHOST_NET=y
CONFIG_VHOST_VSOCK=y
CONFIG_VHOST=y
CONFIG_VHOST_MENU=y
CONFIG_VFIO=y
CONFIG_VFIO_IOMMU_TYPE1=y
CONFIG_VFIO_PCI=y

# Memory optimizations for 64GB RAM
CONFIG_HIGHMEM64G=y
CONFIG_X86_64=y
CONFIG_ZONE_DMA32=y
CONFIG_BOUNCE=y
CONFIG_NR_CPUS=48

# Network optimizations for homelab
CONFIG_NET=y
CONFIG_INET=y
CONFIG_IP_MULTICAST=y
CONFIG_IP_ADVANCED_ROUTER=y
CONFIG_NETFILTER=y
CONFIG_NETFILTER_ADVANCED=y
CONFIG_BRIDGE=y
CONFIG_VLAN_8021Q=y

# Storage optimizations for NVMe and high-performance I/O
CONFIG_BLK_DEV_NVME=y
CONFIG_NVME_CORE=y
CONFIG_NVME_MULTIPATH=y
CONFIG_NVME_HWMON=y
CONFIG_BLK_CGROUP=y
CONFIG_BLK_DEV_THROTTLING=y
CONFIG_BLK_WBT=y
CONFIG_BLK_WBT_MQ=y

# I/O schedulers for performance
CONFIG_IOSCHED_DEADLINE=y
CONFIG_IOSCHED_CFQ=y
CONFIG_MQ_IOSCHED_DEADLINE=y
CONFIG_MQ_IOSCHED_KYBER=y
CONFIG_DEFAULT_IOSCHED="mq-deadline"

# Container support (for your awesome-stack)
CONFIG_NAMESPACES=y
CONFIG_UTS_NS=y
CONFIG_IPC_NS=y
CONFIG_PID_NS=y
CONFIG_NET_NS=y
CONFIG_CGROUPS=y
CONFIG_CGROUP_CPUACCT=y
CONFIG_CGROUP_DEVICE=y
CONFIG_CGROUP_FREEZER=y
CONFIG_CGROUP_SCHED=y
CONFIG_CPUSETS=y
CONFIG_MEMCG=y

# Disable unnecessary features for performance
# CONFIG_DEBUG_KERNEL is not set
# CONFIG_DEBUG_INFO is not set
# CONFIG_KPROBES is not set
# CONFIG_FTRACE is not set

# Enable performance features
CONFIG_NO_HZ=y
CONFIG_NO_HZ_IDLE=y
CONFIG_HIGH_RES_TIMERS=y
CONFIG_GENERIC_CLOCKEVENTS_BROADCAST=y

# Intel graphics (for your integrated GPU)
CONFIG_DRM=y
CONFIG_DRM_I915=y
CONFIG_DRM_I915_GVT=y

# Audio optimizations
CONFIG_SND=y
CONFIG_SND_HDA_INTEL=y
CONFIG_SND_HDA_CODEC_REALTEK=y

# Wireless (if needed)
CONFIG_CFG80211=y
CONFIG_MAC80211=y

# USB support
CONFIG_USB_SUPPORT=y
CONFIG_USB=y
CONFIG_USB_XHCI_HCD=y

# Enable specific optimizations for 13th gen Intel
CONFIG_X86_INTEL_TSX_MODE_OFF=n
CONFIG_X86_INTEL_TSX_MODE_ON=y
CONFIG_X86_INTEL_TSX_MODE_AUTO=y

# ===== LLM AND AI/ML OPTIMIZATIONS =====
# Large memory page support for LLM inference
CONFIG_HUGETLBFS=y
CONFIG_HUGETLB_PAGE=y
CONFIG_TRANSPARENT_HUGEPAGE=y
CONFIG_TRANSPARENT_HUGEPAGE_ALWAYS=y
CONFIG_TRANSPARENT_HUGEPAGE_MADVISE=y
CONFIG_COMPACTION=y
CONFIG_MIGRATION=y

# Memory management for large models (70B+ parameters)
CONFIG_MEMORY_HOTPLUG=y
CONFIG_MEMORY_HOTREMOVE=y
CONFIG_SPARSEMEM_VMEMMAP=y
CONFIG_SPARSEMEM=y
CONFIG_FLATMEM=n

# NUMA optimizations for multi-socket AI workloads
CONFIG_NUMA=y
CONFIG_NUMA_BALANCING=y
CONFIG_NUMA_BALANCING_DEFAULT_ENABLED=y

# CPU features essential for AI workloads
CONFIG_X86_FEATURE_NAMES=y
CONFIG_X86_INTEL_PSTATE=y
CONFIG_X86_AMD_PSTATE=y
CONFIG_CPU_FREQ_STAT=y

# Advanced vector extensions for AI computations
CONFIG_X86_INTEL_MPX=y
CONFIG_X86_INTEL_CET=y
CONFIG_X86_INTEL_MEMORY_PROTECTION_KEYS=y

# Enhanced scheduler for AI workloads
CONFIG_SCHED_SMT=y
CONFIG_SCHED_MC=y
CONFIG_SCHED_MC_PRIO=y
CONFIG_SCHED_AUTOGROUP=y

# Memory bandwidth optimization
CONFIG_X86_INTEL_RAPL=y
CONFIG_INTEL_RAPL_CORE=y
CONFIG_INTEL_RAPL_PKG=y
CONFIG_INTEL_RAPL_RAM=y

# Performance monitoring for AI workloads
CONFIG_PERF_EVENTS=y
CONFIG_PERF_EVENTS_INTEL_UNCORE=y
CONFIG_PERF_EVENTS_INTEL_RAPL=y
CONFIG_PERF_EVENTS_INTEL_CSTATE=y

# Enhanced swap for large model loading
CONFIG_SWAP=y
CONFIG_ZSWAP=y
CONFIG_ZSWAP_DEFAULT_ON=y
CONFIG_ZPOOL=y
CONFIG_ZBUD=y
CONFIG_Z3FOLD=y
CONFIG_ZSMALLOC=y

# Low-latency features for real-time AI inference
CONFIG_PREEMPT_RT=y
CONFIG_IRQ_FORCED_THREADING=y
CONFIG_IRQ_ALL_CPUS=y

# GPU compute support (for GPU-accelerated LLMs)
CONFIG_DRM=y
CONFIG_DRM_I915=y
CONFIG_DRM_I915_GVT=y
CONFIG_DRM_I915_USERPTR=y
CONFIG_DRM_I915_GVT_KVMGT=y

# Enhanced power management for sustained AI workloads
CONFIG_PM=y
CONFIG_PM_ADVANCED_DEBUG=y
CONFIG_PM_SLEEP=y
CONFIG_PM_AUTOSLEEP=y
CONFIG_CPU_IDLE=y
CONFIG_CPU_IDLE_GOV_LADDER=y
CONFIG_CPU_IDLE_GOV_MENU=y

# Network optimizations for distributed AI (model serving)
CONFIG_NET_RX_BUSY_POLL=y
CONFIG_NET_BUSY_POLL=y
CONFIG_BQL=y
CONFIG_NET_FLOW_LIMIT=y

# Security features for AI model protection
CONFIG_SECURITY=y
CONFIG_SECURITYFS=y
CONFIG_SECURITY_NETWORK=y
CONFIG_SECURITY_CAPABILITIES=y
CONFIG_SECURITY_APPARMOR=y

EOF

    # Apply optimizations and resolve dependencies
    make olddefconfig
    
    # Enable additional performance optimizations through menuconfig-style tweaks
    log_info "Applying final performance optimizations..."
    
    # Set optimal values for high-core-count systems
    scripts/config --set-val NR_CPUS 48
    scripts/config --set-val LOG_CPU_MAX_BUF_SHIFT 12
    
    # Enable performance governors
    scripts/config --enable CPU_FREQ_DEFAULT_GOV_PERFORMANCE
    
    # Optimize for Intel 13th gen
    scripts/config --enable X86_INTEL_PSTATE
    scripts/config --enable INTEL_RAPL
    
    # Make sure we have optimal memory management
    scripts/config --enable TRANSPARENT_HUGEPAGE
    scripts/config --enable TRANSPARENT_HUGEPAGE_ALWAYS
    
    # Re-run olddefconfig to ensure consistency
    make olddefconfig
    
    log_success "Custom configuration created"
}

# Compile kernel
compile_kernel() {
    log_info "Starting kernel compilation with ${MAKE_JOBS} parallel jobs..."
    log_warning "This will take 15-30 minutes on your i9-13900HX..."
    
    cd "$KERNEL_BUILD_DIR/linux-stable"
    
    # Set compilation optimizations
    export CFLAGS="-O3 -march=native -mtune=native"
    export KCFLAGS="-O3 -march=native"
    
    # Show estimated time
    log_info "Estimated compilation time with ${CPU_CORES} cores and ${TOTAL_MEMORY_GB}GB RAM: 12-25 minutes"
    
    # Compile kernel with optimizations
    log_info "Phase 1: Compiling kernel image..."
    time make -j${MAKE_JOBS} LOCALVERSION="-custom-$(date +%Y%m%d)"
    
    # Compile modules
    log_info "Phase 2: Compiling kernel modules..."
    time make -j${MAKE_JOBS} modules
    
    log_success "Kernel compilation completed!"
    log_info "Kernel size: $(du -h arch/x86/boot/bzImage | cut -f1)"
}

# Install kernel
install_kernel() {
    log_info "Installing custom kernel..."
    
    cd "$KERNEL_BUILD_DIR/linux-stable"
    
    # Install modules
    sudo make modules_install
    
    # Install kernel
    sudo make install
    
    # Update bootloader
    sudo grub-mkconfig -o /boot/grub/grub.cfg
    
    log_success "Custom kernel installed!"
    
    # Show kernel info
    KERNEL_VERSION=$(make kernelversion)
    log_info "Custom kernel version: ${KERNEL_VERSION}"
    log_info "Kernel image: /boot/vmlinuz-${KERNEL_VERSION}"
    log_info "Modules: /lib/modules/${KERNEL_VERSION}"
}

# Create benchmark script
create_benchmark() {
    log_info "Creating kernel performance benchmark script..."
    
    cat > "$HOME/benchmark-custom-kernel.sh" << 'EOF'
#!/bin/bash
echo "🚀 Custom Kernel Performance Benchmark"
echo "======================================"

echo "Kernel Version: $(uname -r)"
echo "CPU Info: $(lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
echo "Memory: $(free -h | grep Mem | awk '{print $2}')"
echo ""

echo "=== CPU Performance Test ==="
echo "Running 30-second CPU stress test..."
timeout 30 stress-ng --cpu $(nproc) --metrics-brief

echo -e "\n=== Memory Performance Test ==="
echo "Running memory benchmark..."
timeout 15 stress-ng --vm 4 --vm-bytes 1G --metrics-brief

echo -e "\n=== Disk I/O Test ==="
echo "Running disk performance test..."
dd if=/dev/zero of=/tmp/testfile bs=1G count=1 conv=fdatasync 2>&1 | grep -E "(copied|MB/s|GB/s)"
rm -f /tmp/testfile

echo -e "\n=== Network Stack Test ==="
echo "Testing network performance..."
iperf3 -s -1 -p 5201 > /dev/null 2>&1 &
sleep 1
iperf3 -c localhost -p 5201 -t 10 2>/dev/null | grep -E "(sender|receiver)"

echo -e "\n=== Virtualization Test ==="
if command -v kvm-ok > /dev/null 2>&1; then
    kvm-ok
else
    echo "KVM acceleration: $([ -r /dev/kvm ] && echo 'Available' || echo 'Not available')"
fi

echo -e "\n=== System Load ==="
uptime
EOF

    chmod +x "$HOME/benchmark-custom-kernel.sh"
    
    log_success "Benchmark script created: ~/benchmark-custom-kernel.sh"
}

# Main execution
main() {
    echo "Starting custom kernel compilation for your i9-13900HX system..."
    echo ""
    echo "This kernel will be optimized for:"
    echo "  🖥️  Intel i9-13900HX (24 cores, 48 threads)"
    echo "  🚀 Virtualization and KVM"
    echo "  🐳 Container workloads (Docker/LXC)"
    echo "  🧠 AI/ML workloads"
    echo "  🏠 Self-hosting applications"
    echo "  💾 64GB RAM optimization"
    echo "  💿 NVMe storage optimization"
    echo ""
    
    read -p "Continue with custom kernel compilation? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Kernel compilation cancelled"
        exit 0
    fi
    
    install_dependencies
    download_kernel
    create_custom_config
    compile_kernel
    install_kernel
    create_benchmark
    
    log_success "🎉 Custom kernel compilation complete!"
    echo ""
    echo "=========================================="
    echo "🎯 Next Steps:"
    echo "1. Reboot to use the new custom kernel"
    echo "2. Run ~/benchmark-custom-kernel.sh to test performance"
    echo "3. Monitor system stability for a few days"
    echo "4. If issues occur, select the previous kernel from GRUB menu"
    echo ""
    echo "🔧 Your custom kernel is optimized for:"
    echo "  • 🎮 Low-latency gaming performance"
    echo "  • 🚀 Maximum virtualization performance (GPU passthrough ready)"
    echo "  • 💻 Development workflow optimization"
    echo "  • 🧠 AI/ML workload acceleration"
    echo "  • 🏠 Self-hosting infrastructure efficiency"
    echo "  • 💾 64GB RAM and NVMe optimization"
    echo "=========================================="
}

main "$@"
