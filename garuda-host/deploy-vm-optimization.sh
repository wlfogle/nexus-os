#!/bin/bash

# Deployment script for Proxmox VM Optimization
# This script will transfer and execute the optimization script on your Proxmox VM

echo "🚀 Deploying VM Optimization for Awesome Stack Infrastructure"

# Configuration
VM_NAME="ProxMox-Stack"
OPTIMIZATION_SCRIPT="/tmp/optimize-proxmox-vm.sh"

echo "📋 Pre-deployment checklist:"
echo "   • VM Name: $VM_NAME"
echo "   • Optimization script: $OPTIMIZATION_SCRIPT"

# Check if VM is running
echo "🔍 Checking VM status..."
VM_STATUS=$(sudo virsh domstate "$VM_NAME" 2>/dev/null || echo "not found")

if [ "$VM_STATUS" != "running" ]; then
    echo "❌ VM '$VM_NAME' is not running (status: $VM_STATUS)"
    echo "Please start the VM first with: sudo virsh start $VM_NAME"
    exit 1
fi

echo "✅ VM '$VM_NAME' is running"

# Get VM IP address
echo "🔍 Finding VM IP address..."
VM_IP=$(sudo virsh domifaddr "$VM_NAME" 2>/dev/null | grep -oP '192\.168\.[0-9]+\.[0-9]+|10\.[0-9]+\.[0-9]+\.[0-9]+|172\.[0-9]+\.[0-9]+\.[0-9]+' | head -1)

if [ -z "$VM_IP" ]; then
    echo "⚠️  Could not automatically detect VM IP address"
    echo "Please provide the VM IP address:"
    read -p "VM IP: " VM_IP
    
    if [ -z "$VM_IP" ]; then
        echo "❌ IP address is required"
        exit 1
    fi
fi

echo "✅ VM IP address: $VM_IP"

# Check connectivity
echo "🔍 Testing connectivity..."
if ! ping -c 1 "$VM_IP" &>/dev/null; then
    echo "❌ Cannot reach VM at $VM_IP"
    echo "Please check:"
    echo "   • VM is fully booted"
    echo "   • Network is configured"
    echo "   • SSH is enabled"
    exit 1
fi

echo "✅ VM is reachable"

# Get SSH credentials
echo "🔑 SSH Configuration:"
echo "Please provide SSH credentials for the VM:"
read -p "Username (default: root): " SSH_USER
SSH_USER=${SSH_USER:-root}

echo "Attempting to connect to $SSH_USER@$VM_IP..."

# Test SSH connection
if ! ssh -o ConnectTimeout=10 -o StrictHostKeyChecking=no "$SSH_USER@$VM_IP" "echo 'SSH connection successful'" 2>/dev/null; then
    echo "❌ SSH connection failed"
    echo "Please ensure:"
    echo "   • SSH is installed and running on the VM"
    echo "   • Credentials are correct"
    echo "   • Firewall allows SSH connections"
    exit 1
fi

echo "✅ SSH connection successful"

# Copy optimization script to VM
echo "📤 Copying optimization script to VM..."
if scp -o StrictHostKeyChecking=no "$OPTIMIZATION_SCRIPT" "$SSH_USER@$VM_IP:/tmp/optimize-proxmox-vm.sh"; then
    echo "✅ Optimization script copied successfully"
else
    echo "❌ Failed to copy optimization script"
    exit 1
fi

# Make script executable
echo "🔧 Making script executable..."
ssh -o StrictHostKeyChecking=no "$SSH_USER@$VM_IP" "chmod +x /tmp/optimize-proxmox-vm.sh"

# Ask for confirmation before running
echo ""
echo "📋 Ready to optimize VM!"
echo "This will:"
echo "   • Apply system-level performance optimizations"
echo "   • Configure container and Docker settings"
echo "   • Install monitoring tools"
echo "   • Optimize for media stack workloads"
echo ""
read -p "Do you want to proceed with optimization? (y/N): " CONFIRM

if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    echo "⏸️  Optimization cancelled"
    echo "You can run the script manually later:"
    echo "   ssh $SSH_USER@$VM_IP"
    echo "   sudo /tmp/optimize-proxmox-vm.sh"
    exit 0
fi

# Execute optimization script
echo "🚀 Running optimization script on VM..."
echo "This may take several minutes..."
echo ""

if ssh -o StrictHostKeyChecking=no "$SSH_USER@$VM_IP" "sudo /tmp/optimize-proxmox-vm.sh"; then
    echo ""
    echo "🎉 ===== VM OPTIMIZATION COMPLETE ===== 🎉"
    echo ""
    echo "✅ All optimizations applied successfully"
    echo ""
    echo "📋 Next Steps:"
    echo "1. Reboot the VM to apply all changes:"
    echo "   sudo virsh reboot $VM_NAME"
    echo ""
    echo "2. After reboot, verify optimizations:"
    echo "   ssh $SSH_USER@$VM_IP '/usr/local/bin/system-health-check.sh'"
    echo ""
    echo "3. Monitor performance:"
    echo "   ssh $SSH_USER@$VM_IP 'tail -f /var/log/proxmox-performance.log'"
    echo ""
    echo "🎯 Your VM is now optimized for:"
    echo "   • Plex/Jellyfin media streaming"
    echo "   • Docker container workloads"
    echo "   • Home Assistant automation"
    echo "   • AI/ML processing"
    echo "   • Development workflows"
    echo ""
else
    echo "❌ Optimization failed"
    echo "Please check the error messages above"
    echo "You may need to run the script manually:"
    echo "   ssh $SSH_USER@$VM_IP 'sudo /tmp/optimize-proxmox-vm.sh'"
    exit 1
fi
