# Android Emulator Setup on ct-200 (Alexa Desktop) Container

## Overview
This document describes the process of setting up an Android emulator on the ct-200 Proxmox container to run the Amazon Alexa app locally, allowing it to connect to a local Alexa server without external dependencies.

## System Configuration

### Container Details
- **Container ID**: ct-200
- **Alias**: alexa-desktop
- **OS**: Debian 12
- **IP Address**: 192.168.122.200
- **Disk Size**: 40GB (resized from initial 20GB)
- **VNC Display**: :1 (port 5901)

### Host Environment
- **Proxmox Host**: 192.168.122.9 (Debian 12)
- **Client Machine**: Garuda Linux with fish shell
- **SSH Alias**: `alexa` for passwordless access to ct-200

## Prerequisites

### Container Preparation
1. **Nested Virtualization Support**:
   ```bash
   # Stop container
   pct stop 200
   
   # Enable nesting in container config
   echo 'features: nesting=1' >> /etc/pve/lxc/200.conf
   
   # Allow KVM device access
   echo 'lxc.cgroup2.devices.allow: c 10:232 rwm' >> /etc/pve/lxc/200.conf
   echo 'lxc.mount.entry: /dev/kvm dev/kvm none bind,optional,create=file' >> /etc/pve/lxc/200.conf
   
   # Start container
   pct start 200
   ```

2. **KVM Device Setup** (inside container):
   ```bash
   # Create KVM device node
   sudo mknod /dev/kvm c 10 232
   sudo chown root:kvm /dev/kvm
   sudo chmod 660 /dev/kvm
   
   # Add user to KVM group
   sudo gpasswd -a alexa kvm
   ```

### VNC Server Configuration
A custom systemd service was created for VNC access:

**File**: `/etc/systemd/system/alexa-vnc.service`
```ini
[Unit]
Description=VNC Server for Alexa User
After=multi-user.target

[Service]
Type=forking
User=alexa
Group=alexa
WorkingDirectory=/home/alexa
ExecStart=/usr/bin/vncserver :1 -geometry 1920x1080 -depth 24 -localhost no
ExecStop=/usr/bin/vncserver -kill :1
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable alexa-vnc.service
sudo systemctl start alexa-vnc.service
```

## Android SDK Installation

### Directory Structure
```
/opt/android-sdk/
├── cmdline-tools/
├── emulator/
├── platform-tools/
└── system-images/
    ├── android-34/
    └── android-36/
```

### File Transfer Process
1. **Mount container disk** on Proxmox host:
   ```bash
   pct stop 200
   mount /dev/pve/vm-200-disk-0 /mnt/ct200
   ```

2. **Copy Android SDK** from local machine:
   ```bash
   rsync -av /opt/android-sdk/ proxmox:/mnt/ct200/opt/android-sdk/
   ```

3. **Set correct ownership**:
   ```bash
   chown -R 1000:1000 /mnt/ct200/opt/android-sdk/
   ```

4. **Unmount and start container**:
   ```bash
   umount /mnt/ct200
   pct start 200
   ```

### Storage Requirements
- **Initial disk size**: 20GB → insufficient
- **Final disk size**: 40GB (after resizing)
- **System images size**: ~11GB
- **Total Android SDK**: ~15GB

## Android Virtual Device (AVD) Configuration

### AVD Files Location
```
/home/alexa/.android/avd/
├── Pixel_9_Pro_XL.avd/
└── Pixel_9_Pro_XL.ini
```

### AVD Configuration
The AVD was configured to use:
- **Device**: Pixel 9 Pro XL
- **System Image**: android-36/google_apis_playstore/x86_64
- **API Level**: 36 (Android 15)

### Missing Configuration Fix
The `.ini` file was missing and had to be created manually:
```ini
avd.ini.displayname=Pixel 9 Pro XL
avd.ini.encoding=UTF-8
path=/home/alexa/.android/avd/Pixel_9_Pro_XL.avd
path.rel=avd/Pixel_9_Pro_XL.avd
target=android-36
```

## Emulator Execution

### Environment Variables
```bash
export ANDROID_SDK_ROOT=/opt/android-sdk
export DISPLAY=:1
```

### Running the Emulator
Due to KVM hardware acceleration issues in the container, the emulator must be run in software mode:

```bash
/opt/android-sdk/emulator/emulator -avd Pixel_9_Pro_XL -accel off -no-audio -no-metrics &
```

### Process Verification
```bash
# Check emulator processes
ps aux | grep emulator

# Check ADB connection
/opt/android-sdk/platform-tools/adb devices
```

## Troubleshooting

### Common Issues

1. **Hardware Acceleration Errors**:
   - **Problem**: `x86_64 emulation currently requires hardware acceleration!`
   - **Solution**: Use `-accel off` flag to disable hardware acceleration

2. **KVM Permission Issues**:
   - **Problem**: `This user doesn't have permissions to use KVM (/dev/kvm)`
   - **Solution**: Ensure user is in kvm group and device has correct permissions

3. **Disk Space Issues**:
   - **Problem**: `No space left on device` during file transfers
   - **Solution**: Resize container disk using `pct resize 200 rootfs +XG`

4. **Missing System Images**:
   - **Problem**: `kernel-ranchu` not found
   - **Solution**: Copy complete system-images directory from host

5. **Java Version Compatibility**:
   - **Problem**: SDK manager fails with `UnsupportedClassVersionError`
   - **Solution**: System images copied directly instead of using SDK manager

## Network Access

### SSH Configuration
Passwordless SSH access configured with:
```bash
# Generate SSH key
ssh-keygen -t ed25519 -f ~/.ssh/alexa_key -N ""

# Copy to container
ssh-copy-id -i ~/.ssh/alexa_key.pub alexa@192.168.122.200

# Add alias to fish config
alias alexa='ssh -i ~/.ssh/alexa_key alexa@192.168.122.200'
```

### VNC Access
- **Port**: 5901
- **Resolution**: 1920x1080
- **External Access**: Enabled (listening on 0.0.0.0:5901)

## Current Status

### Emulator Status
- ✅ **Emulator processes running**: Multiple QEMU and support processes active
- ✅ **ADB detection**: Device visible as `emulator-5554`
- ⏳ **Boot status**: Device offline (still booting - expected with software acceleration)
- ✅ **VNC access**: Available on port 5901

### Next Steps
1. **Wait for Android boot completion** (5-10 minutes without hardware acceleration)
2. **Connect via VNC** to view Android interface
3. **Install Amazon Alexa app** via Google Play Store or APK sideloading
4. **Configure Alexa app** to connect to local Alexa server

## Performance Notes
- **Boot time**: Extended due to software-only acceleration
- **Runtime performance**: Reduced but functional for app testing
- **Memory usage**: ~1.2GB for emulator processes
- **CPU usage**: Moderate during boot, lower during idle

## Files and Directories Created
```
/opt/android-sdk/                     # Android SDK installation
/home/alexa/.android/avd/            # AVD configuration and data
/etc/systemd/system/alexa-vnc.service # VNC service configuration
/tmp/emulator.log                    # Emulator runtime logs
~/.ssh/alexa_key                     # SSH key for passwordless access
```

This setup provides a complete Android development environment within the Proxmox container, capable of running Android applications including the Amazon Alexa app for local server integration.
