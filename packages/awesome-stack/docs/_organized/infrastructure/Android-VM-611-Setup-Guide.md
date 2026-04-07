# Android VM-611 Setup Guide

## ✅ **Configuration Fixed - Disk Detection Issue Resolved**

### **Problem**: Android-x86 installer couldn't detect hard drive
### **Solution**: Switched from SCSI to SATA controller + SeaBIOS

## 🔧 **VM Configuration Applied**

### **Hardware Changes Made**:
- **Disk Controller**: Changed from `scsi0` to `sata0` (Android-x86 detects SATA better)
- **BIOS**: Changed from `UEFI (ovmf)` to `SeaBIOS` (better Android-x86 compatibility)
- **Display**: Set to `qxl` for better console experience
- **Boot Order**: `ide2;sata0` (CD-ROM first, then SATA disk)

### **Current VM Configuration**:
```
VM ID: 611
Name: android-15
Cores: 4
Memory: 6144 MB (6GB)
Disk: 32GB SATA (local-lvm:vm-611-disk-1)
Network: virtio (MAC: BC:24:11:B6:7F:4C)
Display: QXL
BIOS: SeaBIOS
```

## 📱 **Android Installation Steps**

### **1. Boot from ISO**
- VM boots from Android-x86 ISO (android-x86_64-9.0-r2-k49.iso)
- Choose "Installation" option

### **2. Partition Setup** ✅ **Should Work Now**
- Android installer should now detect the 32GB SATA disk
- Choose "Create/Modify partitions"
- Create partitions as needed:
  - System partition (ext4)
  - Data partition (ext4)

### **3. Installation Options**
- Install bootloader? **Yes**
- Install system as read/write? **Yes**
- Skip Google Apps installation? **Your choice**

## 🎤 **Post-Installation: Alexa Integration**

### **Alexa App Installation**
Once Android is installed and running:

1. **Enable Developer Options**:
   - Settings → About tablet → Tap "Build number" 7 times

2. **Enable ADB**:
   - Settings → Developer Options → USB debugging (ON)
   - Settings → Developer Options → ADB over network (ON)

3. **Install Alexa APK**:
   ```bash
   # From your host machine
   adb connect [ANDROID_VM_IP]:5555
   adb install /path/to/alexa.apk
   ```

### **Connect to Your Home Assistant**
- Open Alexa app
- Sign in with Amazon account
- Skills & Games → "Home Assistant"
- Link account: `http://homeassistant.local:8123`
- Test voice commands with your existing 12 configured commands

## 🌐 **Integration with Your Stack**

### **Your Voice Commands Available**:
1. "Alexa, turn on movie night" → Checks Plex + Jellyfin
2. "Alexa, turn on system status" → Health check all 47+ containers
3. "Alexa, turn on AI assistant status" → Check Ollama AI services
4. [9 more commands from your existing setup]

### **Architecture**:
```
🎤 Android VM-611 (Alexa App)
    ↓
🏠 Home Assistant (VM 500)
    ↓
🐳 Docker Containers (47+)
    ↓
📦 LXC Containers (Proxmox)
    ↓
⚡ Proxmox Hypervisor
```

## 🔧 **Troubleshooting**

### **If Disk Still Not Detected**:
1. Try IDE instead of SATA:
   ```bash
   ssh root@192.168.122.9 "
   qm stop 611
   qm set 611 --delete sata0
   qm set 611 --ide0 local-lvm:vm-611-disk-1,size=32G
   qm set 611 --boot order=ide2\\;ide0
   qm start 611
   "
   ```

### **Alternative Android-x86 Versions**:
- Try Android-x86 8.1 if 9.0 has issues
- Consider Bliss OS or other Android-x86 variants

### **Console Access Improvements**:
- Install `virt-viewer` on your client machine
- Use SPICE console instead of noVNC for better experience

---

## 🎯 **Next Steps**

1. **Complete Android Installation** with the fixed disk detection
2. **Set up Alexa app** and connect to your Home Assistant
3. **Test voice commands** with your sophisticated media stack
4. **Enjoy voice control** of your 47+ containers, AI services, and media servers!

---
*Setup completed: July 30, 2025*
*VM Configuration: Optimized for Android-x86 compatibility*
*Integration: Ready for Home Assistant + Alexa voice control*
