# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is the **AI-Enhanced Ultimate Rescue USB** project - a revolutionary system recovery and diagnostic tool that combines traditional rescue capabilities with cutting-edge AI and machine learning. The project includes 237+ functions across 10 major categories, from neural hardware diagnostics to behavioral security analysis.

## Architecture

The project follows a modular architecture with these core components:

- **AI Core Engine** (`ai-iso-installer.py`, `intelligent-optimizer.py`) - Neural diagnostic models, predictive maintenance, and behavioral security
- **Garuda Optimizations** (`garuda-ai-optimizations/`) - USB-specific performance optimizations for Garuda Linux
- **Neural Diagnostics** (`neural-diagnostics.sh`) - Hardware failure prediction and system health monitoring
- **Installation Systems** (`install-optimizations.sh`) - Automated deployment and configuration

## Development Commands

### Primary Development Tasks

```bash
# Install the AI optimizations (requires root)
sudo ./garuda-ai-optimizations/install-optimizations.sh

# Run neural diagnostics scan
sudo /opt/ai-rescue-tools/neural-diagnostics.sh

# Execute AI system optimizer
sudo python3 /opt/ai-rescue-tools/intelligent-optimizer.py

# Launch AI NLP ISO installer
python3 ai-iso-installer.py

# Test USB device detection
sudo ./garuda-ai-optimizations/usb-device-detector.sh

# Apply runtime USB optimizations
sudo ./garuda-ai-optimizations/usb-rescue-optimizer.sh
```

### Testing and Validation

```bash
# Check optimization service status
sudo systemctl status usb-performance-optimization.service

# View AI optimization logs
sudo tail -f /var/log/ai-rescue.log

# Check neural diagnostic reports
cat /var/log/system-health-report.json | jq '.'

# Test MobaLiveCD integration (from testing notes)
sudo qemu-system-x86_64 -m 4096M -machine pc-q35-2.12,accel=kvm -cpu host \
  -display gtk -vga std -boot order=c,menu=on -hda /dev/sdb \
  -usb -device usb-tablet -netdev user,id=net0 -device rtl8139,netdev=net0 -no-reboot
```

### System Analysis Commands

```bash
# Hardware health analysis
sensors  # CPU temperature data
free -h  # Memory usage
iostat -x 1 2  # Disk I/O analysis
lsusb  # USB device enumeration

# Performance monitoring
uptime  # Load averages
ps aux | awk '$3 > 50.0 {print $11}'  # High CPU processes
ss -tuln  # Network connections
df /  # Disk usage
```

## Code Architecture Details

### AI Core Components

The system uses a **3-layer AI architecture**:

1. **Neural Diagnostic Engine** - Hardware failure prediction (95%+ accuracy), component lifespan estimation, thermal pattern analysis
2. **Behavioral Security AI** - Zero-day malware detection, process behavior profiling, network traffic analysis  
3. **Intelligent Automation** - Contextual tool selection, skill-adaptive interface, workflow optimization

### Key Design Patterns

- **Modular Plugin System** - Each AI function is independent and can be enabled/disabled
- **Configuration-Driven** - JSON configs in `/etc/ai-rescue-config.json` and `/opt/ai-rescue-tools/`
- **Logging-First** - Comprehensive logging to `/var/log/ai-rescue.log` and `/var/log/neural-diagnostics.log`
- **Fail-Safe Operations** - All optimizations preserve rescue USB functionality

### Performance Optimization Strategy

The system implements **USB-Specific Optimizations**:

```python
# Core sysctl optimizations
optimizations = [
    ("vm.swappiness", "1"),           # Minimal swap usage
    ("vm.dirty_ratio", "3"),          # Faster writes  
    ("vm.dirty_background_ratio", "1"), # Immediate sync
    ("vm.vfs_cache_pressure", "200"), # Aggressive cache cleanup
]
```

USB I/O scheduler optimization:
```bash
echo mq-deadline > /sys/block/sd*/queue/scheduler  # For all USB devices
echo 32 > /sys/block/sd*/queue/nr_requests        # Prevent I/O congestion
echo 128 > /sys/block/sd*/queue/read_ahead_kb     # Optimal readahead
```

### Data Flow Architecture

1. **Hardware Analysis** → System profile collection via `psutil` and `/sys/` interfaces
2. **AI Processing** → Neural network analysis in `intelligent-optimizer.py`
3. **Optimization Application** → Real-time sysctl/udev rule application
4. **Monitoring Loop** → Continuous health reporting via `neural-diagnostics.sh`
5. **Report Generation** → JSON health reports for dashboard visualization

## Development Environment Setup

### Dependencies

The project requires these system packages:
- `python3`, `python3-psutil` for AI components
- `qemu-system-x86_64` for MobaLiveCD testing
- `sensors`, `iostat` for hardware monitoring
- `systemd`, `udev` for service management

### File Structure Understanding

```
├── ai-iso-installer.py              # NLP-powered OS installer (GUI)
├── garuda-ai-optimizations/         # USB performance optimization suite
│   ├── install-optimizations.sh     # Main installer script
│   ├── intelligent-optimizer.py     # AI system optimizer
│   ├── neural-diagnostics.sh       # Hardware prediction engine
│   ├── usb-rescue-optimizer.sh     # Runtime USB optimizations
│   └── usb-device-detector.sh      # USB device enumeration
├── CAPABILITIES_MATRIX.md           # Complete function documentation
├── TESTING_STATUS.md               # QA validation status
└── README.md                       # Project overview
```

## Project-Specific Guidelines

### AI Model Training Data

The system references comprehensive datasets:
- **Hardware Failure Patterns** - 10M+ component lifecycles
- **Security Threat Intelligence** - 100M+ malware samples
- **Performance Benchmarks** - Multi-platform optimization data

### Testing Philosophy

- **Physical Boot Testing** - Always test on actual rescue USB (2-5 minute startup target)
- **MobaLiveCD Integration** - Use QEMU validation before physical testing
- **Performance Validation** - Compare before/after metrics (boot time, I/O latency)
- **AI Accuracy Testing** - 95%+ prediction accuracy requirement

### Optimization Safety

All optimizations follow **rescue-first principles**:
- Preserve data recovery capabilities
- Maintain cross-platform compatibility
- Enable fail-safe rollback mechanisms
- Local processing only (no cloud dependencies)

### Security Model

- **Local AI Processing** - No external data transmission
- **Behavioral Analysis** - Pattern recognition without signature dependence
- **Quantum-Ready** - Future-proof encryption protocols
- **Transparency** - All AI decisions must be explainable

## Performance Targets

The system targets these benchmarks:
- **Diagnostic Speed** - <2 minutes for full system analysis
- **Prediction Accuracy** - 95%+ for hardware failures (6+ months ahead)
- **False Positive Rate** - <1% for security threats
- **Recovery Success Rate** - 90%+ for common issues
- **Boot Time Optimization** - 30-50% faster startup on USB

## Troubleshooting

### Common Issues

1. **Python AI optimizer fails** - Check `python3-psutil` installation
2. **USB optimizations not applied** - Verify udev rules reload: `sudo udevadm control --reload-rules`
3. **Service won't start** - Check systemd logs: `journalctl -u usb-performance-optimization.service`
4. **QEMU testing fails** - Ensure KVM acceleration: `lscpu | grep Virtualization`

### Diagnostic Commands

```bash
# Check AI system status
sudo python3 -c "import psutil; print('AI dependencies OK')"

# Verify USB optimization
cat /sys/block/sd*/queue/scheduler | grep -o '\[.*\]'

# Monitor real-time performance
watch -n 1 'cat /var/log/system-health-report.json | jq .health_metrics'
```
## AI Powerhouse Setup Integration

The Ultimate Rescue USB now integrates with AI Powerhouse Setup for enhanced development capabilities.

### AI Powerhouse Features

- **ZFS Root Filesystem** - Superior snapshots and rollback capabilities
- **AI/ML Development** - CUDA acceleration, local LLMs, PyTorch
- **Rust + Tauri + React** - Modern cross-platform app development
- **Virtualization** - KVM/QEMU, Docker, Proxmox VM integration
- **Self-Hosting** - Traefik, Portainer, monitoring stack
- **Gaming Optimized** - Maintains Garuda's gaming performance

### Integration Commands

```bash
# Launch integrated AI Powerhouse Rescue environment
~/bin/ai-powerhouse-rescue

# Build custom AI Powerhouse ISO with rescue tools
cd ~/Downloads/ai-powerhouse-setup && sudo ./installation/build-custom-iso.sh

# Install AI Powerhouse components
cd ~/Downloads/ai-powerhouse-setup && ./ai-ml/ollama-integration.sh

# Setup self-hosting services
cd ~/Downloads/ai-powerhouse-setup/self-hosting && sudo ./install-native-media-stack.sh
```

### AI Powerhouse + Rescue USB Workflow

1. **Search for "AI Powerhouse"** in the AI ISO installer
2. **Build custom ISO** with both rescue tools and development environment
3. **Deploy to USB** for portable AI development + rescue capabilities
4. **Boot from USB** with full ZFS, CUDA, and rescue tools available

### Configuration Files

- `~/.config/ultimate-rescue-usb/ai-powerhouse-config.json` - Integration settings
- `/var/log/ai-powerhouse-integration.log` - Integration logs
- `/opt/ai-rescue-tools/` - AI rescue tools directory

