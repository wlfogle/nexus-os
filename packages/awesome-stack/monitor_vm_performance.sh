#!/bin/bash
# VM Performance monitoring script

echo "=== VM PERFORMANCE MONITOR ==="
echo "Timestamp: $(date)"
echo ""

echo "🖥️ CPU Usage:"
top -bn1 | grep "Cpu(s)" | awk '{print "  Total: " $2 " user, " $4 " system, " $8 " idle"}'

echo ""
echo "💾 Memory Usage:"
free -h | awk 'NR==2{printf "  Used: %s/%s (%.2f%%)\n", $3,$2,$3*100/$2}'

echo ""
echo "📊 System Load:"
uptime | awk -F'load average:' '{print "  " $2}'

echo ""
echo "💿 Storage Usage:"
df -h / | awk 'NR==2{printf "  Root: %s used of %s (%.1f%%)\n", $3, $2, ($3/$2)*100}'

echo ""
echo "🌐 Network Connections:"
ss -tuln | wc -l | awk '{print "  Active connections: " $1}'

echo ""
echo "🔧 Container Status:"
if command -v docker >/dev/null 2>&1; then
    docker ps --format "table {{.Names}}\t{{.Status}}" | head -6
else
    echo "  Docker not available"
fi

echo ""
echo "⚡ Current Optimizations:"
echo "  - Swappiness: $(cat /proc/sys/vm/swappiness)"
echo "  - TCP Congestion: $(sysctl net.ipv4.tcp_congestion_control | cut -d= -f2)"
echo "  - Max Open Files: $(ulimit -n)"
echo "================================"
