# 🚀 Awesome Stack Rebuild System

> **Complete infrastructure rebuild and automation system for the Awesome Stack**

This directory contains a comprehensive Ansible-based system for rebuilding the entire Awesome Stack infrastructure from scratch, including advanced optimization and shrinking capabilities.

## 📋 Overview

The rebuild system automates the deployment of:

- **🏗️ Proxmox VE Infrastructure**: VM and LXC management
- **📦 Golden Image Templates**: Warp Agent + OpenBox containers
- **🎬 Media Stack**: Plex, Jellyfin, Traefik load balancer
- **📊 Monitoring**: Prometheus, Grafana, health checks
- **🧠 AI Services**: Ollama, Open-Interpreter integration
- **🔐 Security**: Multi-layered Docker→LXC→VM→Proxmox architecture

## 🏗️ Architecture

```
Proxmox VM (192.168.122.9)
├── Golden Template (CT-999)
│   ├── Ubuntu 22.04 + OpenBox
│   ├── Warp Agent Bridge
│   ├── Docker-in-LXC
│   └── Agent Communication System
│
├── Core Services
│   ├── Traefik LB (CT-103)
│   ├── Ollama AI (CT-900)
│   └── Agent Comms (CT-950)
│
├── Media Services
│   ├── Plex Server (CT-230)
│   └── Jellyfin (CT-231)
│
├── Monitoring
│   ├── Grafana (CT-240)
│   └── Prometheus (CT-241)
│
└── Home Assistant (VM-500)
```

## 🚀 Quick Start

### Prerequisites
- Ubuntu 22.04+ system
- SSH access to Proxmox at `192.168.122.9`
- Ansible installed
- Root privileges

### Deploy the Stack

```bash
cd Ansible/rebuild-system
./scripts/deploy-awesome-stack.sh
```

### Optimize and Shrink (Post-deployment)

```bash
# After everything is configured and working
ansible-playbook -i inventories/production.yml playbooks/shrink-optimize.yml
```

## 📁 Directory Structure

```
rebuild-system/
├── inventories/
│   └── production.yml              # Host inventory
├── playbooks/
│   ├── master-rebuild.yml          # 9-phase deployment
│   └── shrink-optimize.yml         # Post-deployment optimization
├── roles/
│   ├── golden_image_create/        # Template creation
│   ├── lxc_container_deploy/       # Container deployment
│   ├── container_cleanup/          # Optimization & cleanup
│   ├── lxc_shrink_compress/        # Container compression
│   └── vm_shrink_optimize/         # VM optimization
├── scripts/
│   └── deploy-awesome-stack.sh     # Main deployment script
└── README.md                       # This file
```

## 🔧 Deployment Phases

### Phase 1-2: Infrastructure & Golden Image
- Proxmox environment setup
- Ubuntu 22.04 golden template creation
- OpenBox + Warp Agent installation
- Docker-in-LXC optimization

### Phase 3-6: Service Deployment
- Core services (Traefik, Ollama, Agent Comms)
- Media stack (Plex, Jellyfin)
- Monitoring (Prometheus, Grafana)
- Virtual machines (Home Assistant)

### Phase 7-9: Security & Validation
- SSL certificates and access control
- Health monitoring setup
- Documentation and backup configuration

## 🗜️ Optimization Features

### Container Cleanup
- Remove unnecessary packages and cache
- Clean logs and temporary files
- Docker image and volume pruning
- Python cache optimization

### LXC Compression
- Filesystem shrinking and resizing
- LVM volume optimization
- High-compression backups
- Thin provisioning optimization

### VM Optimization
- Proxmox-specific cleanup
- Performance parameter tuning
- Storage optimization
- Automated maintenance scheduling

## 🤖 Warp Agent System

Each container includes:
- **Warp Agent Bridge**: HTTP API for terminal communication
- **OpenBox Desktop**: GUI environment per container
- **SQLite Messaging**: Cross-container communication
- **Auto-configuration**: Service-specific setup

## 🛠️ Management Commands

```bash
# Health check all services
ansible-playbook -i inventories/production.yml playbooks/health-check.yml

# Deploy specific phases
./scripts/deploy-awesome-stack.sh infrastructure golden core

# Update services
ansible-playbook -i inventories/production.yml playbooks/update-services.yml
```

## 🌐 Service Access

After deployment, services are available at:

- **Traefik Dashboard**: http://192.168.122.103:9080/
- **Media Portal**: http://192.168.122.103:8080/
- **Plex**: http://192.168.122.230:32400/web
- **Jellyfin**: http://192.168.122.231:8096/
- **Grafana**: http://192.168.122.240:3000/
- **Home Assistant**: http://192.168.122.230:8123/
- **Ollama AI**: http://192.168.122.86:11434/

## 🔒 Security Features

- **Multi-layered isolation**: Docker→LXC→VM→Proxmox
- **Service-specific containers**: Isolated environments
- **Traefik load balancing**: Single entry point
- **SSL/TLS termination**: Secure communications
- **Health monitoring**: Automated checks

## 📊 Integration with Existing Stack

This rebuild system integrates with the existing awesome-stack:

- **Compatible** with existing Ansible scripts in `/ansible` and `/Ansible`
- **Extends** the functionality in `rebuild-ultimate-stack.sh`
- **Implements** concepts from `Recommended Stack for LXC + KVM Environments.md`
- **Automates** the optimization described in `Optimize.md`

## 🆘 Troubleshooting

### Common Issues

1. **SSH Connection Failed**
   ```bash
   ssh-copy-id root@192.168.122.9
   ```

2. **Container Won't Start**
   ```bash
   ssh root@192.168.122.9 "pct status <container-id>"
   ```

3. **Service Unreachable**
   ```bash
   curl -I http://192.168.122.103:8080
   ```

### Log Locations
- Deployment: `/var/log/awesome-stack/`
- Optimization: `/var/log/optimization/`
- Health checks: `/var/log/health-reports/`

## 🤝 Contributing

1. Test changes in your environment
2. Update documentation
3. Follow existing code patterns
4. Submit pull requests

## 📖 Related Documentation

- [`../README.md`](../README.md) - Main Ansible directory overview
- [`../Optimize.md`](../Optimize.md) - Optimization strategies
- [`../Recommended Stack for LXC + KVM Environments.md`](../Recommended%20Stack%20for%20LXC%20%2B%20KVM%20Environments.md) - Architecture details
- [`../rebuild-ultimate-stack.sh`](../rebuild-ultimate-stack.sh) - Legacy rebuild script

---

**🚀 Ready to rebuild your awesome stack? Start with `./scripts/deploy-awesome-stack.sh` and automate everything!**
