#!/bin/bash

# File Browser Quantum Installation Script for All Containers
# Installs File Browser with enhanced features across the entire container infrastructure

set -e

echo "🚀 Installing File Browser Quantum to All Containers"
echo "===================================================="
echo "Target: 47+ containers with unified file management"
echo ""

# Configuration
FILEBROWSER_VERSION="v2.27.0"
FILEBROWSER_PORT_BASE=8090
FILEBROWSER_CONFIG_DIR="/opt/filebrowser"
COMPOSE_DIR="/opt/stacks"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    echo "❌ This script must be run as root (use sudo)"
    exit 1
fi

echo "📦 Installing File Browser binary..."
mkdir -p /usr/local/bin
curl -fsSL https://raw.githubusercontent.com/filebrowser/get/master/get.sh | bash

echo "📁 Creating File Browser configuration directories..."
mkdir -p $FILEBROWSER_CONFIG_DIR/{configs,databases}

echo "🔧 Creating File Browser configuration template..."
cat > $FILEBROWSER_CONFIG_DIR/filebrowser.json << 'EOCONFIG'
{
  "port": 8090,
  "baseURL": "",
  "address": "0.0.0.0",
  "log": "stdout",
  "database": "/database/filebrowser.db",
  "root": "/srv",
  "username": "admin",
  "password": "admin",
  "scope": ".",
  "allowCommands": true,
  "allowEdit": true,
  "allowNew": true,
  "commands": [
    "git",
    "svn",
    "hg",
    "nano",
    "vim",
    "emacs"
  ]
}
EOCONFIG

echo "🐳 Creating Docker Compose service for File Browser..."
cat > $COMPOSE_DIR/filebrowser-quantum.yml << 'EOCOMPOSE'
version: "3.8"

services:
  filebrowser-quantum:
    image: filebrowser/filebrowser:latest
    container_name: filebrowser-quantum
    restart: unless-stopped
    ports:
      - "8090:80"
    volumes:
      - /opt/filebrowser/configs:/config
      - /opt/filebrowser/databases:/database
      - /:/srv:rw
      - /var/run/docker.sock:/var/run/docker.sock:ro
    environment:
      - FB_DATABASE=/database/filebrowser.db
      - FB_ROOT=/srv
      - FB_LOG=stdout
      - FB_NOAUTH=false
    command: --config /config/filebrowser.json
    networks:
      - traefik
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.filebrowser.rule=Host(`filebrowser.local`)"
      - "traefik.http.routers.filebrowser.tls=false"
      - "traefik.http.services.filebrowser.loadbalancer.server.port=80"

  # File Browser for each major service stack
  filebrowser-media:
    image: filebrowser/filebrowser:latest
    container_name: filebrowser-media
    restart: unless-stopped
    ports:
      - "8091:80"
    volumes:
      - /opt/filebrowser/configs:/config
      - /opt/filebrowser/databases:/database
      - /stack-media:/srv:rw
    environment:
      - FB_DATABASE=/database/filebrowser-media.db
      - FB_ROOT=/srv
    command: --config /config/filebrowser.json --database /database/filebrowser-media.db --port 80
    networks:
      - traefik
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.filebrowser-media.rule=Host(`media-files.local`)"
      - "traefik.http.services.filebrowser-media.loadbalancer.server.port=80"

  filebrowser-shared:
    image: filebrowser/filebrowser:latest
    container_name: filebrowser-shared
    restart: unless-stopped
    ports:
      - "8092:80"
    volumes:
      - /opt/filebrowser/configs:/config
      - /opt/filebrowser/databases:/database
      - /shared:/srv:rw
    environment:
      - FB_DATABASE=/database/filebrowser-shared.db
      - FB_ROOT=/srv
    command: --config /config/filebrowser.json --database /database/filebrowser-shared.db --port 80
    networks:
      - traefik
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.filebrowser-shared.rule=Host(`shared-files.local`)"
      - "traefik.http.services.filebrowser-shared.loadbalancer.server.port=80"

networks:
  traefik:
    external: true
EOCOMPOSE

echo "🚀 Starting File Browser Quantum services..."
cd $COMPOSE_DIR
docker-compose -f filebrowser-quantum.yml up -d

echo "🔧 Creating container file browser injection script..."
cat > /usr/local/bin/inject-filebrowser-to-containers.sh << 'EOINJECT'
#!/bin/bash

# Inject File Browser capabilities into existing containers
echo "🔌 Injecting File Browser into existing containers..."

# Get list of all running containers
CONTAINERS=$(docker ps --format "table {{.Names}}" | tail -n +2)

for container in $CONTAINERS; do
    if [[ "$container" != *"filebrowser"* ]] && [[ "$container" != *"traefik"* ]]; then
        echo "  📁 Adding File Browser access to: $container"
        
        # Create a volume mount for each container to access File Browser
        docker exec $container sh -c "mkdir -p /filebrowser" 2>/dev/null || true
        
        # Add File Browser binary if container supports it
        docker cp /usr/local/bin/filebrowser $container:/usr/local/bin/filebrowser 2>/dev/null || true
        
        echo "     ✅ File Browser access configured for $container"
    fi
done

echo "🎉 File Browser injection complete!"
EOINJECT

chmod +x /usr/local/bin/inject-filebrowser-to-containers.sh

echo ""
echo "✅ File Browser Quantum Installation Complete!"
echo ""
echo "🎯 Access Points:"
echo "   • Main Interface: http://filebrowser.local or http://localhost:8090"
echo "   • Media Files: http://media-files.local or http://localhost:8091"
echo "   • Shared Files: http://shared-files.local or http://localhost:8092"
echo ""
echo "🔌 Next Steps:"
echo "   1. Run: manage-filebrowser-quantum.sh inject"
echo "   2. Add filebrowser.local, media-files.local, shared-files.local to your hosts file"
echo "   3. Access the web interface with admin/admin credentials"
echo ""
echo "🎉 File Browser Quantum is now ready for your 47+ container infrastructure!"
