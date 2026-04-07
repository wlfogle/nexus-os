#!/bin/bash
# 🔧 ALEXA INTEGRATION FIX SCRIPT
# ==============================
# Fixes configuration errors preventing Alexa device discovery

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔧 FIXING ALEXA INTEGRATION ISSUES${NC}"
echo "===================================="

# Configuration variables
HA_HOST="192.168.122.113"
HA_PORT="8123"
FIXED_CONFIG_DIR="/home/lou/awesome_stack/homeassistant-configs-fixed"

echo -e "${YELLOW}📋 Issues Found in Your Current Configuration:${NC}"
echo "1. ❌ Invalid sensor device_class values (connectivity, problem)"
echo "2. ❌ Deprecated systemmonitor platform"
echo "3. ❌ Invalid REST command method (HEAD not supported)"
echo "4. ❌ Missing notification platform module"
echo "5. ❌ Plex integration should not be in YAML"
echo "6. ❌ Network connectivity issues to media servers"
echo ""

echo -e "${GREEN}✅ Fixed Configuration Created:${NC}"
echo "- Fixed sensor device classes"
echo "- Removed deprecated configurations"
echo "- Corrected REST command methods"
echo "- Fixed notification platform"
echo "- Simplified Alexa integration"
echo ""

echo -e "${YELLOW}📁 Fixed files available in:${NC}"
echo "   $FIXED_CONFIG_DIR/"
echo ""
ls -la $FIXED_CONFIG_DIR/

echo ""
echo -e "${BLUE}🚀 DEPLOYMENT STEPS:${NC}"
echo "==================="
echo ""
echo "1. BACKUP your current configuration:"
echo "   - Go to http://${HA_HOST}:${HA_PORT}"
echo "   - Settings > System > Backups > Create Backup"
echo ""
echo "2. REPLACE configuration files:"
echo "   - Use File Editor add-on or SSH"
echo "   - Copy files from $FIXED_CONFIG_DIR/ to /config/"
echo "   - Replace: configuration.yaml, sensors.yaml, rest_commands.yaml"
echo ""
echo "3. RESTART Home Assistant:"
echo "   - Settings > System > Restart"
echo ""
echo "4. ADD Alexa Integration:"
echo "   - Settings > Devices & Services > Add Integration"
echo "   - Search for 'Amazon Alexa'"
echo "   - Follow setup wizard"
echo ""
echo "5. ENABLE Alexa Skill:"
echo "   - Open Amazon Alexa app"
echo "   - Skills & Games > Search 'Home Assistant'"
echo "   - Enable skill and link account"
echo ""
echo "6. DISCOVER DEVICES:"
echo "   - Say: 'Alexa, discover my devices'"
echo "   - Or use Alexa app: Devices > Discover"
echo ""

echo -e "${GREEN}🎯 EXPECTED RESULT:${NC}"
echo "=================="
echo "After fixing these issues, Alexa should discover:"
echo "- 🎬 script.movie_night ('Alexa, turn on movie night')"
echo "- 🖥️ script.system_status ('Alexa, turn on system status')"
echo "- 🤖 script.ai_assistant_status ('Alexa, turn on AI status')"
echo "- 🎮 script.gaming_mode ('Alexa, turn on gaming mode')"
echo "- 📊 All your sensor entities"
echo ""

echo -e "${YELLOW}⚠️ IMPORTANT NOTES:${NC}"
echo "=================="
echo "1. Some media servers appear offline (Plex, Jellyfin)"
echo "2. Make sure these services are running before testing"
echo "3. Check network connectivity to:"
echo "   - 192.168.122.230:32400 (Plex)"
echo "   - 192.168.122.231:8096 (Jellyfin)"
echo "   - 192.168.122.103:9080 (Traefik)"
echo ""

echo -e "${BLUE}🔍 QUICK TEST COMMANDS:${NC}"
echo "======================"
echo "# Test if media servers are running:"
echo "curl -s -o /dev/null -w \"%{http_code}\" http://192.168.122.230:32400"
echo "curl -s -o /dev/null -w \"%{http_code}\" http://192.168.122.231:8096"
echo "curl -s -o /dev/null -w \"%{http_code}\" http://192.168.122.172:11434"
echo ""

echo -e "${GREEN}🎉 Ready to fix your Alexa integration!${NC}"
echo "Apply these configurations and your Echo devices should start discovering Home Assistant entities."
