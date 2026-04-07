#!/bin/bash

# Auto-deploy SSL certificates to all running LXC containers
CERT_DIR="/home/lou/ssl_certificates"
CA_CERT="/home/lou/ssl_certificates/myCA.crt"

# Function to deploy certificate to a container
deploy_cert() {
    local container_id="$1"
    local service_name="$2"
    
    echo "Deploying SSL certificate for $service_name (container $container_id)..."
    
    # Copy certificate files to temporary location accessible by Proxmox
    cp "$CERT_DIR/${service_name}.crt" "/tmp/${service_name}.crt" 2>/dev/null || echo "Certificate not found for $service_name, skipping..."
    cp "$CERT_DIR/${service_name}.key" "/tmp/${service_name}.key" 2>/dev/null || return
    cp "$CA_CERT" "/tmp/myCA.crt"
    
    # Deploy to container via SSH to Proxmox
    ssh -o ConnectTimeout=5 192.168.122.9 << EOF
        # Create directories
        pct exec $container_id -- mkdir -p /etc/ssl/private /etc/ssl/certs
        
        # Copy files
        pct push $container_id /tmp/${service_name}.crt /etc/ssl/certs/${service_name}.crt
        pct push $container_id /tmp/${service_name}.key /etc/ssl/private/${service_name}.key
        pct push $container_id /tmp/myCA.crt /etc/ssl/certs/myCA.crt
        
        # Set permissions
        pct exec $container_id -- chmod 644 /etc/ssl/certs/${service_name}.crt
        pct exec $container_id -- chmod 600 /etc/ssl/private/${service_name}.key
        pct exec $container_id -- chmod 644 /etc/ssl/certs/myCA.crt
        
        # Clean up temp files
        rm -f /tmp/${service_name}.crt /tmp/${service_name}.key /tmp/myCA.crt
EOF
    
    # Clean up local temp files
    rm -f "/tmp/${service_name}.crt" "/tmp/${service_name}.key" "/tmp/myCA.crt"
    
    echo "✓ Certificate deployed for $service_name"
}

# Deploy certificates to running containers
echo "Deploying SSL certificates to running LXC containers..."
echo "=============================================="

# Key services mapping: container_id -> service_name
deploy_cert 101 "vaultwarden"
deploy_cert 200 "jellyfin" 
deploy_cert 210 "prowlarr"
deploy_cert 214 "sonarr"
deploy_cert 215 "radarr"
deploy_cert 212 "qbittorrent"  # Note: qbittorrent doesn't have a cert, will skip

echo ""
echo "Certificate deployment completed!"
echo ""
echo "Next steps:"
echo "1. Configure each service to use the SSL certificates"
echo "2. Restart the services"
echo "3. Import CA certificate to your browsers: $CA_CERT"
