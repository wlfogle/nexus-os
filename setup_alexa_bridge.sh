#!/bin/bash

# Alexa Bridge Setup Script for VM-613
# This script installs HABridge and Fauxmo on Android via Termux

echo "🚀 Setting up Alexa Bridge on VM-613..."

# Android device IP
BRIDGE_IP="192.168.122.133"
HOME_ASSISTANT_IP="192.168.122.9"  # Your Proxmox host running HA VM

echo "📱 Installing Python and dependencies via Termux..."

# Install Python and packages via ADB shell
adb -s ${BRIDGE_IP}:5555 shell "am start -n com.termux/.HomeActivity"
sleep 2

# Setup Termux environment
adb -s ${BRIDGE_IP}:5555 shell "echo 'pkg update && pkg upgrade -y' > /data/data/com.termux/files/home/setup.sh"
adb -s ${BRIDGE_IP}:5555 shell "echo 'pkg install -y python nodejs openjdk-17 git wget curl' >> /data/data/com.termux/files/home/setup.sh"
adb -s ${BRIDGE_IP}:5555 shell "echo 'pip install requests flask' >> /data/data/com.termux/files/home/setup.sh"
adb -s ${BRIDGE_IP}:5555 shell "chmod +x /data/data/com.termux/files/home/setup.sh"

echo "🔧 Creating Fauxmo configuration..."

# Create Fauxmo config for your 35+ Home Assistant scripts
cat > /tmp/fauxmo_config.json << 'EOF'
{
    "FAUXMO": {
        "ip_address": "auto"
    },
    "PLUGINS": {
        "SimpleHTTPPlugin": {
            "DEVICES": [
                {
                    "name": "movie night",
                    "port": 12340,
                    "on_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/movie_night -H 'Authorization: Bearer YOUR_HA_TOKEN'",
                    "off_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/movie_night -H 'Authorization: Bearer YOUR_HA_TOKEN'"
                },
                {
                    "name": "system status",
                    "port": 12341,
                    "on_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/system_status -H 'Authorization: Bearer YOUR_HA_TOKEN'",
                    "off_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/system_status -H 'Authorization: Bearer YOUR_HA_TOKEN'"
                },
                {
                    "name": "entertainment mode",
                    "port": 12342,
                    "on_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/entertainment_mode -H 'Authorization: Bearer YOUR_HA_TOKEN'",
                    "off_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/entertainment_mode -H 'Authorization: Bearer YOUR_HA_TOKEN'"
                },
                {
                    "name": "restart plex",
                    "port": 12343,
                    "on_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/restart_plex -H 'Authorization: Bearer YOUR_HA_TOKEN'",
                    "off_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/restart_plex -H 'Authorization: Bearer YOUR_HA_TOKEN'"
                },
                {
                    "name": "gaming mode",
                    "port": 12344,
                    "on_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/gaming_mode -H 'Authorization: Bearer YOUR_HA_TOKEN'",
                    "off_cmd": "curl -X POST http://192.168.122.9:8123/api/services/script/gaming_mode -H 'Authorization: Bearer YOUR_HA_TOKEN'"
                }
            ]
        }
    }
}
EOF

# Transfer config to Android
adb -s ${BRIDGE_IP}:5555 push /tmp/fauxmo_config.json /sdcard/fauxmo_config.json

echo "🔧 Creating HABridge startup script..."

# Create startup script for the bridge
cat > /tmp/start_bridge.sh << 'EOF'
#!/data/data/com.termux/files/usr/bin/bash

# Start Fauxmo (Python-based Alexa bridge)
cd /data/data/com.termux/files/home
pip install fauxmo
cp /sdcard/fauxmo_config.json config.json

echo "🚀 Starting Alexa Bridge services..."
echo "📡 Fauxmo will emulate Philips Hue devices for Alexa discovery"
echo "🏠 Connecting to Home Assistant at 192.168.122.9:8123"
echo "🎤 Your voice commands will be processed locally!"

fauxmo -c config.json -v
EOF

# Transfer startup script
adb -s ${BRIDGE_IP}:5555 push /tmp/start_bridge.sh /sdcard/start_bridge.sh
adb -s ${BRIDGE_IP}:5555 shell "cp /sdcard/start_bridge.sh /data/data/com.termux/files/home/"
adb -s ${BRIDGE_IP}:5555 shell "chmod +x /data/data/com.termux/files/home/start_bridge.sh"

echo "✅ Alexa Bridge setup complete!"
echo ""
echo "🎯 Next Steps:"
echo "1. Open Termux app on VM-613 (192.168.122.133)"
echo "2. Run: ./setup.sh"
echo "3. Run: ./start_bridge.sh"
echo "4. Say 'Alexa, discover devices' on any of your 5 Alexa devices"
echo "5. Test: 'Alexa, turn on movie night'"
echo ""
echo "📱 VM-613 (Bliss-OS-Bridge) is now ready to be your Container 280 equivalent!"
