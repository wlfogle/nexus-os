#!/bin/bash

echo "🔍 PROXMOX VM OPTIMIZATION VERIFICATION"
echo "======================================="
echo ""

echo "📊 System Information:"
echo "======================"
echo "   • OS: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2)"
echo "   • Kernel: $(uname -r 2>/dev/null || echo 'N/A (chroot)')"
echo "   • CPU Cores: $(nproc)"
echo "   • Total RAM: $(free -h 2>/dev/null | awk 'NR==2{print $2}' || echo 'N/A (chroot)')"
echo ""

echo "⚙️  System Optimizations:"
echo "========================"

optimizations=(
    "vm.swappiness"
    "vm.nr_hugepages"
    "net.core.rmem_max"
    "net.ipv4.tcp_congestion_control"
    "fs.file-max"
    "kernel.pid_max"
    "user.max_user_namespaces"
)

for opt in "${optimizations[@]}"; do
    if [[ -f "/etc/sysctl.d/99-proxmox-warp-optimization.conf" ]] && grep -q "$opt" /etc/sysctl.d/99-proxmox-warp-optimization.conf; then
        value=$(grep "$opt" /etc/sysctl.d/99-proxmox-warp-optimization.conf | cut -d'=' -f2 | xargs)
        echo "   ✅ $opt = $value"
    else
        echo "   ❌ $opt: Not configured"
    fi
done

echo ""
echo "🐳 Docker Configuration:"
echo "========================"

if [[ -f "/etc/docker/daemon.json" ]]; then
    echo "   ✅ Docker daemon.json: EXISTS"
    echo "   • Storage driver: $(grep -o '"storage-driver":[^,]*' /etc/docker/daemon.json | cut -d':' -f2 | tr -d '" ')"
    echo "   • Default SHM size: $(grep -o '"default-shm-size":[^,]*' /etc/docker/daemon.json | cut -d':' -f2 | tr -d '" ')"
else
    echo "   ❌ Docker daemon.json: MISSING"
fi

echo ""
echo "📁 File Browser Quantum:"
echo "========================"

filebrowser_files=(
    "/usr/local/bin/install-filebrowser-quantum.sh"
    "/usr/local/bin/manage-filebrowser-quantum.sh"
    "/usr/local/bin/inject-filebrowser-to-containers.sh"
)

for file in "${filebrowser_files[@]}"; do
    if [[ -x "$file" ]]; then
        echo "   ✅ $file: EXISTS & EXECUTABLE"
    else
        echo "   ❌ $file: MISSING OR NOT EXECUTABLE"
    fi
done

if [[ -d "/opt/filebrowser" ]]; then
    echo "   ✅ File Browser config directory: EXISTS"
else
    echo "   ❌ File Browser config directory: MISSING"
fi

echo ""
echo "🎯 System Limits:"
echo "=================="

if [[ -f "/etc/security/limits.d/99-proxmox-warp-optimization.conf" ]]; then
    echo "   ✅ Enhanced system limits: CONFIGURED"
    echo "     • Max open files: $(grep 'nofile' /etc/security/limits.d/99-proxmox-warp-optimization.conf | head -1 | awk '{print $4}')"
    echo "     • Max processes: $(grep 'nproc' /etc/security/limits.d/99-proxmox-warp-optimization.conf | head -1 | awk '{print $4}')"
else
    echo "   ❌ Enhanced system limits: NOT CONFIGURED"
fi

echo ""
echo "🚀 Ready for Deployment:"
echo "========================"

config_complete=true
for file in "/etc/sysctl.d/99-proxmox-warp-optimization.conf" "/etc/security/limits.d/99-proxmox-warp-optimization.conf" "/usr/local/bin/install-filebrowser-quantum.sh"; do
    if [[ ! -f "$file" ]]; then
        config_complete=false
        break
    fi
done

if $config_complete; then
    echo "🎉 PROXMOX VM OPTIMIZATION: COMPLETE & READY!"
    echo ""
    echo "📋 Next Steps:"
    echo "   1. Boot the VM and run: /usr/local/bin/install-filebrowser-quantum.sh"
    echo "   2. Access File Browser at: http://192.168.122.9:8090"
    echo "   3. Inject File Browser to containers: manage-filebrowser-quantum.sh inject"
    echo "   4. Your 47+ container infrastructure will have unified file management"
else
    echo "❌ PROXMOX VM OPTIMIZATION: INCOMPLETE"
fi
