#!/usr/bin/env bash
# =============================================================================
# NexusOS Ollama AI Setup
# Installs Ollama with NVIDIA GPU acceleration, pulls a default model,
# creates a systemd service, and verifies CUDA inference.
# =============================================================================
set -euo pipefail

readonly LOG_DIR="/var/log/nexus-os"
readonly LOG_FILE="${LOG_DIR}/ollama-setup.log"
readonly OLLAMA_BIN="/usr/local/bin/ollama"
readonly OLLAMA_SERVICE="/etc/systemd/system/ollama.service"
readonly OLLAMA_USER="ollama"
readonly OLLAMA_HOME="/opt/nexus-os/models"
readonly DEFAULT_MODEL="llama3.2"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log() { printf "[%s] %s\n" "$(date '+%Y-%m-%d %H:%M:%S')" "$*" | tee -a "$LOG_FILE"; }
warn() { log "WARN: $*"; }
die() { log "FATAL: $*"; exit 1; }

need_root() {
    [[ $EUID -eq 0 ]] || die "This script must be run as root (sudo $0)"
}

# ---------------------------------------------------------------------------
# GPU Detection
# ---------------------------------------------------------------------------
detect_gpu() {
    log "=== Detecting GPU ==="

    local gpu_info=""
    local cuda_ok=false

    # Check for NVIDIA GPU
    if command -v nvidia-smi &>/dev/null; then
        gpu_info="$(nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader 2>/dev/null || true)"
        if [[ -n "$gpu_info" ]]; then
            log "NVIDIA GPU detected: $gpu_info"
        fi
    else
        warn "nvidia-smi not found — NVIDIA drivers may not be installed"
    fi

    # Check CUDA toolkit
    if command -v nvcc &>/dev/null; then
        local cuda_ver
        cuda_ver="$(nvcc --version 2>/dev/null | grep -oP 'release \K[0-9.]+' || true)"
        if [[ -n "$cuda_ver" ]]; then
            log "CUDA toolkit version: $cuda_ver"
            cuda_ok=true
        fi
    fi

    # Check CUDA libraries
    if ldconfig -p 2>/dev/null | grep -q libcuda; then
        log "CUDA runtime libraries found"
        cuda_ok=true
    fi

    if [[ "$cuda_ok" == false ]]; then
        warn "CUDA not detected — Ollama will run on CPU only"
        warn "For GPU acceleration, install NVIDIA CUDA toolkit:"
        warn "  sudo nala install nvidia-cuda-toolkit"
    fi

    # Check available VRAM
    if command -v nvidia-smi &>/dev/null; then
        local vram_mb
        vram_mb="$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>/dev/null | head -1 || true)"
        if [[ -n "$vram_mb" ]]; then
            log "GPU VRAM: ${vram_mb}MB"
            if (( vram_mb < 4096 )); then
                warn "Low VRAM (${vram_mb}MB) — smaller models recommended"
            fi
        fi
    fi
}

# ---------------------------------------------------------------------------
# Install Ollama
# ---------------------------------------------------------------------------
install_ollama() {
    log "=== Installing Ollama ==="

    if [[ -x "$OLLAMA_BIN" ]]; then
        local ver
        ver="$("$OLLAMA_BIN" --version 2>/dev/null || echo 'unknown')"
        log "Ollama already installed: $ver"
        log "Updating to latest..."
    fi

    # Use official installer script (handles GPU detection internally)
    if ! curl -fsSL https://ollama.com/install.sh | sh; then
        die "Ollama installation failed"
    fi

    # Verify binary
    if [[ ! -x "$OLLAMA_BIN" ]]; then
        # Check alternate location
        if command -v ollama &>/dev/null; then
            log "Ollama installed at: $(command -v ollama)"
        else
            die "Ollama binary not found after install"
        fi
    fi

    local ver
    ver="$(ollama --version 2>/dev/null || echo 'installed')"
    log "Ollama installed successfully: $ver"
}

# ---------------------------------------------------------------------------
# Create Ollama User & Directories
# ---------------------------------------------------------------------------
setup_ollama_user() {
    log "=== Setting up Ollama user and directories ==="

    # Create dedicated user if it doesn't exist
    if ! id "$OLLAMA_USER" &>/dev/null; then
        useradd -r -s /usr/sbin/nologin -d "$OLLAMA_HOME" -m "$OLLAMA_USER"
        log "Created user: $OLLAMA_USER"
    fi

    # Create model storage directory
    mkdir -p "$OLLAMA_HOME"
    chown -R "${OLLAMA_USER}:${OLLAMA_USER}" "$OLLAMA_HOME"

    # Add ollama user to video and render groups for GPU access
    usermod -aG video "$OLLAMA_USER" 2>/dev/null || true
    usermod -aG render "$OLLAMA_USER" 2>/dev/null || true

    log "Ollama directories ready: $OLLAMA_HOME"
}

# ---------------------------------------------------------------------------
# Systemd Service
# ---------------------------------------------------------------------------
setup_systemd() {
    log "=== Creating Ollama systemd service ==="

    # The official installer may already create a service.
    # We create our own with NexusOS-specific settings if it doesn't exist,
    # or if we want to override model storage location.

    cat > "$OLLAMA_SERVICE" << UNIT_EOF
[Unit]
Description=Ollama LLM Service (NexusOS)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${OLLAMA_USER}
Group=${OLLAMA_USER}
Environment="OLLAMA_MODELS=${OLLAMA_HOME}"
Environment="OLLAMA_HOST=127.0.0.1:11434"
ExecStart=/usr/local/bin/ollama serve
Restart=always
RestartSec=5
LimitNOFILE=65536
LimitMEMLOCK=infinity

# Security hardening
NoNewPrivileges=false
ProtectSystem=full
ProtectHome=true
ReadWritePaths=${OLLAMA_HOME} /tmp

# GPU access
SupplementaryGroups=video render

[Install]
WantedBy=multi-user.target
UNIT_EOF

    systemctl daemon-reload
    systemctl enable ollama
    systemctl restart ollama

    # Wait for service to start
    local retries=0
    while (( retries < 15 )); do
        if curl -sf http://127.0.0.1:11434/api/tags &>/dev/null; then
            log "Ollama service running on 127.0.0.1:11434"
            return 0
        fi
        sleep 1
        (( retries++ ))
    done

    warn "Ollama service started but API not yet responding — may need a moment"
}

# ---------------------------------------------------------------------------
# Pull Default Model
# ---------------------------------------------------------------------------
pull_model() {
    log "=== Pulling default model: ${DEFAULT_MODEL} ==="

    # Check if already pulled
    if ollama list 2>/dev/null | grep -q "$DEFAULT_MODEL"; then
        log "Model $DEFAULT_MODEL already available"
        return 0
    fi

    log "Downloading $DEFAULT_MODEL (this may take several minutes)..."
    if ollama pull "$DEFAULT_MODEL" 2>&1 | tee -a "$LOG_FILE"; then
        log "Model $DEFAULT_MODEL pulled successfully"
    else
        warn "Failed to pull $DEFAULT_MODEL — you can pull manually: ollama pull $DEFAULT_MODEL"
        return 1
    fi
}

# ---------------------------------------------------------------------------
# Verify GPU Inference
# ---------------------------------------------------------------------------
verify_cuda() {
    log "=== Verifying GPU Inference ==="

    # Check if Ollama reports GPU usage
    local ps_output
    ps_output="$(ollama ps 2>/dev/null || true)"
    log "Ollama process status: $ps_output"

    # Quick inference test
    log "Running inference test..."
    local response
    response="$(curl -sf --max-time 30 http://127.0.0.1:11434/api/generate \
        -d "{\"model\": \"${DEFAULT_MODEL}\", \"prompt\": \"Say hello in exactly 5 words.\", \"stream\": false}" \
        2>/dev/null || true)"

    if [[ -n "$response" ]]; then
        log "Inference test passed — model responded"
        # Check if GPU was used
        if command -v nvidia-smi &>/dev/null; then
            local gpu_util
            gpu_util="$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader 2>/dev/null | head -1 || true)"
            if [[ -n "$gpu_util" ]]; then
                log "GPU utilization after inference: $gpu_util"
            fi
        fi
    else
        warn "Inference test did not get a response — model may still be loading"
        warn "Try manually: ollama run $DEFAULT_MODEL"
    fi
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print_summary() {
    log ""
    log "============================================"
    log "  NexusOS Ollama AI Setup Complete"
    log "============================================"
    log "  Ollama Binary:   $(command -v ollama 2>/dev/null || echo 'not found')"
    log "  Service:         $(systemctl is-active ollama 2>/dev/null || echo 'unknown')"
    log "  API Endpoint:    http://127.0.0.1:11434"
    log "  Model Storage:   $OLLAMA_HOME"
    log "  Default Model:   $DEFAULT_MODEL"
    log ""
    log "  Usage:"
    log "    ollama run $DEFAULT_MODEL          # Chat interactively"
    log "    ollama list                        # List models"
    log "    ollama pull mistral                # Pull another model"
    log "    curl http://localhost:11434/api/tags  # API model list"
    log ""
    log "  Log: $LOG_FILE"
    log "============================================"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    need_root
    mkdir -p "$LOG_DIR"

    log "Starting NexusOS Ollama AI setup..."

    detect_gpu
    install_ollama
    setup_ollama_user
    setup_systemd
    pull_model
    verify_cuda
    print_summary
}

main "$@"
