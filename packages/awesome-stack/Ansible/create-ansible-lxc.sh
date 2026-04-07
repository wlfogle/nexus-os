#!/bin/bash
# Proxmox LXC Ansible Control Node Creation Script
# Run this as root on your Proxmox node (the VM running under Garuda)

# CONFIGURE THESE VARIABLES
CTID=120
HOSTNAME="ansible-lxc"
TEMPLATE="local:vztmpl/debian-12-standard_12.2-1_amd64.tar.zst" # Adjust if needed
STORAGE="local-lvm"
DISK_SIZE="8G"
MEMORY="2048"
CORES="2"
NET="name=eth0,bridge=vmbr0,ip=dhcp"
SSH_KEY="/root/.ssh/id_rsa.pub" # Change if needed

# Download template if missing
if ! pveam available | grep -q "${TEMPLATE##*/}"; then
  echo "Downloading template..."
  pveam update
  pveam download local ${TEMPLATE##*/}
fi

echo "Creating LXC $CTID ($HOSTNAME)..."
pct create $CTID $TEMPLATE \
  -hostname $HOSTNAME \
  -storage $STORAGE \
  -rootfs $STORAGE:$DISK_SIZE \
  -memory $MEMORY \
  -cores $CORES \
  -net0 $NET \
  -features nesting=1 \
  -unprivileged 1 \
  -password "changeme" \
  -description "Ansible Automation LXC"

echo "Starting LXC $CTID..."
pct start $CTID
sleep 6

echo "Installing Python and Ansible in $CTID..."
pct exec $CTID -- apt-get update
pct exec $CTID -- apt-get install -y python3 python3-pip git sshpass
pct exec $CTID -- pip3 install ansible

if [ -f "$SSH_KEY" ]; then
  echo "Setting up SSH key for root in $CTID..."
  pct exec $CTID -- mkdir -p /root/.ssh
  pct push $CTID $SSH_KEY /root/.ssh/authorized_keys
  pct exec $CTID -- chmod 600 /root/.ssh/authorized_keys
  pct exec $CTID -- chown root:root /root/.ssh/authorized_keys
fi

echo "Done! LXC $CTID ($HOSTNAME) is ready for Ansible automation."
