#!/bin/bash

# Store initial list of serial devices
initial_devices=$(ls /dev/ttyUSB* /dev/ttyACM* 2>/dev/null)

echo "Now monitoring for new devices. Reset your controller now" >&2

# Loop until a new device appears
while true; do
    sleep 1
    current_devices=$(ls /dev/ttyUSB* /dev/ttyACM* 2>/dev/null)

    # Compare current list with initial list
    for dev in $current_devices; do
        if ! echo "$initial_devices" | grep -q "^$dev$"; then
            NEW_DEVICE="$dev"
            echo "$NEW_DEVICE"
            echo "Detected $NEW_DEVICE, waiting for it to becom writeable" >&2
            exit 0
        fi
    done

    # Update initial_devices to include disconnected then reconnected devices
    initial_devices=$current_devices

done
