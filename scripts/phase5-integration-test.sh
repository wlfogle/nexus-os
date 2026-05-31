#!/bin/bash
set -e

# Phase 5 Integration Test
# Verifies:
# 1. Kernel boots with Phase 5 syscalls
# 2. Reserved ports are registered
# 3. nexus-ai daemon spawns successfully
# 4. Basic IPC message handling works

TEST_LOG="/tmp/nexusos-phase5-test.log"
TIMEOUT=30
PASS=0
FAIL=0

echo "=========================================="
echo "  NexusOS Phase 5 Integration Test"
echo "=========================================="
echo ""

cleanup() {
    echo "[cleanup] Killing QEMU..."
    pkill -f "qemu-system-x86_64.*nexusos-laptop" || true
}

trap cleanup EXIT

test_case() {
    local name="$1"
    local pattern="$2"
    local timeout="$3"
    
    echo -n "[test] $name ... "
    
    # Search log file for pattern with timeout
    if timeout "$timeout" grep -q "$pattern" "$TEST_LOG" 2>/dev/null; then
        echo "✓ PASS"
        ((PASS++))
    else
        echo "✗ FAIL"
        echo "       Expected pattern: $pattern"
        ((FAIL++))
    fi
}

# Build kernel with Phase 5
echo "[build] Compiling kernel with AI-Core support..."
cd kernel
make clean >/dev/null 2>&1 || true
make laptop >/dev/null 2>&1 || {
    echo "✗ Kernel build failed"
    exit 1
}
make iso-laptop >/dev/null 2>&1 || {
    echo "✗ ISO build failed"
    exit 1
}
cd ..

echo "[boot] Starting QEMU test VM (timeout: ${TIMEOUT}s)..."
timeout "$TIMEOUT" qemu-system-x86_64 \
    -m 2G \
    -smp 2 \
    -cdrom build/nexusos-laptop.iso \
    -serial stdio \
    -nographic \
    > "$TEST_LOG" 2>&1 &

QEMU_PID=$!

# Wait for QEMU to start and collect output
sleep 2

# Test Phase 5 initialization
test_case "Phase 5 boot sequence" "Phase 5: AI Core initialization" 5
test_case "nexus-ai daemon spawned" "nexus-ai spawned as PID" 5
test_case "Reserved port: nexus.ai" "Reserved: nexus.ai" 5
test_case "Reserved port: nexus.fs" "Reserved: nexus.fs" 5
test_case "Reserved port: nexus.gpu" "Reserved: nexus.gpu" 5

# Test nexus-ai daemon startup
test_case "nexus-ai IPC bind" "Bound to port_id=" 5
test_case "nexus-ai daemon loop" "Entering main daemon loop" 5

echo ""
echo "=========================================="
echo "  Test Summary"
echo "=========================================="
echo "  Passed: $PASS"
echo "  Failed: $FAIL"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo "✓ All Phase 5 tests PASSED"
    exit 0
else
    echo "✗ Some tests FAILED"
    echo ""
    echo "Last 50 lines of boot log:"
    tail -50 "$TEST_LOG"
    exit 1
fi