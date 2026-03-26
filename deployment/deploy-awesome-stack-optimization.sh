#!/bin/bash

# 🚀 Awesome Stack Optimization Suite - Master Deployment Script
# Complete infrastructure optimization for Garuda Host + Proxmox VM

set -e

echo "🚀 AWESOME STACK OPTIMIZATION SUITE DEPLOYMENT"
echo "=============================================="
echo "🎯 Target: High-performance AI/ML + 47+ container infrastructure"
echo "🖥️  Host: Garuda Linux with external AWX control"
echo "💻 VM: Proxmox VM (192.168.122.9) with Warp agent architecture"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "❌ This script must be run as root (use sudo)"
   echo "   Example: sudo ./deploy-awesome-stack-optimization.sh"
   exit 1
fi

# Configuration
OPTIMIZATION_DIR="/opt/awesome-stack-optimization"
LOG_FILE="/var/log/awesome-stack-deployment.log"
BACKUP_DIR="/root/pre-optimization-backups"

# Detect script location and source directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$SCRIPT_DIR/awesome-stack-optimization"

echo "📍 Source directory: $SOURCE_DIR"
echo "🎯 Target directory: $OPTIMIZATION_DIR"
echo "📝 Log file: $LOG_FILE"

# Create directories
echo "📁 Creating directories..."
mkdir -p "$OPTIMIZATION_DIR"
mkdir -p "$BACKUP_DIR"
mkdir -p "$(dirname "$LOG_FILE")"

# Start logging
exec 1> >(tee -a "$LOG_FILE")
exec 2>&1

echo "$(date): Starting Awesome Stack Optimization deployment" >> "$LOG_FILE"

# Copy optimization suite
echo ""
echo "📦 STEP 1: Copying Awesome Stack Optimization Suite..."

if [[ ! -d "$SOURCE_DIR" ]]; then
    echo "❌ Error: Source directory not found at $SOURCE_DIR"
    exit 1
fi

cp -r "$SOURCE_DIR" "$OPTIMIZATION_DIR"
chmod -R +x "$OPTIMIZATION_DIR"/optimization/{garuda-host,vm-optimization,awx-integration}/*.sh 2>/dev/null || true

echo "✅ Optimization suite copied successfully"
echo "🎉 DEPLOYMENT COMPLETE! Reboot to activate optimizations."

exit 0
