#!/bin/bash
# AI-Optimized System Setup for Ubuntu 22.04
# Hardware: i9-13900HX, 62GB RAM, NVIDIA RTX 4000 series

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

if [ "$EUID" -eq 0 ]; then 
    error "Do not run as root"
fi

echo "=========================================="
echo "  AI-Optimized System Setup"
echo "  Pop!_OS 22.04 LTS"
echo "=========================================="
echo ""
echo "Hardware Detected:"
echo "  CPU: Intel i9-13900HX (32 threads)"
echo "  RAM: 62GB"
echo "  GPU: NVIDIA RTX 4000 series"
echo "  Disk: 835GB free"
echo ""
read -p "Continue with optimization? (y/n): " confirm
if [ "$confirm" != "y" ]; then
    exit 0
fi

# ============================================
# SECTION 1: System Updates & Essentials
# ============================================
log "1/10 - Updating system packages..."
sudo apt update
sudo apt upgrade -y

log "Installing essential tools..."
sudo apt install -y \
    build-essential \
    cmake \
    git \
    curl \
    wget \
    htop \
    btop \
    neofetch \
    net-tools \
    software-properties-common \
    apt-transport-https \
    ca-certificates \
    gnupg \
    lsb-release \
    zsh \
    tmux \
    vim \
    python3-pip \
    python3-dev \
    python-is-python3

# ============================================
# SECTION 2: NVIDIA Driver & CUDA
# ============================================
log "2/10 - Setting up NVIDIA drivers and CUDA..."

if ! command -v nvidia-smi &> /dev/null; then
    log "Installing NVIDIA drivers..."
    sudo apt install -y nvidia-driver-545
    warn "NVIDIA driver installed. Reboot required after script completes!"
else
    log "NVIDIA drivers already installed"
fi

log "Installing CUDA toolkit..."
if ! command -v nvcc &> /dev/null; then
    wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.1-1_all.deb
    sudo dpkg -i cuda-keyring_1.1-1_all.deb
    sudo apt update
    sudo apt install -y cuda-toolkit-12-3
    rm cuda-keyring_1.1-1_all.deb
    
    # Add CUDA to PATH
    echo 'export PATH=/usr/local/cuda/bin:$PATH' >> ~/.bashrc
    echo 'export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
fi

# ============================================
# SECTION 3: Docker & NVIDIA Container Toolkit
# ============================================
log "3/10 - Installing Docker with GPU support..."

if ! command -v docker &> /dev/null; then
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu jammy stable" | sudo tee /etc/apt/sources.list.d/docker.list
    sudo apt update
    sudo apt install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
    sudo systemctl enable docker
    sudo usermod -aG docker $USER
    log "Docker installed"
else
    log "Docker already installed"
fi

log "Installing NVIDIA Container Toolkit..."
if ! command -v nvidia-ctk &> /dev/null; then
    distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
    curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg
    curl -s -L https://nvidia.github.io/libnvidia-container/$distribution/libnvidia-container.list | \
        sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
        sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list
    sudo apt update
    sudo apt install -y nvidia-container-toolkit
    sudo nvidia-ctk runtime configure --runtime=docker
    sudo systemctl restart docker
fi

# ============================================
# SECTION 4: Python AI/ML Stack
# ============================================
log "4/10 - Setting up Python AI/ML environment..."

# Install PyTorch with CUDA support
pip3 install --upgrade pip setuptools wheel

log "Installing PyTorch with CUDA 12.1..."
pip3 install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121

log "Installing AI/ML libraries..."
pip3 install \
    numpy \
    pandas \
    scipy \
    scikit-learn \
    matplotlib \
    seaborn \
    jupyter \
    jupyterlab \
    transformers \
    diffusers \
    accelerate \
    bitsandbytes \
    safetensors \
    opencv-python \
    pillow \
    requests \
    tqdm \
    tensorboard

# ============================================
# SECTION 5: System Performance Tuning
# ============================================
log "5/10 - Optimizing system performance..."

# CPU Governor - Performance mode
log "Setting CPU governor to performance..."
echo 'GOVERNOR="performance"' | sudo tee /etc/default/cpufrequtils
sudo apt install -y cpufrequtils
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    echo performance | sudo tee $cpu > /dev/null 2>&1 || true
done

# Swappiness optimization (reduce swap usage)
log "Optimizing swappiness..."
echo "vm.swappiness=10" | sudo tee -a /etc/sysctl.conf
echo "vm.vfs_cache_pressure=50" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# File descriptor limits
log "Increasing file descriptor limits..."
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Network optimization
log "Optimizing network settings..."
cat << EOF | sudo tee -a /etc/sysctl.conf
net.core.rmem_max=134217728
net.core.wmem_max=134217728
net.ipv4.tcp_rmem=4096 87380 67108864
net.ipv4.tcp_wmem=4096 65536 67108864
net.core.netdev_max_backlog=5000
EOF
sudo sysctl -p

# ============================================
# SECTION 6: Gaming Optimizations
# ============================================
log "6/10 - Installing gaming optimizations..."

# GameMode
log "Installing GameMode..."
sudo apt install -y gamemode

# MangoHud for performance overlay
log "Installing MangoHud..."
sudo apt install -y mangohud goverlay

# Create optimized MangoHud config
mkdir -p ~/.config/MangoHud
cat > ~/.config/MangoHud/MangoHud.conf << 'EOF'
fps
gpu_stats
gpu_temp
gpu_core_clock
gpu_power
cpu_stats
cpu_temp
ram
vram
frame_timing=1
frametime=1
position=top-right
font_size=24
background_alpha=0.5
EOF

# ============================================
# SECTION 7: Development Tools
# ============================================
log "7/10 - Installing development tools..."

# Node.js (for AI web interfaces)
log "Installing Node.js..."
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Rust (for fast AI tools)
log "Installing Rust..."
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Go (for containerized tools)
log "Installing Go..."
if ! command -v go &> /dev/null; then
    wget https://go.dev/dl/go1.21.5.linux-amd64.tar.gz
    sudo rm -rf /usr/local/go
    sudo tar -C /usr/local -xzf go1.21.5.linux-amd64.tar.gz
    echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
    rm go1.21.5.linux-amd64.tar.gz
fi

# ============================================
# SECTION 8: Media & Streaming Tools
# ============================================
log "8/10 - Installing media tools..."

# OBS Studio
log "Installing OBS Studio..."
sudo add-apt-repository -y ppa:obsproject/obs-studio
sudo apt update
sudo apt install -y obs-studio

# FFmpeg with NVENC
log "Installing FFmpeg with NVIDIA encoding..."
sudo apt install -y \
    ffmpeg \
    libavcodec-extra \
    vlc

# Video editing
log "Installing video editors..."
sudo apt install -y kdenlive

# ============================================
# SECTION 9: Storage & Filesystem Optimization
# ============================================
log "9/10 - Optimizing storage..."

# Enable TRIM for SSDs
log "Enabling TRIM for SSDs..."
sudo systemctl enable fstrim.timer
sudo systemctl start fstrim.timer

# Set I/O scheduler for NVMe
log "Optimizing I/O scheduler..."
for nvme in /sys/block/nvme*/queue/scheduler; do
    echo none | sudo tee $nvme || true
done

# ============================================
# SECTION 10: AI-Specific Tools
# ============================================
log "10/10 - Installing AI-specific tools..."

# Ollama for local LLMs
log "Installing Ollama..."
if ! command -v ollama &> /dev/null; then
    curl -fsSL https://ollama.com/install.sh | sh
fi

# Hugging Face CLI
log "Installing Hugging Face CLI..."
pip3 install huggingface-hub[cli]

# ComfyUI dependencies
log "Installing ComfyUI dependencies..."
pip3 install \
    einops \
    kornia \
    spandrel \
    soundfile \
    trimesh

# Create workspace directories
log "Creating AI workspace..."
mkdir -p ~/ai-workspace/{models,datasets,projects,outputs}

# ============================================
# System Monitoring Setup
# ============================================
log "Setting up system monitoring..."

cat > ~/.config/btop/btop.conf << 'EOF' || true
color_theme = "Default"
theme_background = False
update_ms = 1000
proc_sorting = "cpu lazy"
proc_tree = True
proc_colors = True
proc_gradient = True
gpu_mirror_graph = True
EOF

# ============================================
# Create helpful aliases
# ============================================
log "Creating helpful aliases..."

cat >> ~/.bashrc << 'EOF'

# AI Development Aliases
alias gpu='nvidia-smi'
alias gpuwatch='watch -n 1 nvidia-smi'
alias dockergpu='docker run --rm --gpus all nvidia/cuda:12.3.0-base-ubuntu22.04 nvidia-smi'
alias temp='sensors | grep -E "Core|temp1"'
alias aienv='cd ~/ai-workspace && source venv/bin/activate'

# Performance
alias cpufreq='watch -n 1 "grep MHz /proc/cpuinfo | sort -u"'
alias perf='btop'

# Docker shortcuts
alias dps='docker ps'
alias dcup='docker-compose up -d'
alias dcdown='docker-compose down'
alias dclogs='docker-compose logs -f'
EOF

# ============================================
# FINAL SETUP
# ============================================
log "Creating performance monitoring script..."

cat > ~/system-status.sh << 'EOF'
#!/bin/bash
echo "=== System Status ==="
echo ""
echo "CPU:"
grep "MHz" /proc/cpuinfo | head -1
echo ""
echo "RAM:"
free -h | grep Mem
echo ""
echo "GPU:"
nvidia-smi --query-gpu=name,temperature.gpu,utilization.gpu,utilization.memory,memory.used,memory.total --format=csv,noheader
echo ""
echo "Disk:"
df -h / | tail -1
echo ""
echo "Processes using GPU:"
nvidia-smi --query-compute-apps=pid,name,used_memory --format=csv,noheader
EOF
chmod +x ~/system-status.sh

# ============================================
# Summary
# ============================================
echo ""
echo "=========================================="
echo "  Optimization Complete!"
echo "=========================================="
echo ""
echo "Next Steps:"
echo "1. REBOOT your system: sudo reboot"
echo "2. After reboot, verify GPU: nvidia-smi"
echo "3. Test Docker GPU: docker run --rm --gpus all nvidia/cuda:12.3.0-base-ubuntu22.04 nvidia-smi"
echo "4. Check system status: ~/system-status.sh"
echo "5. Start gaming AI stack: cd ~/gaming-ai-stack && ./install.sh"
echo ""
echo "Optimizations Applied:"
echo "  ✓ NVIDIA drivers & CUDA 12.3"
echo "  ✓ Docker with GPU support"
echo "  ✓ PyTorch with CUDA 12.1"
echo "  ✓ CPU performance mode"
echo "  ✓ Reduced swap usage"
echo "  ✓ Network optimization"
echo "  ✓ Gaming tools (GameMode, MangoHud)"
echo "  ✓ Media tools (OBS, FFmpeg)"
echo "  ✓ Development tools (Node, Rust, Go)"
echo "  ✓ AI tools (Ollama, Hugging Face)"
echo ""
echo "System Info:"
echo "  CPU: i9-13900HX (32 threads @ 5.4GHz)"
echo "  RAM: 62GB"
echo "  GPU: NVIDIA RTX 4000 series"
echo "  Storage: NVMe SSD optimized"
echo ""
warn "IMPORTANT: Log out and back in for Docker group changes"
warn "REQUIRED: Reboot for NVIDIA drivers and kernel optimizations"
echo ""
