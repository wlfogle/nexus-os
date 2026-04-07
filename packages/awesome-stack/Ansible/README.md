# Awesome Stack

> **This repository now focuses on the current, supported media stack. Legacy scripts and docs are archived in `/legacy` and `/docs/_archive`.**

## 🚀 Quick Start

- **Stack Overview:** See [`docs/_organized/summary.md`](docs/_organized/summary.md)
- **Architecture:** Proxmox → VM → LXC → Docker → Traefik + Services
- **Automation:** Use the Ansible playbooks and roles in [`ansible/`](ansible/) for provisioning and rebuilds.
- **Main Entry Point:** [`rebuild-ultimate-stack.sh`](rebuild-ultimate-stack.sh) (see docs for full instructions)
- **🆕 Complete Rebuild System:** [`rebuild-system/`](rebuild-system/) - Advanced automation for full stack recreation

## 🛠️ Documentation

- **Implementation/Plan:** [`docs/_organized/summary.md`](docs/_organized/summary.md)
- **AWX/Ansible workflow:** [`ansible/README.md`](ansible/README.md), [`docs/AWX_workflow_example.md`](docs/AWX_workflow_example.md)
- **Proxmox/LXC/KVM stack details:** [`Recommended Stack for LXC + KVM Environments.md`](Recommended%20Stack%20for%20LXC%20%2B%20KVM%20Environments.md)
- **🆕 Rebuild System:** [`rebuild-system/README.md`](rebuild-system/README.md) - Complete infrastructure automation

## 🆕 New: Complete Rebuild System

The [`rebuild-system/`](rebuild-system/) directory contains a comprehensive Ansible-based automation system for rebuilding the entire Awesome Stack from scratch:

### Key Features
- **🏗️ Full Stack Automation**: 9-phase deployment process
- **📦 Golden Image Templates**: Warp Agent + OpenBox containers
- **🗜️ Advanced Optimization**: Container shrinking and VM optimization
- **🤖 Agent Communication**: Cross-container messaging system
- **📊 Complete Monitoring**: Prometheus + Grafana integration

### Quick Deploy
```bash
cd rebuild-system
./scripts/deploy-awesome-stack.sh
```

### Post-Deployment Optimization
```bash
# After everything is configured and working
ansible-playbook -i inventories/production.yml playbooks/shrink-optimize.yml
```

## 🗃️ Legacy

- Moved to [`/legacy`](legacy/) and [`/docs/_archive`](docs/_archive/).

---

**🚀 For new deployments, use the [`rebuild-system/`](rebuild-system/) for complete automation!**
**If you need something from the old stack, look in `/legacy` or `/docs/_archive`. All current work should use the new structure and playbooks!**
