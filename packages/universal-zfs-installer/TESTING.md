# 🧪 Testing Guide

## Quick Start - Test Now!

The easiest way to test the installer:

```bash
./test-vm.sh test
```

This will:
1. ✅ Check/install dependencies (QEMU, Python, GTK)
2. ✅ Clone MobaLiveCD (GUI tool for VM testing)
3. ✅ Download Kubuntu 24.04 ISO (~4GB, one time only)
4. ✅ Create virtual 50GB test disk
5. ✅ Launch MobaLiveCD GUI
6. ✅ Boot Kubuntu in VM with UEFI

## Testing Methods

### Method 1: MobaLiveCD GUI (Recommended) ⭐
**Best for:** Visual testing, easy to use

```bash
./test-vm.sh test
```

- Opens a nice GUI
- Click to boot
- Installer at `/media/installer/` in VM
- Run: `sudo /media/installer/auto-install.sh`

### Method 2: Quick Launch
**Best for:** Fast repeated testing

```bash
./test-vm.sh quick
```

- Launches VM immediately
- No prompts
- Test quickly

### Method 3: Raw QEMU
**Best for:** Advanced users, custom configuration

```bash
MEMORY=16G CPUS=8 ./test-vm.sh qemu
```

- Full control over VM settings
- Direct QEMU commands
- Custom resources

## Inside the VM

Once the VM boots to Kubuntu desktop:

### Option A: Auto Install (Fastest)
```bash
sudo /media/installer/auto-install.sh
```

This uses pre-configured settings:
- Disk: /dev/vda
- Username: testuser  
- Password: testpass
- Hostname: test-vm

### Option B: Manual Install (Custom Settings)
```bash
cd /media/installer
export TARGET_DISK=/dev/vda
export USERNAME=myuser
export USER_PASSWORD=mypass
export HOSTNAME=myhost
sudo -E ./install.sh
```

## After Installation

The installer takes 15-30 minutes. When done:

1. **Reboot the VM**
2. **Test installed system:**
   ```bash
   ./test-vm.sh boot
   ```

3. **Login** with your credentials
4. **Verify everything works:**
   - Desktop loads (KDE Plasma)
   - Docker running: `docker ps`
   - Services available (Jellyfin, etc.)

## Testing Checklist

- [ ] VM boots from Kubuntu ISO
- [ ] Installer runs without errors
- [ ] Installation completes (15-30 min)
- [ ] System reboots successfully
- [ ] ZFSBootMenu appears
- [ ] System boots to login
- [ ] Desktop environment loads
- [ ] Network works
- [ ] Docker is running
- [ ] ZFS pools are healthy: `zpool status`

## Troubleshooting

### "KVM not available"
Your CPU supports KVM, enable it:
```bash
# Check if KVM module is loaded
lsmod | grep kvm

# Add user to kvm group
sudo usermod -a -G kvm $USER

# Reboot
```

### "Missing dependencies"
The script will offer to install them. Say yes!

### "Download failed"
If ISO download fails:
1. Download manually: https://cdimage.ubuntu.com/kubuntu/releases/24.04/release/
2. Place in: `test-env/kubuntu-24.04.1-desktop-amd64.iso`
3. Run test again

### "Installer not found in VM"
MobaLiveCD might not mount the installer ISO. Use QEMU method:
```bash
./test-vm.sh qemu
```

## Clean Up

Remove all test files:
```bash
./test-vm.sh clean
```

This deletes:
- Downloaded ISO (~4GB)
- Virtual disk
- Test environment directory

## Advanced Testing

### Custom VM Resources
```bash
MEMORY=32G CPUS=16 DISK_SIZE=100G ./test-vm.sh test
```

### Test on Different Hardware
The installer detects:
- NVIDIA GPUs
- AVX2 CPUs
- Specific CPU models (i9-13900HX)

To test detection, check logs in VM:
```bash
cat install.log
```

### Test Resume Capability
1. Start installation
2. Force quit VM during install
3. Restart VM
4. Re-run installer - it should resume!

## System Requirements for Testing

**Host System:**
- Linux with KVM support
- 8GB+ RAM (to give VM 8GB)
- 10GB free disk space (for ISO + test disk)
- x86_64 CPU with VT-x/AMD-V

**Your System:**
- ✅ i9-13900HX (32 cores) - Perfect!
- ✅ 62.5GB RAM - Plenty!
- ✅ Kubuntu 24.04 - Native environment!

You can easily run multiple test VMs simultaneously with your hardware!

## Automated Testing (Future)

Coming soon:
- Automated test suite
- CI/CD integration
- Regression testing
- Multiple distro testing

## Need Help?

- Check `install.log` in VM for errors
- Review README.md for full documentation
- Open GitHub issue with logs
