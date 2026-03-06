#!/usr/bin/env python3
"""
Stella — NexusOS Security Guardian
Real-time security monitoring: ports, firewall, logins, updates, Docker health
FastAPI service on port 8601
"""

import logging
import os
import re
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Dict, List

import psutil
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

_log_handlers = [logging.StreamHandler()]
if Path("/var/log/nexus-os").exists():
    _log_handlers.append(logging.FileHandler("/var/log/nexus-os/stella.log", mode="a"))

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [STELLA] %(levelname)s %(message)s",
    handlers=_log_handlers,
)
logger = logging.getLogger("stella")

app = FastAPI(
    title="Stella — NexusOS Security Guardian",
    version="2025.1",
    description="Real-time security monitoring for NexusOS",
)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

_start_time = datetime.now()

# ═══════════════════════════════════════════════════════════════════════════
# SECURITY SCAN FUNCTIONS
# ═══════════════════════════════════════════════════════════════════════════


def scan_open_ports() -> List[Dict]:
    """Scan for open listening ports and their associated processes."""
    ports = []
    for conn in psutil.net_connections(kind="inet"):
        if conn.status == "LISTEN":
            try:
                proc = psutil.Process(conn.pid) if conn.pid else None
                ports.append({
                    "port": conn.laddr.port,
                    "address": conn.laddr.ip,
                    "pid": conn.pid,
                    "process": proc.name() if proc else "unknown",
                    "user": proc.username() if proc else "unknown",
                })
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                ports.append({
                    "port": conn.laddr.port,
                    "address": conn.laddr.ip,
                    "pid": conn.pid,
                    "process": "unknown",
                    "user": "unknown",
                })
    seen: set = set()
    unique: List[Dict] = []
    for p in ports:
        if p["port"] not in seen:
            seen.add(p["port"])
            unique.append(p)
    return sorted(unique, key=lambda x: x["port"])


def scan_failed_logins(max_lines: int = 500) -> Dict:
    """Parse auth log for failed SSH/login attempts."""
    failed: List[Dict] = []
    auth_log = None
    for candidate in ("/var/log/auth.log", "/var/log/secure"):
        if os.path.isfile(candidate) and os.access(candidate, os.R_OK):
            auth_log = candidate
            break

    if not auth_log:
        return {"source": None, "count": 0, "recent": []}

    try:
        with open(auth_log, "r", errors="replace") as fh:
            lines = fh.readlines()[-max_lines:]
        pattern = re.compile(r"Failed password.*from (\S+)")
        for line in lines:
            match = pattern.search(line)
            if match:
                failed.append({"ip": match.group(1), "line": line.strip()[:200]})
    except OSError:
        pass

    return {"source": auth_log, "count": len(failed), "recent": failed[-20:]}


def check_firewall() -> Dict:
    """Check UFW firewall status."""
    try:
        result = subprocess.run(
            ["ufw", "status", "verbose"],
            capture_output=True, text=True, timeout=10,
        )
        if result.returncode == 0:
            output = result.stdout.strip()
            return {"installed": True, "active": "Status: active" in output, "output": output}
        return {"installed": True, "active": False, "output": result.stderr.strip()}
    except FileNotFoundError:
        return {"installed": False, "active": False, "output": "ufw not installed"}
    except subprocess.TimeoutExpired:
        return {"installed": True, "active": False, "output": "timeout"}


def check_package_updates() -> Dict:
    """Check for available package and security updates."""
    try:
        result = subprocess.run(
            ["apt", "list", "--upgradable"],
            capture_output=True, text=True, timeout=30,
            env={**os.environ, "LANG": "C"},
        )
        if result.returncode == 0:
            lines = [l for l in result.stdout.strip().split("\n") if "/" in l]
            security = [l for l in lines if "security" in l.lower()]
            return {
                "total_upgradable": len(lines),
                "security_updates": len(security),
                "packages": lines[:50],
            }
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    return {"total_upgradable": 0, "security_updates": 0, "packages": []}


def check_docker_health() -> List[Dict]:
    """Check health of all Docker containers."""
    try:
        result = subprocess.run(
            ["docker", "ps", "-a", "--format",
             "{{.Names}}\t{{.Status}}\t{{.Image}}\t{{.Ports}}"],
            capture_output=True, text=True, timeout=10,
        )
        if result.returncode != 0:
            return []
        containers: List[Dict] = []
        for line in result.stdout.strip().split("\n"):
            if not line.strip():
                continue
            parts = line.split("\t")
            if len(parts) >= 3:
                containers.append({
                    "name": parts[0],
                    "status": parts[1],
                    "image": parts[2],
                    "ports": parts[3] if len(parts) > 3 else "",
                    "healthy": parts[1].startswith("Up"),
                })
        return containers
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return []


def check_disk_space() -> List[Dict]:
    """Check disk space on all mounted partitions."""
    disks: List[Dict] = []
    for part in psutil.disk_partitions(all=False):
        try:
            usage = psutil.disk_usage(part.mountpoint)
            disks.append({
                "device": part.device,
                "mountpoint": part.mountpoint,
                "fstype": part.fstype,
                "total_gb": round(usage.total / (1024 ** 3), 1),
                "used_gb": round(usage.used / (1024 ** 3), 1),
                "free_gb": round(usage.free / (1024 ** 3), 1),
                "percent": usage.percent,
                "warning": usage.percent > 85,
                "critical": usage.percent > 95,
            })
        except (PermissionError, OSError):
            continue
    return disks


def generate_recommendations() -> List[Dict]:
    """Generate security recommendations based on current system state."""
    recs: List[Dict] = []

    fw = check_firewall()
    if not fw["active"]:
        recs.append({
            "severity": "high",
            "category": "firewall",
            "title": "Firewall is not active",
            "description": "Enable UFW: sudo ufw enable",
        })

    for disk in check_disk_space():
        if disk["critical"]:
            recs.append({
                "severity": "critical",
                "category": "storage",
                "title": f"Critical disk usage on {disk['mountpoint']}",
                "description": f"{disk['percent']}% used — {disk['free_gb']}GB free",
            })
        elif disk["warning"]:
            recs.append({
                "severity": "warning",
                "category": "storage",
                "title": f"High disk usage on {disk['mountpoint']}",
                "description": f"{disk['percent']}% used — {disk['free_gb']}GB free",
            })

    updates = check_package_updates()
    if updates["security_updates"] > 0:
        recs.append({
            "severity": "high",
            "category": "updates",
            "title": f"{updates['security_updates']} security updates available",
            "description": "Run: sudo nala upgrade",
        })

    logins = scan_failed_logins()
    if logins["count"] > 10:
        recs.append({
            "severity": "warning",
            "category": "authentication",
            "title": f"{logins['count']} failed login attempts detected",
            "description": "Review auth logs and consider installing fail2ban",
        })

    open_ports = scan_open_ports()
    external = [p for p in open_ports if p["address"] in ("0.0.0.0", "::")]
    if len(external) > 15:
        recs.append({
            "severity": "info",
            "category": "network",
            "title": f"{len(external)} services listening on all interfaces",
            "description": "Verify all services need external access",
        })

    return recs


# ═══════════════════════════════════════════════════════════════════════════
# API ROUTES
# ═══════════════════════════════════════════════════════════════════════════


@app.get("/api/health")
async def health():
    return {
        "service": "stella",
        "status": "running",
        "version": "2025.1",
        "timestamp": datetime.now().isoformat(),
        "uptime_seconds": int((datetime.now() - _start_time).total_seconds()),
    }


@app.get("/api/scan")
async def full_scan():
    return {
        "timestamp": datetime.now().isoformat(),
        "ports": scan_open_ports(),
        "firewall": check_firewall(),
        "failed_logins": scan_failed_logins(),
        "updates": check_package_updates(),
        "docker": check_docker_health(),
        "disk_space": check_disk_space(),
        "recommendations": generate_recommendations(),
    }


@app.get("/api/ports")
async def get_ports():
    return {"ports": scan_open_ports()}


@app.get("/api/firewall")
async def get_firewall():
    return check_firewall()


@app.get("/api/logins")
async def get_logins():
    return scan_failed_logins()


@app.get("/api/updates")
async def get_updates():
    return check_package_updates()


@app.get("/api/docker")
async def get_docker():
    return {"containers": check_docker_health()}


@app.get("/api/disk")
async def get_disk():
    return {"filesystems": check_disk_space()}


@app.get("/api/recommendations")
async def get_recommendations():
    return {"recommendations": generate_recommendations()}


if __name__ == "__main__":
    import uvicorn

    logger.info("Stella Security Guardian starting on port 8601")
    uvicorn.run(app, host="0.0.0.0", port=8601, log_level="info")
