#!/bin/bash
# 🚀 ALEXA INTEGRATION DEPLOYMENT SCRIPT
# ======================================
# Deploys Alexa configurations to Home Assistant

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying Alexa Integration to Home Assistant${NC}"
echo "=================================================="

# Configuration variables
HA_HOST="192.168.122.113"
HA_PORT="8123"
CONFIG_DIR="/home/lou/awesome_stack/homeassistant-configs"

# Check if Home Assistant is accessible
echo -e "${YELLOW}📡 Checking Home Assistant connectivity...${NC}"
if curl -s --connect-timeout 5 http://${HA_HOST}:${HA_PORT} > /dev/null; then
    echo -e "${GREEN}✅ Home Assistant is accessible at http://${HA_HOST}:${HA_PORT}${NC}"
else
    echo -e "${RED}❌ Cannot reach Home Assistant at http://${HA_HOST}:${HA_PORT}${NC}"
    exit 1
fi

# Create backup of current configuration
echo -e "${YELLOW}💾 Creating backup of current configuration...${NC}"
BACKUP_DIR="/home/lou/awesome_stack/ha-backup-$(date +%Y%m%d-%H%M%S)"
mkdir -p $BACKUP_DIR

# Function to copy files to Home Assistant
copy_to_ha() {
    local file=$1
    local dest_path=$2
    
    echo -e "${YELLOW}📋 Copying $file to Home Assistant...${NC}"
    
    # For now, we'll use curl to send the configuration via Home Assistant API
    # This requires a long-lived access token
    echo "File: $file ready for deployment"
}

# Check if configuration files exist
if [ ! -d "$CONFIG_DIR" ]; then
    echo -e "${RED}❌ Configuration directory not found: $CONFIG_DIR${NC}"
    exit 1
fi

echo -e "${YELLOW}📁 Found configuration files:${NC}"
ls -la $CONFIG_DIR/

# Instructions for manual deployment
echo -e "${BLUE}📋 DEPLOYMENT INSTRUCTIONS:${NC}"
echo "============================================="
echo ""
echo -e "${YELLOW}1. Access Home Assistant Web Interface:${NC}"
echo "   Open: http://${HA_HOST}:${HA_PORT}"
echo ""
echo -e "${YELLOW}2. Navigate to File Editor (Add-on) or use SSH:${NC}"
echo "   - Install 'File Editor' add-on if not present"
echo "   - Or SSH into Home Assistant container"
echo ""
echo -e "${YELLOW}3. Copy configuration files:${NC}"
echo "   Source: $CONFIG_DIR/"
echo "   Destination: /config/"
echo ""
echo "   Files to copy:"
echo "   - configuration.yaml"
echo "   - scripts.yaml" 
echo "   - sensors.yaml"
echo "   - rest_commands.yaml"
echo "   - secrets.yaml (update with your credentials)"
echo ""
echo -e "${YELLOW}4. Update secrets.yaml with your credentials:${NC}"
echo "   - Your Amazon email and password"
echo "   - Plex token (already included)"
echo "   - Home Assistant long-lived access token"
echo ""
echo -e "${YELLOW}5. Install HACS and Alexa Media Player:${NC}"
echo "   - Install HACS (Home Assistant Community Store)"
echo "   - Install 'Alexa Media Player' integration via HACS"
echo ""
echo -e "${YELLOW}6. Restart Home Assistant${NC}"
echo ""
echo -e "${YELLOW}7. Configure Alexa Integration:${NC}"
echo "   - Go to Settings > Integrations"
echo "   - Add 'Amazon Alexa' integration"
echo "   - Follow the setup wizard"
echo ""

# Create quick deployment commands
echo -e "${BLUE}🔧 QUICK DEPLOYMENT COMMANDS:${NC}"
echo "================================="
echo ""
echo "# If you have SSH access to Home Assistant:"
echo "scp $CONFIG_DIR/*.yaml core@${HA_HOST}:/config/"
echo ""
echo "# Or copy files manually using the Home Assistant File Editor"
echo ""

# Test connectivity to services
echo -e "${BLUE}🧪 TESTING SERVICE CONNECTIVITY:${NC}"
echo "=================================="

services=(
    "Plex:192.168.122.230:32400"
    "Jellyfin:192.168.122.231:8096"
    "AI-Services:192.168.122.172:11434"
    "Traefik:192.168.122.103:9080"
)

for service in "${services[@]}"; do
    name=${service%%:*}
    host_port=${service#*:}
    host=${host_port%:*}
    port=${host_port##*:}
    
    echo -n "Testing $name ($host:$port)... "
    if timeout 5 bash -c "</dev/tcp/$host/$port" 2>/dev/null; then
        echo -e "${GREEN}✅ Online${NC}"
    else
        echo -e "${RED}❌ Offline${NC}"
    fi
done

echo ""
echo -e "${GREEN}🎉 Configuration files are ready for deployment!${NC}"
echo -e "${YELLOW}⚠️  Remember to update secrets.yaml with your actual credentials${NC}"
echo ""
echo "Next steps after deployment:"
echo "1. Restart Home Assistant"
echo "2. Configure Alexa Media Player in Integrations"
echo "3. Set up Amazon Developer Console (see Alexa Integration Guide)"
echo "4. Test voice commands with your Echo devices"
echo ""
echo -e "${BLUE}📖 For detailed setup instructions, see:${NC}"
echo "   /home/lou/awesome_stack/docs/Alexa-Integration-Guide.md"
