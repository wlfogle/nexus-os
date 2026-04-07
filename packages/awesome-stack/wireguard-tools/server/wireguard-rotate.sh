#!/bin/bash
# WireGuard IP and Key Rotation Script
# Rotates client keys and IPs to prevent tracking

set -e

# Configuration
WG_INTERFACE="wg0"
WG_CONFIG="/etc/wireguard/wg0.conf"
CLIENT_CONFIG_DIR="/etc/wireguard/clients"
SERVER_CONTAINER="100"
VPN_NETWORK="10.0.0"
LOG_FILE="/var/log/wg-rotation.log"

# Available IP pool (excluding .1 server and .3 gluetun)
AVAILABLE_IPS=("10.0.0.4" "10.0.0.5" "10.0.0.6" "10.0.0.7" "10.0.0.8" "10.0.0.9" "10.0.0.10")

log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

generate_new_keys() {
    local client_name=$1
    
    # Generate new key pair in container using temp files
    pct exec $SERVER_CONTAINER -- sh -c 'wg genkey | tee /tmp/privkey | wg pubkey > /tmp/pubkey' >/dev/null 2>&1
    local private_key=$(pct exec $SERVER_CONTAINER -- cat /tmp/privkey | tr -d '\n')
    local public_key=$(pct exec $SERVER_CONTAINER -- cat /tmp/pubkey | tr -d '\n')
    pct exec $SERVER_CONTAINER -- rm -f /tmp/privkey /tmp/pubkey
    
    echo "$private_key:$public_key"
}

get_random_ip() {
    # Get a random IP from available pool
    local random_index=$((RANDOM % ${#AVAILABLE_IPS[@]}))
    echo "${AVAILABLE_IPS[$random_index]}"
}

update_server_config() {
    local client_name=$1
    local new_public_key=$2
    local new_ip=$3
    
    log_message "Updating server config for $client_name with IP $new_ip"
    
    # Create new server config with rotated peer
    pct exec $SERVER_CONTAINER -- tee /etc/wireguard/wg0.conf << EOF
[Interface]
PrivateKey = kHLNqCd3UaFN33wu+XrUXCQ25G46BLQCmwrWl5iaGmA=
Address = 10.0.0.1/24
ListenPort = 51820
SaveConfig = false
PostUp = iptables -A FORWARD -i %i -j ACCEPT; iptables -A FORWARD -o %i -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
PostDown = iptables -D FORWARD -i %i -j ACCEPT; iptables -D FORWARD -o %i -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE

# $client_name Client (Rotated)
[Peer]
PublicKey = $new_public_key
AllowedIPs = $new_ip/32

# Client 1 (CT-101 Gluetun)
[Peer]
PublicKey = 3jWeOTnH5DgVvfvkdJ3NFAyJoSAqioDHpMsyVHNb0AY=
AllowedIPs = 10.0.0.3/32
PersistentKeepalive = 25
EOF
}

create_client_config() {
    local client_name=$1
    local private_key=$2
    local new_ip=$3
    local server_public_key="4XByD6O1U5OAyuSv1lkxqv9rNd3TF3hCAOHuAEN3KT4="
    
    log_message "Creating new client config for $client_name"
    
    # Create client config in container
    pct exec $SERVER_CONTAINER -- mkdir -p /etc/wireguard/clients
    pct exec $SERVER_CONTAINER -- tee "/etc/wireguard/clients/$client_name-$(date +%Y%m%d-%H%M).conf" << EOF
[Interface]
PrivateKey = $private_key
Address = $new_ip/32
DNS = 1.1.1.1, 8.8.8.8, 9.9.9.9

[Peer]
PublicKey = $server_public_key
Endpoint = 192.168.122.100:51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
EOF
    
    # Also save locally for easy access
    mkdir -p "$CLIENT_CONFIG_DIR"
    tee "$CLIENT_CONFIG_DIR/$client_name-$(date +%Y%m%d-%H%M).conf" << EOF
[Interface]
PrivateKey = $private_key
Address = $new_ip/32
DNS = 1.1.1.1, 8.8.8.8, 9.9.9.9

[Peer]
PublicKey = $server_public_key
Endpoint = 192.168.122.100:51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
EOF
}

restart_wireguard() {
    log_message "Restarting WireGuard service"
    pct exec $SERVER_CONTAINER -- wg-quick down wg0 2>/dev/null || true
    sleep 2
    pct exec $SERVER_CONTAINER -- wg-quick up wg0
}

rotate_client() {
    local client_name=$1
    
    log_message "Starting rotation for client: $client_name"
    
    # Generate new keys
    local key_pair=$(generate_new_keys "$client_name")
    local private_key=$(echo "$key_pair" | cut -d: -f1)
    local public_key=$(echo "$key_pair" | cut -d: -f2)
    
    # Get new IP
    local new_ip=$(get_random_ip)
    
    # Update configurations
    update_server_config "$client_name" "$public_key" "$new_ip"
    create_client_config "$client_name" "$private_key" "$new_ip"
    
    # Restart WireGuard
    restart_wireguard
    
    log_message "Rotation completed for $client_name. New IP: $new_ip"
    
    # Display new config path
    echo ""
    echo "=== NEW CLIENT CONFIG ==="
    echo "Client: $client_name"
    echo "New IP: $new_ip"
    echo "Config: $CLIENT_CONFIG_DIR/$client_name-$(date +%Y%m%d-%H%M).conf"
    echo "========================="
}

# Main execution
case "${1:-}" in
    "garuda-host"|"garuda")
        rotate_client "garuda-host"
        ;;
    "auto")
        log_message "Auto rotation started"
        rotate_client "garuda-host"
        ;;
    "status")
        echo "Current WireGuard status:"
        pct exec $SERVER_CONTAINER -- wg show
        ;;
    *)
        echo "Usage: $0 {garuda-host|auto|status}"
        echo "  garuda-host  - Rotate keys/IP for Garuda host"
        echo "  auto         - Automatic rotation"
        echo "  status       - Show current WireGuard status"
        exit 1
        ;;
esac
