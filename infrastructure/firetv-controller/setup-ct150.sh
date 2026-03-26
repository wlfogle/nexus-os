#!/bin/bash
# ============================================================
# CT-150: Fire TV ADB Controller
# Controls all 3 Fire TV devices (2x Fire TV + Fire TV Cube)
# via Android Debug Bridge (ADB) over the network
# Run INSIDE CT-150 after container creation
# ============================================================

apt update && apt upgrade -y
apt install -y adb python3 python3-pip python3-venv git curl

# Python virtual environment
python3 -m venv /opt/firetv-env
source /opt/firetv-env/bin/activate

pip install androidtv flask requests aiohttp

mkdir -p /opt/firetv-controller
cat > /opt/firetv-controller/app.py << 'PYEOF'
#!/usr/bin/env python3
"""
Fire TV Controller — CT-150
Controls all 3 Fire TV devices via ADB network
REST API on port 5000
"""
import subprocess
import logging
from flask import Flask, request, jsonify

app = Flask(__name__)
logging.basicConfig(level=logging.INFO, format='%(asctime)s %(levelname)s %(message)s')

# Fire TV device registry
# Update IPs to match your actual Fire TV device IPs (check router DHCP table)
DEVICES = {
    "firetv1":  {"ip": "192.168.12.51", "name": "Living Room Fire TV"},
    "firetv2":  {"ip": "192.168.12.52", "name": "Bedroom Fire TV"},
    "cube":     {"ip": "192.168.12.53", "name": "Fire TV Cube"},
}

def adb(device_ip: str, *args):
    """Run an ADB command against a device."""
    cmd = ["adb", "-s", f"{device_ip}:5555"] + list(args)
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        return {"success": result.returncode == 0, "output": result.stdout.strip(), "error": result.stderr.strip()}
    except subprocess.TimeoutExpired:
        return {"success": False, "error": "ADB command timed out"}
    except Exception as e:
        return {"success": False, "error": str(e)}

def connect(device_ip: str):
    """Connect ADB to a device."""
    result = subprocess.run(["adb", "connect", f"{device_ip}:5555"],
                            capture_output=True, text=True, timeout=10)
    return "connected" in result.stdout.lower() or "already" in result.stdout.lower()

@app.route("/devices", methods=["GET"])
def list_devices():
    """List all Fire TV devices and their connection status."""
    status = {}
    for key, device in DEVICES.items():
        connected = connect(device["ip"])
        status[key] = {
            "name": device["name"],
            "ip": device["ip"],
            "connected": connected
        }
    return jsonify(status)

@app.route("/devices/<device_id>/connect", methods=["POST"])
def connect_device(device_id):
    if device_id not in DEVICES:
        return jsonify({"error": "Unknown device"}), 404
    ok = connect(DEVICES[device_id]["ip"])
    return jsonify({"success": ok, "device": device_id})

@app.route("/devices/<device_id>/command", methods=["POST"])
def send_command(device_id):
    """
    Send a command to a Fire TV device.
    Body: {"command": "home|back|play_pause|launch_plex|launch_jellyfin|launch_netflix|..."}
    """
    if device_id not in DEVICES:
        return jsonify({"error": "Unknown device"}), 404

    data = request.get_json() or {}
    command = data.get("command", "")
    ip = DEVICES[device_id]["ip"]

    KEYEVENT_MAP = {
        "home":       "3",
        "back":       "4",
        "menu":       "82",
        "play_pause": "85",
        "rewind":     "89",
        "fast_fwd":   "90",
        "up":         "19",
        "down":       "20",
        "left":       "21",
        "right":      "22",
        "select":     "23",
        "volume_up":  "24",
        "volume_down":"25",
        "mute":       "164",
    }

    APP_MAP = {
        "plex":     "com.plexapp.android",
        "jellyfin": "org.jellyfin.androidtv",
        "netflix":  "com.netflix.ninja",
        "prime":    "com.amazon.avod.thirdpartyclient",
        "disney":   "com.disney.disneyplus",
        "youtube":  "com.amazon.firetv.youtube",
    }

    connect(ip)

    if command in KEYEVENT_MAP:
        return jsonify(adb(ip, "shell", "input", "keyevent", KEYEVENT_MAP[command]))
    elif command.startswith("launch_"):
        app_name = command[7:]
        pkg = APP_MAP.get(app_name, app_name)
        return jsonify(adb(ip, "shell", "monkey", "-p", pkg, "-c",
                           "android.intent.category.LAUNCHER", "1"))
    elif command == "screenshot":
        r = adb(ip, "shell", "screencap", "-p", "/sdcard/screen.png")
        return jsonify(r)
    elif command == "wake":
        return jsonify(adb(ip, "shell", "input", "keyevent", "224"))
    elif command == "sleep":
        return jsonify(adb(ip, "shell", "input", "keyevent", "223"))
    elif command == "install":
        apk_url = data.get("apk_url", "")
        if not apk_url:
            return jsonify({"error": "apk_url required"}), 400
        dl = subprocess.run(["curl", "-L", "-o", "/tmp/install.apk", apk_url],
                            capture_output=True, timeout=60)
        if dl.returncode != 0:
            return jsonify({"success": False, "error": "Download failed"})
        return jsonify(adb(ip, "install", "-r", "/tmp/install.apk"))
    else:
        return jsonify({"error": f"Unknown command: {command}"}), 400

@app.route("/devices/all/command", methods=["POST"])
def broadcast_command():
    """Send the same command to ALL Fire TV devices."""
    data = request.get_json() or {}
    results = {}
    for device_id in DEVICES:
        with app.test_request_context(
            '/devices/{}/command'.format(device_id),
            json=data, method='POST'
        ):
            resp = send_command(device_id)
            results[device_id] = resp.get_json() if hasattr(resp, 'get_json') else {}
    return jsonify(results)

@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "ok", "devices": list(DEVICES.keys())})

if __name__ == "__main__":
    logging.info("Fire TV Controller starting on :5000")
    app.run(host="0.0.0.0", port=5000, debug=False)
PYEOF

# Systemd service
cat > /etc/systemd/system/firetv-controller.service << 'EOF'
[Unit]
Description=Fire TV ADB Controller
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/firetv-controller
Environment=PATH=/opt/firetv-env/bin:/usr/bin:/bin
ExecStart=/opt/firetv-env/bin/python /opt/firetv-controller/app.py
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable firetv-controller
systemctl start firetv-controller

echo ""
echo "=== Fire TV Controller (CT-150) Setup Complete ==="
echo ""
echo "REST API running on: http://192.168.12.150:5000"
echo ""
echo "Before use — enable ADB on each Fire TV:"
echo "  Settings → My Fire TV → Developer Options → ADB Debugging → ON"
echo ""
echo "Then pair devices:"
echo "  curl -X POST http://192.168.12.150:5000/devices/firetv1/connect"
echo "  curl -X POST http://192.168.12.150:5000/devices/firetv2/connect"
echo "  curl -X POST http://192.168.12.150:5000/devices/cube/connect"
echo ""
echo "Test a command:"
echo "  curl -X POST http://192.168.12.150:5000/devices/firetv1/command -H 'Content-Type: application/json' -d '{\"command\":\"home\"}'"
echo ""
echo "Install TiamatsStack APK on all devices:"
echo "  curl -X POST http://192.168.12.150:5000/devices/all/command \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"command\":\"install\",\"apk_url\":\"http://192.168.12.10/tiamats-stack.apk\"}'"
