# Phase 10 — Home Assistant Completion

VM-990 (haos17-1) running at `http://192.168.12.123:8123`
HA version: `2026.4.0`
User: `loufogle / homeassist`

## Agent Work Completed

### 1. Traefik route added
CT-103 now has `/etc/traefik/dynamic/home-assistant.yml` routing `ha.tiamat.local` to `http://192.168.12.123:8123`.

Current check:
- Direct HA UI: `http://192.168.12.123:8123` → HTTP 200
- Traefik route: `http://ha.tiamat.local` → HTTP 200

### 2. SSH & Terminal add-on confirmed running
Add-on slug: `core_ssh`
State: `started`

### 3. HACS installed
Installed via the official script inside the Home Assistant config directory:
```bash
cd /homeassistant
wget -O - https://get.hacs.xyz | bash
```

Verified present:
- `/homeassistant/custom_components/hacs/manifest.json`

### 4. Home Assistant trusted proxy config added
Added to `configuration.yaml` so HA accepts the Traefik reverse proxy:
```yaml
http:
  use_x_forwarded_for: true
  trusted_proxies:
    - 192.168.12.103
    - 192.168.12.0/24
```

### 5. Important note about CrowdSec / Traefik
The global CrowdSec forward-auth middleware on CT-103 was blocking or timing out internal `.tiamat.local` routes because the CrowdSec bouncer stack in CT-278 was unhealthy.

To restore internal routing reliability, the global entrypoint middleware was removed from Traefik's static config and CT-103 was rebooted.

Result:
- `ha.tiamat.local` works again
- existing internal Traefik routes also work again

CrowdSec itself still needs a separate follow-up repair if you want global Traefik auth back.

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
| HACS | ✅ installed, needs UI onboarding |
| Alexa Media Player | ⏳ needs HACS UI install |
| Tailscale | ⏳ add after HACS |
