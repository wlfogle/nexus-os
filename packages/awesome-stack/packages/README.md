# Agent Deployment Packages

This directory contains deployment packages and manifests for inter-agent communication and coordination within the awesome-stack infrastructure.

## 📦 Available Packages

### [alexa-agent/](alexa-agent/)
Complete deployment package for the Alexa Media Bridge agent (CT-200).

- **Package**: alexa-agent-complete-20250805-0642.tar.gz (674MB)
- **Location**: NFS Share `/mnt/nfs_share/garuda_package/`
- **Features**: Media processing, inter-agent communication, message polling
- **Status**: ✅ Ready for deployment

## 🔗 Inter-Agent Communication

All packages are designed to work with:
- **NFS File Server**: Shared storage at `/mnt/nfs_share`
- **Message Broker**: 192.168.122.86:8080
- **HTTP API**: Local agent APIs on port 9090
- **Polling System**: 30-second message polling intervals

## 🚀 Deployment Process

1. **Access NFS Share**: Ensure container has NFS client mounted
2. **Extract Package**: From `/mnt/nfs_share/garuda_package/`
3. **Follow Documentation**: Each package includes setup guides
4. **Test Communication**: Verify inter-agent message polling
5. **Coordinate Operations**: Use shared scripts and logs

## 📋 Package Standards

Each deployment package includes:
- ✅ Complete runtime environment
- ✅ Python virtual environment with dependencies
- ✅ System package requirements list
- ✅ Inter-agent communication scripts
- ✅ NFS integration setup
- ✅ Comprehensive documentation
- ✅ Health monitoring and startup scripts

## 🛠️ Infrastructure Requirements

- **NFS Server**: Proxmox host with `/srv/nfs_share`
- **Container Network**: Access to message broker
- **Python Environment**: 3.11+ with venv support
- **System Packages**: Media processing tools (ffmpeg, vlc, etc.)

---
**Note**: Large package files are stored on NFS share to avoid Git repository bloat. Documentation and manifests are version controlled here.
