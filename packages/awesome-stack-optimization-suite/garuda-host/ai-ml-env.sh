#!/bin/bash

# AI/ML Development Environment Optimizations

# CUDA optimizations
export CUDA_CACHE_MAXSIZE=2147483648
export CUDA_CACHE_PATH=/tmp/cuda-cache
export CUDA_DEVICE_ORDER=PCI_BUS_ID
export CUDA_VISIBLE_DEVICES=0

# PyTorch optimizations
export PYTORCH_CUDA_ALLOC_CONF=max_split_size_mb:512,expandable_segments:True
export OMP_NUM_THREADS=24
export MKL_NUM_THREADS=24
export OPENBLAS_NUM_THREADS=24
export NUMBA_NUM_THREADS=24

# TensorFlow optimizations
export TF_GPU_ALLOCATOR=cuda_malloc_async
export TF_ENABLE_ONEDNN_OPTS=1
export TF_CPP_MIN_LOG_LEVEL=2

# Rust optimizations
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
export CARGO_TARGET_DIR=/tmp/rust-target
export RUST_BACKTRACE=1

# Node.js/Tauri optimizations
export NODE_OPTIONS="--max-old-space-size=32768 --max-semi-space-size=256"
export UV_THREADPOOL_SIZE=32

# Android development
export ANDROID_HOME=/opt/android-sdk
export JAVA_OPTS="-Xmx32g -XX:+UseG1GC -XX:MaxGCPauseMillis=200"

# Docker optimizations
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# Development tools
export MAKEFLAGS="-j24"
export CMAKE_BUILD_PARALLEL_LEVEL=24

# Memory optimizations for large datasets
export MALLOC_ARENA_MAX=4
export MALLOC_TRIM_THRESHOLD_=131072
export MALLOC_TOP_PAD_=131072

# GPU memory management
export NVIDIA_DRIVER_CAPABILITIES=compute,utility,video
export NVIDIA_REQUIRE_CUDA="cuda>=11.0"

mkdir -p /tmp/cuda-cache
mkdir -p /tmp/rust-target
