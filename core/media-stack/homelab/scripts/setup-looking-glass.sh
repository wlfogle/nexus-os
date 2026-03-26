#!/usr/bin/env bash
# =============================================================================
# setup-looking-glass.sh — Tiamat Looking Glass + RX 580 GPU Passthrough
# AMD Ryzen 5 3600 + RX 580 @ 09:00.0 (GPU) / 09:00.1 (Audio)
# AMD-Vi IOMMU already active — no reboot needed for IOMMU
# Creates Windows VM-200 with GPU passthrough + Looking Glass IVSHMEM
# =============================================================================
set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────
VMID=200
VM_NAME="Windows-VM"
VM_RAM=8192          # 8GB — Tiamat has ~7.7GB, leave 1.5GB for host
VM_CORES=10          # 10 of 12 threads
STORAGE_POOL="local-lvm"
LG_SHM_SIZE="64M"   # 64MB — sufficient for 1080p@60

# RX 580 on Tiamat — confirmed 09:00.0/09:00.1
GPU_PCI="09:00.0"
GPU_AUDIO_PCI="09:00.1"
GPU_VENDOR_ID="1002:67df"
GPU_AUDIO_ID="1002:aaf0"

echo ""
echo "╔══════════════════════════════════════════════════╗"
echo "║   Tiamat Looking Glass Setup                     ║"
echo "║   AMD RX 580 GPU Passthrough + KVMFR             ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

# ── Step 1: Install build dependencies for KVMFR ─────────────────────────────
echo "[1/7] Installing KVMFR build dependencies..."
apt-get update -y
apt-get install -y \
    build-essential \
    dkms \
    git \
    cmake \
    pkg-config \
    binutils-dev \
    nettle-dev \
    libgl-dev \
    libgles-dev \
    libspice-protocol-dev \
    libfontconfig-dev \
    libx11-dev \
    libxi-dev \
    libxss-dev \
    libxcursor-dev \
    libxinerama-dev \
    libxrandr-dev \
    libpipewire-0.3-dev \
    libsamplerate0-dev \
    libpulse-dev \
    libwayland-dev \
    linux-headers-$(uname -r) \
    pve-headers-$(uname -r) 2>/dev/null || \
    apt-get install -y linux-headers-generic

# ── Step 2: Clone and build Looking Glass (KVMFR + client) ───────────────────
echo "[2/7] Cloning Looking Glass source..."
LG_VERSION="B7"
LG_DIR="/opt/looking-glass-${LG_VERSION}"

if [[ ! -d "${LG_DIR}" ]]; then
    git clone --depth 1 --branch ${LG_VERSION} \
        https://github.com/gnif/LookingGlass.git "${LG_DIR}" 2>/dev/null || \
    git clone --depth 1 \
        https://github.com/gnif/LookingGlass.git "${LG_DIR}"
fi

echo "[2/7] Building Looking Glass client..."
mkdir -p "${LG_DIR}/client/build"
cd "${LG_DIR}/client/build"
cmake -DENABLE_WAYLAND=no -DENABLE_X11=yes ..
make -j$(nproc)
cp looking-glass-client /usr/local/bin/looking-glass-client
chmod +x /usr/local/bin/looking-glass-client

echo "[2/7] Building KVMFR kernel module..."
mkdir -p "${LG_DIR}/module/build"
cd "${LG_DIR}/module/build"
cmake ..
make -j$(nproc)

# Install via DKMS for persistence across kernel updates
mkdir -p /usr/src/kvmfr-1.0
cp "${LG_DIR}/module/"*.c "${LG_DIR}/module/"*.h /usr/src/kvmfr-1.0/ 2>/dev/null || true
cp "${LG_DIR}/module/build/kvmfr.ko" /lib/modules/$(uname -r)/extra/ 2>/dev/null || true
depmod -a

# Load the module now
modprobe kvmfr static_size_mb=64 || true

# ── Step 3: Configure KVMFR to load at boot ───────────────────────────────────
echo "[3/7] Configuring KVMFR module autoload..."
echo "kvmfr" >> /etc/modules
cat > /etc/modprobe.d/kvmfr.conf << 'EOF'
options kvmfr static_size_mb=64
EOF

# ── Step 4: udev rule for /dev/kvmfr0 permissions ────────────────────────────
echo "[4/7] Creating udev rule for /dev/kvmfr0..."
cat > /etc/udev/rules.d/99-kvmfr.rules << 'EOF'
SUBSYSTEM=="kvmfr", OWNER="root", GROUP="kvm", MODE="0660"
EOF
udevadm control --reload-rules
udevadm trigger

# ── Step 5: Blacklist amdgpu for RX 580 passthrough ─────────────────────────
echo "[5/7] Blacklisting amdgpu / binding RX 580 to vfio-pci..."

cat > /etc/modprobe.d/vfio.conf << EOF
# Bind RX 580 (GPU + Audio) to vfio-pci at boot
options vfio-pci ids=${GPU_VENDOR_ID},${GPU_AUDIO_ID}
softdep amdgpu pre: vfio-pci
softdep snd_hda_intel pre: vfio-pci
EOF

cat > /etc/modprobe.d/blacklist-amdgpu.conf << 'EOF'
# Blacklist amdgpu on host — RX 580 passed through to Windows VM
blacklist amdgpu
blacklist radeon
EOF

# Ensure vfio modules load first
cat > /etc/modules-load.d/vfio.conf << 'EOF'
vfio
vfio_iommu_type1
vfio_pci
EOF

update-initramfs -u -k all

# ── Step 6: Create Windows VM-200 with GPU passthrough + Looking Glass ────────
echo "[6/7] Creating Windows VM-200 with RX 580 passthrough + IVSHMEM..."

# Remove existing VM if present
if qm status ${VMID} &>/dev/null; then
    echo "VM ${VMID} already exists — skipping creation. Run 'qm destroy ${VMID}' first to recreate."
else
    pvesh create /nodes/localhost/qemu -vmid ${VMID} \
        -name "${VM_NAME}" \
        -memory ${VM_RAM} \
        -cores ${VM_CORES} \
        -sockets 1 \
        -cpu "host,hidden=1,flags=+pcid" \
        -machine "q35" \
        -bios "ovmf" \
        -efidisk0 "${STORAGE_POOL}:1,efitype=4m,size=4M" \
        -tpmstate0 "${STORAGE_POOL}:1,version=v2.0,size=4M" \
        -scsihw "virtio-scsi-pci" \
        -scsi0 "${STORAGE_POOL}:60,cache=writeback,discard=on,ssd=1" \
        -ostype "win10" \
        -net0 "virtio,bridge=vmbr0,firewall=1" \
        -vga "none" \
        -tablet 0 \
        -agent "enabled=1,fstrim_cloned_disks=1" \
        -boot "order=scsi0;ide2"

    # Add RX 580 GPU passthrough
    pvesh set /nodes/localhost/qemu/${VMID}/config \
        -hostpci0 "${GPU_PCI},pcie=1,x-vga=1,rombar=1" \
        -hostpci1 "${GPU_AUDIO_PCI},pcie=1"

    # Add Looking Glass IVSHMEM shared memory
    pvesh set /nodes/localhost/qemu/${VMID}/config \
        -args "-device ivshmem-plain,memdev=ivshmem,bus=pcie.0 -object memory-backend-file,id=ivshmem,share=on,mem-path=/dev/shm/looking-glass,size=${LG_SHM_SIZE}"

    # USB keyboard + mouse passthrough
    pvesh set /nodes/localhost/qemu/${VMID}/config \
        -usb0 "host=spice,usb3=1"

    # SPICE for initial Windows setup (before Looking Glass is configured in guest)
    pvesh set /nodes/localhost/qemu/${VMID}/config \
        -spice "port=5930,addr=0.0.0.0,disable-ticketing=1,image-compression=off,streaming-video=off"

    echo "VM-200 created."
fi

# ── Step 7: Set up Looking Glass shared memory ───────────────────────────────
echo "[7/7] Setting up /dev/shm/looking-glass..."
touch /dev/shm/looking-glass
chown root:kvm /dev/shm/looking-glass
chmod 660 /dev/shm/looking-glass

# Persist across reboots via tmpfiles.d
cat > /etc/tmpfiles.d/looking-glass.conf << 'EOF'
# Looking Glass shared memory file
f /dev/shm/looking-glass 0660 root kvm -
EOF

# ── Install Looking Glass client config ──────────────────────────────────────
mkdir -p /root/.config
cat > /root/.config/looking-glass-client.ini << EOF
[app]
renderer=EGL
shmFile=/dev/kvmfr0
allowDMA=yes

[win]
title=Looking Glass — Windows VM
size=1920x1080
fullScreen=no
autoResize=yes
keepAspect=yes

[input]
rawMouse=yes
mouseRedraw=yes
escapeKey=0x0F
grabKeyboard=yes
grabMouse=yes

[egl]
vsync=no
doubleBuffer=yes
multisample=yes

[audio]
micDefault=allow
speakers=yes

[spice]
enable=yes
host=127.0.0.1
port=5930
EOF

# ── Done ─────────────────────────────────────────────────────────────────────
echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║   Looking Glass Setup COMPLETE                                   ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   REBOOT REQUIRED to apply:                                      ║"
echo "║   • KVMFR module autoload                                        ║"
echo "║   • amdgpu blacklist / vfio-pci binding                          ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   After reboot:                                                  ║"
echo "║   1. Upload Windows ISO to Proxmox                               ║"
echo "║      → Web UI: https://$(hostname -I | awk '{print $1}'):8006    ║"
echo "║   2. Attach ISO to VM-200, start VM, install Windows             ║"
echo "║   3. In Windows: install VirtIO + IVSHMEM drivers                ║"
echo "║   4. In Windows: run Looking Glass host service (B7)             ║"
echo "║   5. On Tiamat: looking-glass-client                             ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   SPICE (for initial Windows setup):                             ║"
printf "║   virt-viewer spice://%-43s ║\n" "$(hostname -I | awk '{print $1}'):5930"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "NOTE: Your Windows SSD (when plugged in as /dev/sdb) can be added:"
echo "  qm set ${VMID} -scsi1 /dev/sdb"
echo ""
