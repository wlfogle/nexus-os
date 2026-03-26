#!/bin/bash
# WireGuard Dual-Role (Server + Client) Optimization Script
# For i9-13900HX systems with high-performance networking requirements
# Based on existing awesome-stack WireGuard tools and configurations

set -e

echo "🔥 WireGuard Dual-Role Optimization for i9-13900HX"
echo "=================================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration
SERVER_PORT=51820
CLIENT_PORT=51821
SERVER_NETWORK="10.200.0.0/24"
CLIENT_NETWORK="10.201.0.0/24"
SERVER_IP="10.200.0.1"
CLIENT_IP="10.201.0.100"
EXTERNAL_INTERFACE=$(ip route | grep default | head -1 | awk '{print $5}')

# Apply advanced kernel network optimizations for WireGuard dual-role
apply_kernel_optimizations() {
    log_info "Applying advanced kernel optimizations for WireGuard dual-role..."
    
    # Create optimized sysctl configuration for WireGuard dual-role
    sudo tee /etc/sysctl.d/99-wireguard-dual-role.conf > /dev/null << 'EOF'
# WireGuard Dual-Role Performance Optimizations for i9-13900HX
# ============================================================

# Network Core Optimizations
net.core.rmem_default = 262144
net.core.rmem_max = 268435456
net.core.wmem_default = 262144
net.core.wmem_max = 268435456
net.core.netdev_max_backlog = 30000
net.core.netdev_budget = 600
net.core.netdev_budget_usecs = 8000

# UDP Optimizations for WireGuard
net.core.netdev_tstamp_prequeue = 0
net.ipv4.udp_rmem_min = 8192
net.ipv4.udp_wmem_min = 8192
net.ipv4.udp_mem = 102400 873800 16777216

# TCP Performance (for API traffic over VPN)
net.ipv4.tcp_rmem = 4096 87380 268435456
net.ipv4.tcp_wmem = 4096 65536 268435456
net.ipv4.tcp_congestion_control = bbr
net.core.default_qdisc = cake

# Advanced TCP optimizations
net.ipv4.tcp_slow_start_after_idle = 0
net.ipv4.tcp_fastopen = 3
net.ipv4.tcp_mtu_probing = 1
net.ipv4.tcp_base_mss = 1024

# Multi-path routing for dual-role setup
net.ipv4.ip_forward = 1
net.ipv6.conf.all.forwarding = 1
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0

# Advanced routing for multiple WireGuard interfaces
net.ipv4.conf.all.rp_filter = 2
net.ipv4.conf.default.rp_filter = 2

# Security optimizations
net.ipv4.conf.all.log_martians = 1
net.ipv4.conf.default.log_martians = 1
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.default.accept_redirects = 0
net.ipv4.conf.all.send_redirects = 0
net.ipv4.icmp_echo_ignore_broadcasts = 1
net.ipv4.icmp_ignore_bogus_error_responses = 1

# Netfilter optimizations for dual-role NAT/forwarding
net.netfilter.nf_conntrack_max = 1048576
net.nf_conntrack_max = 1048576
net.netfilter.nf_conntrack_tcp_timeout_established = 28800
net.netfilter.nf_conntrack_generic_timeout = 120

# Memory pressure handling
vm.min_free_kbytes = 65536
vm.swappiness = 10

# CPU scheduler optimizations for network processing
kernel.sched_autogroup_enabled = 1
kernel.sched_migration_cost_ns = 5000000
EOF

    # Apply immediately
    sudo sysctl -p /etc/sysctl.d/99-wireguard-dual-role.conf
    
    log_success "Kernel network optimizations applied"
}

# Configure advanced network queue disciplines
setup_network_qdisc() {
    log_info "Setting up advanced network queue disciplines..."
    
    # Create network optimization script
    sudo tee /usr/local/bin/optimize-network-queues > /dev/null << EOF
#!/bin/bash
# Advanced Network Queue Discipline Setup for WireGuard Dual-Role

# Set CAKE qdisc on external interface for better WireGuard performance
tc qdisc replace dev $EXTERNAL_INTERFACE root cake bandwidth 1gbit

# Set up fq_codel on WireGuard interfaces (will be applied when interfaces come up)
for iface in wg-server wg-client; do
    if ip link show \$iface >/dev/null 2>&1; then
        tc qdisc replace dev \$iface root fq_codel limit 10240 flows 1024 quantum 1514 target 5ms
    fi
done

echo "Network queue disciplines optimized for WireGuard dual-role"
EOF

    sudo chmod +x /usr/local/bin/optimize-network-queues
    sudo /usr/local/bin/optimize-network-queues
    
    log_success "Network queue disciplines configured"
}

# Create systemd service for network optimizations
create_network_service() {
    log_info "Creating network optimization service..."
    
    sudo tee /etc/systemd/system/wireguard-network-optimization.service > /dev/null << 'EOF'
[Unit]
Description=WireGuard Network Optimizations
After=network.target
Before=wg-quick@wg-server.service wg-quick@wg-client.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/optimize-network-queues
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl enable wireguard-network-optimization.service
    
    log_success "Network optimization service created"
}

# Generate WireGuard keys
generate_keys() {
    local name=$1
    local key_dir="/etc/wireguard/keys"
    
    sudo mkdir -p "$key_dir"
    
    if [[ ! -f "$key_dir/${name}_private.key" ]]; then
        log_info "Generating keys for $name..."
        wg genkey | sudo tee "$key_dir/${name}_private.key" | wg pubkey | sudo tee "$key_dir/${name}_public.key"
        sudo chmod 600 "$key_dir/${name}_private.key"
        sudo chmod 644 "$key_dir/${name}_public.key"
    fi
}

# Create optimized WireGuard server configuration
create_server_config() {
    log_info "Creating optimized WireGuard server configuration..."
    
    generate_keys "server"
    
    local server_private=$(sudo cat /etc/wireguard/keys/server_private.key)
    
    sudo tee /etc/wireguard/wg-server.conf > /dev/null << EOF
[Interface]
# Server Configuration - Globe Hopper Style
PrivateKey = $server_private
Address = $SERVER_IP/24
ListenPort = $SERVER_PORT
SaveConfig = false

# Performance optimizations
MTU = 1420

# Advanced iptables rules for dual-role setup
PostUp = iptables -A FORWARD -i %i -j ACCEPT
PostUp = iptables -A FORWARD -o %i -j ACCEPT  
PostUp = iptables -t nat -A POSTROUTING -o $EXTERNAL_INTERFACE -j MASQUERADE
PostUp = iptables -A INPUT -p udp --dport $SERVER_PORT -j ACCEPT

# Apply network optimizations after interface comes up
PostUp = tc qdisc replace dev %i root fq_codel limit 10240 flows 1024 quantum 1514 target 5ms
PostUp = echo performance > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Cleanup rules
PostDown = iptables -D FORWARD -i %i -j ACCEPT
PostDown = iptables -D FORWARD -o %i -j ACCEPT
PostDown = iptables -t nat -D POSTROUTING -o $EXTERNAL_INTERFACE -j MASQUERADE
PostDown = iptables -D INPUT -p udp --dport $SERVER_PORT -j ACCEPT

# Client configurations will be added here
# Example client peer (copy from existing setup):
# [Peer]
# PublicKey = CLIENT_PUBLIC_KEY_HERE
# AllowedIPs = 10.200.0.2/32
# PersistentKeepalive = 25
EOF

    log_success "WireGuard server configuration created"
}

# Create optimized WireGuard client configuration
create_client_config() {
    log_info "Creating optimized WireGuard client configuration..."
    
    generate_keys "client"
    
    local client_private=$(sudo cat /etc/wireguard/keys/client_private.key)
    
    sudo tee /etc/wireguard/wg-client.conf > /dev/null << EOF
[Interface]
# Client Configuration - Connect to external WireGuard servers
PrivateKey = $client_private
Address = $CLIENT_IP/24
MTU = 1420

# DNS for privacy
DNS = 1.1.1.1, 8.8.8.8, 9.9.9.9, 2606:4700:4700::1111

# Performance optimizations
PostUp = tc qdisc replace dev %i root fq_codel limit 10240 flows 1024 quantum 1514 target 5ms
PostUp = echo performance > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Example peer configuration (update with your VPN provider details):
# [Peer]
# PublicKey = EXTERNAL_VPN_SERVER_PUBLIC_KEY
# Endpoint = EXTERNAL_VPN_SERVER_IP:51820
# AllowedIPs = 0.0.0.0/0
# PersistentKeepalive = 25
EOF

    log_success "WireGuard client configuration created"
}

# Create advanced client management script
create_client_management() {
    log_info "Creating advanced client management system..."
    
    sudo tee /usr/local/bin/wg-dual-manager > /dev/null << 'EOF'
#!/bin/bash
# Advanced WireGuard Dual-Role Manager
# Based on awesome-stack tools with performance optimizations

COLORS_RED='\033[0;31m'
COLORS_GREEN='\033[0;32m'
COLORS_YELLOW='\033[1;33m'
COLORS_BLUE='\033[0;34m'
COLORS_NC='\033[0m'

print_header() {
    echo -e "${COLORS_BLUE}"
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║              🛡️  WireGuard Dual-Role Manager  🛡️                    ║"
    echo "║                Server + Client + Performance Monitoring              ║"
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo -e "${COLORS_NC}"
}

check_status() {
    print_header
    echo -e "${COLORS_YELLOW}🔍 System Status:${COLORS_NC}"
    echo
    
    # Check server interface
    if ip link show wg-server >/dev/null 2>&1; then
        echo -e "  ${COLORS_GREEN}✅ WireGuard Server (wg-server): ACTIVE${COLORS_NC}"
        echo "    └─ $(wg show wg-server | grep listening | awk '{print "Port:", $3}')"
        echo "    └─ Connected peers: $(wg show wg-server | grep -c "peer:")"
    else
        echo -e "  ${COLORS_RED}❌ WireGuard Server: INACTIVE${COLORS_NC}"
    fi
    
    # Check client interface  
    if ip link show wg-client >/dev/null 2>&1; then
        echo -e "  ${COLORS_GREEN}✅ WireGuard Client (wg-client): ACTIVE${COLORS_NC}"
        if wg show wg-client | grep -q "endpoint:"; then
            echo "    └─ Connected to: $(wg show wg-client | grep endpoint | awk '{print $2}')"
        fi
    else
        echo -e "  ${COLORS_RED}❌ WireGuard Client: INACTIVE${COLORS_NC}"
    fi
    
    # Network performance
    echo
    echo -e "${COLORS_YELLOW}📊 Performance Metrics:${COLORS_NC}"
    
    # CPU frequency
    local cpu_freq=$(grep "cpu MHz" /proc/cpuinfo | head -1 | awk '{print $4}')
    echo "  🔥 CPU Frequency: ${cpu_freq} MHz"
    
    # Network throughput
    if command -v iftop >/dev/null 2>&1; then
        echo "  📈 Use 'iftop -i wg-server' or 'iftop -i wg-client' for real-time traffic"
    fi
    
    # Connection test
    echo
    echo -e "${COLORS_YELLOW}🌐 Connectivity Test:${COLORS_NC}"
    local ext_ip=$(curl -s --connect-timeout 5 ifconfig.me 2>/dev/null || echo "Unable to detect")
    echo "  📡 External IP: $ext_ip"
}

add_server_client() {
    local client_name=$1
    local client_ip=$2
    
    if [[ -z "$client_name" || -z "$client_ip" ]]; then
        echo "Usage: $0 add-client CLIENT_NAME CLIENT_IP"
        echo "Example: $0 add-client phone 10.200.0.10"
        exit 1
    fi
    
    echo -e "${COLORS_YELLOW}➕ Adding client: $client_name ($client_ip)${COLORS_NC}"
    
    # Generate client keys
    local temp_dir=$(mktemp -d)
    wg genkey | tee "$temp_dir/client_private" | wg pubkey > "$temp_dir/client_public"
    
    local client_private=$(cat "$temp_dir/client_private")
    local client_public=$(cat "$temp_dir/client_public")
    local server_public=$(cat /etc/wireguard/keys/server_public.key)
    
    # Add peer to server config
    echo "" | sudo tee -a /etc/wireguard/wg-server.conf
    echo "# Client: $client_name" | sudo tee -a /etc/wireguard/wg-server.conf
    echo "[Peer]" | sudo tee -a /etc/wireguard/wg-server.conf
    echo "PublicKey = $client_public" | sudo tee -a /etc/wireguard/wg-server.conf
    echo "AllowedIPs = $client_ip/32" | sudo tee -a /etc/wireguard/wg-server.conf
    echo "PersistentKeepalive = 25" | sudo tee -a /etc/wireguard/wg-server.conf
    
    # Create client config
    sudo mkdir -p /etc/wireguard/clients
    sudo tee "/etc/wireguard/clients/$client_name.conf" << EOF
[Interface]
PrivateKey = $client_private
Address = $client_ip/32
DNS = 1.1.1.1, 8.8.8.8
MTU = 1420

[Peer]  
PublicKey = $server_public
Endpoint = $(curl -s ifconfig.me):51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
EOF
    
    rm -rf "$temp_dir"
    
    echo -e "${COLORS_GREEN}✅ Client '$client_name' added successfully${COLORS_NC}"
    echo "  📋 Config saved to: /etc/wireguard/clients/$client_name.conf"
    echo "  🔄 Restart server: sudo systemctl restart wg-quick@wg-server"
}

rotate_keys() {
    echo -e "${COLORS_YELLOW}🔄 Rotating WireGuard keys for enhanced privacy...${COLORS_NC}"
    
    # This would integrate with the existing awesome-stack rotation logic
    if [[ -f "/home/lou/github-repos/awesome-stack/wireguard-tools/server/wireguard-rotate.sh" ]]; then
        echo "Using awesome-stack rotation script..."
        /home/lou/github-repos/awesome-stack/wireguard-tools/server/wireguard-rotate.sh garuda-host
    else
        echo "Manual key rotation required - awesome-stack tools not found"
    fi
}

performance_monitor() {
    echo -e "${COLORS_YELLOW}📈 Starting performance monitor...${COLORS_NC}"
    echo "Press Ctrl+C to stop"
    echo
    
    while true; do
        clear
        print_header
        echo -e "${COLORS_BLUE}Real-time Performance Monitor${COLORS_NC}"
        echo "$(date)"
        echo
        
        # WireGuard status
        if command -v wg >/dev/null 2>&1; then
            echo -e "${COLORS_YELLOW}WireGuard Interfaces:${COLORS_NC}"
            wg show all
            echo
        fi
        
        # Network stats
        echo -e "${COLORS_YELLOW}Network Performance:${COLORS_NC}"
        cat /proc/net/dev | grep -E "(wg-server|wg-client)" | while read line; do
            echo "  $line"
        done
        echo
        
        # CPU utilization
        echo -e "${COLORS_YELLOW}CPU Status:${COLORS_NC}"
        echo "  Load: $(uptime | awk -F'load average:' '{print $2}')"
        echo "  Active cores: $(nproc) @ $(grep MHz /proc/cpuinfo | head -1 | awk '{print $4}') MHz"
        
        sleep 5
    done
}

case "${1:-status}" in
    "status")
        check_status
        ;;
    "add-client")
        add_server_client "$2" "$3"
        ;;
    "rotate")
        rotate_keys
        ;;
    "monitor")
        performance_monitor
        ;;
    "start-server")
        sudo systemctl start wg-quick@wg-server
        echo "WireGuard server started"
        ;;
    "stop-server")
        sudo systemctl stop wg-quick@wg-server
        echo "WireGuard server stopped"
        ;;
    "start-client") 
        sudo systemctl start wg-quick@wg-client
        echo "WireGuard client started"
        ;;
    "stop-client")
        sudo systemctl stop wg-quick@wg-client
        echo "WireGuard client stopped"
        ;;
    *)
        print_header
        echo -e "${COLORS_BLUE}Available Commands:${COLORS_NC}"
        echo "  status              - Show system status"
        echo "  add-client NAME IP  - Add new server client"
        echo "  rotate              - Rotate keys (privacy)"
        echo "  monitor             - Real-time performance monitor"
        echo "  start-server        - Start WireGuard server"
        echo "  stop-server         - Stop WireGuard server"  
        echo "  start-client        - Start WireGuard client"
        echo "  stop-client         - Stop WireGuard client"
        ;;
esac
EOF

    sudo chmod +x /usr/local/bin/wg-dual-manager
    
    log_success "Dual-role management script created"
}

# Create performance monitoring and auto-optimization
create_performance_monitoring() {
    log_info "Creating performance monitoring system..."
    
    sudo tee /usr/local/bin/wg-performance-optimizer > /dev/null << 'EOF'
#!/bin/bash
# WireGuard Performance Auto-Optimizer
# Continuously monitors and optimizes network performance

LOG_FILE="/var/log/wg-performance.log"

log_perf() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" >> "$LOG_FILE"
}

optimize_cpu_governors() {
    # Set performance governor for better crypto performance
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [[ -f "$cpu" ]]; then
            echo performance > "$cpu" 2>/dev/null || true
        fi
    done
    log_perf "CPU governors set to performance mode"
}

optimize_network_interrupts() {
    # Distribute network interrupts across CPU cores for better performance
    local cpu_count=$(nproc)
    local interrupt_count=0
    
    for irq in /proc/irq/*/smp_affinity; do
        if [[ -f "$irq" ]]; then
            printf "%x" $((2**$((interrupt_count % cpu_count)))) > "$irq" 2>/dev/null || true
            ((interrupt_count++))
        fi
    done
    log_perf "Network interrupts optimized across $cpu_count cores"
}

monitor_performance() {
    while true; do
        # Check if WireGuard interfaces are under load
        local server_rx=$(cat /sys/class/net/wg-server/statistics/rx_bytes 2>/dev/null || echo 0)
        local server_tx=$(cat /sys/class/net/wg-server/statistics/tx_bytes 2>/dev/null || echo 0)
        local client_rx=$(cat /sys/class/net/wg-client/statistics/rx_bytes 2>/dev/null || echo 0)  
        local client_tx=$(cat /sys/class/net/wg-client/statistics/tx_bytes 2>/dev/null || echo 0)
        
        # Auto-optimize if high traffic detected
        local total_traffic=$((server_rx + server_tx + client_rx + client_tx))
        if [[ $total_traffic -gt 1000000000 ]]; then  # > 1GB traffic
            optimize_cpu_governors
            optimize_network_interrupts
            log_perf "High traffic detected ($total_traffic bytes), optimizations applied"
        fi
        
        sleep 300  # Check every 5 minutes
    done
}

case "${1:-monitor}" in
    "monitor")
        log_perf "Performance monitoring started"
        monitor_performance
        ;;
    "optimize")
        optimize_cpu_governors
        optimize_network_interrupts
        log_perf "Manual optimization completed"
        ;;
esac
EOF

    sudo chmod +x /usr/local/bin/wg-performance-optimizer
    
    # Create systemd service for auto-optimization
    sudo tee /etc/systemd/system/wg-performance-optimizer.service > /dev/null << 'EOF'
[Unit]
Description=WireGuard Performance Auto-Optimizer
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/wg-performance-optimizer monitor
Restart=always
RestartSec=30

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl enable wg-performance-optimizer.service
    
    log_success "Performance monitoring system created"
}

# Create bandwidth testing and optimization
create_bandwidth_testing() {
    log_info "Creating bandwidth testing tools..."
    
    sudo tee /usr/local/bin/wg-bandwidth-test > /dev/null << 'EOF'
#!/bin/bash
# WireGuard Bandwidth Testing and Optimization Tool

test_crypto_performance() {
    echo "🔐 Testing crypto performance (ChaCha20-Poly1305):"
    openssl speed -evp chacha20-poly1305 2>/dev/null | tail -3
    echo
    
    echo "🔐 Testing AES-256-GCM performance:"  
    openssl speed -evp aes-256-gcm 2>/dev/null | tail -3
    echo
}

test_network_performance() {
    echo "📊 Network Performance Test:"
    
    # Test through WireGuard server interface
    if ip link show wg-server >/dev/null 2>&1; then
        echo "  Server interface (wg-server):"
        iperf3 -c 10.200.0.1 -t 10 -P 4 2>/dev/null || echo "    iperf3 server needed on 10.200.0.1"
    fi
    
    # Test through WireGuard client interface  
    if ip link show wg-client >/dev/null 2>&1; then
        echo "  Client interface (wg-client):"
        curl -w "Speed: %{speed_download} bytes/sec\n" -s -o /dev/null https://speed.cloudflare.com/__down?bytes=100000000
    fi
}

optimize_mtu() {
    echo "🔧 Optimizing MTU sizes..."
    
    # Test optimal MTU for each interface
    for interface in wg-server wg-client; do
        if ip link show "$interface" >/dev/null 2>&1; then
            echo "  Testing $interface:"
            for mtu in 1500 1420 1380 1360 1280; do
                if ping -M do -s $((mtu - 28)) -c 1 -W 2 8.8.8.8 >/dev/null 2>&1; then
                    echo "    Optimal MTU for $interface: $mtu"
                    sudo ip link set dev "$interface" mtu "$mtu"
                    break
                fi
            done
        fi
    done
}

case "${1:-all}" in
    "crypto")
        test_crypto_performance
        ;;
    "network") 
        test_network_performance
        ;;
    "mtu")
        optimize_mtu
        ;;
    "all")
        echo "🚀 Complete WireGuard Performance Test"
        echo "====================================="
        test_crypto_performance
        test_network_performance  
        optimize_mtu
        ;;
esac
EOF

    sudo chmod +x /usr/local/bin/wg-bandwidth-test
    
    log_success "Bandwidth testing tools created"
}

# Main installation function
main() {
    echo "This script will set up optimized WireGuard dual-role configuration:"
    echo "  🔧 Server mode: Host your own VPN server"
    echo "  🔒 Client mode: Connect to external VPN providers"  
    echo "  ⚡ Performance: i9-13900HX optimized networking"
    echo "  📊 Monitoring: Real-time performance tracking"
    echo "  🛡️ Integration: Works with awesome-stack tools"
    echo
    
    read -p "Continue with WireGuard dual-role setup? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Setup cancelled"
        exit 0
    fi
    
    # Check if WireGuard is installed
    if ! command -v wg &> /dev/null; then
        log_error "WireGuard not installed. Install with: sudo rpm-ostree install wireguard-tools"
        exit 1
    fi
    
    apply_kernel_optimizations
    setup_network_qdisc
    create_network_service
    create_server_config
    create_client_config
    create_client_management
    create_performance_monitoring
    create_bandwidth_testing
    
    log_success "🎉 WireGuard dual-role setup complete!"
    echo
    echo "=========================================="
    echo "🎯 Next Steps:"
    echo "1. Configure server clients: sudo wg-dual-manager add-client NAME IP"
    echo "2. Configure external VPN: edit /etc/wireguard/wg-client.conf"
    echo "3. Start services:"
    echo "   • Server: sudo systemctl enable --now wg-quick@wg-server"
    echo "   • Client: sudo systemctl enable --now wg-quick@wg-client"  
    echo "4. Monitor performance: wg-dual-manager monitor"
    echo "5. Test bandwidth: wg-bandwidth-test"
    echo ""
    echo "🔧 Management Commands:"
    echo "  • wg-dual-manager status     - Check system status"
    echo "  • wg-dual-manager rotate     - Rotate keys (awesome-stack integration)"
    echo "  • wg-performance-optimizer   - Auto-optimization service"
    echo "  • wg-bandwidth-test          - Performance testing"
    echo ""
    echo "📁 Configuration Files:"
    echo "  • Server: /etc/wireguard/wg-server.conf"
    echo "  • Client: /etc/wireguard/wg-client.conf"
    echo "  • Clients: /etc/wireguard/clients/*.conf"
    echo "=========================================="
}

main "$@"
