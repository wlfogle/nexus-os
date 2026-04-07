# WireGuard VPN Server Setup Complete

## Container Information
- **Container ID**: 100
- **IP Address**: 192.168.122.100
- **OS**: Alpine Linux 3.22
- **Startup Order**: 6 (after authentication services)

## WireGuard Configuration
- **Server Port**: 51820 (UDP)
- **VPN Network**: 10.0.0.0/24
- **Server VPN IP**: 10.0.0.1
- **Server Public Key**: 4XByD6O1U5OAyuSv1lkxqv9rNd3TF3hCAOHuAEN3KT4=

## Features Configured
✅ WireGuard server installed and running
✅ IP forwarding enabled
✅ iptables NAT rules configured
✅ OpenRC service configured for auto-start
✅ Client configuration generator script
✅ Server information display script

## Management Commands

### Check Server Status
```bash
ssh root@192.168.122.9 'pct exec 100 -- /etc/wireguard/server-info.sh'
ssh root@192.168.122.9 'pct exec 100 -- wg show'
ssh root@192.168.122.9 'pct exec 100 -- rc-service wg-quick.wg0 status'
```

### Add New Client
```bash
ssh root@192.168.122.9 'pct exec 100 -- /etc/wireguard/add-client.sh CLIENT_NAME CLIENT_IP'
# Example:
ssh root@192.168.122.9 'pct exec 100 -- /etc/wireguard/add-client.sh phone 10.0.0.2'
ssh root@192.168.122.9 'pct exec 100 -- rc-service wg-quick.wg0 restart'
```

### View Client Configuration
```bash
ssh root@192.168.122.9 'pct exec 100 -- cat /etc/wireguard/clients/CLIENT_NAME.conf'
```

### Restart WireGuard Service
```bash
ssh root@192.168.122.9 'pct exec 100 -- rc-service wg-quick.wg0 restart'
```

## Network Configuration
- Server listens on all interfaces (0.0.0.0:51820)
- NAT configured for internet access through container's eth0
- IP forwarding enabled for routing client traffic
- Clients get DNS servers: 1.1.1.1, 8.8.8.8

## Security Notes
- Private key secured with 600 permissions
- Server configuration allows full tunnel (0.0.0.0/0)
- Persistent keepalive configured for NAT traversal
- Each client gets unique IP in 10.0.0.x range

## Next Steps
1. Configure external firewall/router to forward port 51820 to 192.168.122.100
2. Update SERVER_ENDPOINT in add-client.sh with your external IP
3. Add clients as needed using the provided script
4. Consider setting up monitoring for the VPN service

## Files Created
- `/etc/wireguard/wg0.conf` - Main server configuration
- `/etc/wireguard/server_private.key` - Server private key (secure)
- `/etc/wireguard/server_public.key` - Server public key
- `/etc/wireguard/add-client.sh` - Client configuration generator
- `/etc/wireguard/server-info.sh` - Server information display
- `/etc/wireguard/clients/` - Directory for client configurations
