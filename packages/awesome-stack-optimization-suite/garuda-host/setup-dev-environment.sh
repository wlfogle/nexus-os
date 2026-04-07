#!/bin/bash

# AI/ML Development Environment Setup Script

set -e

echo "Setting up AI/ML Development Environment..."

# Update system
pacman -Syu --noconfirm

# Install essential development tools
pacman -S --needed --noconfirm \
    base-devel \
    git \
    curl \
    wget \
    unzip \
    htop \
    neofetch \
    tree \
    vim \
    nano \
    tmux \
    screen \
    openssh \
    rsync \
    cmake \
    make \
    gcc \
    clang \
    llvm \
    ninja \
    meson \
    pkg-config

# Install Python and ML libraries
pacman -S --needed --noconfirm \
    python \
    python-pip \
    python-pipenv \
    python-virtualenv \
    python-numpy \
    python-scipy \
    python-matplotlib \
    python-pandas \
    python-scikit-learn \
    python-jupyter \
    python-ipython \
    python-notebook \
    python-pytest

# Install Rust
pacman -S --needed --noconfirm \
    rustup \
    rust-analyzer

# Configure Rust
sudo -u $(logname) rustup default stable
sudo -u $(logname) rustup component add rls rust-analysis rust-src

# Install Node.js and npm
pacman -S --needed --noconfirm \
    nodejs \
    npm \
    yarn

# Install Java (for Android development)
pacman -S --needed --noconfirm \
    jdk17-openjdk \
    gradle \
    maven

# Install Docker and containerization tools
pacman -S --needed --noconfirm \
    docker \
    docker-compose \
    podman \
    buildah \
    skopeo

# Install KVM/QEMU and virtualization
pacman -S --needed --noconfirm \
    qemu-full \
    libvirt \
    virt-manager \
    virt-viewer \
    dnsmasq \
    vde2 \
    bridge-utils \
    openbsd-netcat \
    ebtables \
    dmidecode

# Install VirtualBox
pacman -S --needed --noconfirm \
    virtualbox \
    virtualbox-host-modules-arch \
    virtualbox-guest-iso

# Install NVIDIA drivers and CUDA (if not already installed)
if lspci | grep -i nvidia > /dev/null; then
    pacman -S --needed --noconfirm \
        nvidia \
        nvidia-utils \
        cuda \
        cudnn \
        nvidia-container-toolkit
fi

# Install monitoring and profiling tools
pacman -S --needed --noconfirm \
    htop \
    iotop \
    nethogs \
    iftop \
    ncdu \
    lsof \
    strace \
    perf \
    valgrind \
    gdb \
    sysstat

# Install compression tools
pacman -S --needed --noconfirm \
    zip \
    unzip \
    tar \
    gzip \
    bzip2 \
    xz \
    zstd \
    p7zip

# Install network tools
pacman -S --needed --noconfirm \
    nmap \
    wireshark-cli \
    tcpdump \
    netcat \
    socat \
    curl \
    wget \
    aria2

# Install database tools
pacman -S --needed --noconfirm \
    postgresql \
    mysql \
    sqlite \
    redis \
    mongodb

# Install Android development tools
if [ ! -d "/opt/android-sdk" ]; then
    mkdir -p /opt/android-sdk
    cd /opt/android-sdk
    wget https://dl.google.com/android/repository/commandlinetools-linux-latest.zip
    unzip commandlinetools-latest.zip
    rm commandlinetools-latest.zip
    chown -R $(logname):$(logname) /opt/android-sdk
fi

# Enable and start services
systemctl enable docker
systemctl enable libvirtd
systemctl enable cpu-performance-optimization

# Add user to required groups
usermod -aG docker,kvm,libvirt $(logname)

# Install Python ML packages
pip install --upgrade pip
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu118
pip install tensorflow-gpu
pip install transformers
pip install datasets
pip install accelerate
pip install bitsandbytes
pip install xformers
pip install flash-attn

# Install Tauri CLI
cargo install tauri-cli

# Install additional Node.js tools
npm install -g @tauri-apps/cli
npm install -g typescript
npm install -g ts-node
npm install -g nodemon
npm install -g pm2

echo "Development environment setup complete!"
echo "Please reboot your system to apply all optimizations."
