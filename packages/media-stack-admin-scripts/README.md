# 🛠️ Media Stack Admin Scripts

A comprehensive collection of production-ready scripts for managing self-hosted media infrastructure, containers, and Home Assistant automation.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Bash](https://img.shields.io/badge/Bash-4.0%2B-green.svg)](https://www.gnu.org/software/bash/)
[![Tested on](https://img.shields.io/badge/Tested%20on-Garuda%20Linux-blue.svg)](https://garudalinux.org/)

## 🎯 **Overview**

This repository contains **16 production-tested scripts** (1,882+ lines) for automating media stack infrastructure management. These tools have been battle-tested in real-world self-hosting environments and provide robust solutions for common administrative tasks.

## 📁 **Script Categories**

### 🏠 **Home Assistant & Smart Home**
- **`apply-homeassistant-fixes.sh`** (187 lines) - Comprehensive HA troubleshooting and repair
- **`fix-homeassistant-connectivity.sh`** (186 lines) - Network connectivity diagnostics and fixes
- **`setup_alexa_bridge.sh`** (107 lines) - Alexa integration bridge setup

### 🐳 **Container Management**
- **`fix-all-containers.sh`** (328 lines) - Automated container health checks and repairs
- **`fix-media-services.sh`** (184 lines) - Media service troubleshooting and optimization

### 🌐 **Network & Infrastructure**
- **`port-forwarding-rules.sh`** (44 lines) - iptables port forwarding automation
- **`port-forwarding-health-check.sh`** (143 lines) - Comprehensive network health monitoring
- **`update-duckdns.sh`** (18 lines) - Dynamic DNS updates for DuckDNS

### 🖥️ **System Optimization**
- **`hardware_optimization.sh`** (313 lines) - System performance tuning and hardware optimization
- **`monitor_performance.sh`** (36 lines) - Real-time performance monitoring

### ☁️ **Proxmox Management**
- **`configure-proxmox-vm.sh`** (52 lines) - VM configuration automation
- **`create-proxmox-vm-final.sh`** (78 lines) - Advanced VM creation
- **`create-proxmox-vm.sh`** (73 lines) - Standard VM creation
- **`create-proxmox-vm-simple.sh`** (77 lines) - Simplified VM setup
- **`fix-proxmox-ssl.sh`** (56 lines) - SSL certificate management

## 🚀 **Quick Start**

### Prerequisites
```bash
# Ensure you have required tools
sudo pacman -S docker docker-compose iptables curl wget
# or
sudo apt install docker.io docker-compose iptables curl wget
```

### Installation
```bash
# Clone the repository
git clone https://github.com/your-username/media-stack-admin-scripts.git
cd media-stack-admin-scripts

# Make scripts executable
chmod +x *.sh

# Run individual scripts as needed
sudo ./fix-all-containers.sh
./hardware_optimization.sh
```

## 📋 **Usage Examples**

### Container Health Management
```bash
# Fix all container issues automatically
sudo ./fix-all-containers.sh

# Specific media service troubleshooting
sudo ./fix-media-services.sh
```

### Network Administration
```bash
# Apply port forwarding rules
sudo ./port-forwarding-rules.sh

# Run comprehensive health check
./port-forwarding-health-check.sh
```

### Home Assistant Management
```bash
# Apply all HA fixes and optimizations
sudo ./apply-homeassistant-fixes.sh

# Fix connectivity issues
./fix-homeassistant-connectivity.sh
```

### System Optimization
```bash
# Optimize hardware performance
sudo ./hardware_optimization.sh

# Monitor system performance
./monitor_performance.sh
```

## 🔧 **Features**

### ✅ **Production Ready**
- Comprehensive error handling and logging
- Safe operation modes with confirmation prompts
- Detailed status reporting and diagnostics
- Rollback capabilities where applicable

### ✅ **Self-Hosting Focused**
- Docker and container management
- Reverse proxy configurations
- SSL certificate automation
- Network security hardening

### ✅ **Infrastructure as Code**
- Idempotent operations
- Configuration templating
- Automated service discovery
- Health monitoring integration

## 🏗️ **Architecture**

These scripts are designed for typical self-hosted media stack architectures:

```
Internet → Router → Reverse Proxy → Services
                ↓
            [Monitoring & Management Scripts]
                ↓
    Docker Containers ← → Proxmox VMs ← → Home Assistant
```

## 📊 **Script Statistics**

| Category | Scripts | Lines of Code | Primary Focus |
|----------|---------|---------------|---------------|
| Home Assistant | 3 | 400+ | Smart home automation |
| Container Management | 2 | 512+ | Docker infrastructure |
| Network & Infrastructure | 3 | 205+ | Network management |
| System Optimization | 2 | 349+ | Performance tuning |
| Proxmox Management | 5 | 336+ | Virtualization |
| Utilities | 1 | 18+ | DNS management |
| **Total** | **16** | **1,882+** | **Complete infrastructure** |

## 🎯 **Target Use Cases**

- **Self-hosted media servers** (Plex, Jellyfin, Emby)
- **Home automation systems** (Home Assistant, Node-RED)
- **Container orchestration** (Docker Compose, Portainer)
- **Virtualization management** (Proxmox, QEMU/KVM)
- **Network infrastructure** (reverse proxies, VPNs)

## 🛡️ **Security Features**

- Input validation and sanitization
- Privilege escalation controls
- Secure credential handling
- Network security hardening
- SSL/TLS configuration management

## 📚 **Documentation**

Each script includes:
- Comprehensive inline documentation
- Usage examples and parameter descriptions
- Error codes and troubleshooting guides
- Integration notes for related services

## 🤝 **Contributing**

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Test your changes in a safe environment
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## 📝 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ **Disclaimer**

These scripts are provided as-is and should be tested in a safe environment before production use. Always backup your configurations before running administrative scripts.

## 🆘 **Support**

- **Issues**: [GitHub Issues](https://github.com/your-username/media-stack-admin-scripts/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/media-stack-admin-scripts/discussions)
- **Wiki**: [Project Wiki](https://github.com/your-username/media-stack-admin-scripts/wiki)

## 🌟 **Acknowledgments**

Built for the self-hosting community. Tested on real infrastructure managing:
- Multiple Docker containers
- Home Assistant installations
- Proxmox virtualization clusters
- Media streaming services
- Network security infrastructure

---

**⭐ If these scripts help your infrastructure, please consider starring the repository!**
