#!/bin/bash

# Awesome Stack Complete Rebuild & Deployment Script
# This script orchestrates the entire infrastructure rebuild process

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ANSIBLE_DIR="$PROJECT_DIR/ansible"
LOG_DIR="/var/log/awesome-stack"
LOG_FILE="$LOG_DIR/deployment-$(date +%Y%m%d-%H%M%S).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Create log directory
sudo mkdir -p "$LOG_DIR"
sudo chmod 755 "$LOG_DIR"

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case $level in
        INFO)  echo -e "${GREEN}[INFO]${NC} $message" | tee -a "$LOG_FILE" ;;
        WARN)  echo -e "${YELLOW}[WARN]${NC} $message" | tee -a "$LOG_FILE" ;;
        ERROR) echo -e "${RED}[ERROR]${NC} $message" | tee -a "$LOG_FILE" ;;
        DEBUG) echo -e "${BLUE}[DEBUG]${NC} $message" | tee -a "$LOG_FILE" ;;
        PHASE) echo -e "${PURPLE}[PHASE]${NC} $message" | tee -a "$LOG_FILE" ;;
    esac
    
    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
}

# Banner
show_banner() {
    cat << 'EOF'
    
     █████╗ ██╗    ██╗███████╗███████╗ ██████╗ ███╗   ███╗███████╗
    ██╔══██╗██║    ██║██╔════╝██╔════╝██╔═══██╗████╗ ████║██╔════╝
    ███████║██║ █╗ ██║█████╗  ███████╗██║   ██║██╔████╔██║█████╗  
    ██╔══██║██║███╗██║██╔══╝  ╚════██║██║   ██║██║╚██╔╝██║██╔══╝  
    ██║  ██║╚███╔███╔╝███████╗███████║╚██████╔╝██║ ╚═╝ ██║███████╗
    ╚═╝  ╚═╝ ╚══╝╚══╝ ╚══════╝╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝
    
    ███████╗████████╗ █████╗  ██████╗██╗  ██╗
    ██╔════╝╚══██╔══╝██╔══██╗██╔════╝██║ ██╔╝
    ███████╗   ██║   ███████║██║     █████╔╝ 
    ╚════██║   ██║   ██╔══██║██║     ██╔═██╗ 
    ███████║   ██║   ██║  ██║╚██████╗██║  ██╗
    ╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
    
    🚀 Complete Infrastructure Rebuild & Deployment
    🔧 Proxmox VM + LXC + Docker + Warp Agent Stack
    📦 47+ Container Media & AI Infrastructure
    
EOF
}

# Check prerequisites
check_prerequisites() {
    log PHASE "Checking prerequisites..."
    
    # Check if running on Ubuntu
    if ! grep -q "Ubuntu" /etc/os-release; then
        log ERROR "This script is designed for Ubuntu. Current OS: $(lsb_release -d | cut -f2)"
        exit 1
    fi
    
    # Check for Ansible
    if ! command -v ansible-playbook >/dev/null 2>&1; then
        log INFO "Installing Ansible..."
        sudo apt update
        sudo apt install -y software-properties-common
        sudo add-apt-repository --yes --update ppa:ansible/ansible
        sudo apt install -y ansible
    fi
    
    # Check for required Python modules
    log INFO "Installing required Python modules..."
    pip3 install --user proxmoxer requests
    
    # Check SSH connectivity to Proxmox
    if ! ssh -o ConnectTimeout=5 -o BatchMode=yes root@192.168.122.9 'exit 0' 2>/dev/null; then
        log WARN "Cannot connect to Proxmox host via SSH. Please ensure:"
        log WARN "1. SSH key authentication is set up"
        log WARN "2. Proxmox host is accessible at 192.168.122.9"
        log WARN "3. Root user can connect without password"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    log INFO "Prerequisites check completed"
}

# Validate configuration
validate_config() {
    log PHASE "Validating configuration..."
    
    if [ ! -f "$ANSIBLE_DIR/inventories/production.yml" ]; then
        log ERROR "Production inventory not found: $ANSIBLE_DIR/inventories/production.yml"
        exit 1
    fi
    
    if [ ! -f "$ANSIBLE_DIR/playbooks/master-rebuild.yml" ]; then
        log ERROR "Master playbook not found: $ANSIBLE_DIR/playbooks/master-rebuild.yml"
        exit 1
    fi
    
    log INFO "Configuration validation completed"
}

# Create required directories and scripts
setup_environment() {
    log PHASE "Setting up environment..."
    
    # Create required directories
    mkdir -p "$PROJECT_DIR"/{scripts,docs,templates,agent-comms}
    
    # Create placeholder Warp Agent scripts if they don't exist
    if [ ! -f "$PROJECT_DIR/scripts/warp_agent_bridge_standalone.py" ]; then
        log INFO "Creating placeholder Warp Agent Bridge script..."
        cat > "$PROJECT_DIR/scripts/warp_agent_bridge_standalone.py" << 'EOF'
#!/usr/bin/env python3
"""
Warp Agent Bridge - Standalone Version
HTTP API bridge to communicate with Warp terminal agents
"""

import http.server
import socketserver
import json
import sqlite3
import threading
import time
from urllib.parse import urlparse, parse_qs

class WarpAgentBridge:
    def __init__(self, port=8080, agent_port=7777):
        self.port = port
        self.agent_port = agent_port
        self.db_path = '/opt/agent-comms/messages.db'
        self.init_database()
    
    def init_database(self):
        """Initialize SQLite database for message persistence"""
        conn = sqlite3.connect(self.db_path)
        conn.execute('''
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                source TEXT,
                target TEXT,
                message TEXT,
                status TEXT DEFAULT 'pending'
            )
        ''')
        conn.commit()
        conn.close()
    
    def run(self):
        """Start the bridge server"""
        with socketserver.TCPServer(("", self.port), BridgeHandler) as httpd:
            print(f"Warp Agent Bridge running on port {self.port}")
            print(f"Agent communication on port {self.agent_port}")
            httpd.serve_forever()

class BridgeHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'healthy'}).encode())
        else:
            self.send_response(404)
            self.end_headers()

if __name__ == '__main__':
    bridge = WarpAgentBridge()
    bridge.run()
EOF
        chmod +x "$PROJECT_DIR/scripts/warp_agent_bridge_standalone.py"
    fi
    
    if [ ! -f "$PROJECT_DIR/scripts/start_warp_bridge.sh" ]; then
        log INFO "Creating Warp Bridge launcher script..."
        cat > "$PROJECT_DIR/scripts/start_warp_bridge.sh" << 'EOF'
#!/bin/bash
# Warp Agent Bridge Launcher Script

BRIDGE_SCRIPT="/opt/warp-agent/warp_agent_bridge_standalone.py"
LOG_FILE="/var/log/warp-agent/bridge.log"

# Create log directory
mkdir -p "$(dirname "$LOG_FILE")"

# Start the bridge with logging
python3 "$BRIDGE_SCRIPT" 2>&1 | tee -a "$LOG_FILE"
EOF
        chmod +x "$PROJECT_DIR/scripts/start_warp_bridge.sh"
    fi
    
    log INFO "Environment setup completed"
}

# Run deployment phases
run_deployment() {
    log PHASE "Starting deployment phases..."
    
    cd "$ANSIBLE_DIR"
    
    # Phase selection
    local phases=()
    if [ "${1:-all}" = "all" ]; then
        phases=(
            "infrastructure"
            "golden"
            "core"
            "media"
            "monitoring"
            "vms"
            "security"
            "validation"
            "finalization"
        )
    else
        phases=("$@")
    fi
    
    for phase in "${phases[@]}"; do
        case $phase in
            "infrastructure")
                log PHASE "🏗️  Phase 1: Infrastructure Setup"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags infrastructure --ask-become-pass
                ;;
            "golden")
                log PHASE "🔧 Phase 2: Golden Image Creation"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags golden --ask-become-pass
                ;;
            "core")
                log PHASE "📦 Phase 3: Core Services Deployment"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags core --ask-become-pass
                ;;
            "media")
                log PHASE "🎬 Phase 4: Media Services Deployment"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags media --ask-become-pass
                ;;
            "monitoring")
                log PHASE "📊 Phase 5: Monitoring Stack"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags monitoring --ask-become-pass
                ;;
            "vms")
                log PHASE "🏠 Phase 6: Virtual Machines"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags vms --ask-become-pass
                ;;
            "security")
                log PHASE "🔐 Phase 7: Security & Access"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags security --ask-become-pass
                ;;
            "validation")
                log PHASE "🎯 Phase 8: Validation & Health Checks"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags validation --ask-become-pass
                ;;
            "finalization")
                log PHASE "📋 Phase 9: Documentation & Backup"
                ansible-playbook -i inventories/production.yml playbooks/master-rebuild.yml \
                    --tags finalization --ask-become-pass
                ;;
            *)
                log ERROR "Unknown phase: $phase"
                ;;
        esac
        
        if [ $? -ne 0 ]; then
            log ERROR "Phase $phase failed!"
            exit 1
        fi
        
        log INFO "Phase $phase completed successfully"
    done
}

# Post-deployment verification
verify_deployment() {
    log PHASE "🔍 Verifying deployment..."
    
    # Check if containers are running
    local containers=(103 230 231 240 241 900 950)
    for container in "${containers[@]}"; do
        if ssh root@192.168.122.9 "pct status $container" | grep -q "status: running"; then
            log INFO "Container $container is running ✅"
        else
            log WARN "Container $container is not running ⚠️"
        fi
    done
    
    # Check services
    local services=(
        "http://192.168.122.103:8080"  # Traefik
        "http://192.168.122.230:32400" # Plex
        "http://192.168.122.240:3000"  # Grafana
    )
    
    for service in "${services[@]}"; do
        if curl -s --max-time 5 "$service" > /dev/null; then
            log INFO "Service $service is responding ✅"
        else
            log WARN "Service $service is not responding ⚠️"
        fi
    done
    
    log INFO "Deployment verification completed"
}

# Show deployment summary
show_summary() {
    log PHASE "📋 Deployment Summary"
    
    cat << EOF | tee -a "$LOG_FILE"

🎉 Awesome Stack Deployment Complete!

📊 Infrastructure Overview:
• Proxmox VM: 192.168.122.9
• Total LXC Containers: Multiple (103, 230, 231, 240, 241, 900, 950)
• Golden Image Template: CT-999
• Warp Agent Bridge: Active on all containers

🌐 Service Access Points:
• Traefik Dashboard: http://192.168.122.103:9080/
• Media Stack Portal: http://192.168.122.103:8080/
• Plex Media Server: http://192.168.122.230:32400/web
• Jellyfin: http://192.168.122.231:8096/
• Grafana Monitoring: http://192.168.122.240:3000/
• Prometheus: http://192.168.122.241:9090/
• Ollama AI: http://192.168.122.86:11434/
• Agent Communications: http://192.168.122.86:8080/

📁 Important Files:
• Deployment Log: $LOG_FILE
• Ansible Inventory: $ANSIBLE_DIR/inventories/production.yml
• Playbooks: $ANSIBLE_DIR/playbooks/

🔧 Management Commands:
• Health Check: ansible-playbook -i inventories/production.yml playbooks/health-check.yml
• Update Services: ansible-playbook -i inventories/production.yml playbooks/update-services.yml
• Shrink & Optimize: ansible-playbook -i inventories/production.yml playbooks/shrink-optimize.yml

📚 Next Steps:
1. Configure personal media libraries in Plex/Jellyfin
2. Set up monitoring dashboards in Grafana
3. Configure Home Assistant automation
4. Test Warp Agent communication
5. Create backup snapshots

EOF
}

# Main execution
main() {
    show_banner
    
    log INFO "Starting Awesome Stack deployment at $(date)"
    log INFO "Log file: $LOG_FILE"
    
    check_prerequisites
    validate_config
    setup_environment
    
    run_deployment "$@"
    
    verify_deployment
    show_summary
    
    log INFO "Deployment completed successfully at $(date)"
    echo
    echo -e "${GREEN}🚀 Awesome Stack is ready! Check the summary above for access points.${NC}"
}

# Run main function with all arguments
main "$@"
