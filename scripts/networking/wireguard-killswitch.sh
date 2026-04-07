#!/bin/bash

# WireGuard Killswitch Script
# This script toggles WireGuard connection and implements internet killswitch

WG_INTERFACE="wg0"
LOCKFILE="/tmp/wg-killswitch.lock"
STATE_FILE="/tmp/wg-killswitch.state"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if WireGuard is running
is_wg_running() {
    sudo systemctl is-active --quiet wg-quick@${WG_INTERFACE}
}

# Function to get current killswitch state
get_killswitch_state() {
    if [[ -f "$STATE_FILE" ]]; then
        cat "$STATE_FILE"
    else
        echo "disabled"
    fi
}

# Function to enable killswitch (block all traffic except VPN)
enable_killswitch() {
    echo "enabled" > "$STATE_FILE"
    
    # Block all traffic except VPN
    sudo iptables -P INPUT DROP
    sudo iptables -P FORWARD DROP
    sudo iptables -P OUTPUT DROP
    
    # Allow loopback
    sudo iptables -I INPUT -i lo -j ACCEPT
    sudo iptables -I OUTPUT -o lo -j ACCEPT
    
    # Allow established connections
    sudo iptables -I INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
    sudo iptables -I OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
    
    # Allow VPN interface
    sudo iptables -I INPUT -i ${WG_INTERFACE} -j ACCEPT
    sudo iptables -I OUTPUT -o ${WG_INTERFACE} -j ACCEPT
    
    # Allow VPN port (51820)
    sudo iptables -I OUTPUT -p udp --dport 51820 -j ACCEPT
    sudo iptables -I INPUT -p udp --sport 51820 -j ACCEPT
    
    echo -e "${RED}🛡️  KILLSWITCH ENABLED${NC}" >&2
}

# Function to disable killswitch (restore normal traffic)
disable_killswitch() {
    echo "disabled" > "$STATE_FILE"
    
    # Reset iptables to default (ACCEPT all)
    sudo iptables -P INPUT ACCEPT
    sudo iptables -P FORWARD ACCEPT
    sudo iptables -P OUTPUT ACCEPT
    
    # Flush all rules
    sudo iptables -F
    
    echo -e "${GREEN}🌐 KILLSWITCH DISABLED${NC}" >&2
}

# Function to toggle WireGuard
toggle_wireguard() {
    if is_wg_running; then
        sudo systemctl stop wg-quick@${WG_INTERFACE}
        if [[ "$(get_killswitch_state)" == "enabled" ]]; then
            # If killswitch is enabled, keep it enabled (block internet)
            enable_killswitch
        else
            disable_killswitch
        fi
        echo -e "${RED}🔴 VPN OFF${NC}"
    else
        sudo systemctl start wg-quick@${WG_INTERFACE}
        disable_killswitch  # Allow internet when VPN is on
        echo -e "${GREEN}🟢 VPN ON${NC}"
    fi
}

# Function to toggle killswitch
toggle_killswitch() {
    current_state=$(get_killswitch_state)
    if [[ "$current_state" == "enabled" ]]; then
        disable_killswitch
    else
        enable_killswitch
    fi
}

# Function to get status for panel display
get_status() {
    if is_wg_running; then
        echo "🟢 VPN"
    else
        killswitch_state=$(get_killswitch_state)
        if [[ "$killswitch_state" == "enabled" ]]; then
            echo "🛡️ KILL"
        else
            echo "🔴 OFF"
        fi
    fi
}

# Quick recovery: stop WG and disable killswitch to restore networking
restore_network() {
    sudo systemctl stop wg-quick@${WG_INTERFACE} >/dev/null 2>&1 || true
    disable_killswitch
    echo -e "${GREEN}🌐 Network restored${NC}" >&2
}

# Main logic
case "$1" in
    "toggle-vpn")
        toggle_wireguard
        ;;
    "toggle-killswitch")
        toggle_killswitch
        ;;
    "status")
        get_status
        ;;
    "enable-killswitch")
        enable_killswitch
        ;;
    "disable-killswitch")
        disable_killswitch
        ;;
    "restore-network")
        restore_network
        ;;
    *)
        echo "WireGuard Killswitch Control"
        echo ""
        echo "Usage: $0 {toggle-vpn|toggle-killswitch|status|enable-killswitch|disable-killswitch|restore-network}"
        echo ""
        echo "Commands:"
        echo "  toggle-vpn         - Toggle WireGuard VPN on/off"
        echo "  toggle-killswitch  - Toggle internet killswitch"
        echo "  status             - Show current status for panel"
        echo "  enable-killswitch  - Force enable killswitch"
        echo "  disable-killswitch - Force disable killswitch"
        echo "  restore-network    - Stop wg0 and disable killswitch (fast recovery)"
        echo ""
        echo "Panel Status Icons:"
        echo "  🟢 VPN  - VPN is connected"
        echo "  🔴 OFF  - VPN is off, internet allowed"
        echo "  🛡️ KILL - VPN is off, internet blocked"
        exit 1
        ;;
esac
