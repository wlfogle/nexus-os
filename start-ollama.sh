#!/bin/bash

# Set the OLLAMA_MODELS environment variable to point to the models directory
export OLLAMA_MODELS="/mnt/media/workspace/models"

# Find the ollama binary
OLLAMA_BIN=""

# Check common locations for ollama
for location in \
    "$(which ollama 2>/dev/null)" \
    "/usr/local/bin/ollama" \
    "/usr/bin/ollama" \
    "/opt/ollama/bin/ollama" \
    "$HOME/.local/bin/ollama" \
    "$HOME/bin/ollama" \
    "./ollama" \
    "../ollama"; do
    
    if [ -x "$location" ]; then
        OLLAMA_BIN="$location"
        break
    fi
done

if [ -z "$OLLAMA_BIN" ]; then
    echo "Error: ollama binary not found. Please install ollama or add it to your PATH."
    exit 1
fi

echo "Starting Ollama with models directory: $OLLAMA_MODELS"
echo "Using ollama binary: $OLLAMA_BIN"

# Kill any existing ollama processes
pkill ollama 2>/dev/null || true

# Wait a moment for processes to terminate
sleep 1

# Start ollama serve in the background
exec "$OLLAMA_BIN" serve
