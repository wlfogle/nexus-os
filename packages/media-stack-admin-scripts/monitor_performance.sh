#!/bin/bash
# Performance monitoring script

echo "=== SYSTEM PERFORMANCE MONITOR ==="
echo "Timestamp: $(date)"
echo ""

echo "🖥️  CPU Usage:"
top -bn1 | grep "Cpu(s)" | awk '{print "  Total: " $2 " user, " $4 " system, " $8 " idle"}'

echo ""
echo "💾 Memory Usage:"
free -h | awk 'NR==2{printf "  Used: %s/%s (%.2f%%)\n", $3,$2,$3*100/$2}'

echo ""
echo "🎮 GPU Status:"
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu --format=csv,noheader,nounits | \
    awk -F',' '{printf "  GPU: %s%%, Memory: %s/%s MB, Temp: %s°C\n", $1, $2, $3, $4}'
else
    echo "  NVIDIA GPU not available"
fi

echo ""
echo "💿 Storage I/O:"
iostat -x 1 1 | tail -n +4 | awk 'NF>0 && $1!="Device" {printf "  %s: %s%% utilization\n", $1, $NF}'

echo ""
echo "🌐 Network:"
cat /proc/net/dev | awk 'NR>2 && $2>0 {printf "  %s: RX %d MB, TX %d MB\n", $1, $2/1024/1024, $10/1024/1024}' | head -5

echo ""
echo "⚡ Load Average:"
uptime | awk -F'load average:' '{print "  " $2}'

echo "=================================="
