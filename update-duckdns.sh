#!/bin/bash

# DuckDNS Configuration
DOMAIN="lou-fogle-media-stack"
TOKEN="32106606-1a70-4e0b-8bac-12ef5f2a7d5e"

# Get current public IP
CURRENT_IP=$(curl -s https://api.ipify.org)

# Update DuckDNS
RESPONSE=$(curl -s "https://www.duckdns.org/update?domains=${DOMAIN}&token=${TOKEN}&ip=${CURRENT_IP}")

# Log the result
if [ "$RESPONSE" = "OK" ]; then
    echo "$(date): DuckDNS updated successfully with IP: $CURRENT_IP"
else
    echo "$(date): DuckDNS update failed. Response: $RESPONSE"
fi
