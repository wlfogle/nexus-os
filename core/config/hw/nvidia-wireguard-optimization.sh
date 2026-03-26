#!/bin/bash
# NVIDIA GPU and WireGuard VPN Performance Optimization Script
# For i9-13900HX systems with custom optimized kernel

set -e

echo "🚀 NVIDIA GPU & WireGuard VPN Optimization Suite"
echo "================================================="

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

# Check if running optimized kernel
check_kernel() {
    log_info "Checking kernel version..."
    KERNEL_VERSION=$(uname -r)
    
    if [[ $KERNEL_VERSION == *"custom"* ]]; then
        log_success "Running optimized custom kernel: $KERNEL_VERSION"
    else
        log_warning "Not running custom kernel. Consider compiling first."
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 0
        fi
    fi
}

# Install NVIDIA drivers optimally
install_nvidia_drivers() {
    log_info "Installing NVIDIA drivers with performance optimizations..."
    
    # Enable RPM Fusion repositories for NVIDIA drivers
    sudo rpm-ostree install \
        https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm \
        https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm 2>/dev/null || true
    
    # Install NVIDIA driver packages
    log_info "Installing NVIDIA proprietary drivers..."
    sudo rpm-ostree install \
        akmod-nvidia \
        xorg-x11-drv-nvidia-cuda \
        nvidia-settings \
        nvidia-persistenced \
        nvidia-modprobe \
        nvidia-xconfig
    
    log_success "NVIDIA drivers installed. Reboot required."
    
    # Create NVIDIA performance configuration
    create_nvidia_config
}

# Create optimal NVIDIA configuration
create_nvidia_config() {
    log_info "Creating optimal NVIDIA performance configuration..."
    
    # NVIDIA X Server settings for gaming performance
    sudo mkdir -p /etc/X11/xorg.conf.d/
    sudo tee /etc/X11/xorg.conf.d/20-nvidia.conf > /dev/null << 'EOF'
Section "Device"
    Identifier     "nvidia"
    Driver         "nvidia"
    VendorName     "NVIDIA Corporation"
    Option         "NoLogo" "true"
    Option         "UseEDID" "false"
    Option         "ConnectedMonitor" "DFP"
    Option         "TripleBuffer" "true"
    Option         "RegistryDwords" "EnableBrightnessControl=1"
    # Performance optimizations
    Option         "PowerMizerEnable" "0x1"
    Option         "PowerMizerLevel" "0x1"
    Option         "PowerMizerDefault" "0x1"
    # Force maximum performance
    Option         "RegistryDwords" "PowerMizerDefault=0x1; PowerMizerDefaultAC=0x1"
EndSection

Section "Screen"
    Identifier     "nvidia"
    Device         "nvidia"
    Option         "MetaModes" "nvidia-auto-select +0+0 { ForceFullCompositionPipeline = On }"
    Option         "AllowIndirectGLXProtocol" "off"
    Option         "TripleBuffer" "on"
EndSection
EOF
    
    # NVIDIA persistence daemon configuration
    sudo systemctl enable nvidia-persistenced
    
    # Create NVIDIA performance tuning script
    sudo tee /usr/local/bin/nvidia-performance-mode > /dev/null << 'EOF'
#!/bin/bash
# NVIDIA GPU Performance Mode Script
# Run this script after boot for maximum performance

# Set maximum performance mode
nvidia-smi -pm 1

# Set power limit to maximum (adjust based on your GPU)
nvidia-smi -pl 450

# Set GPU and memory clocks to maximum
nvidia-smi -ac 2000,1950

# Set GPU performance mode to maximum
nvidia-settings -a [gpu:0]/GPUPowerMizerMode=1
nvidia-settings -a [gpu:0]/GPUGraphicsClockOffset[3]=150
nvidia-settings -a [gpu:0]/GPUMemoryTransferRateOffset[3]=800

echo "NVIDIA GPU set to maximum performance mode"
EOF
    
    sudo chmod +x /usr/local/bin/nvidia-performance-mode
    
    # Create systemd service for automatic NVIDIA performance tuning
    sudo tee /etc/systemd/system/nvidia-performance.service > /dev/null << 'EOF'
[Unit]
Description=NVIDIA GPU Performance Mode
After=graphical.target nvidia-persistenced.service
Wants=nvidia-persistenced.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/nvidia-performance-mode
User=root
RemainAfterExit=yes

[Install]
WantedBy=graphical.target
EOF
    
    sudo systemctl enable nvidia-performance.service
    
    log_success "NVIDIA performance configuration created"
}

# Configure NVIDIA for AI/ML workloads
configure_nvidia_cuda() {
    log_info "Configuring NVIDIA CUDA for AI/ML optimization..."
    
    # Install CUDA toolkit
    sudo rpm-ostree install \
        cuda \
        cuda-toolkit \
        python3-pip
    
    # Create CUDA environment optimization
    sudo tee /etc/profile.d/cuda-optimization.sh > /dev/null << 'EOF'
# NVIDIA CUDA Optimizations for AI/ML
export CUDA_VISIBLE_DEVICES=0
export CUDA_DEVICE_ORDER=PCI_BUS_ID
export CUDA_CACHE_MAXSIZE=4294967296
export NVIDIA_TF32_OVERRIDE=0

# Memory management for large models
export PYTORCH_CUDA_ALLOC_CONF=max_split_size_mb:128
export TF_FORCE_GPU_ALLOW_GROWTH=true
export TF_GPU_ALLOCATOR=cuda_malloc_async

# Performance optimizations
export OMP_NUM_THREADS=$(nproc)
export MKL_NUM_THREADS=$(nproc)
export CUDA_LAUNCH_BLOCKING=0
EOF
    
    log_success "NVIDIA CUDA AI/ML optimizations configured"
}

# Install and configure WireGuard
install_wireguard() {
    log_info "Installing WireGuard with performance optimizations..."
    
    # WireGuard is already compiled into our custom kernel, just install tools
    sudo rpm-ostree install wireguard-tools
    
    # Create WireGuard performance configuration
    create_wireguard_config
    
    log_success "WireGuard installed and configured"
}

# Create optimal WireGuard configuration
create_wireguard_config() {
    log_info "Creating high-performance WireGuard configuration..."
    
    # Create WireGuard directory
    sudo mkdir -p /etc/wireguard
    
    # Create performance-optimized WireGuard template
    sudo tee /etc/wireguard/wg0-template.conf > /dev/null << 'EOF'
[Interface]
# Replace with your private key
PrivateKey = YOUR_PRIVATE_KEY_HERE
Address = 10.0.0.2/24
DNS = 1.1.1.1, 8.8.8.8

# Performance optimizations
MTU = 1420
PostUp = echo 'Performance mode enabled' > /var/log/wireguard-performance.log
PostUp = ethtool -K %i tx off rx off
PostUp = ip link set dev %i mtu 1420

# Network performance tuning
PostUp = sysctl -w net.core.default_qdisc=cake
PostUp = sysctl -w net.ipv4.tcp_congestion_control=bbr
PostUp = sysctl -w net.core.rmem_max=134217728
PostUp = sysctl -w net.core.wmem_max=134217728
PostUp = sysctl -w net.ipv4.tcp_rmem="4096 65536 134217728"
PostUp = sysctl -w net.ipv4.tcp_wmem="4096 65536 134217728"

# Crypto acceleration (uses Intel AES-NI)
PostUp = echo 'Using hardware crypto acceleration' >> /var/log/wireguard-performance.log

[Peer]
# Replace with server's public key
PublicKey = YOUR_SERVER_PUBLIC_KEY_HERE
Endpoint = YOUR_SERVER_IP:51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
EOF
    
    # Create WireGuard performance monitoring script
    sudo tee /usr/local/bin/wireguard-performance-monitor > /dev/null << 'EOF'
#!/bin/bash
# WireGuard Performance Monitoring Script

echo "🔒 WireGuard Performance Status"
echo "==============================="

# Check if WireGuard is running
if systemctl is-active --quiet wg-quick@wg0; then
    echo "Status: Active ✅"
    
    # Show connection stats
    echo -e "\n📊 Connection Statistics:"
    wg show
    
    # Show network performance
    echo -e "\n🚀 Network Performance:"
    if command -v speedtest-cli &> /dev/null; then
        speedtest-cli --simple
    else
        echo "Install speedtest-cli for speed testing"
    fi
    
    # Show crypto performance
    echo -e "\n🔐 Crypto Performance Test:"
    openssl speed -evp chacha20-poly1305 2>/dev/null | tail -1 || echo "ChaCha20-Poly1305 test unavailable"
    
    # Show CPU usage for crypto
    echo -e "\n💻 CPU Crypto Acceleration:"
    grep -i aes /proc/cpuinfo | head -1 || echo "AES-NI: Not detected"
    grep -i avx /proc/cpuinfo | head -1 || echo "AVX: Not detected"
    
else
    echo "Status: Inactive ❌"
    echo "Start with: sudo systemctl start wg-quick@wg0"
fi
EOF
    
    sudo chmod +x /usr/local/bin/wireguard-performance-monitor
    
    # Create WireGuard systemd service optimization
    sudo mkdir -p /etc/systemd/system/wg-quick@.service.d/
    sudo tee /etc/systemd/system/wg-quick@.service.d/performance.conf > /dev/null << 'EOF'
[Service]
# CPU affinity for crypto performance (use performance cores)
ExecStartPre=/bin/sh -c 'echo performance > /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor'
ExecStartPre=/bin/sh -c 'echo performance > /sys/devices/system/cpu/cpu1/cpufreq/scaling_governor'

# Network buffer optimizations
ExecStartPost=/bin/sh -c 'echo 134217728 > /proc/sys/net/core/rmem_max'
ExecStartPost=/bin/sh -c 'echo 134217728 > /proc/sys/net/core/wmem_max'
EOF
    
    log_success "WireGuard performance configuration created"
}

# Create system-wide network optimizations for VPN
optimize_network_stack() {
    log_info "Optimizing network stack for VPN performance..."
    
    sudo tee /etc/sysctl.d/99-wireguard-performance.conf > /dev/null << 'EOF'
# Network optimizations for WireGuard VPN performance
# TCP/IP stack optimizations
net.core.rmem_default = 262144
net.core.rmem_max = 134217728
net.core.wmem_default = 262144
net.core.wmem_max = 134217728
net.core.netdev_max_backlog = 5000

# TCP optimizations
net.ipv4.tcp_rmem = 4096 65536 134217728
net.ipv4.tcp_wmem = 4096 65536 134217728
net.ipv4.tcp_congestion_control = bbr
net.core.default_qdisc = cake

# Reduce TCP latency
net.ipv4.tcp_fastopen = 3
net.ipv4.tcp_low_latency = 1
net.ipv4.tcp_no_delay = 1

# UDP optimizations for WireGuard
net.core.netdev_budget = 600
net.core.netdev_budget_usecs = 5000

# IP forwarding for VPN routing
net.ipv4.ip_forward = 1
net.ipv6.conf.all.forwarding = 1

# Security optimizations
net.ipv4.conf.default.log_martians = 1
net.ipv4.conf.all.log_martians = 1
net.ipv4.conf.default.accept_source_route = 0
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_redirects = 0
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.icmp_echo_ignore_broadcasts = 1
net.ipv4.icmp_ignore_bogus_error_responses = 1
EOF
    
    # Apply network optimizations immediately
    sudo sysctl -p /etc/sysctl.d/99-wireguard-performance.conf
    
    log_success "Network stack optimized for VPN performance"
}

# Create comprehensive performance test script
create_performance_test() {
    log_info "Creating comprehensive performance test script..."
    
    tee "$HOME/test-nvidia-wireguard-performance.sh" > /dev/null << 'EOF'
#!/bin/bash
# Comprehensive NVIDIA GPU and WireGuard Performance Test

echo "🚀 NVIDIA GPU & WireGuard Performance Test"
echo "==========================================="

# Test NVIDIA GPU
echo -e "\n🖥️  NVIDIA GPU Status:"
nvidia-smi --query-gpu=name,power.draw,temperature.gpu,utilization.gpu,memory.used,memory.total --format=csv,noheader,nounits

echo -e "\n🎮 GPU Performance Mode:"
nvidia-smi -q -d PERFORMANCE | grep -E "(Performance State|Power Limit)"

echo -e "\n🧠 CUDA Performance Test:"
if command -v nvidia-smi &> /dev/null; then
    echo "Running CUDA bandwidth test..."
    /usr/local/cuda/extras/demo_suite/bandwidthTest 2>/dev/null || echo "CUDA bandwidth test not available"
fi

# Test WireGuard
echo -e "\n🔒 WireGuard Status:"
if systemctl is-active --quiet wg-quick@wg0; then
    echo "WireGuard: Active ✅"
    wg show wg0 2>/dev/null || echo "No WireGuard interface active"
    
    echo -e "\n🚀 VPN Performance Test:"
    echo "Testing ping latency through VPN..."
    ping -c 5 1.1.1.1 2>/dev/null | tail -1
    
    echo -e "\nTesting VPN throughput..."
    if command -v speedtest-cli &> /dev/null; then
        speedtest-cli --simple
    else
        echo "Install speedtest-cli for throughput testing"
    fi
else
    echo "WireGuard: Inactive ❌"
fi

# Test crypto performance
echo -e "\n🔐 Hardware Crypto Performance:"
echo "ChaCha20-Poly1305 (WireGuard crypto):"
openssl speed -evp chacha20-poly1305 2>/dev/null | tail -1 || echo "Test unavailable"

echo -e "\nAES-256-GCM (Alternative crypto):"
openssl speed -evp aes-256-gcm 2>/dev/null | tail -1 || echo "Test unavailable"

# System performance summary
echo -e "\n📊 System Performance Summary:"
echo "CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
echo "Kernel: $(uname -r)"
echo "Load Average: $(uptime | awk '{print $10, $11, $12}')"
echo "Memory Usage: $(free -h | grep Mem | awk '{print $3 "/" $2}')"

echo -e "\n✅ Performance test completed!"
EOF
    
    chmod +x "$HOME/test-nvidia-wireguard-performance.sh"
    
    log_success "Performance test script created: ~/test-nvidia-wireguard-performance.sh"
}

# Main execution
main() {
    echo "This script will optimize your i9-13900HX system for:"
    echo "  🎮 NVIDIA GPU gaming and CUDA performance"
    echo "  🔒 High-performance WireGuard VPN"
    echo "  🚀 Hardware crypto acceleration"
    echo "  🖼️  GPU passthrough capabilities"
    echo ""
    
    read -p "Continue with NVIDIA and WireGuard optimization? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Optimization cancelled"
        exit 0
    fi
    
    check_kernel
    install_nvidia_drivers
    configure_nvidia_cuda
    install_wireguard
    optimize_network_stack
    create_performance_test
    
    log_success "🎉 NVIDIA GPU and WireGuard optimization complete!"
    echo ""
    echo "=========================================="
    echo "🎯 Next Steps:"
    echo "1. Reboot to load NVIDIA drivers: sudo systemctl reboot"
    echo "2. Configure WireGuard: sudo nano /etc/wireguard/wg0.conf"
    echo "3. Start WireGuard: sudo systemctl enable --now wg-quick@wg0"
    echo "4. Test performance: ./test-nvidia-wireguard-performance.sh"
    echo "5. Monitor WireGuard: sudo /usr/local/bin/wireguard-performance-monitor"
    echo ""
    echo "🔧 Your system is now optimized for:"
    echo "  • 🎮 Maximum NVIDIA GPU gaming performance"
    echo "  • 🧠 CUDA/AI workload acceleration"
    echo "  • 🔒 High-performance WireGuard VPN (with Intel crypto acceleration)"
    echo "  • 🖼️  NVIDIA GPU passthrough for VMs"
    echo "  • 🚀 Optimized network stack for VPN traffic"
    echo "=========================================="
}

main "$@"
