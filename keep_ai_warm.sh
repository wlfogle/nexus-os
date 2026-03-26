#!/bin/bash

# Keep AI model warm script
# This script sends periodic requests to keep the model loaded in memory

AI_HOST="192.168.122.172:11434"
MODEL_NAME="codellama:7b"
KEEP_ALIVE="15m"

echo "Starting AI keep-warm service..."
echo "Target: $AI_HOST"
echo "Model: $MODEL_NAME"
echo "Keep alive: $KEEP_ALIVE"

while true; do
    echo "$(date): Sending keep-warm request..."
    
    response=$(curl -s -X POST "http://$AI_HOST/api/generate" \
        -H "Content-Type: application/json" \
        -d "{\"model\":\"$MODEL_NAME\",\"prompt\":\"ping\",\"stream\":false,\"keep_alive\":\"$KEEP_ALIVE\"}" \
        --max-time 60)
    
    if [ $? -eq 0 ]; then
        echo "$(date): Keep-warm successful"
    else
        echo "$(date): Keep-warm failed"
    fi
    
    # Wait 10 minutes before next keep-warm
    sleep 600
done
