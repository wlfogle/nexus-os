# Phase 10 — Home Assistant Completion

VM-990 (haos17-1) running at http://192.168.12.123:8123
HA version: 2026.4.0
User: loufogle / homeassist

## Tiamat Agent Tasks

### 1. Add Traefik route for Home Assistant
On CT-103, create `/etc/traefik/dynamic/home-assistant.yml`:
```yaml
http:
  routers:
    homeassistant:
      rule: "Host(`ha.tiamat.local`)"
      service: homeassistant
      entryPoints:
        - web
  services:
    homeassistant:
      loadBalancer:
        servers:
          - url: "http://192.168.12.123:8123"
```

### 2. Install SSH & Terminal add-on (enables remote CLI)
Via Proxmox VM console for VM-990:
```
ha addons install core_ssh
ha addons config core_ssh --options '{"authorized_keys":["ssh-ed25519 AAAA... root@tiamat"]}'
ha addons start core_ssh
```

### 3. Install HACS via SSH (once SSH add-on is running)
```
ha addons exec core_ssh -- wget -O - https://get.hacs.xyz | bash -
```
Then restart HA: `ha core restart`

### 4. Commit changes

## Browser Tasks (user must do)

### Set Static IP
Settings → System → Network → IPv4 → Static:
- IP: 192.168.12.250/24
- Gateway: 192.168.12.1
- DNS: 192.168.12.244 (AdGuard on Bahamut)

### Add AdGuard Home Integration
Settings → Devices & Services → Add Integration → AdGuard Home
- Host: 192.168.12.244
- Port: 8080
- Username: adguard
- Password: (your AdGuard password)

### Configure HACS (after install)
Settings → Devices & Services → Add Integration → HACS
- Follow GitHub OAuth flow
- Then install: Alexa Media Player custom component

### Add Alexa Integration (via HACS)
HACS → Integrations → search "Alexa Media Player" → Install
Restart HA, then:
Settings → Devices & Services → Add Integration → Alexa Media Player
- Sign in with Amazon account

### Set URLs
Settings → System → General:
- Internal URL: http://192.168.12.250:8123
- External URL: https://tiamat-tailscale.tail9d8b73.ts.net

### Useful Automations to Create
- Plex/Jellyfin webhook → dim lights when playing
- AdGuard → auto-enable protection at bedtime
- WireGuard status monitoring
- Media server health alerts

## Integrations Status
| Integration | Status |
|---|---|
| Jellyfin | ✅ loaded |
| Plex | ✅ loaded |
| HDHomeRun | ✅ auto-discovered |
| EPSON Printer | ✅ auto-discovered |
| AdGuard Home | ⏳ needs browser setup |
| HACS | ⏳ needs SSH add-on first |
| Alexa Media Player | ⏳ needs HACS first |
| Tailscale | ⏳ add after HACS |
