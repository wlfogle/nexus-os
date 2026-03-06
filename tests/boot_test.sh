#!/bin/bash
# Boot test script - verifies kernel boots and produces expected serial output

set -e

KERNEL_BIN="${1:-./build/kernel.bin}"
TIMEOUT="${2:-5}"

if [ ! -f "$KERNEL_BIN" ]; then
    echo "ERROR: Kernel binary not found at $KERNEL_BIN"
    exit 1
fi

echo "=== NexusOS Boot Test ==="
echo "Kernel: $KERNEL_BIN"
echo "Timeout: ${TIMEOUT}s"
echo

# Run QEMU with kernel and serial output to a file
SERIAL_OUTPUT="/tmp/nexus_serial.log"
rm -f "$SERIAL_OUTPUT"

echo "Booting with QEMU..."
timeout $TIMEOUT qemu-system-i386 \
    -kernel "$KERNEL_BIN" \
    -serial file:"$SERIAL_OUTPUT" \
    -display none \
    -m 128M \
    2>&1 || true

# Read serial output
if [ ! -f "$SERIAL_OUTPUT" ]; then
    echo "ERROR: Serial output file not created"
    exit 1
fi
OUTPUT=$(cat "$SERIAL_OUTPUT")

echo "Serial Output:"
echo "---"
echo "$OUTPUT"
echo "---"
echo

# Check for expected boot messages
EXPECTED_MESSAGES=(
    "NexusOS Boot Sequence Started"
    "Multiboot validation: OK"
    "Kernel initialization complete"
)

FAILED=0
for msg in "${EXPECTED_MESSAGES[@]}"; do
    if echo "$OUTPUT" | grep -q "$msg"; then
        echo "✓ Found: $msg"
    else
        echo "✗ Missing: $msg"
        FAILED=1
    fi
done

echo

if [ $FAILED -eq 0 ]; then
    echo "✓ Boot test PASSED"
    exit 0
else
    echo "✗ Boot test FAILED"
    exit 1
fi
