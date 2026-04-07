#!/usr/bin/env bash

if [ "${1}" == "pre" ]; then
	echo "nothing here..."
elif [ "${1}" == "post" ]; then
	sleep 1s
	bluetoothctl connect #REPLACE THIS COMMENTED TXT WITH YOUR BLUETOOTH SPEAKER MAC ADDRESS
fi
