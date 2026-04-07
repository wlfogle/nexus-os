#!/bin/bash

# 🚀 Awesome Stack Optimization Suite - Complete Deployment Script
# Deploys all optimizations: Garuda Host + Proxmox VM + File Browser Quantum

set -e

echo "🚀 AWESOME STACK OPTIMIZATION SUITE - COMPLETE DEPLOYMENT"
echo "=========================================================="
echo "🎯 Deploying: Host optimizations + VM tuning + File Browser Quantum"
echo "🖥️  Target: Enterprise-grade 47+ container infrastructure"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    echo "❌ This script must be run as root (use sudo)"
    echo "   Example: sudo ./deploy-complete-suite.sh"
    exit 1
fi

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_FILE="/var/log/awesome-stack-complete-deployment.log"
BACKUP_DIR="/root/pre-optimization-backups"

echo "📍 Project root: $PROJECT_ROOT"
echo "📝 Log file: $LOG_FILE"
echo ""

# Create directories
mkdir -p "$BACKUP_DIR"
mkdir -p "$(dirname "$LOG_FILE")"

# Start logging
exec 1> >(tee -a "$LOG_FILE")
exec 2>&1

echo "$(date): Starting Complete Awesome Stack deployment" >> "$LOG_FILE"

echo "🏗️ PHASE 1: GARUDA HOST OPTIMIZATION"
echo "===================================="

# Backup current configurations
echo "📋 Creating configuration backups..."
cp /etc/sysctl.conf "$BACKUP_DIR/sysctl.conf.backup" 2>/dev/null || true
cp /etc/security/limits.conf "$BACKUP_DIR/limits.conf.backup" 2>/dev/null || true

# Apply Garuda Host optimizations
if [[ -f "$PROJECT_ROOT/garuda-host/99-ai-ml-optimization.conf" ]]; then
    echo "🧠 Applying AI/ML kernel optimizations..."
    cp "$PROJECT_ROOT/garuda-host/99-ai-ml-optimization.conf" /etc/sysctl.d/
    sysctl -p /etc/sysctl.d/99-ai-ml-optimization.conf
fi

# Configure Docker optimization
if [[ -f "$PROJECT_ROOT/garuda-host/daemon.json" ]]; then
    echo "🐳 Configuring Docker optimizations..."
    mkdir -p /etc/docker
    cp "$PROJECT_ROOT/garuda-host/daemon.json" /etc/docker/
fi

# Install and configure iptables
echo "🌐 Configuring iptables for Proxmox VM forwarding..."

# Create iptables rules
cat > /etc/iptables/iptables.rules << 'EOF'
# Garuda Host iptables rules for Proxmox VM forwarding
*filter
:INPUT ACCEPT [0:0]
:FORWARD ACCEPT [0:0]
:OUTPUT ACCEPT [0:0]
# Accept forwarded traffic to Proxmox VM
-A FORWARD -d 192.168.122.9/32 -p tcp -m tcp --dport 80 -j ACCEPT
-A FORWARD -d 192.168.122.9/32 -p tcp -m tcp --dport 443 -j ACCEPT
-A FORWARD -d 192.168.122.9/32 -p tcp -m tcp --dport 8080 -j ACCEPT
-A FORWARD -d 192.168.122.9/32 -p tcp -m tcp --dport 8090 -j ACCEPT
-A FORWARD -d 192.168.122.9/32 -p tcp -m tcp --dport 8091 -j ACCEPT
-A FORWARD -d 192.168.122.9/32 -p tcp -m tcp --dport 8092 -j ACCEPT
COMMIT

*nat
:PREROUTING ACCEPT [0:0]
:INPUT ACCEPT [0:0]
:OUTPUT ACCEPT [0:0]
:POSTROUTING ACCEPT [0:0]
# Port forwarding rules for Proxmox VM services
-A PREROUTING -i enp4s0 -p tcp -m tcp --dport 80 -j DNAT --to-destination 192.168.122.9:80
-A PREROUTING -i enp4s0 -p tcp -m tcp --dport 443 -j DNAT --to-destination 192.168.122.9:443
-A PREROUTING -i enp4s0 -p tcp -m tcp --dport 8080 -j DNAT --to-destination 192.168.122.9:8080
-A PREROUTING -i enp4s0 -p tcp -m tcp --dport 8090 -j DNAT --to-destination 192.168.122.9:8090
-A PREROUTING -i enp4s0 -p tcp -m tcp --dport 8091 -j DNAT --to-destination 192.168.122.9:8091
-A PREROUTING -i enp4s0 -p tcp -m tcp --dport 8092 -j DNAT --to-destination 192.168.122.9:8092
# MASQUERADE for return traffic
-A POSTROUTING -o enp4s0 -j MASQUERADE
COMMIT
EOF

# Enable IP forwarding
echo 'net.ipv4.ip_forward=1' > /etc/sysctl.d/99-ip-forward.conf
sysctl -p /etc/sysctl.d/99-ip-forward.conf

# Enable services
systemctl enable iptables.service 2>/dev/null || true
systemctl enable docker.service

echo "✅ Phase 1 Complete: Garuda Host optimized"
echo ""

echo "🎯 PHASE 2: PROXMOX VM PREPARATION"
echo "=================================="

# Copy VM optimization scripts to accessible location
echo "📦 Preparing VM optimization scripts..."
mkdir -p /opt/awesome-stack-vm-scripts
cp -r "$PROJECT_ROOT/proxmox-vm"/* /opt/awesome-stack-vm-scripts/ 2>/dev/null || true
cp -r "$PROJECT_ROOT/file-browser-quantum"/* /opt/awesome-stack-vm-scripts/ 2>/dev/null || true
cp -r "$PROJECT_ROOT/verification"/* /opt/awesome-stack-vm-scripts/ 2>/dev/null || true

# Make scripts executable
find /opt/awesome-stack-vm-scripts -name "*.sh" -exec chmod +x {} \;

echo "✅ Phase 2 Complete: VM scripts prepared"
echo ""

echo "🚀 PHASE 3: FINAL CONFIGURATION"
echo "==============================="

# Create convenient aliases
cat >> /etc/bash.bashrc << 'EOF'

# Awesome Stack Optimization Suite aliases
alias stack-health='awesome-stack-health-check 2>/dev/null || echo "Health check not available - reboot required"'
alias stack-optimize-vm='echo "Run on Proxmox VM: /opt/awesome-stack-vm-scripts/verify-proxmox-optimization.sh"'
alias stack-logs='tail -f /var/log/awesome-stack-complete-deployment.log'
alias proxmox-start='systemctl start iptables.service && echo "Proxmox forwarding started"'
alias proxmox-stop='iptables -t nat -F && echo "Forwarding stopped"'
alias proxmox-status='iptables -t nat -L -n'
EOF

# Create health check script
cat > /usr/local/bin/awesome-stack-health-check << 'EOF'
#!/bin/bash

echo "🔍 AWESOME STACK HEALTH CHECK"
echo "============================="
echo ""

echo "📊 Garuda Host Status:"
echo "   • IP Forwarding: $(cat /proc/sys/net/ipv4/ip_forward)"
echo "   • Docker: $(systemctl is-active docker 2>/dev/null || echo 'inactive')"
echo "   • iptables: $(systemctl is-active iptables 2>/dev/null || echo 'inactive')"
echo ""

echo "🌐 Proxmox VM Connectivity:"
if ping -c 1 192.168.122.9 &>/dev/null; then
    echo "   • VM Reachable: ✅ Yes"
    
    # Test File Browser ports
    for port in 8090 8091 8092; do
        if nc -z 192.168.122.9 $port 2>/dev/null; then
            echo "   • Port $port: ✅ Open"
        else
            echo "   • Port $port: ❌ Closed"
        fi
    done
else
    echo "   • VM Reachable: ❌ No"
fi

echo ""
echo "🎯 Next Steps:"
echo "   1. Reboot to activate all optimizations"
echo "   2. Start Proxmox VM" 
echo "   3. On VM, run: /opt/awesome-stack-vm-scripts/install-filebrowser-quantum.sh"
EOF

chmod +x /usr/local/bin/awesome-stack-health-check

echo "✅ Phase 3 Complete: Final configuration applied"
echo ""

echo "🎉 DEPLOYMENT COMPLETE!"
echo "======================"
echo ""
echo "✅ What's been deployed:"
echo "   • Garuda Host optimizations (kernel, Docker, iptables)"
echo "   • Proxmox VM scripts and File Browser Quantum ready"
echo "   • Health monitoring and management commands"
echo ""
echo "🔄 REBOOT REQUIRED to activate all optimizations:"
echo "   sudo reboot"
echo ""
echo "📋 After reboot:"
echo "   1. Check status: stack-health"
echo "   2. Start your Proxmox VM"
echo "   3. SSH to VM and run: /opt/awesome-stack-vm-scripts/install-filebrowser-quantum.sh"
echo "   4. Access File Browser: http://your-ip:8090"
echo ""
echo "🎯 Your infrastructure is now enterprise-ready! 🚀"

exit 0
