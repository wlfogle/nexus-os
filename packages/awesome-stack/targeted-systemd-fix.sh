#!/bin/bash
#
# Targeted systemd fixes based on audit findings

set -euo pipefail

PROXMOX_HOST="proxmox"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Safely execute command in container
container_exec() {
    local ct_id=$1
    shift
    ssh "$PROXMOX_HOST" "pct exec $ct_id -- bash -c '$*'"
}

# Fix VNC server issues
fix_vnc_issues() {
    local ct_id=$1
    echo -e "${BLUE}Fixing VNC issues in container $ct_id${NC}"
    
    # Stop and disable problematic VNC service
    container_exec "$ct_id" "systemctl stop vncserver@:1.service 2>/dev/null || true"
    container_exec "$ct_id" "systemctl disable vncserver@:1.service 2>/dev/null || true"
    
    # Reset failed state
    container_exec "$ct_id" "systemctl reset-failed 2>/dev/null || true"
    
    echo "VNC service disabled and reset"
}

# Fix journal corruption
fix_journal_corruption() {
    local ct_id=$1
    echo -e "${BLUE}Fixing journal corruption in container $ct_id${NC}"
    
    # Stop systemd-journald temporarily
    container_exec "$ct_id" "systemctl stop systemd-journald.service"
    
    # Remove corrupted journal files
    container_exec "$ct_id" "find /var/log/journal -name '*.journal' -type f -delete 2>/dev/null || true"
    
    # Restart journald to recreate clean journals
    container_exec "$ct_id" "systemctl start systemd-journald.service"
    
    # Reset any failed states
    container_exec "$ct_id" "systemctl reset-failed 2>/dev/null || true"
    
    echo "Journal corruption fixed"
}

# Check container health
check_container() {
    local ct_id=$1
    echo -e "${BLUE}Checking container $ct_id${NC}"
    
    # Test basic access
    if ! ssh "$PROXMOX_HOST" "pct exec $ct_id -- echo 'Container accessible'" 2>/dev/null; then
        echo -e "${RED}Container $ct_id is not accessible - may need restart${NC}"
        return 1
    fi
    
    # Check systemd status
    local status
    status=$(container_exec "$ct_id" "systemctl is-system-running 2>/dev/null || echo 'error'")
    echo "Systemd status: $status"
    
    if [[ "$status" == "degraded" ]]; then
        # Check for specific failed services
        local failed_services
        failed_services=$(container_exec "$ct_id" "systemctl --failed --no-legend --no-pager 2>/dev/null || echo ''")
        
        if echo "$failed_services" | grep -q "vncserver"; then
            fix_vnc_issues "$ct_id"
        fi
        
        # Reset all failed services
        container_exec "$ct_id" "systemctl reset-failed 2>/dev/null || true"
        
        # Check final status
        sleep 2
        local final_status
        final_status=$(container_exec "$ct_id" "systemctl is-system-running 2>/dev/null || echo 'error'")
        
        if [[ "$final_status" == "running" ]]; then
            echo -e "${GREEN}✓ Container $ct_id fixed - systemd now running${NC}"
        else
            echo -e "${YELLOW}⚠ Container $ct_id still shows: $final_status${NC}"
        fi
    else
        echo -e "${GREEN}✓ Container $ct_id systemd status is acceptable${NC}"
    fi
}

# Main execution
echo "Starting targeted systemd fixes..."

# Problematic containers from audit
CONTAINERS=("104" "107" "220" "224" "230" "231" "260" "274" "900" "950")

# Special case: Fix journal corruption in container 274 first
echo -e "${YELLOW}Special fix for container 274 (journal corruption)${NC}"
fix_journal_corruption "274"
echo

# Fix VNC issues in specific containers
echo -e "${YELLOW}Fixing VNC issues in containers 260 and 950${NC}"
fix_vnc_issues "260"
fix_vnc_issues "950"
echo

# Check all containers
for ct in "${CONTAINERS[@]}"; do
    check_container "$ct"
    echo
done

# Check the inaccessible containers
echo -e "${YELLOW}Checking previously inaccessible containers${NC}"
INACCESSIBLE=("100" "103" "106" "234")

for ct in "${INACCESSIBLE[@]}"; do
    echo -e "${BLUE}Checking container $ct${NC}"
    if ssh "$PROXMOX_HOST" "pct exec $ct -- echo 'Now accessible'" 2>/dev/null; then
        echo -e "${GREEN}✓ Container $ct is now accessible${NC}"
        check_container "$ct"
    else
        echo -e "${RED}✗ Container $ct still inaccessible - may need restart${NC}"
        # Check if container is running
        local ct_status
        ct_status=$(ssh "$PROXMOX_HOST" "pct status $ct" | awk '{print $2}')
        echo "Container status: $ct_status"
        
        if [[ "$ct_status" == "running" ]]; then
            echo -e "${YELLOW}Container is running but not responding - may need restart${NC}"
        fi
    fi
    echo
done

echo "Targeted fixes completed!"
