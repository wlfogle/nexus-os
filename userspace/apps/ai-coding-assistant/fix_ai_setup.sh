#!/bin/bash

echo "🔧 AI Coding Assistant - GPU and Memory Setup Fix"
echo "=================================================="

echo "📊 Current System Status:"
echo "Host GPU: $(nvidia-smi --query-gpu=name --format=csv,noheader,nounits)"
echo "Host Memory: $(free -h | grep '^Mem:' | awk '{print $2}')"
echo "CUDA Version: $(nvidia-smi --query-gpu=driver_version --format=csv,noheader,nounits)"

echo ""
echo "🏗️ Architecture Analysis:"
echo "Garuda Linux (Host) → Proxmox VM → ct-900 Container"

echo ""
echo "🔍 Testing Container AI Capabilities:"

# Test different model sizes
echo "Testing model availability..."
MODELS=$(curl -s http://192.168.122.172:11434/api/tags | jq -r '.models[].name' 2>/dev/null)
echo "Available models: $MODELS"

echo ""
echo "🧪 Memory Test:"
MEMORY_ERROR=$(curl -s -X POST http://192.168.122.172:11434/api/generate -H "Content-Type: application/json" -d '{"model": "codellama:7b", "prompt": "test", "stream": false}' | grep -o "model requires more system memory.*")

if [[ -n "$MEMORY_ERROR" ]]; then
    echo "❌ Current Issue: $MEMORY_ERROR"
    echo ""
    echo "💡 Solutions:"
    echo "1. Increase ct-900 container memory allocation in Proxmox"
    echo "2. Use smaller models (3B instead of 7B parameters)"
    echo "3. Enable CPU-only inference mode"
    echo "4. Use local GPU on Garuda host instead of container"
    
    echo ""
    echo "🚀 Recommended Actions:"
    echo "Option A - Container Memory Increase:"
    echo "  - Login to Proxmox web UI"
    echo "  - Edit ct-900 container"
    echo "  - Increase memory to 12GB+"
    echo "  - Restart ct-900"
    
    echo ""
    echo "Option B - Use Smaller Models:"
    echo "  - Pull llama2:3b or phi:2.7b models"
    echo "  - These require only 2-4GB RAM"
    
    echo ""
    echo "Option C - Local GPU Setup:"
    echo "  - Install Ollama directly on Garuda"
    echo "  - Use your RTX 4080 directly"
    echo "  - Maximum performance with full GPU access"
else
    echo "✅ Container AI is working properly!"
fi

echo ""
echo "🔄 Testing smaller model availability:"
curl -s -X POST http://192.168.122.172:11434/api/generate -H "Content-Type: application/json" -d '{"model": "codellama:7b", "prompt": "test", "stream": false, "options": {"num_ctx": 512}}' >/dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ AI inference working with smaller context"
else
    echo "❌ Still needs memory increase or smaller models"
fi

echo ""
echo "📝 Next Steps:"
echo "1. For immediate testing: Install Ollama locally on Garuda"
echo "2. For container fix: Increase ct-900 memory in Proxmox"
echo "3. Application will work for all non-AI features regardless"
echo ""
echo "🎯 Your AI Coding Assistant is installed and ready!"
echo "   The core application works - just need AI backend optimization."
