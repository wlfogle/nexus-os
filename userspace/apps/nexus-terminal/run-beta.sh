#!/bin/bash

# NexusTerminal Beta Run Script
# This script ensures all environment variables are properly loaded

set -e

echo "🚀 Starting NexusTerminal v1.0.0-beta.1"
echo "🔧 Loading environment configuration..."

# Load environment variables from .env file
if [ -f .env ]; then
    echo "✅ Loading .env file..."
    export $(cat .env | grep -v '^#' | xargs)
    echo "✅ Environment variables loaded"
    echo "   OLLAMA_MODELS: ${OLLAMA_MODELS:-'not set'}"
    echo "   OLLAMA_HOST: ${OLLAMA_HOST:-'not set'}"
    echo "   OLLAMA_MODEL: ${OLLAMA_MODEL:-'not set'}"
else
    echo "⚠️  .env file not found, using system environment"
fi

echo ""
echo "🎯 Starting development server..."

# Run the application
npm run tauri:dev
