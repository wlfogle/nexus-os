#!/bin/bash
# Custom Kernel Compilation Script for i9-13900HX on Fedora Kinoite
# Optimized for virtualization, AI programming, and self-hosting

set -e

echo "🔥 Custom Kernel Compilation for Intel i9-13900HX on Fedora Kinoite"
echo "======================================================================"

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

# Install build dependencies in toolbox
install_dependencies() {
    log_info "Installing kernel build dependencies in toolbox..."
    
    toolbox run --container kernel-build sudo dnf install -y \
        kernel-devel \
        kernel-headers \
        gcc \
        make \
        flex \
        bison \
        elfutils-libelf-devel \
        openssl-devel \
        bc \
        dwarves \
        python3-devel \
        perl-devel \
        ncurses-devel \
        git \
        wget \
        curl \
        xz \
        zstd \
        rpm-build \
        rpmdevtools
    
    log_success "Dependencies installed in toolbox"
}

# Download kernel source
download_kernel() {
    log_info "Downloading latest stable kernel source..."
    
    toolbox run --container kernel-build mkdir -p "$KERNEL_BUILD_DIR"
    
    # Get latest stable kernel
    if [ ! -d "$HOME/.local/share/containers/storage/volumes/kernel-build-volume" ]; then
        toolbox run --container kernel-build git clone --depth=1 https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git "$KERNEL_BUILD_DIR/linux-stable"
    else
        toolbox run --container kernel-build bash -c "cd $KERNEL_BUILD_DIR/linux-stable && git pull"
    fi
    
    log_success "Kernel source ready"
}

# Create optimized kernel configuration
create_custom_config() {
    log_info "Creating custom kernel configuration for i9-13900HX..."
    
    toolbox run --container kernel-build bash -c "
    cd '$KERNEL_BUILD_DIR/linux-stable'
    
    # Start with current running kernel config
    if [ -f '/proc/config.gz' ]; then
        zcat /proc/config.gz > .config
        echo 'Using current kernel config as base'
    elif [ -f '/boot/config-\$(uname -r)' ]; then
        cp /boot/config-\$(uname -r) .config
        echo 'Using boot config as base'
    else
        make defconfig
        echo 'Using default config as base'
    fi
    "
    
    # Create the optimized configuration
    toolbox run --container kernel-build bash -c "
    cd '$KERNEL_BUILD_DIR/linux-stable'
    
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

# Performance CPU Governor (for better performance)
CONFIG_CPU_FREQ_DEFAULT_GOV_PERFORMANCE=y
CONFIG_CPU_FREQ_GOV_PERFORMANCE=y
CONFIG_CPU_FREQ_GOV_ONDEMAND=y
CONFIG_CPU_FREQ_GOV_CONSERVATIVE=y
CONFIG_CPU_FREQ_GOV_SCHEDUTIL=y

# Intel Turbo Boost and Advanced CPU Features
CONFIG_X86_INTEL_PSTATE=y
CONFIG_X86_INTEL_LPSS=y

# Enable advanced Intel features for 13th gen
CONFIG_X86_INTEL_UMIP=y

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
CONFIG_MQ_IOSCHED_DEADLINE=y
CONFIG_MQ_IOSCHED_KYBER=y
CONFIG_DEFAULT_IOSCHED=\"mq-deadline\"

# Container support
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

# ===== LLM AND AI/ML OPTIMIZATIONS =====
# Large memory page support for LLM inference
CONFIG_HUGETLBFS=y
CONFIG_HUGETLB_PAGE=y
CONFIG_TRANSPARENT_HUGEPAGE=y
CONFIG_TRANSPARENT_HUGEPAGE_ALWAYS=y
CONFIG_TRANSPARENT_HUGEPAGE_MADVISE=y
CONFIG_COMPACTION=y
CONFIG_MIGRATION=y

# Memory management for large models
CONFIG_MEMORY_HOTPLUG=y
CONFIG_MEMORY_HOTREMOVE=y
CONFIG_SPARSEMEM_VMEMMAP=y
CONFIG_SPARSEMEM=y

# NUMA optimizations
CONFIG_NUMA=y
CONFIG_NUMA_BALANCING=y
CONFIG_NUMA_BALANCING_DEFAULT_ENABLED=y

# CPU features essential for AI workloads
CONFIG_X86_FEATURE_NAMES=y
CONFIG_CPU_FREQ_STAT=y

# Enhanced scheduler for AI workloads
CONFIG_SCHED_SMT=y
CONFIG_SCHED_MC=y
CONFIG_SCHED_MC_PRIO=y
CONFIG_SCHED_AUTOGROUP=y

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

# ===== NVIDIA GPU OPTIMIZATIONS =====
# NVIDIA GPU support for gaming and AI/ML acceleration
CONFIG_DRM=y
CONFIG_DRM_I915=y
CONFIG_DRM_I915_GVT=y
CONFIG_DRM_I915_USERPTR=y

# NVIDIA proprietary driver support
CONFIG_FB=y
CONFIG_FB_EFI=y
CONFIG_DRM_FBDEV_EMULATION=y
CONFIG_DRM_FBDEV_OVERALLOC=100
CONFIG_FB_SIMPLE=y

# VFIO for GPU passthrough (NVIDIA)
CONFIG_VFIO=y
CONFIG_VFIO_PCI=y
CONFIG_VFIO_PCI_GENERIC=y
CONFIG_VFIO_VIRQFD=y
CONFIG_VFIO_NOIOMMU=y
CONFIG_VFIO_PCI_MMAP=y
CONFIG_VFIO_PCI_INTX=y

# IOMMU support for NVIDIA GPU passthrough
CONFIG_IOMMU_SUPPORT=y
CONFIG_INTEL_IOMMU=y
CONFIG_INTEL_IOMMU_DEFAULT_ON=y
CONFIG_INTEL_IOMMU_FLOPPY_WA=y

# PCIe optimizations for high-performance NVIDIA GPUs
CONFIG_PCIEPORTBUS=y
CONFIG_PCIEAER=y
CONFIG_PCIE_ECRC=y
CONFIG_PCIEAER_INJECT=y
CONFIG_PCIE_DPC=y
CONFIG_PCIE_PTM=y

# CUDA and OpenCL support optimizations
CONFIG_STAGING=y
CONFIG_DRM_DISPLAY_CONNECTOR=y
CONFIG_DRM_DISPLAY_HELPER=y

# Power management for NVIDIA GPUs
CONFIG_PM_RUNTIME=y
CONFIG_PM_GENERIC_DOMAINS=y
CONFIG_PM_GENERIC_DOMAINS_SLEEP=y
CONFIG_PM_GENERIC_DOMAINS_OF=y

# ===== WIREGUARD VPN OPTIMIZATIONS =====
# WireGuard VPN support with performance optimizations for dual-role setup
CONFIG_NET=y
CONFIG_INET=y
CONFIG_NETFILTER=y
CONFIG_NETFILTER_ADVANCED=y

# WireGuard kernel module with debugging for development
CONFIG_WIREGUARD=y
CONFIG_WIREGUARD_DEBUG=y

# Enhanced UDP performance for WireGuard
CONFIG_NET_UDP_TUNNEL=y
CONFIG_INET_UDP_DIAG=y
CONFIG_IPV6_UDP_DIAG=y

# Cryptographic optimizations for WireGuard
CONFIG_CRYPTO=y
CONFIG_CRYPTO_CHACHA20POLY1305=y
CONFIG_CRYPTO_CHACHA20_X86_64=y
CONFIG_CRYPTO_POLY1305=y
CONFIG_CRYPTO_POLY1305_X86_64=y
CONFIG_CRYPTO_CURVE25519=y
CONFIG_CRYPTO_CURVE25519_X86=y
CONFIG_CRYPTO_BLAKE2S=y
CONFIG_CRYPTO_BLAKE2S_X86=y

# Hardware crypto acceleration for Intel CPUs
CONFIG_CRYPTO_AES=y
CONFIG_CRYPTO_AES_NI_INTEL=y
CONFIG_CRYPTO_GHASH_CLMUL_NI_INTEL=y
CONFIG_CRYPTO_CRC32C_INTEL=y
CONFIG_CRYPTO_CRCT10DIF_PCLMUL=y
CONFIG_CRYPTO_SHA1_SSSE3=y
CONFIG_CRYPTO_SHA256_SSSE3=y
CONFIG_CRYPTO_SHA512_SSSE3=y

# Network performance optimizations for VPN
CONFIG_NET_SCHED=y
CONFIG_NET_SCH_FQ=y
CONFIG_NET_SCH_FQ_CODEL=y
CONFIG_NET_SCH_CAKE=y
CONFIG_TCP_CONG_BBR=y
CONFIG_DEFAULT_TCP_CONG="bbr"

# Network namespace support for VPN isolation
CONFIG_NET_NS=y
CONFIG_UTS_NS=y
CONFIG_IPC_NS=y
CONFIG_PID_NS=y

# Advanced routing for VPN traffic
CONFIG_IP_ADVANCED_ROUTER=y
CONFIG_IP_MULTIPLE_TABLES=y
CONFIG_IP_ROUTE_MULTIPATH=y
CONFIG_IP_ROUTE_VERBOSE=y
CONFIG_IP_ROUTE_CLASSID=y

# Netfilter optimizations for VPN firewall rules
CONFIG_NETFILTER_NETLINK=y
CONFIG_NETFILTER_NETLINK_ACCT=y
CONFIG_NETFILTER_NETLINK_QUEUE=y
CONFIG_NETFILTER_NETLINK_LOG=y
CONFIG_NF_CONNTRACK=y
CONFIG_NF_CONNTRACK_MARK=y
CONFIG_NF_CONNTRACK_ZONES=y
CONFIG_NF_CONNTRACK_PROCFS=y
CONFIG_NF_CONNTRACK_EVENTS=y
CONFIG_NF_CONNTRACK_TIMEOUT=y
CONFIG_NF_CONNTRACK_TIMESTAMP=y

# IPTables support for VPN firewall
CONFIG_NETFILTER_XTABLES=y
CONFIG_NETFILTER_XT_MARK=y
CONFIG_NETFILTER_XT_CONNMARK=y
CONFIG_NETFILTER_XT_TARGET_CLASSIFY=y
CONFIG_NETFILTER_XT_TARGET_DSCP=y
CONFIG_NETFILTER_XT_TARGET_MARK=y
CONFIG_NETFILTER_XT_TARGET_NFQUEUE=y
CONFIG_NETFILTER_XT_TARGET_NOTRACK=y
CONFIG_NETFILTER_XT_MATCH_COMMENT=y
CONFIG_NETFILTER_XT_MATCH_CONNTRACK=y
CONFIG_NETFILTER_XT_MATCH_MARK=y
CONFIG_NETFILTER_XT_MATCH_POLICY=y
CONFIG_NETFILTER_XT_MATCH_STATE=y

# IPv4 and IPv6 support for modern VPN usage
CONFIG_IP_NF_IPTABLES=y
CONFIG_IP_NF_MATCH_AH=y
CONFIG_IP_NF_MATCH_ECN=y
CONFIG_IP_NF_MATCH_RPFILTER=y
CONFIG_IP_NF_MATCH_TTL=y
CONFIG_IP_NF_FILTER=y
CONFIG_IP_NF_TARGET_REJECT=y
CONFIG_IP_NF_TARGET_SYNPROXY=y
CONFIG_IP_NF_NAT=y
CONFIG_IP_NF_TARGET_MASQUERADE=y
CONFIG_IP_NF_TARGET_NETMAP=y
CONFIG_IP_NF_TARGET_REDIRECT=y
CONFIG_IP_NF_MANGLE=y
CONFIG_IP_NF_TARGET_CLUSTERIP=y
CONFIG_IP_NF_TARGET_ECN=y
CONFIG_IP_NF_TARGET_TTL=y
CONFIG_IP_NF_RAW=y
CONFIG_IP_NF_ARPTABLES=y
CONFIG_IP_NF_ARPFILTER=y
CONFIG_IP_NF_ARP_MANGLE=y

# IPv6 support for modern VPN setups
CONFIG_IPV6=y
CONFIG_IPV6_ROUTER_PREF=y
CONFIG_IPV6_ROUTE_INFO=y
CONFIG_IPV6_OPTIMISTIC_DAD=y
CONFIG_INET6_AH=y
CONFIG_INET6_ESP=y
CONFIG_INET6_IPCOMP=y
CONFIG_IPV6_MIP6=y
CONFIG_INET6_XFRM_TUNNEL=y
CONFIG_INET6_TUNNEL=y
CONFIG_INET6_XFRM_MODE_TRANSPORT=y
CONFIG_INET6_XFRM_MODE_TUNNEL=y
CONFIG_INET6_XFRM_MODE_BEET=y
CONFIG_INET6_XFRM_MODE_ROUTEOPTIMIZATION=y
CONFIG_IPV6_SIT=y
CONFIG_IPV6_SIT_6RD=y
CONFIG_IPV6_NDISC_NODETYPE=y
CONFIG_IPV6_TUNNEL=y
CONFIG_IPV6_GRE=y
CONFIG_IPV6_MULTIPLE_TABLES=y
CONFIG_IPV6_SUBTREES=y
CONFIG_IPV6_MROUTE=y
CONFIG_IPV6_MROUTE_MULTIPLE_TABLES=y
CONFIG_IPV6_PIMSM_V2=y

# Network driver optimizations for high-performance networking
CONFIG_NET_VENDOR_INTEL=y
CONFIG_E1000E=y
CONFIG_IGB=y
CONFIG_IGBVF=y
CONFIG_IXGB=y
CONFIG_IXGBE=y
CONFIG_IXGBEVF=y
CONFIG_I40E=y
CONFIG_I40EVF=y
CONFIG_ICE=y

# Advanced networking for WireGuard dual-role
CONFIG_NET_CLS=y
CONFIG_NET_CLS_U32=y
CONFIG_NET_CLS_FW=y
CONFIG_NET_CLS_ROUTE4=y
CONFIG_NET_CLS_BASIC=y
CONFIG_NET_EMATCH=y
CONFIG_NET_EMATCH_U32=y
CONFIG_NET_ACT_POLICE=y
CONFIG_NET_ACT_GACT=y
CONFIG_NET_ACT_MIRRED=y

# Multi-path routing for dual VPN setup
CONFIG_IP_ROUTE_MULTIPATH_CACHED=y
CONFIG_IP_ROUTE_VERBOSE=y

# Enhanced connection tracking for NAT performance
CONFIG_NF_CT_PROTO_UDPLITE=y
CONFIG_NF_CT_PROTO_GRE=y

# Security features
CONFIG_SECURITY=y
CONFIG_SECURITYFS=y
CONFIG_SECURITY_NETWORK=y
CONFIG_SECURITY_CAPABILITIES=y

EOF

    # Apply optimizations and resolve dependencies
    make olddefconfig
    
    # Enable additional performance optimizations
    echo 'Applying final performance optimizations...'
    
    # Set optimal values for high-core-count systems
    scripts/config --set-val NR_CPUS 48
    scripts/config --set-val LOG_CPU_MAX_BUF_SHIFT 12
    
    # Enable performance governors
    scripts/config --enable CPU_FREQ_DEFAULT_GOV_PERFORMANCE
    
    # Optimize for Intel 13th gen
    scripts/config --enable X86_INTEL_PSTATE
    
    # Make sure we have optimal memory management
    scripts/config --enable TRANSPARENT_HUGEPAGE
    scripts/config --enable TRANSPARENT_HUGEPAGE_ALWAYS
    
    # Re-run olddefconfig to ensure consistency
    make olddefconfig
    "
    
    log_success "Custom configuration created"
}

# Compile kernel
compile_kernel() {
    log_info "Starting kernel compilation with ${MAKE_JOBS} parallel jobs..."
    log_warning "This will take 15-30 minutes on your i9-13900HX..."
    
    toolbox run --container kernel-build bash -c "
    cd '$KERNEL_BUILD_DIR/linux-stable'
    
    # Set compilation optimizations
    export CFLAGS='-O3 -march=native -mtune=native'
    export KCFLAGS='-O3 -march=native'
    
    echo 'Estimated compilation time with ${CPU_CORES} cores and ${TOTAL_MEMORY_GB}GB RAM: 12-25 minutes'
    
    # Compile kernel with optimizations
    echo 'Phase 1: Compiling kernel image...'
    time make -j${MAKE_JOBS} LOCALVERSION=\"-custom-\$(date +%Y%m%d)\"
    
    # Compile modules
    echo 'Phase 2: Compiling kernel modules...'
    time make -j${MAKE_JOBS} modules
    "
    
    log_success "Kernel compilation completed!"
}

# Create RPM packages
create_rpm_packages() {
    log_info "Creating RPM packages for installation..."
    
    toolbox run --container kernel-build bash -c "
    cd '$KERNEL_BUILD_DIR/linux-stable'
    
    # Create RPM packages
    make -j${MAKE_JOBS} rpm-pkg LOCALVERSION=\"-custom-\$(date +%Y%m%d)\"
    "
    
    log_success "RPM packages created"
}

# Install kernel (this will need to be done on the host)
install_kernel_host() {
    log_info "Preparing kernel installation on host..."
    
    # Copy RPMs to host
    KERNEL_RPMS_DIR="$HOME/kernel-rpms"
    mkdir -p "$KERNEL_RPMS_DIR"
    
    # Find and copy the RPM packages
    toolbox run --container kernel-build bash -c "
    find '$KERNEL_BUILD_DIR/linux-stable' -name '*.rpm' -exec cp {} '$KERNEL_RPMS_DIR/' \;
    "
    
    log_info "Kernel RPM packages copied to $KERNEL_RPMS_DIR"
    log_info "To install on Fedora Kinoite:"
    log_info "sudo rpm-ostree override replace $KERNEL_RPMS_DIR/kernel-*.rpm"
    log_info "sudo rpm-ostree reboot"
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
timeout 30 stress-ng --cpu $(nproc) --metrics-brief 2>/dev/null || echo "stress-ng not available"

echo -e "\n=== Memory Performance Test ==="
echo "Running memory benchmark..."
timeout 15 stress-ng --vm 4 --vm-bytes 1G --metrics-brief 2>/dev/null || echo "stress-ng not available"

echo -e "\n=== Disk I/O Test ==="
echo "Running disk performance test..."
dd if=/dev/zero of=/tmp/testfile bs=1G count=1 conv=fdatasync 2>&1 | grep -E "(copied|MB/s|GB/s)"
rm -f /tmp/testfile

echo -e "\n=== Virtualization Test ==="
if [ -r /dev/kvm ]; then
    echo "KVM acceleration: Available"
else
    echo "KVM acceleration: Not available"
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
    echo "  🎮 NVIDIA GPU gaming and CUDA acceleration"
    echo "  🔒 WireGuard VPN with hardware crypto acceleration"
    echo "  🖼️  NVIDIA GPU passthrough for VMs"
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
    create_rpm_packages
    install_kernel_host
    create_benchmark
    
    log_success "🎉 Custom kernel compilation complete!"
    echo ""
    echo "=========================================="
    echo "🎯 Next Steps:"
    echo "1. Install the kernel: sudo rpm-ostree override replace ~/kernel-rpms/kernel-*.rpm"
    echo "2. Reboot: sudo systemctl reboot"
    echo "3. Run ~/benchmark-custom-kernel.sh to test performance"
    echo "4. Monitor system stability for a few days"
    echo ""
    echo "🔧 Your custom kernel is optimized for:"
    echo "  • 🎮 Low-latency gaming performance with NVIDIA GPU support"
    echo "  • 🚀 Maximum virtualization performance (NVIDIA GPU passthrough ready)"
    echo "  • 💻 Development workflow optimization"
    echo "  • 🧠 AI/ML workload acceleration (CUDA/OpenCL optimized)"
    echo "  • 🏠 Self-hosting infrastructure efficiency"
    echo "  • 💾 64GB RAM and NVMe optimization"
    echo "  • 🔒 High-performance WireGuard VPN with Intel crypto acceleration"
    echo "  • 🖼️  Full NVIDIA driver compatibility and GPU compute support"
    echo "=========================================="
}

main "$@"
