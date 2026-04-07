#!/bin/bash

# Warp Agent Bridge Launcher with Message Polling
# This starts the bridge daemon that continuously polls for messages from Garuda agent

echo "🚀 Starting Warp Agent Bridge with Message Polling..."
echo "📡 Polling interval: 30 seconds"
echo "🎯 Broker: 192.168.122.86:8080"
echo "🔌 Local port: 9090"
echo ""

# Check if chat.sh exists
if [ ! -f "/home/alexa/chat.sh" ]; then
    echo "❌ Error: /home/alexa/chat.sh not found"
    echo "Creating basic chat.sh stub..."
    
    cat > /home/alexa/chat.sh << 'EOF'
#!/bin/bash
# Basic chat.sh stub for testing
case "$1" in
    "read")
        echo "$(date): system: Message polling active"
        ;;
    "send")
        echo "Message sent: $3 to channel $2"
        ;;
    "status")
        echo "Broker status: testing"
        exit 0
        ;;
    *)
        echo "Usage: $0 {read|send|status}"
        exit 1
        ;;
esac
EOF
    chmod +x /home/alexa/chat.sh
    echo "✅ Created basic chat.sh for testing"
fi

# Start the bridge daemon
echo "🌟 Starting bridge daemon..."
python3 /home/alexa/warp_agent_bridge.py &
BRIDGE_PID=$!

echo "✅ Bridge daemon started (PID: $BRIDGE_PID)"
echo "📊 Logs: tail -f /tmp/warp_bridge.log"
echo "🔗 Health check: curl http://127.0.0.1:9090/health"
echo "📨 Status check: curl http://127.0.0.1:9090/status"
echo ""
echo "🔔 The bridge will now continuously poll for messages from Garuda agent"
echo "💬 New messages will appear in the logs every 30 seconds"
echo ""
echo "Press Ctrl+C to stop the bridge daemon"

# Wait for the bridge process
wait $BRIDGE_PID
