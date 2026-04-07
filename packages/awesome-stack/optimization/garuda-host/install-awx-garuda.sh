#!/bin/bash

# AWX Installation and Migration Script for Garuda Host
# External control of entire Proxmox stack from Garuda host
# Migrates existing Ansible configuration from VM-800

set -e

echo "🚀 Installing AWX on Garuda Host for External Stack Control"
echo "This will provide complete oversight of your Proxmox infrastructure"

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo "Please run as regular user (not root). AWX will use sudo when needed."
   exit 1
fi

# Configuration
AWX_PROJECT_DIR="/opt/awx"
ANSIBLE_DIR="/opt/ansible"
VM_800_IP="192.168.122.3"  # Adjust if different
PROXMOX_VM_IP="192.168.122.9"  # ProxMox-Stack VM
AWX_ADMIN_USER="admin"
AWX_ADMIN_PASSWORD="awx_admin_$(openssl rand -hex 8)"

echo "📋 Configuration:"
echo "   • AWX Directory: $AWX_PROJECT_DIR"
echo "   • Ansible Directory: $ANSIBLE_DIR"
echo "   • VM-800 IP: $VM_800_IP"
echo "   • ProxMox-Stack IP: $PROXMOX_VM_IP"
echo "   • AWX Admin Password: $AWX_ADMIN_PASSWORD"

# 1. Install required packages
echo "📦 Installing required packages..."
sudo pacman -Syu --noconfirm
sudo pacman -S --needed --noconfirm \
    docker \
    docker-compose \
    python \
    python-pip \
    python-virtualenv \
    ansible \
    git \
    curl \
    wget \
    openssh \
    rsync \
    jq \
    make \
    gcc \
    nodejs \
    npm

# Enable and start Docker
sudo systemctl enable --now docker
sudo usermod -aG docker $(whoami)

# Create AWX directories
echo "📁 Creating AWX directories..."
sudo mkdir -p $AWX_PROJECT_DIR
sudo mkdir -p $ANSIBLE_DIR
sudo chown -R $(whoami):$(whoami) $AWX_PROJECT_DIR
sudo chown -R $(whoami):$(whoami) $ANSIBLE_DIR

# 2. Install AWX
echo "🏗️ Installing AWX..."
cd $AWX_PROJECT_DIR

# Clone AWX
git clone https://github.com/ansible/awx.git
cd awx

# Install AWX CLI
pip install --user awxkit

# Create AWX docker-compose configuration
cat > docker-compose.yml << EOF
version: '3.8'

services:
  web:
    image: quay.io/ansible/awx:latest
    container_name: awx_web
    depends_on:
      - redis
      - postgres
    ports:
      - "8080:8052"
    hostname: awxweb
    user: root
    restart: unless-stopped
    volumes:
      - supervisor-socket:/var/run/supervisor
      - rsyslog-socket:/var/run/awx-rsyslog/
      - rsyslog-config:/var/lib/awx/rsyslog/
      - "$AWX_PROJECT_DIR/projects:/var/lib/awx/projects:rw"
      - "$AWX_PROJECT_DIR/job_status:/var/lib/awx/job_status:rw"
    environment:
      http_proxy: 
      https_proxy: 
      no_proxy: 
      SECRET_KEY: awxsecret
      DATABASE_NAME: awx
      DATABASE_USER: awx
      DATABASE_PASSWORD: awxpass
      DATABASE_HOST: postgres
      DATABASE_PORT: 5432
      MEMCACHED_HOST: redis
      RABBITMQ_HOST: redis
      AWX_ADMIN_USER: $AWX_ADMIN_USER
      AWX_ADMIN_PASSWORD: $AWX_ADMIN_PASSWORD

  task:
    image: quay.io/ansible/awx:latest
    container_name: awx_task
    depends_on:
      - redis
      - postgres
      - web
    command: /usr/bin/launch_awx_task.sh
    hostname: awx
    user: root
    restart: unless-stopped
    volumes:
      - supervisor-socket:/var/run/supervisor
      - rsyslog-socket:/var/run/awx-rsyslog/
      - rsyslog-config:/var/lib/awx/rsyslog/
      - "$AWX_PROJECT_DIR/projects:/var/lib/awx/projects:rw"
      - "$AWX_PROJECT_DIR/job_status:/var/lib/awx/job_status:rw"
    environment:
      http_proxy: 
      https_proxy: 
      no_proxy: 
      SECRET_KEY: awxsecret
      DATABASE_NAME: awx
      DATABASE_USER: awx
      DATABASE_PASSWORD: awxpass
      DATABASE_HOST: postgres
      DATABASE_PORT: 5432
      MEMCACHED_HOST: redis
      RABBITMQ_HOST: redis
      AWX_ADMIN_USER: $AWX_ADMIN_USER
      AWX_ADMIN_PASSWORD: $AWX_ADMIN_PASSWORD

  redis:
    image: redis
    container_name: awx_redis
    restart: unless-stopped
    environment:
      http_proxy: 
      https_proxy: 
      no_proxy: 

  postgres:
    image: postgres:13
    container_name: awx_postgres
    restart: unless-stopped
    volumes:
      - awx-db:/var/lib/postgresql/data:Z
    environment:
      POSTGRES_USER: awx
      POSTGRES_PASSWORD: awxpass
      POSTGRES_DB: awx
      http_proxy: 
      https_proxy: 
      no_proxy: 

volumes:
  supervisor-socket:
  rsyslog-socket:
  rsyslog-config:
  awx-db:
EOF

# Create projects directory structure
echo "📁 Creating project structure..."
mkdir -p $AWX_PROJECT_DIR/projects/{vm-optimization,infrastructure,monitoring}
mkdir -p $AWX_PROJECT_DIR/inventories
mkdir -p $AWX_PROJECT_DIR/job_status

# 3. Start AWX
echo "🚀 Starting AWX..."
docker-compose up -d

# Wait for AWX to be ready
echo "⏳ Waiting for AWX to be ready..."
sleep 60

# Check if AWX is running
for i in {1..30}; do
    if curl -s http://localhost:8080/api/v2/ping/ | grep -q "OK"; then
        echo "✅ AWX is ready!"
        break
    fi
    echo "   Waiting... ($i/30)"
    sleep 10
done

# 4. Copy existing Ansible configuration from VM-800
echo "📋 Migrating Ansible configuration from VM-800..."

# Create migration script for VM-800
cat > migrate_from_vm800.sh << 'EOF'
#!/bin/bash
# Migration script to copy Ansible config from VM-800

VM_800_IP="192.168.122.86"
MIGRATION_DIR="/tmp/ansible_migration"

echo "🔄 Migrating Ansible configuration from VM-800..."

# Create migration directory
mkdir -p $MIGRATION_DIR

# Copy from VM-800 (adjust paths as needed)
echo "📥 Copying inventories from VM-800..."
scp -r root@$VM_800_IP:/etc/ansible/inventories/* $MIGRATION_DIR/ 2>/dev/null || \
scp -r root@$VM_800_IP:/opt/ansible/inventories/* $MIGRATION_DIR/ 2>/dev/null || \
scp -r root@$VM_800_IP:~/ansible/inventories/* $MIGRATION_DIR/ 2>/dev/null || \
echo "   No inventories found in standard locations"

echo "📥 Copying playbooks from VM-800..."
scp -r root@$VM_800_IP:/etc/ansible/playbooks/* $MIGRATION_DIR/playbooks/ 2>/dev/null || \
scp -r root@$VM_800_IP:/opt/ansible/playbooks/* $MIGRATION_DIR/playbooks/ 2>/dev/null || \
scp -r root@$VM_800_IP:~/ansible/playbooks/* $MIGRATION_DIR/playbooks/ 2>/dev/null || \
echo "   No playbooks found in standard locations"

echo "📥 Copying roles from VM-800..."
scp -r root@$VM_800_IP:/etc/ansible/roles/* $MIGRATION_DIR/roles/ 2>/dev/null || \
scp -r root@$VM_800_IP:/opt/ansible/roles/* $MIGRATION_DIR/roles/ 2>/dev/null || \
scp -r root@$VM_800_IP:~/ansible/roles/* $MIGRATION_DIR/roles/ 2>/dev/null || \
echo "   No roles found in standard locations"

echo "📥 Copying ansible.cfg..."
scp root@$VM_800_IP:/etc/ansible/ansible.cfg $MIGRATION_DIR/ 2>/dev/null || \
scp root@$VM_800_IP:~/ansible/ansible.cfg $MIGRATION_DIR/ 2>/dev/null || \
echo "   No ansible.cfg found"

# Copy to AWX projects directory
echo "📋 Installing migrated configuration..."
if [ -d "$MIGRATION_DIR" ]; then
    cp -r $MIGRATION_DIR/* /opt/awx/projects/infrastructure/ 2>/dev/null || true
fi

echo "✅ Migration from VM-800 complete!"
EOF

chmod +x migrate_from_vm800.sh

# 5. Create comprehensive inventory for entire stack
echo "📋 Creating comprehensive inventory..."
cat > $AWX_PROJECT_DIR/inventories/complete_stack.yml << EOF
---
# Complete Awesome Stack Inventory - External Control from Garuda Host
# Manages entire Proxmox infrastructure from outside

all:
  children:
    # Proxmox VMs
    proxmox_vms:
      hosts:
        proxmox-stack:
          ansible_host: "$PROXMOX_VM_IP"
          ansible_user: root
          vm_id: "proxmox-stack"
          vm_role: "main_infrastructure"
          expected_containers: 50
          
        vm-800:
          ansible_host: "$VM_800_IP"
          ansible_user: root
          vm_id: "vm-800"
          vm_role: "ansible_controller"
          services: ["ansible", "awx", "automation"]
          
    # Service categories
    automation_controllers:
      hosts:
        vm-800:
      vars:
        automation_type: "ansible"
        
    container_hosts:
      hosts:
        proxmox-stack:
      vars:
        container_runtime: "docker"
        max_containers: 60
        
    warp_agent_hosts:
      hosts:
        proxmox-stack:
      vars:
        warp_agents_enabled: true
        agent_bridge_port: 8080
        message_broker_host: "$VM_800_IP"
        
    media_stack_hosts:
      hosts:
        proxmox-stack:
      vars:
        plex_enabled: true
        jellyfin_enabled: true
        arr_services_enabled: true
        traefik_enabled: true
        
  vars:
    # Global variables
    ansible_python_interpreter: /usr/bin/python3
    ansible_become: true
    ansible_ssh_common_args: '-o StrictHostKeyChecking=no'
    
    # External control settings
    control_host: "garuda_awx"
    external_control: true
    
    # Network configuration
    network_range: "192.168.122.0/24"
    gateway: "192.168.122.1"
    
    # AWX integration
    awx_enabled: true
    awx_host: "localhost:8080"
    awx_user: "$AWX_ADMIN_USER"
EOF

# 6. Create optimization playbooks for AWX
echo "📝 Creating optimization playbooks..."

# Main VM optimization playbook
cat > $AWX_PROJECT_DIR/projects/vm-optimization/optimize_stack.yml << 'EOF'
---
- name: Complete Stack Optimization - External Control
  hosts: all
  gather_facts: yes
  vars:
    awx_external_control: true
    optimization_timestamp: "{{ ansible_date_time.iso8601 }}"
    
  tasks:
    - name: System Information Gathering
      ansible.builtin.debug:
        msg:
          - "Host: {{ ansible_hostname }}"
          - "IP: {{ ansible_default_ipv4.address }}"
          - "OS: {{ ansible_distribution }} {{ ansible_distribution_version }}"
          - "RAM: {{ ansible_memtotal_mb }}MB"
          - "CPU: {{ ansible_processor_vcpus }} cores"
          
    - name: Deploy optimization based on host role
      include_tasks: "optimize_{{ vm_role }}.yml"
      when: vm_role is defined
      
    - name: External control verification
      ansible.builtin.uri:
        url: "http://{{ awx_host }}/api/v2/ping/"
        method: GET
      register: awx_ping
      delegate_to: localhost
      run_once: true
      
    - name: Report optimization status
      ansible.builtin.debug:
        msg: "Optimization complete for {{ ansible_hostname }} - controlled from {{ awx_host }}"
EOF

# Create infrastructure management playbook
cat > $AWX_PROJECT_DIR/projects/infrastructure/manage_infrastructure.yml << 'EOF'
---
- name: Infrastructure Management - Complete Stack
  hosts: all
  tasks:
    - name: Check VM status
      ansible.builtin.command: systemctl is-active qemu-guest-agent
      register: vm_status
      ignore_errors: yes
      
    - name: Verify container services
      ansible.builtin.systemd:
        name: docker
      register: docker_status
      when: "'container_hosts' in group_names"
      
    - name: Check Warp agents
      ansible.builtin.command: pgrep -f warp
      register: warp_status
      ignore_errors: yes
      when: "'warp_agent_hosts' in group_names"
      
    - name: Report infrastructure status
      ansible.builtin.debug:
        var: "{{ item }}"
      loop:
        - vm_status
        - docker_status
        - warp_status
      when: item is defined
EOF

# 7. Create AWX configuration script
echo "⚙️ Creating AWX configuration..."
cat > configure_awx.py << EOF
#!/usr/bin/env python3
"""
AWX Configuration Script
Sets up projects, inventories, and job templates for external stack control
"""

import requests
import json
import time
import sys

AWX_URL = "http://localhost:8080"
AWX_USER = "$AWX_ADMIN_USER"
AWX_PASS = "$AWX_ADMIN_PASSWORD"

def awx_request(method, endpoint, data=None):
    """Make request to AWX API"""
    url = f"{AWX_URL}/api/v2{endpoint}"
    auth = (AWX_USER, AWX_PASS)
    
    if method.upper() == 'GET':
        response = requests.get(url, auth=auth)
    elif method.upper() == 'POST':
        response = requests.post(url, auth=auth, json=data)
    elif method.upper() == 'PUT':
        response = requests.put(url, auth=auth, json=data)
    
    return response

def configure_awx():
    """Configure AWX for external stack control"""
    print("🔧 Configuring AWX for external stack control...")
    
    # Create organization
    org_data = {
        "name": "Awesome Stack",
        "description": "Complete self-hosting infrastructure management"
    }
    response = awx_request('POST', '/organizations/', org_data)
    print(f"Organization: {response.status_code}")
    
    # Create project
    project_data = {
        "name": "VM Optimization Project",
        "description": "External control of Proxmox VMs and containers",
        "organization": 1,
        "scm_type": "manual",
        "local_path": "vm-optimization"
    }
    response = awx_request('POST', '/projects/', project_data)
    print(f"Project: {response.status_code}")
    
    # Create inventory
    inventory_data = {
        "name": "Complete Stack Inventory",
        "description": "All VMs and services in the stack",
        "organization": 1
    }
    response = awx_request('POST', '/inventories/', inventory_data)
    print(f"Inventory: {response.status_code}")
    
    # Create job template
    template_data = {
        "name": "Optimize Complete Stack",
        "description": "External optimization of entire infrastructure",
        "job_type": "run",
        "inventory": 1,
        "project": 1,
        "playbook": "optimize_stack.yml",
        "verbosity": 1
    }
    response = awx_request('POST', '/job_templates/', template_data)
    print(f"Job Template: {response.status_code}")
    
    print("✅ AWX configuration complete!")

if __name__ == "__main__":
    # Wait for AWX to be ready
    for i in range(30):
        try:
            response = requests.get(f"{AWX_URL}/api/v2/ping/")
            if response.status_code == 200:
                break
        except:
            pass
        print(f"Waiting for AWX... ({i+1}/30)")
        time.sleep(10)
    
    configure_awx()
EOF

chmod +x configure_awx.py

# 8. Create management scripts
echo "📝 Creating management scripts..."

# AWX management script
cat > manage_awx.sh << EOF
#!/bin/bash
# AWX Management Script for Garuda Host

AWX_DIR="$AWX_PROJECT_DIR/awx"

case "\$1" in
    start)
        echo "🚀 Starting AWX..."
        cd \$AWX_DIR && docker-compose up -d
        ;;
    stop)
        echo "🛑 Stopping AWX..."
        cd \$AWX_DIR && docker-compose down
        ;;
    restart)
        echo "🔄 Restarting AWX..."
        cd \$AWX_DIR && docker-compose restart
        ;;
    status)
        echo "📊 AWX Status:"
        cd \$AWX_DIR && docker-compose ps
        ;;
    logs)
        echo "📋 AWX Logs:"
        cd \$AWX_DIR && docker-compose logs -f
        ;;
    migrate)
        echo "🔄 Migrating from VM-800..."
        ./migrate_from_vm800.sh
        ;;
    *)
        echo "Usage: \$0 {start|stop|restart|status|logs|migrate}"
        exit 1
        ;;
esac
EOF

chmod +x manage_awx.sh

# 9. Final setup and verification
echo "🔧 Running final setup..."

# Install Python AWX kit
pip install --user awxkit

# Copy our optimization scripts to AWX projects
cp /usr/local/bin/optimize-proxmox-vm-enhanced.sh $AWX_PROJECT_DIR/projects/vm-optimization/ 2>/dev/null || true
cp /usr/local/bin/optimize-proxmox-vm.sh $AWX_PROJECT_DIR/projects/vm-optimization/ 2>/dev/null || true

# Set proper permissions
sudo chown -R $(whoami):$(whoami) $AWX_PROJECT_DIR
chmod -R 755 $AWX_PROJECT_DIR/projects

echo ""
echo "🎉 ===== AWX INSTALLATION COMPLETE ===== 🎉"
echo ""
echo "✅ AWX installed on Garuda host for external control"
echo "✅ Docker containers started"
echo "✅ Project structure created"
echo "✅ Migration scripts prepared"
echo "✅ Management scripts available"
echo ""
echo "🔧 AWX Access:"
echo "   • URL: http://localhost:8080"
echo "   • Username: $AWX_ADMIN_USER"
echo "   • Password: $AWX_ADMIN_PASSWORD"
echo ""
echo "📋 Next Steps:"
echo "1. Run migration: ./migrate_from_vm800.sh"
echo "2. Configure AWX: python3 configure_awx.py"
echo "3. Access AWX at: http://localhost:8080"
echo "4. Import your existing playbooks from VM-800"
echo ""
echo "🎯 Management Commands:"
echo "   • Start AWX: ./manage_awx.sh start"
echo "   • Stop AWX: ./manage_awx.sh stop"
echo "   • View status: ./manage_awx.sh status"
echo "   • Migrate config: ./manage_awx.sh migrate"
echo ""
echo "🚀 Your Garuda host now has complete external control of the entire stack!"
echo ""

# Save important info
cat > $AWX_PROJECT_DIR/INSTALLATION_INFO.txt << EOF
AWX Installation Information
===========================

Installation Date: $(date)
AWX URL: http://localhost:8080
Admin Username: $AWX_ADMIN_USER
Admin Password: $AWX_ADMIN_PASSWORD

Directories:
- AWX Project: $AWX_PROJECT_DIR
- Ansible Config: $ANSIBLE_DIR

VM Targets:
- VM-800 (Ansible): $VM_800_IP
- ProxMox-Stack: $PROXMOX_VM_IP

Management:
- Start AWX: cd $AWX_PROJECT_DIR && ./manage_awx.sh start
- Configure: cd $AWX_PROJECT_DIR && python3 configure_awx.py
- Migrate: cd $AWX_PROJECT_DIR && ./migrate_from_vm800.sh

Your Garuda host now has complete external control of your entire infrastructure!
EOF

echo "📄 Installation info saved to: $AWX_PROJECT_DIR/INSTALLATION_INFO.txt"
EOF
