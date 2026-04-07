#!/bin/bash

# Container 230: Plex Media Server
# Base: Ubuntu Server 24.04 LTS
# Purpose: Media streaming server with transcoding capabilities
# Target: Proxmox VM at 192.168.122.9

echo "Creating Container 230: Plex Media Server on Proxmox..."

# SSH into Proxmox and create container
ssh root@192.168.122.9 << 'EOF'

# Create container with Ubuntu template
pct create 230 /var/lib/vz/template/cache/ubuntu-24.04-standard_24.04-2_amd64.tar.zst \
  --hostname plex \
  --memory 8192 \
  --cores 4 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.122.230/24,gw=192.168.122.1 \
  --storage local-lvm \
  --rootfs local-lvm:32 \
  --unprivileged 1 \
  --features nesting=1

# Add bind mounts for media and config
pct set 230 --mp0 /mnt/data/media/movies,mp=/media/movies,ro=1
pct set 230 --mp1 /mnt/data/media/tv,mp=/media/tv,ro=1
pct set 230 --mp2 /mnt/data/media/music,mp=/media/music,ro=1
pct set 230 --mp3 /mnt/data/configs/plex,mp=/config

# Start container
pct start 230

# Wait for container to boot
sleep 15

# Install Plex Media Server
pct exec 230 -- bash -c '
# Update system
apt update && apt upgrade -y

# Install dependencies
apt install -y curl wget gnupg2 software-properties-common

# Add Plex repository
curl https://downloads.plex.tv/plex-keys/PlexSign.key | gpg --dearmor | tee /usr/share/keyrings/plex-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/plex-archive-keyring.gpg] https://downloads.plex.tv/repo/deb public main" | tee /etc/apt/sources.list.d/plexmediaserver.list

# Install Plex Media Server
apt update
apt install -y plexmediaserver

# Create necessary directories
mkdir -p /config /media/{movies,tv,music}

# Add plex user to necessary groups
usermod -a -G audio,video plex

# Configure Plex service
systemctl stop plexmediaserver
sed -i "s|PLEX_MEDIA_SERVER_APPLICATION_SUPPORT_DIR=.*|PLEX_MEDIA_SERVER_APPLICATION_SUPPORT_DIR=/config|" /etc/default/plexmediaserver
sed -i "s|PLEX_MEDIA_SERVER_USER=.*|PLEX_MEDIA_SERVER_USER=plex|" /etc/default/plexmediaserver

# Set proper permissions
chown -R plex:plex /config
chmod -R 755 /config

# Install transcoding dependencies
apt install -y ffmpeg intel-media-va-driver-non-free

# Enable and start Plex
systemctl enable plexmediaserver
systemctl start plexmediaserver

# Configure basic firewall
ufw --force enable
ufw allow 32400/tcp
ufw allow 3005/tcp
ufw allow 8324/tcp
ufw allow 32469/tcp
ufw allow 1900/udp
ufw allow 32410:32414/udp

echo "Plex Media Server installation completed"
echo "Service status:"
systemctl status plexmediaserver --no-pager -l
'

echo "Container 230 (Plex) setup completed!"
echo "Access: http://192.168.122.230:32400/web"

EOF

echo "Container 230 (Plex) created successfully on Proxmox!"
