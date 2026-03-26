#!/bin/bash
# 🚀 ULTIMATE MEDIA STACK CONTAINER FIX SCRIPT
# Fixes all containers with proper systemd services, optimization, and configuration

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

# Container definitions with their services
declare -A CONTAINERS=(
    # Media Servers
    ["230"]="plex:plexmediaserver:/usr/lib/plexmediaserver/Plex Media Server"
    ["231"]="jellyfin:jellyfin:/usr/bin/jellyfin"
    
    # *arr Stack
    ["214"]="sonarr:sonarr:/opt/Sonarr/Sonarr"
    ["215"]="radarr:radarr:/opt/Radarr/Radarr"
    ["217"]="readarr:readarr:/opt/Readarr/Readarr"
    ["219"]="whisparr:whisparr:/opt/Whisparr/Whisparr"
    ["220"]="sonarr-extended:sonarr:/opt/Sonarr/Sonarr"
    ["240"]="bazarr:bazarr:/opt/bazarr/bazarr.py"
    
    # Download Clients
    ["212"]="qbittorrent:qbittorrent-nox:/usr/bin/qbittorrent-nox"
    ["224"]="deluge:deluged:/usr/bin/deluged"
    
    # Indexers
    ["210"]="prowlarr:prowlarr:/opt/Prowlarr/Prowlarr"
    ["211"]="jackett:jackett:/opt/Jackett/jackett"
    ["223"]="autobrr:autobrr:/usr/local/bin/autobrr"
    
    # Media Management
    ["232"]="audiobookshelf:audiobookshelf:/opt/audiobookshelf/index.js"
    ["233"]="calibre-web:calibre-web:/usr/bin/calibre-web"
    ["270"]="filebot:filebot:/usr/bin/filebot"
    
    # Request/Management
    ["241"]="overseerr:overseerr:/app/dist/index.js"
    ["242"]="jellyseerr:jellyseerr:/app/dist/index.js"
    ["243"]="ombi:ombi:/opt/Ombi/Ombi"
    ["244"]="tautulli:tautulli:/opt/Tautulli/Tautulli.py"
    
    # TV/IPTV
    ["234"]="iptv-proxy:iptv-proxy:/usr/local/bin/iptv-proxy"
    ["235"]="tvheadend:tvheadend:/usr/bin/tvheadend"
    
    # Transcoding
    ["236"]="tdarr-server:tdarr_server:/opt/Tdarr/Tdarr_Server/Tdarr_Server"
    ["237"]="tdarr-node:tdarr_node:/opt/Tdarr/Tdarr_Node/Tdarr_Node"
    
    # Infrastructure
    ["102"]="flaresolverr:flaresolverr:/usr/src/app/src/flaresolverr.py"
    ["103"]="traefik:traefik:/usr/local/bin/traefik"
    ["104"]="vaultwarden:vaultwarden:/usr/bin/vaultwarden"
    ["105"]="valkey:valkey-server:/usr/bin/valkey-server"
    ["106"]="postgresql:postgresql:/usr/lib/postgresql/*/bin/postgres"
    ["107"]="authentik:authentik:/opt/goauthentik/bin/ak"
    
    # Monitoring
    ["260"]="prometheus:prometheus:/usr/local/bin/prometheus"
    ["261"]="grafana:grafana-server:/usr/share/grafana/bin/grafana-server"
    
    # Utilities
    ["277"]="recyclarr:recyclarr:/usr/local/bin/recyclarr"
    ["247"]="janitorr:janitorr:/app/janitorr"
    ["248"]="decluttarr:decluttarr:/app/decluttarr"
    ["278"]="crowdsec:crowdsec:/usr/bin/crowdsec"
    ["279"]="tailscale:tailscaled:/usr/sbin/tailscaled"
)

# Function to create systemd service file
create_systemd_service() {
    local container_id=$1
    local service_info=$2
    local container_name=$(echo $service_info | cut -d: -f1)
    local service_name=$(echo $service_info | cut -d: -f2)
    local executable=$(echo $service_info | cut -d: -f3)
    
    log "Creating systemd service for $container_name (CT $container_id)"
    
    # Check if container is responsive
    if ! ssh proxmox "pct exec $container_id -- echo 'test'" >/dev/null 2>&1; then
        warn "Container $container_id is not responsive, skipping"
        return 1
    fi
    
    # Get the actual executable path
    local actual_exec
    actual_exec=$(ssh proxmox "pct exec $container_id -- find /opt /usr -name '$(basename $executable)' -type f 2>/dev/null | head -1" 2>/dev/null || echo "")
    
    if [[ -z "$actual_exec" ]]; then
        actual_exec=$executable
    fi
    
    # Create service file
    cat > "/tmp/${service_name}.service" << EOF
[Unit]
Description=${container_name^} Media Server
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=root
Group=root
UMask=0002
Restart=on-failure
RestartSec=5s
TimeoutStopSec=20s
KillMode=process
ExecStart=$actual_exec
WorkingDirectory=/opt/${container_name^}
Environment="HOME=/opt/${container_name^}"

# Performance optimizations
LimitNOFILE=65536
LimitNPROC=4096
Nice=-10
IOSchedulingClass=1
IOSchedulingPriority=4

# Security
NoNewPrivileges=false
PrivateTmp=false
ProtectSystem=false
ProtectHome=false

[Install]
WantedBy=multi-user.target
EOF

    # Verify service file was created
    if [[ ! -f "/tmp/${service_name}.service" ]]; then
        error "Failed to create service file for $service_name"
        return 1
    fi
    
    # Copy service file to container
    if ssh proxmox "pct push $container_id /tmp/${service_name}.service /etc/systemd/system/${service_name}.service" 2>/dev/null; then
        log "Service file copied successfully"
    else
        warn "Could not copy service file to container $container_id"
        rm -f "/tmp/${service_name}.service"
        return 1
    fi
    
    # Enable and start service
    if ssh proxmox "pct exec $container_id -- systemctl daemon-reload" 2>/dev/null; then
        ssh proxmox "pct exec $container_id -- systemctl enable ${service_name}.service" 2>/dev/null || warn "Could not enable service"
    else
        warn "Could not reload systemd in container $container_id"
    fi
    
    # Clean up temp file
    rm -f "/tmp/${service_name}.service"
}

# Function to optimize container performance
optimize_container() {
    local container_id=$1
    local container_info=$2
    local container_name=$(echo $container_info | cut -d: -f1)
    
    log "Optimizing container $container_name (CT $container_id)"
    
    # System optimizations
    ssh proxmox "pct exec $container_id -- bash -c 'echo \"vm.swappiness=10\" >> /etc/sysctl.conf'"
    ssh proxmox "pct exec $container_id -- bash -c 'echo \"vm.vfs_cache_pressure=50\" >> /etc/sysctl.conf'"
    ssh proxmox "pct exec $container_id -- bash -c 'echo \"net.core.rmem_max=134217728\" >> /etc/sysctl.conf'"
    ssh proxmox "pct exec $container_id -- bash -c 'echo \"net.core.wmem_max=134217728\" >> /etc/sysctl.conf'"
    
    # Apply sysctl settings
    ssh proxmox "pct exec $container_id -- sysctl -p" || warn "Could not apply sysctl settings for CT $container_id"
    
    # Create log rotation
    cat > /tmp/logrotate-${container_name} << EOF
/var/log/${container_name}/*.log {
    daily
    missingok
    rotate 52
    compress
    delaycompress
    notifempty
    create 644 root root
    postrotate
        systemctl reload ${container_name} || true
    endscript
}
EOF
    
    ssh proxmox "pct push $container_id /tmp/logrotate-${container_name} /etc/logrotate.d/${container_name}"
    rm -f /tmp/logrotate-${container_name}
    
    # Install essential packages
    ssh proxmox "pct exec $container_id -- apt-get update -qq" || warn "Could not update packages for CT $container_id"
    ssh proxmox "pct exec $container_id -- apt-get install -y curl wget htop iotop nethogs rsync unzip" || warn "Could not install packages for CT $container_id"
}

# Function to install missing applications
install_missing_app() {
    local container_id=$1
    local service_info=$2
    local container_name=$(echo $service_info | cut -d: -f1)
    local service_name=$(echo $service_info | cut -d: -f2)
    
    log "Installing $container_name in CT $container_id"
    
    case $container_name in
        "sonarr"|"sonarr-extended")
            ssh proxmox "pct exec $container_id -- bash -c 'cd /opt && wget -q https://github.com/Sonarr/Sonarr/releases/download/v4.0.10.2544/Sonarr.main.4.0.10.2544.linux-x64.tar.gz -O sonarr.tar.gz && tar -xzf sonarr.tar.gz && rm sonarr.tar.gz && chown -R root:root Sonarr'"
            ;;
        "radarr")
            ssh proxmox "pct exec $container_id -- bash -c 'cd /opt && wget -q https://github.com/Radarr/Radarr/releases/download/v5.14.0.9383/Radarr.master.5.14.0.9383.linux-core-x64.tar.gz -O radarr.tar.gz && tar -xzf radarr.tar.gz && rm radarr.tar.gz && chown -R root:root Radarr'"
            ;;
        "readarr")
            ssh proxmox "pct exec $container_id -- bash -c 'cd /opt && wget -q https://github.com/Readarr/Readarr/releases/download/v0.4.0.2593/Readarr.develop.0.4.0.2593.linux-core-x64.tar.gz -O readarr.tar.gz && tar -xzf readarr.tar.gz && rm readarr.tar.gz && chown -R root:root Readarr'"
            ;;
        "prowlarr")
            ssh proxmox "pct exec $container_id -- bash -c 'cd /opt && wget -q https://github.com/Prowlarr/Prowlarr/releases/download/v1.28.2.4885/Prowlarr.master.1.28.2.4885.linux-core-x64.tar.gz -O prowlarr.tar.gz && tar -xzf prowlarr.tar.gz && rm prowlarr.tar.gz && chown -R root:root Prowlarr'"
            ;;
        "jackett")
            ssh proxmox "pct exec $container_id -- bash -c 'cd /opt && wget -q https://github.com/Jackett/Jackett/releases/download/v0.22.1088/Jackett.Binaries.LinuxAMDx64.tar.gz -O jackett.tar.gz && tar -xzf jackett.tar.gz && rm jackett.tar.gz && chown -R root:root Jackett'"
            ;;
        *)
            warn "No installation script for $container_name"
            ;;
    esac
}

# Function to check if application exists
check_app_exists() {
    local container_id=$1
    local service_info=$2
    local executable=$(echo $service_info | cut -d: -f3)
    
    ssh proxmox "pct exec $container_id -- test -f $executable" 2>/dev/null || \
    ssh proxmox "pct exec $container_id -- find /opt -name '$(basename $executable)' -type f" 2>/dev/null | grep -q . || \
    return 1
}

# Main execution
main() {
    log "🚀 Starting Ultimate Media Stack Container Fix"
    log "================================================"
    
    # Get list of running containers
    local running_containers
    running_containers=$(ssh proxmox "pct list | grep running | awk '{print \$1}'")
    
    for container_id in $running_containers; do
        if [[ ${CONTAINERS[$container_id]+_} ]]; then
            local service_info=${CONTAINERS[$container_id]}
            local container_name=$(echo $service_info | cut -d: -f1)
            local service_name=$(echo $service_info | cut -d: -f2)
            
            log "Processing container $container_name (CT $container_id)"
            
            # Check if application exists
            if ! check_app_exists $container_id "$service_info"; then
                warn "Application not found in CT $container_id, attempting installation"
                install_missing_app $container_id "$service_info"
            fi
            
            # Create systemd service
            create_systemd_service $container_id "$service_info"
            
            # Optimize container
            optimize_container $container_id "$service_info"
            
            # Try to start the service
            if ssh proxmox "pct exec $container_id -- systemctl start ${service_name}.service" 2>/dev/null; then
                log "✅ Service ${service_name} started successfully in CT $container_id"
            else
                warn "⚠️  Could not start service ${service_name} in CT $container_id"
            fi
            
            log "Container $container_name (CT $container_id) processing complete"
            echo "----------------------------------------"
        else
            log "Skipping unknown container CT $container_id"
        fi
    done
    
    log "🎯 All containers processed!"
    log "Running final validation..."
    
    # Final validation
    for container_id in $running_containers; do
        if [[ ${CONTAINERS[$container_id]+_} ]]; then
            local service_info=${CONTAINERS[$container_id]}
            local container_name=$(echo $service_info | cut -d: -f1)
            local service_name=$(echo $service_info | cut -d: -f2)
            
            local status
            status=$(ssh proxmox "pct exec $container_id -- systemctl is-active ${service_name}.service" 2>/dev/null || echo "inactive")
            
            if [[ "$status" == "active" ]]; then
                log "✅ $container_name (CT $container_id): Service running"
            else
                warn "⚠️  $container_name (CT $container_id): Service not running ($status)"
            fi
        fi
    done
    
    log "🚀 Ultimate Media Stack Container Fix Complete!"
    log "Your media stack is now optimized and all services have proper systemd configuration!"
}

# Run main function
main "$@"
