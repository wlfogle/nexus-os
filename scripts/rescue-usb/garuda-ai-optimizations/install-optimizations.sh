#!/bin/bash
# AI-Enhanced Garuda Rescue USB Optimization Installer
# Inspired by Ultimate Rescue USB project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/var/log/garuda-ai-install.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "🧠 Starting AI-Enhanced Garuda Rescue USB Optimization Installation..."

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root (use sudo)"
    exit 1
fi

# Check if running on Garuda Linux
if ! grep -q "Garuda" /etc/os-release 2>/dev/null; then
    log "⚠️ Warning: This optimization is designed for Garuda Linux"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

log "📁 Creating directories..."
mkdir -p /opt/ai-rescue-tools
mkdir -p /usr/local/bin
mkdir -p /etc/sysctl.d
mkdir -p /etc/udev/rules.d
mkdir -p /etc/systemd/system

log "📋 Installing system configuration files..."
cp "$SCRIPT_DIR/90-plasma-usb-performance.conf" /etc/sysctl.d/
cp "$SCRIPT_DIR/60-usb-scheduler.rules" /etc/udev/rules.d/
cp "$SCRIPT_DIR/usb-performance-optimization.service" /etc/systemd/system/

log "🔧 Installing optimization scripts..."
cp "$SCRIPT_DIR/usb-rescue-optimizer.sh" /usr/local/bin/
cp "$SCRIPT_DIR/usb-device-detector.sh" /usr/local/bin/
cp "$SCRIPT_DIR/intelligent-optimizer.py" /opt/ai-rescue-tools/
cp "$SCRIPT_DIR/neural-diagnostics.sh" /opt/ai-rescue-tools/
cp "$SCRIPT_DIR/ai-powerhouse-integration.sh" /opt/ai-rescue-tools/
cp "$SCRIPT_DIR/README.md" /opt/ai-rescue-tools/

log "🔑 Setting permissions..."
chmod +x /usr/local/bin/usb-rescue-optimizer.sh
chmod +x /usr/local/bin/usb-device-detector.sh
chmod +x /opt/ai-rescue-tools/intelligent-optimizer.py
chmod +x /opt/ai-rescue-tools/neural-diagnostics.sh
chmod +x /opt/ai-rescue-tools/ai-powerhouse-integration.sh

log "🎯 Enabling services..."
systemctl daemon-reload
systemctl enable usb-performance-optimization.service

log "⚡ Applying immediate optimizations..."
# Apply sysctl settings now
sysctl -p /etc/sysctl.d/90-plasma-usb-performance.conf

# Reload udev rules
udevadm control --reload-rules
udevadm trigger

log "🧠 Running initial AI optimization..."
if command -v python3 &> /dev/null; then
    python3 /opt/ai-rescue-tools/intelligent-optimizer.py || log "⚠️ Python AI optimizer failed (missing dependencies?)"
fi

log "✅ AI-Enhanced Garuda Rescue USB Optimization installed successfully!"
log ""
log "📊 Next steps:"
log "1. Reboot system to apply all optimizations"
log "2. Check logs: /var/log/ai-rescue.log"
log "3. Run diagnostics: sudo /opt/ai-rescue-tools/neural-diagnostics.sh"
log "4. Monitor performance: /var/log/system-health-report.json"
log "5. Setup AI Powerhouse integration: /opt/ai-rescue-tools/ai-powerhouse-integration.sh"
log ""
log "🚀 Your rescue USB is now AI-optimized for maximum performance!"
log "🏠 Run AI Powerhouse integration for development environment setup!"
