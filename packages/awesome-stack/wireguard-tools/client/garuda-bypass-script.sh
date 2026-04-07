#!/bin/bash
# Garuda Host Anti-Tracking Bypass Script
# Deploy this on your Garuda system to rotate WireGuard configs automatically

PROXMOX_HOST="192.168.122.9"
PROXMOX_USER="root"
WG_CONFIG_DIR="/etc/wireguard"
BACKUP_DIR="/etc/wireguard/backups"

# Function to fetch and apply new rotated config
update_wireguard_config() {
    echo "$(date) - Fetching new WireGuard configuration..."
    
    # Create backup of current config
    mkdir -p "$BACKUP_DIR"
    if [[ -f "$WG_CONFIG_DIR/wg0.conf" ]]; then
        cp "$WG_CONFIG_DIR/wg0.conf" "$BACKUP_DIR/wg0-$(date +%Y%m%d-%H%M).conf.bak"
    fi
    
    # Fetch latest client config from Proxmox host
    latest_config=$(ssh "$PROXMOX_USER@$PROXMOX_HOST" "ls -1t /etc/wireguard/clients/garuda-host-*.conf | head -1")
    
    if [[ -n "$latest_config" ]]; then
        echo "Downloading new config: $latest_config"
        scp "$PROXMOX_USER@$PROXMOX_HOST:$latest_config" "$WG_CONFIG_DIR/wg0.conf"
        
        # Restart WireGuard with new config
        echo "Applying new configuration..."
        wg-quick down wg0 2>/dev/null || true
        sleep 2
        wg-quick up wg0
        
        echo "$(date) - WireGuard configuration updated successfully"
    else
        echo "$(date) - No new configuration found"
    fi
}

# Function to force rotation on server and update locally
force_rotate_and_update() {
    echo "$(date) - Forcing server-side rotation..."
    
    # Trigger rotation on Proxmox server
    ssh "$PROXMOX_USER@$PROXMOX_HOST" "/root/wireguard-rotate.sh garuda-host"
    
    # Wait a moment for the rotation to complete
    sleep 5
    
    # Update local config
    update_wireguard_config
}

# Function to check if Warp Terminal is blocking AI access
test_ai_access() {
    echo "Testing AI service access..."
    
    # Test various AI service domains
    domains=("claude.ai" "api.openai.com" "chat.openai.com" "api.anthropic.com")
    
    for domain in "${domains[@]}"; do
        if timeout 5 curl -s --connect-timeout 3 "https://$domain" >/dev/null; then
            echo "✅ $domain - Accessible"
        else
            echo "❌ $domain - Blocked or unreachable"
        fi
    done
}

# Function to start stealth mode (rapid rotation)
start_stealth_mode() {
    echo "$(date) - Starting stealth mode (rapid rotation every 30 minutes)"
    
    while true; do
        force_rotate_and_update
        echo "$(date) - Stealth rotation completed, sleeping 30 minutes..."
        sleep 1800  # 30 minutes
    done
}

# Main menu
case "${1:-}" in
    "update")
        update_wireguard_config
        ;;
    "rotate")
        force_rotate_and_update
        ;;
    "test")
        test_ai_access
        ;;
    "stealth")
        start_stealth_mode
        ;;
    "status")
        echo "Current WireGuard status:"
        wg show
        echo ""
        echo "Current external IP:"
        curl -s ifconfig.me
        echo ""
        ;;
    *)
        echo "Garuda Host Anti-Tracking Bypass Script"
        echo "Usage: $0 {update|rotate|test|stealth|status}"
        echo ""
        echo "Commands:"
        echo "  update   - Fetch and apply latest rotated config"
        echo "  rotate   - Force server rotation and update locally"
        echo "  test     - Test AI service accessibility"
        echo "  stealth  - Enable rapid rotation mode (every 30 min)"
        echo "  status   - Show current WireGuard and IP status"
        echo ""
        echo "To bypass Warp Terminal detection:"
        echo "1. Run: sudo $0 rotate"
        echo "2. Test with: $0 test"
        echo "3. For continuous protection: $0 stealth"
        exit 1
        ;;
esac
