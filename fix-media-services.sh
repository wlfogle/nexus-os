#!/bin/bash
# 🎯 FOCUSED MEDIA STACK SERVICE FIX
# Targets key media services for proper systemd configuration

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1"; }
warn() { echo -e "${YELLOW}[$(date +'%H:%M:%S')] WARNING:${NC} $1"; }
error() { echo -e "${RED}[$(date +'%H:%M:%S')] ERROR:${NC} $1"; }

# Function to fix Sonarr
fix_sonarr() {
    local ct_id=214
    log "Fixing Sonarr (CT $ct_id)"
    
    # Check if Sonarr is installed
    if ! ssh proxmox "pct exec $ct_id -- test -d /opt/Sonarr" 2>/dev/null; then
        log "Installing Sonarr..."
        ssh proxmox "pct exec $ct_id -- mkdir -p /opt"
        ssh proxmox "pct exec $ct_id -- bash -c 'cd /opt && wget -q https://github.com/Sonarr/Sonarr/releases/download/v4.0.10.2544/Sonarr.main.4.0.10.2544.linux-x64.tar.gz -O sonarr.tar.gz'"
        ssh proxmox "pct exec $ct_id -- bash -c 'cd /opt && tar -xzf sonarr.tar.gz && rm sonarr.tar.gz'"
        ssh proxmox "pct exec $ct_id -- chown -R root:root /opt/Sonarr"
    fi
    
    # Create systemd service
    cat > /tmp/sonarr.service << 'EOF'
[Unit]
Description=Sonarr Daemon
After=network.target

[Service]
User=root
Group=root
Type=forking
ExecStart=/opt/Sonarr/Sonarr -nobrowser -data=/config
TimeoutStopSec=20
KillMode=process
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF
    
    ssh proxmox "pct push $ct_id /tmp/sonarr.service /etc/systemd/system/sonarr.service"
    ssh proxmox "pct exec $ct_id -- systemctl daemon-reload"
    ssh proxmox "pct exec $ct_id -- systemctl enable sonarr"
    ssh proxmox "pct exec $ct_id -- systemctl start sonarr" || warn "Could not start Sonarr"
    rm /tmp/sonarr.service
}

# Function to fix Radarr
fix_radarr() {
    local ct_id=215
    log "Fixing Radarr (CT $ct_id)"
    
    # Check if Radarr is installed
    if ! ssh proxmox "pct exec $ct_id -- test -d /opt/Radarr" 2>/dev/null; then
        log "Installing Radarr..."
        ssh proxmox "pct exec $ct_id -- mkdir -p /opt"
        ssh proxmox "pct exec $ct_id -- bash -c 'cd /opt && wget -q https://github.com/Radarr/Radarr/releases/download/v5.14.0.9383/Radarr.master.5.14.0.9383.linux-core-x64.tar.gz -O radarr.tar.gz'"
        ssh proxmox "pct exec $ct_id -- bash -c 'cd /opt && tar -xzf radarr.tar.gz && rm radarr.tar.gz'"
        ssh proxmox "pct exec $ct_id -- chown -R root:root /opt/Radarr"
    fi
    
    # Create systemd service
    cat > /tmp/radarr.service << 'EOF'
[Unit]
Description=Radarr Daemon
After=network.target

[Service]
User=root
Group=root
Type=forking
ExecStart=/opt/Radarr/Radarr -nobrowser -data=/config
TimeoutStopSec=20
KillMode=process
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF
    
    ssh proxmox "pct push $ct_id /tmp/radarr.service /etc/systemd/system/radarr.service"
    ssh proxmox "pct exec $ct_id -- systemctl daemon-reload"
    ssh proxmox "pct exec $ct_id -- systemctl enable radarr"
    ssh proxmox "pct exec $ct_id -- systemctl start radarr" || warn "Could not start Radarr"
    rm /tmp/radarr.service
}

# Function to fix Prowlarr
fix_prowlarr() {
    local ct_id=210
    log "Fixing Prowlarr (CT $ct_id)"
    
    # Check if Prowlarr is installed
    if ! ssh proxmox "pct exec $ct_id -- test -d /opt/Prowlarr" 2>/dev/null; then
        log "Installing Prowlarr..."
        ssh proxmox "pct exec $ct_id -- mkdir -p /opt"
        ssh proxmox "pct exec $ct_id -- bash -c 'cd /opt && wget -q https://github.com/Prowlarr/Prowlarr/releases/download/v1.28.2.4885/Prowlarr.master.1.28.2.4885.linux-core-x64.tar.gz -O prowlarr.tar.gz'"
        ssh proxmox "pct exec $ct_id -- bash -c 'cd /opt && tar -xzf prowlarr.tar.gz && rm prowlarr.tar.gz'"
        ssh proxmox "pct exec $ct_id -- chown -R root:root /opt/Prowlarr"
    fi
    
    # Create systemd service
    cat > /tmp/prowlarr.service << 'EOF'
[Unit]
Description=Prowlarr Daemon
After=network.target

[Service]
User=root
Group=root
Type=forking
ExecStart=/opt/Prowlarr/Prowlarr -nobrowser -data=/config
TimeoutStopSec=20
KillMode=process
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF
    
    ssh proxmox "pct push $ct_id /tmp/prowlarr.service /etc/systemd/system/prowlarr.service"
    ssh proxmox "pct exec $ct_id -- systemctl daemon-reload"
    ssh proxmox "pct exec $ct_id -- systemctl enable prowlarr"
    ssh proxmox "pct exec $ct_id -- systemctl start prowlarr" || warn "Could not start Prowlarr"
    rm /tmp/prowlarr.service
}

# Function to optimize container
optimize_container() {
    local ct_id=$1
    local name=$2
    log "Optimizing $name (CT $ct_id)"
    
    # Basic optimization
    ssh proxmox "pct exec $ct_id -- bash -c 'echo \"vm.swappiness=10\" >> /etc/sysctl.conf'" 2>/dev/null || true
    ssh proxmox "pct exec $ct_id -- sysctl -p" 2>/dev/null || true
    
    # Ensure essential packages
    ssh proxmox "pct exec $ct_id -- apt-get update -qq" 2>/dev/null || true
    ssh proxmox "pct exec $ct_id -- apt-get install -y curl wget" 2>/dev/null || true
}

# Main execution
main() {
    log "🎯 Starting Focused Media Stack Service Fix"
    log "============================================"
    
    # Fix key services
    fix_sonarr
    optimize_container 214 "Sonarr"
    
    fix_radarr
    optimize_container 215 "Radarr"
    
    fix_prowlarr
    optimize_container 210 "Prowlarr"
    
    log "Checking service status..."
    
    # Check status
    for ct_service in "214:sonarr" "215:radarr" "210:prowlarr"; do
        ct_id=$(echo $ct_service | cut -d: -f1)
        service=$(echo $ct_service | cut -d: -f2)
        
        status=$(ssh proxmox "pct exec $ct_id -- systemctl is-active $service" 2>/dev/null || echo "inactive")
        if [[ "$status" == "active" ]]; then
            log "✅ $service (CT $ct_id): Running"
        else
            warn "⚠️  $service (CT $ct_id): $status"
        fi
    done
    
    log "🚀 Key media services fix complete!"
}

main "$@"
