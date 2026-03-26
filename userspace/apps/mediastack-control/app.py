"""MediaStack Control - Master control panel for media stack Docker services."""

import os

from flask import Flask, render_template, jsonify, request
import docker
import psutil

app = Flask(__name__)

# ---------------------------------------------------------------------------
# Service Catalog — maps container names to category, icon, display name,
# and (optional) web-UI port / path.
# ---------------------------------------------------------------------------
SERVICE_CATALOG = {
    # Media Servers
    "plex":        {"category": "Media Servers", "icon": "🎬", "display": "Plex",        "web_port": 32400, "web_path": "/web"},
    "jellyfin":    {"category": "Media Servers", "icon": "🎬", "display": "Jellyfin",    "web_port": 8096},
    "emby":        {"category": "Media Servers", "icon": "🎬", "display": "Emby",        "web_port": 8096},
    "tvheadend":   {"category": "Media Servers", "icon": "📺", "display": "TVHeadend",   "web_port": 9981},
    # Arr Suite
    "sonarr":      {"category": "Arr Suite", "icon": "📺", "display": "Sonarr",    "web_port": 8989},
    "radarr":      {"category": "Arr Suite", "icon": "🎥", "display": "Radarr",    "web_port": 7878},
    "lidarr":      {"category": "Arr Suite", "icon": "🎵", "display": "Lidarr",    "web_port": 8686},
    "readarr":     {"category": "Arr Suite", "icon": "📚", "display": "Readarr",   "web_port": 8787},
    "bazarr":      {"category": "Arr Suite", "icon": "💬", "display": "Bazarr",    "web_port": 6767},
    "prowlarr":    {"category": "Arr Suite", "icon": "🔍", "display": "Prowlarr",  "web_port": 9696},
    "whisparr":    {"category": "Arr Suite", "icon": "🔞", "display": "Whisparr",  "web_port": 6969},
    # Indexers
    "jackett":      {"category": "Indexers", "icon": "🔎", "display": "Jackett",      "web_port": 9117},
    "flaresolverr": {"category": "Indexers", "icon": "🛡️", "display": "FlareSolverr", "web_port": 8191},
    # Downloads
    "qbittorrent":  {"category": "Downloads", "icon": "⬇️", "display": "qBittorrent",  "web_port": 8080},
    "deluge":       {"category": "Downloads", "icon": "⬇️", "display": "Deluge",       "web_port": 8112},
    "transmission": {"category": "Downloads", "icon": "⬇️", "display": "Transmission", "web_port": 9091},
    "sabnzbd":      {"category": "Downloads", "icon": "⬇️", "display": "SABnzbd",      "web_port": 8080},
    # Requests & Discovery
    "overseerr":  {"category": "Requests", "icon": "🎯", "display": "Overseerr",  "web_port": 5055},
    "jellyseerr": {"category": "Requests", "icon": "🎯", "display": "Jellyseerr", "web_port": 5055},
    "ombi":       {"category": "Requests", "icon": "🎯", "display": "Ombi",       "web_port": 3579},
    "doplarr":    {"category": "Requests", "icon": "🤖", "display": "Doplarr"},
    # Monitoring
    "tautulli":   {"category": "Monitoring", "icon": "📊", "display": "Tautulli",   "web_port": 8181},
    "grafana":    {"category": "Monitoring", "icon": "📈", "display": "Grafana",    "web_port": 3000},
    "prometheus": {"category": "Monitoring", "icon": "📉", "display": "Prometheus", "web_port": 9090},
    # Automation
    "flexget":      {"category": "Automation", "icon": "⚡", "display": "FlexGet",       "web_port": 5050},
    "autobrr":      {"category": "Automation", "icon": "📡", "display": "Autobrr",       "web_port": 7474},
    "recyclarr":    {"category": "Automation", "icon": "♻️", "display": "Recyclarr"},
    "unpackerr":    {"category": "Automation", "icon": "📦", "display": "Unpackerr"},
    "kometa":       {"category": "Automation", "icon": "🏷️", "display": "Kometa"},
    "autoscan":     {"category": "Automation", "icon": "🔄", "display": "Autoscan"},
    "decluttarr":   {"category": "Automation", "icon": "🧹", "display": "Decluttarr"},
    "janitorr":     {"category": "Automation", "icon": "🧹", "display": "Janitorr"},
    "watchlistarr": {"category": "Automation", "icon": "👀", "display": "Watchlistarr"},
    "checkrr":      {"category": "Automation", "icon": "✅", "display": "Checkrr"},
    "webgrabplus":  {"category": "Automation", "icon": "📋", "display": "WebGrab+Plus"},
    # Media Management
    "calibre-web":    {"category": "Media Management", "icon": "📖", "display": "Calibre-Web",    "web_port": 8083},
    "audiobookshelf": {"category": "Media Management", "icon": "🎧", "display": "Audiobookshelf", "web_port": 80},
    "tdarr":          {"category": "Media Management", "icon": "🎞️", "display": "Tdarr",          "web_port": 8265},
    # VPN & Network
    "gluetun":   {"category": "Network", "icon": "🔒", "display": "Gluetun"},
    "wireguard": {"category": "Network", "icon": "🛡️", "display": "WireGuard"},
    "wg-easy":   {"category": "Network", "icon": "🌐", "display": "WG-Easy", "web_port": 51821},
    "tailscale": {"category": "Network", "icon": "🔗", "display": "Tailscale"},
    # Dashboards
    "organizr":    {"category": "Dashboards", "icon": "🗂️", "display": "Organizr",     "web_port": 80},
    "homarr":      {"category": "Dashboards", "icon": "🏠", "display": "Homarr",       "web_port": 7575},
    "homepage":    {"category": "Dashboards", "icon": "🏡", "display": "Homepage",     "web_port": 3000},
    "htpcmanager": {"category": "Dashboards", "icon": "🖥️", "display": "HTPC Manager", "web_port": 8085},
    # Infrastructure
    "homeassistant": {"category": "Infrastructure", "icon": "🏠", "display": "Home Assistant", "web_port": 8123},
    "vaultwarden":   {"category": "Infrastructure", "icon": "🔐", "display": "Vaultwarden",   "web_port": 80},
    "ollama-gpt":    {"category": "Infrastructure", "icon": "🤖", "display": "Ollama GPT",    "web_port": 8080},
    "ollama":        {"category": "Infrastructure", "icon": "🧠", "display": "Ollama"},
    "crowdsec":      {"category": "Infrastructure", "icon": "🛡️", "display": "CrowdSec"},
    # Photos
    "immich":            {"category": "Photos", "icon": "📷", "display": "Immich",      "web_port": 8080},
    "immich_postgres15": {"category": "Photos", "icon": "🗄️", "display": "Immich DB"},
    "immich_redis":      {"category": "Photos", "icon": "⚡", "display": "Immich Cache"},
    # Tools
    "compose-toolbox": {"category": "Tools", "icon": "🧰", "display": "Compose Toolbox", "web_port": 3000},
}

CATEGORY_ORDER = [
    "Media Servers", "Arr Suite", "Indexers", "Downloads", "Requests",
    "Monitoring", "Automation", "Media Management", "Network",
    "Dashboards", "Photos", "Infrastructure", "Tools", "Other",
]

DISK_PATH = os.environ.get("DISK_PATH", "/")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def get_docker_client():
    try:
        return docker.from_env()
    except Exception:
        return None


def get_container_web_url(container, service_info):
    """Resolve a clickable URL to the service's web UI."""
    if not service_info or "web_port" not in service_info:
        return None

    ports = container.attrs.get("NetworkSettings", {}).get("Ports") or {}
    web_port = service_info["web_port"]
    web_path = service_info.get("web_path", "")

    # 1) Exact match on the declared web port
    key = f"{web_port}/tcp"
    if key in ports and ports[key]:
        host_port = ports[key][0]["HostPort"]
        return f"http://localhost:{host_port}{web_path}"

    # 2) Host-network mode — port is used directly
    net_mode = container.attrs.get("HostConfig", {}).get("NetworkMode", "")
    if net_mode == "host":
        return f"http://localhost:{web_port}{web_path}"

    # 3) Fallback — first TCP port that is host-bound (handles custom remaps)
    for pk, bindings in ports.items():
        if bindings and "/tcp" in pk:
            host_port = bindings[0]["HostPort"]
            return f"http://localhost:{host_port}{web_path}"

    return None


def get_container_info(container):
    name = container.name
    info = SERVICE_CATALOG.get(name)

    state = container.attrs.get("State", {})
    status = container.status
    started_at = state.get("StartedAt", "")
    health = state.get("Health", {}).get("Status", "") if "Health" in state else ""

    # Ports
    ports = container.attrs.get("NetworkSettings", {}).get("Ports") or {}
    port_list, seen = [], set()
    for port_key, bindings in ports.items():
        if bindings:
            for b in bindings:
                if b["HostIp"] in ("0.0.0.0", ""):
                    m = f"{b['HostPort']}:{port_key}"
                    if m not in seen:
                        seen.add(m)
                        port_list.append(m)
                    break

    # Image (short form)
    tags = container.image.tags if container.image.tags else []
    image = (tags[0] if tags else container.attrs.get("Config", {}).get("Image", "unknown"))
    short_image = image.split("/")[-1]

    web_url = get_container_web_url(container, info)

    if info:
        category, display_name, icon = info["category"], info["display"], info["icon"]
    else:
        category = "Other"
        display_name = name.replace("-", " ").replace("_", " ").title()
        icon = "📦"

    return {
        "name": name,
        "display_name": display_name,
        "icon": icon,
        "category": category,
        "status": status,
        "health": health,
        "started_at": started_at,
        "image": short_image,
        "ports": port_list,
        "web_url": web_url,
    }


# ---------------------------------------------------------------------------
# Routes
# ---------------------------------------------------------------------------

@app.route("/")
def dashboard():
    return render_template("index.html")


@app.route("/api/services")
def api_services():
    client = get_docker_client()
    if not client:
        return jsonify({"error": "Cannot connect to Docker"}), 503

    try:
        containers = client.containers.list(all=True)
        services = [get_container_info(c) for c in containers]

        grouped = {}
        for svc in services:
            grouped.setdefault(svc["category"], []).append(svc)

        ordered = []
        for cat in CATEGORY_ORDER:
            if cat in grouped:
                ordered.append({
                    "category": cat,
                    "services": sorted(grouped[cat], key=lambda x: x["display_name"]),
                })

        total = len(services)
        running = sum(1 for s in services if s["status"] == "running")
        return jsonify({
            "groups": ordered,
            "stats": {"total": total, "running": running, "stopped": total - running},
        })
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@app.route("/api/services/<name>/<action>", methods=["POST"])
def api_service_action(name, action):
    client = get_docker_client()
    if not client:
        return jsonify({"error": "Cannot connect to Docker"}), 503
    if action not in ("start", "stop", "restart"):
        return jsonify({"error": "Invalid action"}), 400

    try:
        container = client.containers.get(name)
        getattr(container, action)()
        container.reload()
        return jsonify({"status": "ok", "container_status": container.status})
    except docker.errors.NotFound:
        return jsonify({"error": f"Container '{name}' not found"}), 404
    except docker.errors.APIError as e:
        return jsonify({"error": str(e)}), 500


@app.route("/api/services/<name>/logs")
def api_service_logs(name):
    client = get_docker_client()
    if not client:
        return jsonify({"error": "Cannot connect to Docker"}), 503

    lines = min(max(request.args.get("lines", 100, type=int), 1), 5000)

    try:
        container = client.containers.get(name)
        logs = container.logs(tail=lines, timestamps=True).decode("utf-8", errors="replace")
        return jsonify({"logs": logs})
    except docker.errors.NotFound:
        return jsonify({"error": f"Container '{name}' not found"}), 404
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@app.route("/api/system")
def api_system():
    try:
        cpu = psutil.cpu_percent(interval=0.5)
        mem = psutil.virtual_memory()
        disk = psutil.disk_usage(DISK_PATH)
        return jsonify({
            "cpu":    {"percent": cpu, "cores": psutil.cpu_count()},
            "memory": {"total": mem.total, "used": mem.used, "percent": mem.percent},
            "disk":   {"total": disk.total, "used": disk.used, "free": disk.free, "percent": disk.percent},
        })
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@app.route("/api/health")
def api_health():
    client = get_docker_client()
    return jsonify({"status": "ok", "docker": client is not None})


# ---------------------------------------------------------------------------
if __name__ == "__main__":
    port = int(os.environ.get("MSC_PORT", 9900))
    debug = os.environ.get("MSC_DEBUG", "false").lower() == "true"
    app.run(host="0.0.0.0", port=port, debug=debug)
