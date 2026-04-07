#!/bin/bash

# Set your Bluetooth device MAC address
DEVICE_MAC="" ########## ADD your speaker device's MAC address

# Attempt connection
echo -e "connect $DEVICE_MAC\nquit" | bluetoothctl > null
# Add a short delay to give time for connection attempt
sleep 1
# Check if the device is connected
if echo -e "info $DEVICE_MAC\nquit" | bluetoothctl | grep -q "Connected: yes"; then
    echo "Connection successful"
    exit 0
fi

# Set the maximum wait time (in seconds) for reconnection
MAX_WAIT=120
WAIT_INTERVAL=2
TOTAL_WAIT=0

systemctl --user restart pulseaudio
paplay /usr/share/sounds/Pop/stereo/notification/message-new-instant.oga

# Power on Bluetooth
echo "Press Bluetooth button on device..."
bluetoothctl power on

# Disconnect and remove the device
echo "Attempting to disconnect and remove device: $DEVICE_MAC"
bluetoothctl disconnect $DEVICE_MAC
bluetoothctl remove $DEVICE_MAC

# Adding a delay to allow the device to reset
echo "Waiting for a moment before attempting to reconnect..."
sleep 1

echo "Device removed successfully."

# Start scanning for devices
bluetoothctl --timeout $MAX_WAIT scan on &

# Wait until the device is found and paired
while [ $TOTAL_WAIT -lt $MAX_WAIT ]; do
    if echo "devices" | bluetoothctl | grep -q "$DEVICE_MAC"; then
        echo "Device found: $DEVICE_MAC"

        # Trust the device before connecting
        echo "Trusting device: $DEVICE_MAC"
        bluetoothctl trust $DEVICE_MAC

        # Connect to the device
        echo "Connecting to device: $DEVICE_MAC"
        if bluetoothctl connect $DEVICE_MAC | grep -q "Connection successful"; then
            exit 0
        else
            echo "Connection failed."
        fi
    fi

    # Wait for a while before the next attempt
    sleep $WAIT_INTERVAL
    TOTAL_WAIT=$((TOTAL_WAIT + WAIT_INTERVAL))
done

echo "Failed to reconnect to device after $MAX_WAIT seconds"
exit 1
