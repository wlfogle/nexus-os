#!/bin/bash

# Container Audit Script
# Checks each container for binaries in /opt and /usr and systemd status
# Created: $(date)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Log file
LOG_FILE="./container-audit-$(date +%Y%m%d-%H%M%S).log"

echo -e "${BLUE}=== Container Audit Script ===${NC}"
echo "Log file: $LOG_FILE"
echo "Started: $(date)"
echo

# Function to log and print
log_print() {
    echo -e "$1" | tee -a "$LOG_FILE"
}

# Proxmox host (use alias or IP)
PROXMOX_HOST="proxmox"

# Function to check if container exists and is running
check_container_status() {
    local ct_id=$1
    if ssh "$PROXMOX_HOST" "pct status '$ct_id'" &>/dev/null; then
        local status=$(ssh "$PROXMOX_HOST" "pct status '$ct_id'" | awk '{print $2}')
        echo "$status"
    else
        echo "not_found"
    fi
}

# Function to safely execute command in container
safe_exec() {
    local ct_id=$1
    local cmd=$2
    local timeout_duration=${3:-10}
    
    timeout "$timeout_duration" ssh "$PROXMOX_HOST" "pct exec '$ct_id' -- bash -c '$cmd'" 2>/dev/null || echo "ERROR_OR_TIMEOUT"
}

# Function to check binaries in directory
check_binaries() {
    local ct_id=$1
    local dir=$2
    local description=$3
    
    log_print "    ${YELLOW}Checking $description ($dir):${NC}"
    
    # Check if directory exists
    local dir_check=$(safe_exec "$ct_id" "[ -d '$dir' ] && echo 'EXISTS' || echo 'NOT_EXISTS'")
    
    if [[ "$dir_check" == "NOT_EXISTS" ]]; then
        log_print "      Directory $dir does not exist"
        return
    fi
    
    # Find executable files
    local binaries=$(safe_exec "$ct_id" "find '$dir' -type f -executable 2>/dev/null | head -20" 15)
    
    if [[ "$binaries" == "ERROR_OR_TIMEOUT" ]]; then
        log_print "      ${RED}Error or timeout checking $dir${NC}"
        return
    fi
    
    if [[ -z "$binaries" ]]; then
        log_print "      No executable binaries found in $dir"
    else
        local count=$(echo "$binaries" | wc -l)
        log_print "      ${GREEN}Found $count executable files:${NC}"
        echo "$binaries" | head -10 | while read -r binary; do
            if [[ -n "$binary" ]]; then
                log_print "        $binary"
            fi
        done
        
        if [[ $count -gt 10 ]]; then
            log_print "        ... and $((count - 10)) more"
        fi
    fi
}

# Function to check systemd status
check_systemd() {
    local ct_id=$1
    
    log_print "    ${YELLOW}Checking systemd status:${NC}"
    
    # Check if systemd is installed
    local systemd_installed=$(safe_exec "$ct_id" "which systemctl >/dev/null 2>&1 && echo 'INSTALLED' || echo 'NOT_INSTALLED'")
    
    if [[ "$systemd_installed" == "NOT_INSTALLED" ]]; then
        log_print "      ${RED}systemd not installed${NC}"
        return
    fi
    
    log_print "      ${GREEN}systemd is installed${NC}"
    
    # Check if systemd is running
    local systemd_running=$(safe_exec "$ct_id" "systemctl is-system-running 2>/dev/null || echo 'NOT_RUNNING'")
    
    if [[ "$systemd_running" == "NOT_RUNNING" || "$systemd_running" == "ERROR_OR_TIMEOUT" ]]; then
        log_print "      ${YELLOW}systemd may not be running or accessible${NC}"
    else
        log_print "      ${GREEN}systemd status: $systemd_running${NC}"
    fi
    
    # Get some basic systemd info
    local systemd_version=$(safe_exec "$ct_id" "systemctl --version 2>/dev/null | head -1")
    if [[ "$systemd_version" != "ERROR_OR_TIMEOUT" && -n "$systemd_version" ]]; then
        log_print "      Version: $systemd_version"
    fi
    
    # Check active services count
    local active_services=$(safe_exec "$ct_id" "systemctl list-units --type=service --state=active --no-pager --no-legend 2>/dev/null | wc -l")
    if [[ "$active_services" != "ERROR_OR_TIMEOUT" && "$active_services" =~ ^[0-9]+$ ]]; then
        log_print "      ${GREEN}Active services: $active_services${NC}"
    fi
    
    # List some key services
    local key_services=$(safe_exec "$ct_id" "systemctl list-units --type=service --state=active --no-pager --no-legend 2>/dev/null | head -5 | awk '{print \$1}'")
    if [[ "$key_services" != "ERROR_OR_TIMEOUT" && -n "$key_services" ]]; then
        log_print "      Key active services:"
        echo "$key_services" | while read -r service; do
            if [[ -n "$service" ]]; then
                log_print "        $service"
            fi
        done
    fi
}

# Function to get container info
get_container_info() {
    local ct_id=$1
    
    # Get container description/hostname
    local description=$(safe_exec "$ct_id" "hostname 2>/dev/null")
    if [[ "$description" == "ERROR_OR_TIMEOUT" ]]; then
        description="Unknown"
    fi
    
    # Get OS info
    local os_info=$(safe_exec "$ct_id" "cat /etc/os-release 2>/dev/null | grep '^PRETTY_NAME' | cut -d'=' -f2 | tr -d '\"'")
    if [[ "$os_info" == "ERROR_OR_TIMEOUT" ]]; then
        os_info="Unknown OS"
    fi
    
    log_print "    Hostname: $description"
    log_print "    OS: $os_info"
}

# Main execution
main() {
    log_print "${BLUE}Starting container audit...${NC}\n"
    
    # Get list of all containers
    local containers
    if ! containers=$(ssh "$PROXMOX_HOST" "pct list" 2>/dev/null); then
        log_print "${RED}Error: Unable to list containers via SSH to $PROXMOX_HOST.${NC}"
        exit 1
    fi
    
    # Parse container IDs (skip header)
    local ct_ids=$(echo "$containers" | tail -n +2 | awk '{print $1}')
    
    if [[ -z "$ct_ids" ]]; then
        log_print "${YELLOW}No containers found.${NC}"
        exit 0
    fi
    
    local total_containers=$(echo "$ct_ids" | wc -l)
    log_print "Found $total_containers containers to audit\n"
    
    local running_count=0
    local stopped_count=0
    local error_count=0
    
    # Process each container
    for ct_id in $ct_ids; do
        log_print "${BLUE}=== Container $ct_id ===${NC}"
        
        local status=$(check_container_status "$ct_id")
        
        case "$status" in
            "running")
                log_print "  ${GREEN}Status: Running${NC}"
                running_count=$((running_count + 1))
                
                # Get basic container info
                get_container_info "$ct_id"
                
                # Check binaries in /opt
                check_binaries "$ct_id" "/opt" "Optional software packages"
                
                # Check binaries in /usr
                check_binaries "$ct_id" "/usr" "User binaries"
                
                # Check systemd
                check_systemd "$ct_id"
                ;;
            "stopped")
                log_print "  ${YELLOW}Status: Stopped - Skipping checks${NC}"
                stopped_count=$((stopped_count + 1))
                ;;
            "not_found")
                log_print "  ${RED}Status: Container not found${NC}"
                error_count=$((error_count + 1))
                ;;
            *)
                log_print "  ${YELLOW}Status: $status - Skipping checks${NC}"
                error_count=$((error_count + 1))
                ;;
        esac
        
        log_print ""
    done
    
    # Summary
    log_print "${BLUE}=== Audit Summary ===${NC}"
    log_print "Total containers: $total_containers"
    log_print "${GREEN}Running: $running_count${NC}"
    log_print "${YELLOW}Stopped: $stopped_count${NC}"
    log_print "${RED}Errors/Other: $error_count${NC}"
    log_print ""
    log_print "Audit completed: $(date)"
    log_print "Full log saved to: $LOG_FILE"
}

# Check SSH connectivity to Proxmox
if ! ssh -o ConnectTimeout=5 "$PROXMOX_HOST" "echo 'SSH connection OK'" &>/dev/null; then
    echo -e "${RED}Error: Cannot connect to Proxmox host '$PROXMOX_HOST' via SSH${NC}"
    exit 1
fi

# Run main function
main "$@"
