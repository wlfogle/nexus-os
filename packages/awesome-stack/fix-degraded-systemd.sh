#!/bin/bash
#
# Script to fix degraded systemd services in Proxmox containers.

set -euo pipefail

# --- Configuration ---
PROXMOX_HOST="proxmox"
DEGRADED_CTS=("104" "107" "220" "224" "230" "231" "260" "274" "900" "950")
INACCESSIBLE_CTS=("100" "103" "106" "234")
ALL_CTS=("${DEGRADED_CTS[@]}" "${INACCESSIBLE_CTS[@]}")

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# --- Functions ---

# Safely execute a command on the Proxmox host
proxmox_exec() {
    ssh "$PROXMOX_HOST" "$@"
}

# Safely execute a command inside a container
container_exec() {
    local ct_id=$1
    shift
    proxmox_exec "pct exec $ct_id -- bash -c '$*'"
}

# Check and fix a single container
fix_container() {
    local ct_id=$1
    echo -e "${BLUE}--- Processing Container $ct_id ---${NC}"

    # 1. Check current systemd status
    local initial_status
    initial_status=$(container_exec "$ct_id" "systemctl is-system-running --wait 2>/dev/null || echo 'error'")
    echo "Initial status: $initial_status"

    if [[ "$initial_status" == "degraded" ]]; then
        # 2. List failed units
        echo -e "  ${YELLOW}Listing failed units...${NC}"
        local failed_units
        failed_units=$(container_exec "$ct_id" "systemctl --failed --no-legend --no-pager")

        if [[ -n "$failed_units" ]]; then
            echo "$failed_units"
            
            # 3. Attempt to reset failed units
            echo -e "  ${YELLOW}Attempting to reset failed units...${NC}"
            container_exec "$ct_id" "systemctl reset-failed"
            echo "  Reset command sent."
        else
            echo "  No failed units listed, but status is degraded. May be a transient issue."
        fi

    elif [[ "$initial_status" == "running" || "$initial_status" == "online" ]]; then
        echo -e "  ${GREEN}Systemd is already running correctly.${NC}"
        return
    else
        echo -e "  ${RED}Systemd is not in a 'degraded' state (Status: $initial_status). Checking journal...${NC}"
        local journal
        journal=$(container_exec "$ct_id" "journalctl -b -n 5 --no-pager")
        echo "  Last 5 journal lines:"
        echo "$journal"
        return
    fi
    
    # 4. Verify the new status
    echo -e "  ${YELLOW}Verifying new status...${NC}"
    sleep 3 # Give systemd a moment
    local final_status
    final_status=$(container_exec "$ct_id" "systemctl is-system-running --wait 2>/dev/null || echo 'error'")

    if [[ "$final_status" == "running" ]]; then
        echo -e "  ${GREEN}Success! Container $ct_id systemd is now running.${NC}"
    else
        echo -e "  ${RED}Failed. Container $ct_id systemd is still '$final_status'. Manual investigation needed.${NC}"
    fi
}

# --- Main Execution ---

echo "Starting systemd fix script..."
echo "Checking ${#ALL_CTS[@]} containers."

for ct in "${ALL_CTS[@]}"; do
    fix_container "$ct"
    echo
done

echo "Script finished."

