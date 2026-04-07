#!/bin/bash
# WireGuard IP and Key Rotation Script
# Adapted for Pop!_OS with Docker-based WireGuard (linuxserver/wireguard)

set -e

# Configuration
WG_CONTAINER="wireguard"
WG_INTERFACE="wg0"
CLIENT_CONFIG_DIR="$HOME/wireguard-tools/clients"
LOG_FILE="$HOME/wireguard-tools/wg-rotation.log"
VPN_NETWORK="99.99.99"

# Available IP pool (excluding .1 server)
AVAILABLE_IPS=("99.99.99.2" "99.99.99.3" "99.99.99.4" "99.99.99.5" "99.99.99.6" "99.99.99.7" "99.99.99.8" "99.99.99.9" "99.99.99.10")

log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

docker_exec() {
    docker exec "$WG_CONTAINER" "$@"
}

generate_new_keys() {
    local client_name=$1

    local private_key=$(docker_exec wg genkey)
    local public_key=$(echo "$private_key" | docker_exec wg pubkey)

    echo "$private_key:$public_key"
}

get_server_public_key() {
    docker_exec wg show "$WG_INTERFACE" public-key 2>/dev/null || echo ""
}

get_server_endpoint() {
    # Detect the host's LAN IP for the endpoint
    ip -4 route get 1.1.1.1 2>/dev/null | awk '{print $7; exit}'
}

get_random_ip() {
    local random_index=$((RANDOM % ${#AVAILABLE_IPS[@]}))
    echo "${AVAILABLE_IPS[$random_index]}"
}

get_server_listen_port() {
    docker_exec wg show "$WG_INTERFACE" listen-port 2>/dev/null || echo "51820"
}

update_server_peer() {
    local client_name=$1
    local new_public_key=$2
    local new_ip=$3

    log_message "Updating server peer for $client_name with IP $new_ip"

    # Remove existing peer with the same allowed IPs (if any) then add new one
    local existing_peers
    existing_peers=$(docker_exec wg show "$WG_INTERFACE" allowed-ips 2>/dev/null || true)

    # Strip any old peer that had an IP from our pool
    for ip in "${AVAILABLE_IPS[@]}"; do
        local old_key
        old_key=$(echo "$existing_peers" | grep "$ip" | awk '{print $1}')
        if [[ -n "$old_key" ]]; then
            docker_exec wg set "$WG_INTERFACE" peer "$old_key" remove 2>/dev/null || true
        fi
    done

    # Add the new peer
    docker_exec wg set "$WG_INTERFACE" peer "$new_public_key" allowed-ips "$new_ip/32"

    log_message "Server peer updated for $client_name"
}

create_client_config() {
    local client_name=$1
    local private_key=$2
    local new_ip=$3

    local server_public_key
    server_public_key=$(get_server_public_key)
    local server_endpoint
    server_endpoint=$(get_server_endpoint)
    local server_port
    server_port=$(get_server_listen_port)

    if [[ -z "$server_public_key" ]]; then
        log_message "ERROR: Could not retrieve server public key"
        return 1
    fi

    log_message "Creating new client config for $client_name"

    mkdir -p "$CLIENT_CONFIG_DIR"

    local config_file="$CLIENT_CONFIG_DIR/$client_name-$(date +%Y%m%d-%H%M).conf"

    cat > "$config_file" << EOF
[Interface]
PrivateKey = $private_key
Address = $new_ip/32
DNS = 1.1.1.1, 8.8.8.8, 9.9.9.9

[Peer]
PublicKey = $server_public_key
Endpoint = $server_endpoint:$server_port
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
EOF

    chmod 600 "$config_file"
    log_message "Client config saved: $config_file"
}

rotate_client() {
    local client_name=$1

    log_message "Starting rotation for client: $client_name"

    # Verify container is running
    if ! docker ps --format '{{.Names}}' | grep -q "^${WG_CONTAINER}$"; then
        log_message "ERROR: WireGuard container '$WG_CONTAINER' is not running"
        exit 1
    fi

    # Generate new keys
    local key_pair
    key_pair=$(generate_new_keys "$client_name")
    local private_key
    private_key=$(echo "$key_pair" | cut -d: -f1)
    local public_key
    public_key=$(echo "$key_pair" | cut -d: -f2)

    # Get new IP
    local new_ip
    new_ip=$(get_random_ip)

    # Update server peer
    update_server_peer "$client_name" "$public_key" "$new_ip"

    # Create client config
    create_client_config "$client_name" "$private_key" "$new_ip"

    log_message "Rotation completed for $client_name. New IP: $new_ip"

    echo ""
    echo "=== NEW CLIENT CONFIG ==="
    echo "Client: $client_name"
    echo "New IP: $new_ip"
    echo "Config: $CLIENT_CONFIG_DIR/$client_name-$(date +%Y%m%d-%H%M).conf"
    echo "========================="
}

show_status() {
    echo "Current WireGuard status:"
    docker_exec wg show 2>/dev/null || echo "WireGuard container not running or interface not up"
    echo ""
    echo "Container status:"
    docker ps --filter "name=$WG_CONTAINER" --format "  {{.Names}}: {{.Status}}"
}

# Main execution
case "${1:-}" in
    "rotate")
        rotate_client "${2:-default-peer}"
        ;;
    "auto")
        log_message "Auto rotation started"
        rotate_client "auto-peer"
        ;;
    "status")
        show_status
        ;;
    "init")
        log_message "Initializing wireguard-tools"
        mkdir -p "$CLIENT_CONFIG_DIR"
        echo "Initialized. Client configs will be stored in: $CLIENT_CONFIG_DIR"
        echo "Log file: $LOG_FILE"
        show_status
        ;;
    *)
        echo "Usage: $0 {rotate [client-name]|auto|status|init}"
        echo "  rotate [name]  - Rotate keys/IP for a client"
        echo "  auto           - Automatic rotation (for cron)"
        echo "  status         - Show current WireGuard status"
        echo "  init           - Initialize directory structure"
        exit 1
        ;;
esac
