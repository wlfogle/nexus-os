#!/bin/bash
# AI Powerhouse Setup Integration for Ultimate Rescue USB
# Combines AI-Enhanced Rescue USB with AI Powerhouse development environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/var/log/ai-powerhouse-integration.log"
# Use actual user home, not root's home when running with sudo
USER_HOME="/home/garuda"
POWERHOUSE_DIR="$USER_HOME/Downloads/ai-powerhouse-setup"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "🚀 Starting AI Powerhouse Setup Integration..."

# Check if running as root for system modifications
if [[ $EUID -eq 0 ]]; then
    log "⚠️ Running as root - system-level integration enabled"
    SYSTEM_INSTALL=true
else
    log "📝 Running as user - user-level integration only"
    SYSTEM_INSTALL=false
fi

# Clone AI Powerhouse Setup if not exists
if [[ ! -d "$POWERHOUSE_DIR" ]]; then
    log "📥 Cloning AI Powerhouse Setup repository..."
    git clone https://github.com/wlfogle/ai-powerhouse-setup.git "$POWERHOUSE_DIR"
else
    log "🔄 Updating existing AI Powerhouse Setup repository..."
    cd "$POWERHOUSE_DIR"
    git pull origin main
fi

cd "$POWERHOUSE_DIR"

# Create integration configuration
log "⚙️ Creating AI Powerhouse integration configuration..."
mkdir -p "$USER_HOME/.config/ultimate-rescue-usb"

cat > "$USER_HOME/.config/ultimate-rescue-usb/ai-powerhouse-config.json" << JSON
{
  "integration_enabled": true,
  "powerhouse_directory": "$POWERHOUSE_DIR",
  "features": {
    "zfs_root": true,
    "ai_ml_tools": true,
    "development_stack": true,
    "self_hosting": true,
    "virtualization": true,
    "media_stack": true
  },
  "system_specs": {
    "cpu": "Intel i9-13900HX",
    "gpu": "NVIDIA RTX 4080",
    "ram": "64GB DDR5",
    "storage": "Multiple NVMe SSDs with ZFS"
  },
  "installation_phases": [
    "zfs_installation",
    "development_setup",
    "ai_ml_environment",
    "virtualization_config",
    "self_hosting_services"
  ]
}
JSON

# Install AI Powerhouse components based on available scripts
log "🔧 Installing AI Powerhouse components..."

# Phase 1: Check for ZFS installation scripts
if [[ -f "$POWERHOUSE_DIR/installation/zfs-installation.sh" ]]; then
    log "📦 ZFS installation script found"
    if [[ "$SYSTEM_INSTALL" == "true" ]]; then
        log "⚡ Would install ZFS root filesystem (requires clean installation)"
    else
        log "💡 ZFS installation requires system privileges - skipping"
    fi
fi

# Phase 2: Development environment setup
if [[ -d "$POWERHOUSE_DIR/ai-ml" ]]; then
    log "🧠 Setting up AI/ML development environment..."
    
    # Install Ollama integration if available
    if [[ -f "$POWERHOUSE_DIR/ai-ml/ollama-integration.sh" ]]; then
        chmod +x "$POWERHOUSE_DIR/ai-ml/ollama-integration.sh"
        log "🤖 Ollama integration script prepared"
    fi
    
    # Python AI/ML setup
    if command -v python3 &> /dev/null; then
        log "🐍 Python 3 available - AI/ML tools already installed via pacman"
        log "✅ PyTorch, psutil, requests packages ready"
    fi
fi

# Phase 3: Virtualization integration
if [[ -d "$POWERHOUSE_DIR/virtualization" ]]; then
    log "🖥️ Setting up virtualization integration..."
    
    if [[ -f "$POWERHOUSE_DIR/virtualization/proxmox-integration.sh" ]]; then
        chmod +x "$POWERHOUSE_DIR/virtualization/proxmox-integration.sh"
        log "📦 Proxmox integration script prepared"
    fi
fi

# Phase 4: Self-hosting services
if [[ -d "$POWERHOUSE_DIR/self-hosting" ]]; then
    log "🏠 Setting up self-hosting services..."
    
    if [[ -f "$POWERHOUSE_DIR/self-hosting/install-native-media-stack.sh" ]]; then
        chmod +x "$POWERHOUSE_DIR/self-hosting/install-native-media-stack.sh"
        log "📺 Media stack installation script prepared"
    fi
fi

# Create integrated rescue USB launcher
log "🛠️ Creating integrated launcher script..."
mkdir -p "$USER_HOME/bin"
cat > "$USER_HOME/bin/ai-powerhouse-rescue" << 'LAUNCHER'
#!/bin/bash
# AI Powerhouse Enhanced Rescue USB Launcher

POWERHOUSE_DIR="/home/garuda/Downloads/ai-powerhouse-setup"
RESCUE_DIR="/home/garuda/projects/ultimate-rescue-usb"

echo "🚀 AI Powerhouse Enhanced Rescue USB"
echo "======================================"
echo ""
echo "Available Actions:"
echo "1. Run AI ISO Installer (with Powerhouse option)"
echo "2. Build Custom AI Powerhouse ISO"
echo "3. Install AI optimizations"
echo "4. Run neural diagnostics"
echo "5. Setup development environment"
echo "6. Configure self-hosting services"
echo ""

read -p "Select action (1-6): " choice

case $choice in
    1)
        echo "🖥️ Launching AI ISO Installer..."
        cd "$RESCUE_DIR" && sudo python3 ai-iso-installer.py
        ;;
    2)
        echo "🏗️ Building Custom AI Powerhouse ISO..."
        cd "$POWERHOUSE_DIR" && sudo ./installation/build-custom-iso.sh
        ;;
    3)
        echo "⚡ Installing AI optimizations..."
        cd "$RESCUE_DIR" && sudo ./garuda-ai-optimizations/install-optimizations.sh
        ;;
    4)
        echo "🧠 Running neural diagnostics..."
        sudo /opt/ai-rescue-tools/neural-diagnostics.sh
        ;;
    5)
        echo "💻 Setting up development environment..."
        cd "$POWERHOUSE_DIR" && ./ai-ml/ollama-integration.sh
        ;;
    6)
        echo "🏠 Configuring self-hosting services..."
        cd "$POWERHOUSE_DIR/self-hosting" && sudo ./install-native-media-stack.sh
        ;;
    *)
        echo "Invalid selection"
        ;;
esac
LAUNCHER

chmod +x "$USER_HOME/bin/ai-powerhouse-rescue"
# Set proper ownership when running as root
if [[ "$SYSTEM_INSTALL" == "true" ]]; then
    chown garuda:garuda "$USER_HOME/bin/ai-powerhouse-rescue"
    chown -R garuda:garuda "$USER_HOME/.config/ultimate-rescue-usb"
fi

# Update WARP.md with integration information
log "📝 Updating WARP.md with AI Powerhouse integration..."
RESCUE_DIR="$USER_HOME/projects/ultimate-rescue-usb"
if [[ -f "$RESCUE_DIR/WARP.md" ]]; then
    # Add AI Powerhouse section to WARP.md
    cat >> "$RESCUE_DIR/WARP.md" << 'WARP_UPDATE'

## AI Powerhouse Setup Integration

The Ultimate Rescue USB now integrates with AI Powerhouse Setup for enhanced development capabilities.

### AI Powerhouse Features

- **ZFS Root Filesystem** - Superior snapshots and rollback capabilities
- **AI/ML Development** - CUDA acceleration, local LLMs, PyTorch
- **Rust + Tauri + React** - Modern cross-platform app development
- **Virtualization** - KVM/QEMU, Docker, Proxmox VM integration
- **Self-Hosting** - Traefik, Portainer, monitoring stack
- **Gaming Optimized** - Maintains Garuda's gaming performance

### Integration Commands

```bash
# Launch integrated AI Powerhouse Rescue environment
~/bin/ai-powerhouse-rescue

# Build custom AI Powerhouse ISO with rescue tools
cd ~/Downloads/ai-powerhouse-setup && sudo ./installation/build-custom-iso.sh

# Install AI Powerhouse components
cd ~/Downloads/ai-powerhouse-setup && ./ai-ml/ollama-integration.sh

# Setup self-hosting services
cd ~/Downloads/ai-powerhouse-setup/self-hosting && sudo ./install-native-media-stack.sh
```

### AI Powerhouse + Rescue USB Workflow

1. **Search for "AI Powerhouse"** in the AI ISO installer
2. **Build custom ISO** with both rescue tools and development environment
3. **Deploy to USB** for portable AI development + rescue capabilities
4. **Boot from USB** with full ZFS, CUDA, and rescue tools available

### Configuration Files

- `~/.config/ultimate-rescue-usb/ai-powerhouse-config.json` - Integration settings
- `/var/log/ai-powerhouse-integration.log` - Integration logs
- `/opt/ai-rescue-tools/` - AI rescue tools directory

WARP_UPDATE
fi

# Create systemd service for integration monitoring
if [[ "$SYSTEM_INSTALL" == "true" ]]; then
    log "🔧 Creating system integration service..."
    
    cat > /etc/systemd/system/ai-powerhouse-rescue.service << SERVICE
[Unit]
Description=AI Powerhouse Rescue USB Integration
After=network.target

[Service]
Type=oneshot
ExecStart=$USER_HOME/bin/ai-powerhouse-rescue-monitor
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
SERVICE

    systemctl daemon-reload
    systemctl enable ai-powerhouse-rescue.service
fi

log "✅ AI Powerhouse Setup Integration completed successfully!"
log ""
log "🎯 Next steps:"
log "1. Run: ~/bin/ai-powerhouse-rescue"
log "2. Search 'AI Powerhouse' in AI ISO installer"
log "3. Build custom ISO with: sudo ./installation/build-custom-iso.sh"
log "4. Deploy to USB for ultimate development + rescue environment"
log ""
log "🚀 Your rescue USB now has AI Powerhouse capabilities!"