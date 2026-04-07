# ISO Build Troubleshooting Guide

## 🚨 Major Issue: garuda-tools buildiso vs archiso

### ❌ garuda-tools buildiso - FAILS in Live Environment

**Problem**: Overlayfs nesting restriction in Linux kernel
```
==> ERROR: overlayfs mount: filesystem not supported as upperdir
mount: fsconfig() failed: overlay: filesystem on desktopfs not supported as upperdir
```

**Root Cause**: 
- Live Garuda ISO runs on overlayfs root filesystem
- garuda-tools buildiso attempts to create overlayfs on top of overlayfs
- Linux kernel prevents this for filesystem consistency

**Environment Where It Fails**:
```bash
# Live environment with overlayfs root
$ mount | grep "on / "
overlay on / type overlay (rw,relatime,lowerdir=...,upperdir=...,workdir=...)
```

### ✅ archiso (mkarchiso) - WORKS in Live Environment

**Solution**: Use archiso which doesn't create nested overlayfs mounts
```bash
# This works in live environment
sudo mkarchiso -v -w /path/to/workdir -o /path/to/output .
```

**Why It Works**:
- archiso uses different filesystem handling approach
- No overlayfs nesting conflicts
- Same method proven in blendOS-dragonized-iso build

## 🔧 Common Issues & Solutions

### 1. Package Not Found Errors

**Problem**:
```
error: target not found: package-name
```

**Solutions**:
```bash
# Check if package exists
pacman -Ss package-name

# Check package name variations
pacman -Ss partial-name | grep package

# Remove non-existent packages from packages.x86_64
# Common missing packages and alternatives:
# garuda-keyring → (remove, already in repo)
# python-jupyter → jupyter-console
# python-uvicorn → (remove, part of python-fastapi)
```

### 2. Disk Space Issues

**Problem**:
```
No space left on device
```

**Solutions**:
```bash
# Check available space
df -h /tmp
df -h /run/media/garuda/Data

# Use external drive with more space
WORK_DIR=/run/media/garuda/Data/tmp/workdir \
OUTPUT_DIR=/run/media/garuda/Data/tmp/out \
./build-ai-powerhouse-iso.sh

# Clean up previous builds
sudo rm -rf /tmp/ai-powerhouse-*
```

### 3. Permission Errors

**Problem**:
```
Permission denied accessing /var/cache/pacman/pkg/
```

**Solutions**:
```bash
# Run with proper sudo
sudo mkarchiso -v -w workdir -o output .

# Fix ownership if needed
sudo chown -R $USER:$USER output/
```

### 4. Network/Repository Issues

**Problem**:
```
failed to retrieve some files
could not resolve host
```

**Solutions**:
```bash
# Update package databases
sudo pacman -Sy

# Check internet connectivity
ping -c 3 archlinux.org

# Try different mirror
sudo reflector --latest 20 --protocol https --sort rate --save /etc/pacman.d/mirrorlist
```

### 5. Memory Issues

**Problem**:
```
Cannot allocate memory
Process killed
```

**Solutions**:
```bash
# Check available memory
free -h

# Reduce parallel downloads in pacman.conf
ParallelDownloads = 2

# Close unnecessary applications
# Ensure at least 8GB RAM available
```

## 📊 Build Environment Requirements

### Minimum Requirements
- **RAM**: 8GB (16GB recommended)
- **Storage**: 25GB free space
- **Network**: Stable internet connection
- **Permissions**: sudo access

### Recommended Setup
```bash
# Use external drive for build
BUILD_DIR=/run/media/garuda/Data/build
mkdir -p $BUILD_DIR

# Set up environment variables
export WORK_DIR="$BUILD_DIR/workdir"
export OUTPUT_DIR="$BUILD_DIR/output"

# Build with monitoring
./build-ai-powerhouse-iso.sh 2>&1 | tee build-$(date +%Y%m%d-%H%M).log
```

## 🎯 Verification Steps

### 1. Pre-Build Checks
```bash
# Verify archiso installation
which mkarchiso

# Check package file syntax
grep -v "^#" packages.x86_64 | grep -v "^$" | wc -l

# Verify profiledef.sh syntax
bash -n profiledef.sh
```

### 2. During Build Monitoring
```bash
# Monitor disk space
watch -n 30 'df -h /tmp'

# Monitor build progress
tail -f /path/to/build.log

# Monitor system resources
htop
```

### 3. Post-Build Verification
```bash
# Check ISO file
file output/*.iso
ls -lh output/*.iso

# Verify ISO structure
isoinfo -d -i output/*.iso

# Test mount
sudo mkdir -p /mnt/test-iso
sudo mount -o loop output/*.iso /mnt/test-iso
ls /mnt/test-iso
sudo umount /mnt/test-iso
```

## 🚀 Success Indicators

### ✅ Successful Build
- Package installation completes without errors
- ISO file created (~4-6GB size)
- No filesystem errors in build log
- ISO mounts successfully
- Contains expected files and structure

### ❌ Failed Build Signs
- "target not found" errors for packages
- "No space left" errors
- "Permission denied" errors
- Overlayfs mount failures
- Empty or corrupted ISO file

## 📞 Getting Help

### Debug Information to Collect
```bash
# System information
uname -a
free -h
df -h
mount | grep overlay

# Build environment
ls -la build-directory/
cat packages.x86_64 | wc -l
tail -50 build.log

# Package repository status
pacman -Sy
pacman -Ss garuda | head -5
```

### Common Solutions Quick Reference
1. **Overlayfs error** → Use archiso instead of garuda-tools
2. **Package not found** → Check package name and remove if necessary  
3. **Space issues** → Use external drive with adequate space
4. **Permission issues** → Use sudo and check ownership
5. **Network issues** → Update mirrors and check connectivity

This troubleshooting guide should resolve 95% of build issues encountered.