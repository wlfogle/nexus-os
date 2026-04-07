# **Complete Fire TV Cube Integration Guide in Proxmox**

---

## 1. Overview and Architecture

You have a media stack running in various Proxmox LXCs and VMs. The goal is to treat the Fire TV Cube as an integrated controllable node within this stack, capable of automation, monitoring, and interaction via network APIs.

**High-level architecture example:**

```
+-----------------+       +--------------------+       +---------------------+
|                 |       |                    |       |                     |
| Proxmox Host    |       | LXC Container:     |       | Fire TV Cube        |
| - Media Stack   | <----> | Fire TV Controller | <----> | (Network IP)        |
| - Home Assistant|       | (Python ADB daemon)|       |                     |
+-----------------+       +--------------------+       +---------------------+
```

---

## 2. Prepare Fire TV Cube

### Enable Developer Options & ADB Debugging

1. Navigate on Fire TV Cube:
   - Settings → My Fire TV → About → click on "Fire TV Stick" or Device 7 times to enable Developer Options.
2. Go back to Settings → My Fire TV → Developer Options.
3. Enable **ADB Debugging**.
4. Enable **Apps from Unknown Sources** (optional, if you install 3rd party tools).
5. Connect Fire TV to your home network (ensure same LAN as Proxmox LXCs).
6. Note the Fire TV IP address (Settings → Network).

---

## 3. Network Setup on Proxmox

- Use a bridged network for all LXCs/VMs (`vmbr0`), on same IP subnet as Fire TV, e.g., 192.168.1.x.
- Verify that LXCs and Fire TV can ping each other:

```bash
ping 192.168.1.50  # Fire TV IP from Proxmox host or LXC
```

- Configure firewall to allow TCP port 5555 (ADB default), and relevant control ports (e.g., 5000 for REST API):

```bash
iptables -A INPUT -p tcp -s 192.168.1.0/24 --dport 5555 -j ACCEPT
iptables -A INPUT -p tcp -s 192.168.1.0/24 --dport 5000 -j ACCEPT
```

---

## 4. Create Fire TV Controller LXC (e.g. 150)

### Create LXC Container

```bash
pct create 150 local:vztmpl/ubuntu-22.04-standard_22.04-1_amd64.tar.zst \
  --hostname firetv-controller \
  --memory 1024 --cores 2 \
  --net0 name=eth0,bridge=vmbr0,ip=dhcp \
  --storage local-lvm:8G \
  --features nesting=1 \
  --startup order=50
pct start 150
pct enter 150
```

Note: `nesting=1` allows running services that require cgroups manipulation.

### Install Software and Dependencies

Inside the LXC:

```bash
apt update && apt upgrade -y
apt install -y adb python3 python3-pip python3-venv git curl

# Create Python virtual environment
python3 -m venv /opt/firetv-env
source /opt/firetv-env/bin/activate

# Install required Python packages
pip install androidtv firetv flask requests paho-mqtt websockets aiohttp asyncio-mqtt
```

---

## 5. Fire TV Controller Service Implementation

### Python REST API Service

Create `/opt/firetv-controller/firetv_service.py`:

```python
import asyncio
from flask import Flask, request, jsonify
from androidtv import AndroidTV
import logging

app = Flask(__name__)
logging.basicConfig(level=logging.INFO)

class FireTVController:
    def __init__(self, host='192.168.1.50'):
        self.host = host
        self.device = AndroidTV(self.host)
        self.connected = False

    async def connect(self):
        try:
            await self.device.adb_connect()
            self.connected = True
            logging.info(f"Connected to Fire TV at {self.host}")
            return True
        except Exception as e:
            logging.error(f"Failed connecting to Fire TV: {e}")
            self.connected = False
            return False

    async def get_status(self):
        if not self.connected:
            await self.connect()
        if not self.connected:
            return {"connected": False}
        status = await self.device.get_state()
        current_app = await self.device.get_current_app()
        volume = await self.device.get_volume_level()
        return {
            "connected": True,
            "state": status,
            "current_app": current_app,
            "volume": volume,
        }

    async def send_command(self, command, **kwargs):
        if not self.connected:
            await self.connect()
        if not self.connected:
            return {"success": False, "error": "Cannot connect to Fire TV"}
        try:
            if command == "home":
                await self.device.home()
            elif command == "back":
                await self.device.back()
            elif command == "menu":
                await self.device.menu()
            elif command == "play_pause":
                await self.device.media_play_pause()
            elif command == "power":
                await self.device.power()
            elif command == "launch_app" and "app_id" in kwargs:
                await self.device.start_intent(kwargs["app_id"])
            elif command == "volume_up":
                await self.device.volume_up()
            elif command == "volume_down":
                await self.device.volume_down()
            elif command == "text" and "text" in kwargs:
                await self.device.send_text(kwargs["text"])
            else:
                return {"success": False, "error": "Unknown command or missing parameters"}
            return {"success": True}
        except Exception as e:
            return {"success": False, "error": str(e)}

controller = FireTVController()

@app.route("/status", methods=["GET"])
def status():
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    result = loop.run_until_complete(controller.get_status())
    return jsonify(result)

@app.route("/command", methods=["POST"])
def command():
    data = request.get_json()
    cmd = data.get("command")
    params = data.get("params", {})
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    result = loop.run_until_complete(controller.send_command(cmd, **params))
    return jsonify(result)

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
```

### Create Systemd Service

Create `/etc/systemd/system/firetv-controller.service`:

```
[Unit]
Description=Fire TV Controller Service
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/firetv-controller
Environment=PATH=/opt/firetv-env/bin
ExecStart=/opt/firetv-env/bin/python /opt/firetv-controller/firetv_service.py
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable service:

```bash
systemctl daemon-reload
systemctl enable firetv-controller
systemctl start firetv-controller
systemctl status firetv-controller
```

---

## 6. Home Assistant Integration (LXC 151)

Add to your Home Assistant configuration (assuming it runs in an LXC on the same network):

```yaml
# configuration.yaml

rest_command:
  firetv_command:
    url: "http://192.168.1.150:5000/command"
    method: POST
    headers:
      Content-Type: application/json
    payload: >
      {
        "command": "{{ command }}",
        "params": {{ params | default({}) | to_json }}
      }

sensor:
  - platform: rest
    resource: "http://192.168.1.150:5000/status"
    name: "Fire TV Status"
    json_attributes_path: "$"
    json_attributes:
      - state
      - current_app
      - volume
      - connected
    value_template: "{{ value_json.state }}"
    scan_interval: 60

switch:
  - platform: template
    switches:
      firetv_power:
        friendly_name: "Fire TV Power"
        value_template: "{{ is_state('sensor.fire_tv_status', 'on') }}"
        turn_on:
          service: rest_command.firetv_command
          data:
            command: "power"
        turn_off:
          service: rest_command.firetv_command
          data:
            command: "power"

media_player:
  - platform: template
    media_players:
      fire_tv_cube:
        friendly_name: "Fire TV Cube"
        value_template: >
          {% if is_state('sensor.fire_tv_status', 'on') %}
            playing
          {% else %}
            off
          {% endif %}
        turn_on:
          service: rest_command.firetv_command
          data:
            command: "power"
        turn_off:
          service: rest_command.firetv_command
          data:
            command: "power"
        media_play:
          service: rest_command.firetv_command
          data:
            command: "play_pause"
        media_pause:
          service: rest_command.firetv_command
          data:
            command: "play_pause"
        volume_up:
          service: rest_command.firetv_command
          data:
            command: "volume_up"
        volume_down:
          service: rest_command.firetv_command
          data:
            command: "volume_down"
```

**Reload and verify sensors and switches on Home Assistant UI.**

---

## 7. Plex / Sonarr / Radarr Integration

### Tautulli - Fire TV Playback Monitoring

- Tautulli running in LXC/VM can monitor streaming sessions.
- No additional configuration required if Fire TV is playing via Plex.

### Sonarr/Radarr Custom Scripts for Notifications & Playback

Create script `/opt/scripts/firetv-notification.sh` in the Proxmox host or appropriate LXC:

```bash
#!/bin/bash
FIRETV_API="http://192.168.1.150:5000/command"
TITLE="$sonarr_series_title - S${sonarr_seasonnumber}E${sonarr_episodenumber}"

# Optionally send notification via key event (customize for your setup)
curl -X POST "$FIRETV_API" -H "Content-Type: application/json" \
  -d '{"command": "text", "params": {"text": "New episode downloaded: '"$TITLE"'"}}'

# Launch Plex app automatically after download
sleep 5
curl -X POST "$FIRETV_API" -H "Content-Type: application/json" \
  -d '{"command": "launch_app", "params": {"app_id": "com.plexapp.android"}}'
```

Configure Sonarr/Radarr to call this script on download completion.

---

## 8. Advanced Automation & Monitoring (Optional)

### Fire TV Monitoring LXC (e.g., 152)

- Create new LXC dedicated to monitoring Fire TV usage.
- Run a Python async script that polls `/status` REST endpoint.
- Log user activity in SQL or InfluxDB.
- Trigger HA automations or notifications.

Example: `/opt/firetv-monitor/monitor.py`

```python
import asyncio
import aiohttp
import sqlite3
from datetime import datetime

DATABASE = "firetv_usage.db"

def init_db():
    conn = sqlite3.connect(DATABASE)
    conn.execute('''CREATE TABLE IF NOT EXISTS usage (
                        timestamp TEXT,
                        state TEXT,
                        current_app TEXT,
                        volume INTEGER
                    )''')
    conn.commit()
    conn.close()

async def poll_status():
    async with aiohttp.ClientSession() as session:
        while True:
            async with session.get("http://192.168.1.150:5000/status") as resp:
                data = await resp.json()
                conn = sqlite3.connect(DATABASE)
                conn.execute("INSERT INTO usage VALUES (?, ?, ?, ?)",
                             (datetime.now().isoformat(),
                              data.get("state"),
                              data.get("current_app"),
                              data.get("volume")))
                conn.commit()
                conn.close()
            await asyncio.sleep(30)

if __name__ == "__main__":
    init_db()
    asyncio.run(poll_status())
```

---

## 9. Security Recommendations

- Restrict ADB and REST API access to your LAN only.
- Use firewall rules in Proxmox to block outside access.
- If exposing REST API outside LAN, add authentication proxy (e.g., Traefik, Nginx with basic auth).
- Regularly update all containers and LXCs.

---

## 10. Maintenance Tips

- Monitor logs of Fire TV Controller service via `journalctl -u firetv-controller -f`.
- Restart service if connectivity issues occur.
- Keep Fire TV and LXCs up to date.
- Backup Proxmox LXC configurations.

---

# Summary

- You run a **Fire TV Controller LXC** on Proxmox with Python ADB integration.
- Provides REST API to control Fire TV functions (launch apps, volume, power, text).
- Home Assistant LXC uses REST commands to control Fire TV via this API.
- Plex/Sonarr/Radarr integrate by triggering commands or tracking playback.
- Optional monitoring LXC tracks Fire TV usage analytics.
- Network and security carefully configured for best performance and safety.

If you want, I can generate configuration files, scripts, or even automate the entire setup from your Proxmox host—just ask!
