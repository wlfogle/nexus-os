#!/bin/bash
# ============================================================
# DuckDNS Dynamic DNS Setup
# Domain: lou-fogle-media-stack.duckdns.org
# Keeps your public IP updated so remote access always works
# Run on Proxmox host as root
# ============================================================

DUCKDNS_TOKEN="${DUCKDNS_TOKEN:-}"
DUCKDNS_DOMAIN="${DUCKDNS_DOMAIN:-lou-fogle-media-stack}"

if [ -z "$DUCKDNS_TOKEN" ]; then
  echo "ERROR: Set DUCKDNS_TOKEN environment variable first"
  echo "  Get your token from: https://www.duckdns.org"
  echo "  export DUCKDNS_TOKEN=your-token-here"
  echo "  bash scripts/setup-duckdns.sh"
  exit 1
fi

echo "==> Setting up DuckDNS for: ${DUCKDNS_DOMAIN}.duckdns.org"

# Create update script
mkdir -p /opt/duckdns
cat > /opt/duckdns/update.sh << EOF
#!/bin/bash
TOKEN="${DUCKDNS_TOKEN}"
DOMAIN="${DUCKDNS_DOMAIN}"
LOG="/var/log/duckdns.log"

RESULT=\$(curl -s "https://www.duckdns.org/update?domains=\${DOMAIN}&token=\${TOKEN}&ip=")
TIMESTAMP=\$(date '+%Y-%m-%d %H:%M:%S')

echo "\${TIMESTAMP} \${RESULT}" >> "\${LOG}"

if [ "\${RESULT}" = "OK" ]; then
  echo "\${TIMESTAMP} DuckDNS update: OK" >> "\${LOG}"
else
  echo "\${TIMESTAMP} DuckDNS update FAILED: \${RESULT}" >> "\${LOG}"
fi
EOF

chmod +x /opt/duckdns/update.sh

# Run immediately to verify token works
echo "==> Testing DuckDNS update..."
bash /opt/duckdns/update.sh
cat /var/log/duckdns.log

# Add cron job (every 5 minutes)
echo "==> Adding cron job (every 5 minutes)..."
(crontab -l 2>/dev/null | grep -v duckdns; echo "*/5 * * * * /opt/duckdns/update.sh") | crontab -

echo ""
echo "=== DuckDNS Setup Complete ==="
echo ""
echo "Domain:    https://${DUCKDNS_DOMAIN}.duckdns.org"
echo "Updates:   every 5 minutes via cron"
echo "Log:       /var/log/duckdns.log"
echo ""
echo "Port forwarding needed on T-Mobile router:"
echo "  80  → 192.168.12.10:80   (HTTP/Traefik)"
echo "  443 → 192.168.12.10:443  (HTTPS/Traefik)"
echo "  51820/UDP → 192.168.12.20:51820 (WireGuard remote access)"
echo ""
echo "Note: T-Mobile 5G Home Internet uses CGNAT — port forwarding"
echo "may not work. Use WireGuard instead for reliable remote access."
