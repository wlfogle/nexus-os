// System Controller - Integrating i9-13900HX optimizations
// Based on https://github.com/wlfogle/i9-13900hx-optimizations

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::fs::{self, OpenOptions};
use std::io::Write;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use tokio::process::Command as AsyncCommand;
use crate::ai::SystemState;

pub mod kernel;
pub mod ollama;
pub mod gaming;
pub mod virtualization;
pub mod security;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemController {
    pub current_governor: String,
    pub kernel_optimizations: KernelOptimizations,
    pub ollama_config: OllamaConfig,
    pub gaming_mode: bool,
    pub virtualization_enabled: bool,
    pub security_hardening: bool,
    pub huge_pages_enabled: bool,
    pub performance_profile: PerformanceProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelOptimizations {
    pub custom_kernel_installed: bool,
    pub kernel_version: String,
    pub optimizations_enabled: Vec<String>,
    pub compilation_flags: Vec<String>,
    pub performance_tweaks: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub optimized: bool,
    pub huge_pages_gb: u32,
    pub models_installed: Vec<String>,
    pub performance_governor_set: bool,
    pub monitoring_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceProfile {
    Gaming,
    Development,
    LLMInference,
    Virtualization,
    Balanced,
}

impl SystemController {
    pub async fn new_garuda() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🚀 Initializing System Controller with Garuda Linux optimizations...");
        
        let mut controller = Self {
            current_governor: "powersave".to_string(),
            kernel_optimizations: KernelOptimizations {
                custom_kernel_installed: false,
                kernel_version: String::new(),
                optimizations_enabled: Vec::new(),
                compilation_flags: Vec::new(),
                performance_tweaks: HashMap::new(),
            },
            ollama_config: OllamaConfig {
                optimized: false,
                huge_pages_gb: 0,
                models_installed: Vec::new(),
                performance_governor_set: false,
                monitoring_enabled: false,
            },
            gaming_mode: false,
            virtualization_enabled: false,
            security_hardening: false,
            huge_pages_enabled: false,
            performance_profile: PerformanceProfile::Balanced,
        };
        
        // Detect current system state
        controller.detect_current_configuration().await?;
        
        // Initialize i9-13900HX specific optimizations
        controller.initialize_i9_optimizations().await?;
        
        Ok(controller)
    }
    
    async fn detect_current_configuration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("🔍 Detecting current system configuration...");
        
        // Check current CPU governor
        if let Ok(governor) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
            self.current_governor = governor.trim().to_string();
        }
        
        // Check kernel version
        if let Ok(version) = fs::read_to_string("/proc/version") {
            self.kernel_optimizations.kernel_version = version.lines().next().unwrap_or("unknown").to_string();
            
            // Check if custom kernel is installed (look for specific optimizations)
            if version.contains("zen") || version.contains("performance") {
                self.kernel_optimizations.custom_kernel_installed = true;
            }
        }
        
        // Check huge pages
        if let Ok(hugepages) = fs::read_to_string("/proc/meminfo") {
            if hugepages.contains("HugePages_Total:") && !hugepages.contains("HugePages_Total:        0") {
                self.huge_pages_enabled = true;
                
                // Extract huge pages size for Ollama
                for line in hugepages.lines() {
                    if line.starts_with("HugePages_Total:") {
                        if let Some(total_str) = line.split_whitespace().nth(1) {
                            if let Ok(total) = total_str.parse::<u32>() {
                                // Each huge page is typically 2MB
                                self.ollama_config.huge_pages_gb = (total * 2) / 1024;
                                if self.ollama_config.huge_pages_gb > 0 {
                                    self.ollama_config.optimized = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Check if Ollama is installed and configured
        if let Ok(_) = AsyncCommand::new("which").arg("ollama").output().await {
            self.ollama_config.models_installed = self.get_ollama_models().await?;
        }
        
        // Check virtualization support
        if let Ok(virt_check) = fs::read_to_string("/proc/cpuinfo") {
            if virt_check.contains("vmx") || virt_check.contains("svm") {
                self.virtualization_enabled = true;
            }
        }
        
        Ok(())
    }
    
    async fn initialize_i9_optimizations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔧 Initializing i9-13900HX specific optimizations...");
        
        // Set up performance tweaks specific to i9-13900HX
        self.kernel_optimizations.performance_tweaks.insert(
            "intel_pstate".to_string(), 
            "active".to_string()
        );
        
        self.kernel_optimizations.performance_tweaks.insert(
            "cpu_freq_min".to_string(), 
            "800000".to_string() // 800 MHz minimum
        );
        
        self.kernel_optimizations.performance_tweaks.insert(
            "cpu_freq_max".to_string(), 
            "5400000".to_string() // 5.4 GHz maximum
        );
        
        // i9-13900HX has 24 cores (8 P-cores + 16 E-cores)
        self.kernel_optimizations.performance_tweaks.insert(
            "cores_total".to_string(), 
            "24".to_string()
        );
        
        self.kernel_optimizations.performance_tweaks.insert(
            "threads_total".to_string(), 
            "32".to_string() // 8P-cores with HT + 16 E-cores
        );
        
        Ok(())
    }
    
    pub async fn compile_custom_kernel(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        info!("🔨 Starting custom kernel compilation for i9-13900HX...");
        
        // Based on compile-custom-kernel.sh from i9-13900hx-optimizations
        let compilation_script = r#"#!/bin/bash
# Custom Kernel Compilation for i9-13900HX
# Integrated from https://github.com/wlfogle/i9-13900hx-optimizations

set -e

echo "🚀 Starting custom kernel compilation for i9-13900HX..."
echo "💻 Detected: 24 cores (8 P-cores + 16 E-cores), 32 threads"
echo "⚡ Expected compilation time: 12-25 minutes"

# Create build directory
BUILD_DIR="$HOME/kernel-build"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

# Download latest stable kernel
echo "📥 Downloading latest stable kernel..."
if [ ! -d "linux-stable" ]; then
    git clone https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git linux-stable
fi

cd linux-stable
git fetch
git checkout $(git tag | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1)

# Copy current config as base
if [ -f "/proc/config.gz" ]; then
    zcat /proc/config.gz > .config
elif [ -f "/boot/config-$(uname -r)" ]; then
    cp "/boot/config-$(uname -r)" .config
else
    make defconfig
fi

# Enable i9-13900HX specific optimizations
echo "🔧 Applying i9-13900HX optimizations..."

# CPU optimizations
echo "CONFIG_GENERIC_CPU=n" >> .config
echo "CONFIG_MALDERLAKE=y" >> .config  # Intel 13th gen
echo "CONFIG_X86_INTEL_PSTATE=y" >> .config
echo "CONFIG_CPU_FREQ_DEFAULT_GOV_PERFORMANCE=y" >> .config

# Gaming optimizations
echo "CONFIG_PREEMPT_VOLUNTARY=n" >> .config
echo "CONFIG_PREEMPT=y" >> .config
echo "CONFIG_PREEMPT_COUNT=y" >> .config
echo "CONFIG_PREEMPT_RCU=y" >> .config
echo "CONFIG_HIGH_RES_TIMERS=y" >> .config
echo "CONFIG_NO_HZ_FULL=y" >> .config

# Virtualization support
echo "CONFIG_KVM=y" >> .config
echo "CONFIG_KVM_INTEL=y" >> .config
echo "CONFIG_VFIO=y" >> .config
echo "CONFIG_VFIO_PCI=y" >> .config
echo "CONFIG_VFIO_MDEV=y" >> .config

# AI/ML optimizations
echo "CONFIG_TRANSPARENT_HUGEPAGE=y" >> .config
echo "CONFIG_TRANSPARENT_HUGEPAGE_ALWAYS=y" >> .config
echo "CONFIG_NUMA_BALANCING=y" >> .config
echo "CONFIG_NUMA_BALANCING_DEFAULT_ENABLED=y" >> .config

# Storage optimizations
echo "CONFIG_BLK_DEV_NVME=y" >> .config
echo "CONFIG_NVME_CORE=y" >> .config
echo "CONFIG_BLK_MQ_VIRTIO=y" >> .config
echo "CONFIG_IOSCHED_DEADLINE=y" >> .config
echo "CONFIG_MQ_IOSCHED_DEADLINE=y" >> .config

# Container support
echo "CONFIG_NAMESPACES=y" >> .config
echo "CONFIG_CGROUPS=y" >> .config
echo "CONFIG_DOCKER=y" >> .config

# Security features
echo "CONFIG_SECURITY_APPARMOR=y" >> .config
echo "CONFIG_DEFAULT_SECURITY_APPARMOR=y" >> .config

# Resolve config dependencies
make olddefconfig

# Compile with optimal job count for i9-13900HX
JOBS=$(($(nproc) + 4))  # 24 + 4 = 28 jobs
echo "🔥 Compiling with $JOBS parallel jobs..."

time make -j$JOBS

echo "📦 Installing modules..."
sudo make modules_install

echo "🏗️ Installing kernel..."
sudo make install

echo "🔄 Updating GRUB..."
sudo grub-mkconfig -o /boot/grub/grub.cfg

echo "✅ Custom kernel compilation completed!"
echo "⚠️  Please reboot to use the new kernel."
"#;
        
        // Write and execute the compilation script
        let script_path = "/tmp/compile_i9_kernel.sh";
        fs::write(script_path, compilation_script)?;
        
        // Make executable
        AsyncCommand::new("chmod")
            .arg("+x")
            .arg(script_path)
            .output()
            .await?;
        
        // Execute compilation
        let output = AsyncCommand::new("bash")
            .arg(script_path)
            .output()
            .await?;
        
        if output.status.success() {
            self.kernel_optimizations.custom_kernel_installed = true;
            self.kernel_optimizations.optimizations_enabled = vec![
                "i9-13900HX CPU optimization".to_string(),
                "Gaming performance".to_string(),
                "Virtualization support".to_string(),
                "AI/ML acceleration".to_string(),
                "NVMe storage optimization".to_string(),
                "Container support".to_string(),
                "Security hardening".to_string(),
            ];
            
            Ok("✅ Custom kernel compiled successfully! Reboot required.".to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Kernel compilation failed: {}", error).into())
        }
    }
    
    pub async fn optimize_for_ollama(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        info!("🧠 Optimizing system for Ollama LLM inference...");
        
        // Based on optimize-ollama-system.sh from i9-13900hx-optimizations
        let optimization_script = r#"#!/bin/bash
# Ollama System Optimization for i9-13900HX
# Integrated from https://github.com/wlfogle/i9-13900hx-optimizations

set -e

echo "🧠 Optimizing system for LLM inference on i9-13900HX..."
echo "💾 System RAM: 64GB - Configuring for 70B+ models"

# Memory optimization for large models
TOTAL_MEMORY_GB=64
HUGEPAGE_SIZE_GB=$((TOTAL_MEMORY_GB * 40 / 100))  # 40% of RAM for huge pages

echo "📊 Configuring ${HUGEPAGE_SIZE_GB}GB of huge pages..."

# Configure huge pages
echo $((HUGEPAGE_SIZE_GB * 1024 / 2)) | sudo tee /sys/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages

# Make persistent
echo "vm.nr_hugepages=$((HUGEPAGE_SIZE_GB * 1024 / 2))" | sudo tee -a /etc/sysctl.conf

# CPU optimization for inference
echo "⚡ Setting performance governor for all cores..."
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    if [ -f "$cpu" ]; then
        echo performance | sudo tee "$cpu"
    fi
done

# Disable CPU idle states for consistent performance
echo "🔥 Disabling CPU idle states for consistent latency..."
for cpu in /sys/devices/system/cpu/cpu*/cpuidle/state*/disable; do
    if [ -f "$cpu" ]; then
        echo 1 | sudo tee "$cpu" 2>/dev/null || true
    fi
done

# Optimize kernel parameters for LLM workloads
echo "🔧 Optimizing kernel parameters..."
cat << EOF | sudo tee -a /etc/sysctl.conf
# LLM Inference Optimizations
vm.swappiness=1
vm.dirty_ratio=15
vm.dirty_background_ratio=5
vm.vfs_cache_pressure=50
kernel.numa_balancing=1
kernel.sched_autogroup_enabled=0
kernel.sched_migration_cost_ns=5000000
EOF

# Apply sysctl changes
sudo sysctl -p

# Install and configure Ollama if not present
if ! command -v ollama &> /dev/null; then
    echo "📦 Installing Ollama..."
    curl -fsSL https://ollama.ai/install.sh | sh
fi

# Configure Ollama environment for optimal performance
sudo mkdir -p /etc/systemd/system/ollama.service.d/
cat << EOF | sudo tee /etc/systemd/system/ollama.service.d/override.conf
[Service]
Environment="OLLAMA_HOST=0.0.0.0:11434"
Environment="OLLAMA_NUM_PARALLEL=4"
Environment="OLLAMA_MAX_LOADED_MODELS=3"
Environment="OLLAMA_FLASH_ATTENTION=1"
Environment="OLLAMA_KV_CACHE_TYPE=f16"
LimitNOFILE=1048576
LimitNPROC=1048576
EOF

# Enable and start Ollama service
sudo systemctl daemon-reload
sudo systemctl enable ollama
sudo systemctl start ollama

# Create monitoring script
cat << 'EOF' > "$HOME/monitor-llm-performance.sh"
#!/bin/bash
# LLM Performance Monitor

echo "🖥️  i9-13900HX LLM Performance Monitor"
echo "======================================"

while true; do
    clear
    echo "$(date)"
    echo
    
    # CPU usage and frequency
    echo "🔥 CPU Status:"
    echo "Usage: $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)%"
    echo "Freq:  $(cat /proc/cpuinfo | grep "cpu MHz" | head -1 | awk '{print $4}') MHz"
    
    # Memory usage
    echo
    echo "💾 Memory Status:"
    free -h | grep -E "(Mem|Swap)"
    
    # Huge pages
    echo
    echo "📊 Huge Pages:"
    grep -E "(HugePages_Total|HugePages_Free)" /proc/meminfo
    
    # Temperature
    echo
    echo "🌡️  Temperature:"
    if command -v sensors &> /dev/null; then
        sensors | grep "Package id 0:" || echo "Temp monitoring not available"
    fi
    
    # Ollama status
    echo
    echo "🧠 Ollama Status:"
    if systemctl is-active --quiet ollama; then
        echo "Status: Running"
        echo "Models: $(ollama list 2>/dev/null | wc -l) loaded"
    else
        echo "Status: Not running"
    fi
    
    sleep 2
done
EOF

chmod +x "$HOME/monitor-llm-performance.sh"

# Create model management script
cat << 'EOF' > "$HOME/manage-llm-models.sh"
#!/bin/bash
# LLM Model Management

case "$1" in
    "install-recommended")
        echo "📥 Installing recommended models for 64GB system..."
        ollama pull llama3.1:8b
        ollama pull codellama:13b
        ollama pull mistral:7b
        ;;
    "install-large")
        echo "📥 Installing large models (requires significant RAM)..."
        ollama pull llama3.1:70b
        ;;
    "list-sizes")
        echo "📊 Model size recommendations for 64GB RAM:"
        echo "Safe (multiple models): 7B, 8B, 13B"
        echo "Large (single model): 30B, 70B"
        echo "Maximum: 70B (requires ~40GB RAM)"
        ;;
    "benchmark")
        echo "🚀 Running performance benchmark..."
        echo "Testing llama3.1:8b inference speed..."
        time ollama run llama3.1:8b "Write a short python function"
        ;;
    *)
        echo "Usage: $0 {install-recommended|install-large|list-sizes|benchmark}"
        ;;
esac
EOF

chmod +x "$HOME/manage-llm-models.sh"

echo "✅ Ollama optimization completed!"
echo "🔄 Reboot recommended to ensure all changes take effect"
echo "📊 Run ~/monitor-llm-performance.sh to monitor performance"
echo "📦 Run ~/manage-llm-models.sh install-recommended to install models"
"#;
        
        // Execute optimization script
        let script_path = "/tmp/optimize_ollama.sh";
        fs::write(script_path, optimization_script)?;
        
        AsyncCommand::new("chmod")
            .arg("+x")
            .arg(script_path)
            .output()
            .await?;
        
        let output = AsyncCommand::new("bash")
            .arg(script_path)
            .output()
            .await?;
        
        if output.status.success() {
            self.ollama_config.optimized = true;
            self.ollama_config.huge_pages_gb = 25; // 40% of 64GB
            self.ollama_config.performance_governor_set = true;
            self.ollama_config.monitoring_enabled = true;
            
            Ok("✅ System optimized for Ollama LLM inference!".to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Ollama optimization failed: {}", error).into())
        }
    }
    
    pub async fn set_cpu_governor(&mut self, governor: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("⚡ Setting CPU governor to: {}", governor);
        
        // Validate governor
        let valid_governors = ["performance", "powersave", "ondemand", "conservative", "schedutil"];
        if !valid_governors.contains(&governor) {
            return Err(format!("Invalid governor: {}. Valid options: {:?}", governor, valid_governors).into());
        }
        
        // Set governor for all CPUs (i9-13900HX has 24 cores)
        for cpu_id in 0..24 {
            let governor_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", cpu_id);
            if let Err(e) = fs::write(&governor_path, governor) {
                warn!("Failed to set governor for CPU {}: {}", cpu_id, e);
            }
        }
        
        // Verify the change
        if let Ok(current) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
            self.current_governor = current.trim().to_string();
            if self.current_governor == governor {
                Ok(format!("✅ CPU governor set to: {}", governor))
            } else {
                Err("Failed to verify governor change".into())
            }
        } else {
            Err("Failed to read current governor".into())
        }
    }
    
    pub async fn optimize_for_gaming(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        info!("🎮 Optimizing system for gaming performance...");
        
        // Set performance governor
        self.set_cpu_governor("performance").await?;
        
        // Gaming-specific optimizations
        let gaming_script = r#"#!/bin/bash
# Gaming optimizations for i9-13900HX

echo "🎮 Applying gaming optimizations..."

# Disable CPU idle states for lowest latency
for cpu in /sys/devices/system/cpu/cpu*/cpuidle/state*/disable; do
    if [ -f "$cpu" ]; then
        echo 1 | sudo tee "$cpu" 2>/dev/null || true
    fi
done

# Set I/O scheduler to performance for NVMe drives
for nvme in /sys/block/nvme*/queue/scheduler; do
    if [ -f "$nvme" ]; then
        echo none | sudo tee "$nvme" || echo mq-deadline | sudo tee "$nvme"
    fi
done

# Optimize kernel parameters for gaming
sysctl -w kernel.sched_migration_cost_ns=5000000
sysctl -w kernel.sched_autogroup_enabled=0
sysctl -w vm.dirty_ratio=15
sysctl -w vm.dirty_background_ratio=5

echo "✅ Gaming optimizations applied!"
"#;
        
        let script_path = "/tmp/gaming_optimization.sh";
        fs::write(script_path, gaming_script)?;
        
        AsyncCommand::new("sudo")
            .arg("bash")
            .arg(script_path)
            .output()
            .await?;
        
        self.gaming_mode = true;
        self.performance_profile = PerformanceProfile::Gaming;
        
        Ok("✅ System optimized for gaming performance!".to_string())
    }
    
    pub async fn optimize_for_development(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        info!("💻 Optimizing system for development workload...");
        
        // Balanced performance for development
        self.set_cpu_governor("ondemand").await?;
        
        let dev_script = r#"#!/bin/bash
# Development optimizations

echo "💻 Applying development optimizations..."

# Enable better file watching for development tools
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
echo fs.file-max=2097152 | sudo tee -a /etc/sysctl.conf

# Optimize for compilation workloads
sysctl -w kernel.sched_child_runs_first=1

# I/O optimizations for code compilation
echo mq-deadline | sudo tee /sys/block/nvme*/queue/scheduler

sysctl -p

echo "✅ Development optimizations applied!"
"#;
        
        let script_path = "/tmp/dev_optimization.sh";
        fs::write(script_path, dev_script)?;
        
        AsyncCommand::new("sudo")
            .arg("bash")
            .arg(script_path)
            .output()
            .await?;
        
        self.gaming_mode = false;
        self.performance_profile = PerformanceProfile::Development;
        
        Ok("✅ System optimized for development workload!".to_string())
    }
    
    async fn get_ollama_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        use crate::system::ollama::OllamaManager;
        
        // Use the new dynamic Ollama manager
        let ollama_manager = OllamaManager::new().await?;
        
        // Get models from the manager
        let models = ollama_manager.get_discovered_models()
            .iter()
            .map(|model| model.name.clone())
            .collect();
        
        Ok(models)
    }
    
    pub async fn get_system_status(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut status = HashMap::new();
        
        // CPU governor
        status.insert("cpu_governor".to_string(), self.current_governor.clone());
        
        // Kernel version
        status.insert("kernel_version".to_string(), self.kernel_optimizations.kernel_version.clone());
        status.insert("custom_kernel".to_string(), self.kernel_optimizations.custom_kernel_installed.to_string());
        
        // Performance profile
        status.insert("performance_profile".to_string(), format!("{:?}", self.performance_profile));
        
        // Gaming mode
        status.insert("gaming_mode".to_string(), self.gaming_mode.to_string());
        
        // Ollama configuration
        status.insert("ollama_optimized".to_string(), self.ollama_config.optimized.to_string());
        status.insert("ollama_huge_pages_gb".to_string(), self.ollama_config.huge_pages_gb.to_string());
        status.insert("ollama_models_count".to_string(), self.ollama_config.models_installed.len().to_string());
        
        // Huge pages
        status.insert("huge_pages_enabled".to_string(), self.huge_pages_enabled.to_string());
        
        // Virtualization
        status.insert("virtualization_enabled".to_string(), self.virtualization_enabled.to_string());
        
        Ok(status)
    }
    
    pub async fn emergency_cooling(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        warn!("🚨 Emergency cooling activated!");
        
        // Set powersave governor to reduce heat
        self.set_cpu_governor("powersave").await?;
        
        // Enable all CPU idle states
        let cooling_script = r#"#!/bin/bash
# Emergency cooling for i9-13900HX

echo "🚨 Activating emergency cooling..."

# Enable deep CPU idle states
for cpu in /sys/devices/system/cpu/cpu*/cpuidle/state*/disable; do
    if [ -f "$cpu" ]; then
        echo 0 | sudo tee "$cpu" 2>/dev/null || true
    fi
done

# Reduce CPU max frequency temporarily
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq; do
    if [ -f "$cpu" ]; then
        echo 3000000 | sudo tee "$cpu" # Limit to 3GHz
    fi
done

echo "✅ Emergency cooling measures applied!"
"#;
        
        let script_path = "/tmp/emergency_cooling.sh";
        fs::write(script_path, cooling_script)?;
        
        AsyncCommand::new("sudo")
            .arg("bash")
            .arg(script_path)
            .output()
            .await?;
        
        Ok("🚨 Emergency cooling activated - CPU frequency limited to 3GHz".to_string())
    }
}

<citations>
<document>
<document_type>WEB_PAGE</document_type>
<document_id>https://github.com/wlfogle/i9-13900hx-optimizations</document_id>
</document>
</citations>
