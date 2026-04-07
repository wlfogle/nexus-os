🎛️ Automated Bash Script for VirtualBox→KVM Conversion
This bash script automates exporting a VirtualBox VM (by name), converting its disk to QCOW2, and importing it into KVM via virt-install.

📋 Prerequisites
• VirtualBox and VBoxManage installed
• qemu-img, libvirt, virt-install installed
• User has permission to run virsh and virt-install (often via sudo or libvirt group)

🚀 Usage
Save the script below as vb2kvm.sh and make it executable:
bash

Copy Code
chmod +x vb2kvm.sh
Run with:
bash

Copy Code
./vb2kvm.sh <VM_NAME> <MEM_MB> <VCPUS> <OS_VARIANT>
Example:
bash

Copy Code
./vb2kvm.sh my-vbox-vm 4096 2 ubuntu22.04
🖥️ Script: vb2kvm.sh
bash

Copy Code
#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 4 ]; then
  echo "Usage: $0 <VM_NAME> <MEM_MB> <VCPUS> <OS_VARIANT>"
  exit 1
fi

VM_NAME="$1"
MEM_MB="$2"
VCPUS="$3"
OS_VARIANT="$4"
WORKDIR="$HOME/vm-convert-$VM_NAME"

echo "==> Creating working directory at $WORKDIR"
mkdir -p "$WORKDIR"
cd "$WORKDIR"

echo "==> Exporting VirtualBox VM '$VM_NAME' to OVA"
VBoxManage export "$VM_NAME" --output "${VM_NAME}.ova"

echo "==> Extracting disk from OVA"
tar xf "${VM_NAME}.ova"

VMDK_FILE=$(find . -maxdepth 1 -name '*.vmdk' | head -n1)
if [ -z "$VMDK_FILE" ]; then
  echo "Error: No VMDK found in OVA"
  exit 1
fi

QCOW2_FILE="${VM_NAME}.qcow2"
echo "==> Converting $VMDK_FILE → $QCOW2_FILE"
qemu-img convert -O qcow2 "$VMDK_FILE" "$QCOW2_FILE"

echo "==> Importing into KVM via virt-install"
virt-install \
  --name "$VM_NAME" \
  --memory "$MEM_MB" \
  --vcpus "$VCPUS" \
  --disk path="$WORKDIR/$QCOW2_FILE",format=qcow2 \
  --import \
  --os-type linux \
  --os-variant "$OS_VARIANT" \
  --network network=default \
  --noautoconsole

echo "==> Creation complete. VM '$VM_NAME' defined in libvirt."
echo "Start it with: virsh start $VM_NAME"
After running, verify with:

bash

Copy Code
virsh list --all
virsh start <VM_NAME>
Inside the guest, install the guest agent for best performance:

bash

Copy Code
sudo apt update && sudo apt install -y qemu-guest-agent




