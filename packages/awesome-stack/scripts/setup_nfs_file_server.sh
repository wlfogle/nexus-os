#!/bin/bash

# NFS File Server Setup Script for Proxmox Host and All Containers
# This script sets up an NFS server on the Proxmox host and configures all containers as clients

set -e

PROXMOX_HOST="proxmox"
PROXMOX_IP="192.168.122.9"
NFS_SHARE_DIR="/srv/nfs_share"
NFS_MOUNT_POINT="/mnt/nfs_share"
CONTAINER_IDS=(200 210 214 215 216 217)  # Add your container IDs here

echo "🚀 Setting up NFS File Server System"
echo "====================================="

# Function to run commands on Proxmox host
run_on_proxmox() {
    ssh root@$PROXMOX_HOST "$1"
}

# Function to run commands in container
run_in_container() {
    local ct_id=$1
    local command=$2
    ssh root@$PROXMOX_HOST "pct exec $ct_id -- bash -c '$command'"
}

echo "📋 Step 1: Setting up NFS Server on Proxmox Host ($PROXMOX_HOST)"
echo "================================================================="

echo "Installing NFS server packages..."
run_on_proxmox "apt update && apt install -y nfs-kernel-server"

echo "Creating shared directory: $NFS_SHARE_DIR"
run_on_proxmox "mkdir -p $NFS_SHARE_DIR"
run_on_proxmox "chown nobody:nogroup $NFS_SHARE_DIR"
run_on_proxmox "chmod 755 $NFS_SHARE_DIR"

echo "Configuring NFS exports..."
run_on_proxmox "echo '$NFS_SHARE_DIR *(rw,sync,no_subtree_check,no_root_squash)' > /etc/exports"

echo "Applying export configuration..."
run_on_proxmox "exportfs -ra"

echo "Enabling and starting NFS server..."
run_on_proxmox "systemctl enable nfs-kernel-server"
run_on_proxmox "systemctl start nfs-kernel-server"
run_on_proxmox "systemctl status nfs-kernel-server --no-pager"

echo "✅ NFS Server setup complete!"
echo ""

echo "📋 Step 2: Setting up NFS Clients in Containers"
echo "==============================================="

for ct_id in "${CONTAINER_IDS[@]}"; do
    echo "🔧 Configuring Container CT-$ct_id..."
    
    # Check if container is running
    if run_on_proxmox "pct status $ct_id | grep -q running"; then
        echo "   Container CT-$ct_id is running, proceeding..."
        
        # Install NFS client
        echo "   Installing NFS client packages..."
        run_in_container $ct_id "apt update && apt install -y nfs-common"
        
        # Create mount point
        echo "   Creating mount point: $NFS_MOUNT_POINT"
        run_in_container $ct_id "mkdir -p $NFS_MOUNT_POINT"
        
        # Mount NFS share
        echo "   Mounting NFS share..."
        run_in_container $ct_id "mount -t nfs $PROXMOX_IP:$NFS_SHARE_DIR $NFS_MOUNT_POINT"
        
        # Add to fstab for permanent mounting
        echo "   Adding to /etc/fstab for permanent mounting..."
        run_in_container $ct_id "echo '$PROXMOX_IP:$NFS_SHARE_DIR $NFS_MOUNT_POINT nfs defaults 0 0' >> /etc/fstab"
        
        # Test the mount
        echo "   Testing NFS mount..."
        run_in_container $ct_id "df -h | grep nfs_share || echo 'Mount test failed'"
        
        echo "   ✅ Container CT-$ct_id configured successfully!"
    else
        echo "   ⚠️  Container CT-$ct_id is not running, skipping..."
    fi
    echo ""
done

echo "📋 Step 3: Creating Test Files and Verification"
echo "==============================================="

echo "Creating test file on NFS server..."
run_on_proxmox "echo 'NFS File Server Test - $(date)' > $NFS_SHARE_DIR/test_file.txt"
run_on_proxmox "echo 'Inter-container communication ready!' > $NFS_SHARE_DIR/README.txt"

echo "Creating directory structure for agent communication..."
run_on_proxmox "mkdir -p $NFS_SHARE_DIR/agent_communication"
run_on_proxmox "mkdir -p $NFS_SHARE_DIR/shared_scripts"
run_on_proxmox "mkdir -p $NFS_SHARE_DIR/logs"
run_on_proxmox "chmod -R 777 $NFS_SHARE_DIR"

echo "Testing file access from containers..."
for ct_id in "${CONTAINER_IDS[@]}"; do
    if run_on_proxmox "pct status $ct_id | grep -q running"; then
        echo "Testing CT-$ct_id access..."
        run_in_container $ct_id "ls -la $NFS_MOUNT_POINT/ && cat $NFS_MOUNT_POINT/test_file.txt" || echo "Access test failed for CT-$ct_id"
    fi
done

echo ""
echo "🎉 NFS File Server Setup Complete!"
echo "==================================="
echo ""
echo "📁 NFS Server Details:"
echo "   Host: $PROXMOX_HOST"
echo "   Share Directory: $NFS_SHARE_DIR"
echo "   Mount Point in Containers: $NFS_MOUNT_POINT"
echo ""
echo "📂 Directory Structure Created:"
echo "   $NFS_SHARE_DIR/agent_communication/ - For inter-agent messages"
echo "   $NFS_SHARE_DIR/shared_scripts/ - For shared scripts"
echo "   $NFS_SHARE_DIR/logs/ - For shared logs"
echo ""
echo "🔧 Usage Examples:"
echo "   # Copy files from CT-200 to shared space:"
echo "   pct exec 200 -- cp /home/alexa/warp_agent_bridge.py $NFS_MOUNT_POINT/shared_scripts/"
echo ""
echo "   # Access shared files from any container:"
echo "   ls $NFS_MOUNT_POINT/shared_scripts/"
echo ""
echo "✅ All containers can now share files through $NFS_MOUNT_POINT!"
