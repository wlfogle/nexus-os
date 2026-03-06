#!/usr/bin/env python3
"""
NexusOS Orchestrator — Central Management Service
Aggregates Stella (security) + Max Jr (performance), Docker container management,
systemd service discovery, combined dashboard API.
FastAPI service on port 8600
"""

import asyncio
import logging
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

import httpx
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

_log_handlers = [logging.StreamHandler()]
if Path("/var/log/nexus-os").exists():
    _log_handlers.append(logging.FileHandler("/var/log/nexus-os/orchestrator.log", mode="a"))

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [ORCHESTRATOR] %(levelname)s %(message)s",
    handlers=_log_handlers,
)
logger = logging.getLogger("orchestrator")

app = FastAPI(
    title="NexusOS Orchestrator",
    version="2025.1",
    description="Central management — aggregates Stella + Max Jr, Docker, systemd",
)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

_start_time = datetime.now()

SERVICE_ENDPOINTS = {
    "stella": "http://127.0.0.1:8601",
    "maxjr": "http://127.0.0.1:8602",
}

NEXUS_SYSTEMD_PREFIX = "nexus-"

# ═══════════════════════════════════════════════════════════════════════════
# HELPER: ASYNC HTTP CLIENT
# ═══════════════════════════════════════════════════════════════════════════


async def _fetch(url: str, timeout: float = 8.0) -> Optional[Dict[str, Any]]:
    """Fetch JSON from an internal service. Returns None on any failure."""
    try:
        async with httpx.AsyncClient(timeout=timeout) as client:
            resp = await client.get(url)
            resp.raise_for_status()
            return resp.json()
    except (httpx.HTTPError, httpx.TimeoutException, ValueError) as exc:
        logger.warning("fetch %s failed: %s", url, exc)
        return None


# ═══════════════════════════════════════════════════════════════════════════
# SERVICE HEALTH
# ═══════════════════════════════════════════════════════════════════════════


async def _service_health(name: str) -> Dict[str, Any]:
    """Check a child service health endpoint."""
    base = SERVICE_ENDPOINTS.get(name, "")
    data = await _fetch(f"{base}/api/health")
    if data:
        return {"name": name, "status": "running", "details": data}
    return {"name": name, "status": "unreachable", "details": None}


# ═══════════════════════════════════════════════════════════════════════════
# DOCKER MANAGEMENT (subprocess, no docker python lib)
# ═══════════════════════════════════════════════════════════════════════════


def docker_list() -> List[Dict]:
    """List all Docker containers with status."""
    try:
        result = subprocess.run(
            ["docker", "ps", "-a", "--format",
             "{{.Names}}\t{{.Status}}\t{{.Image}}\t{{.Ports}}\t{{.ID}}"],
            capture_output=True, text=True, timeout=10,
        )
        if result.returncode != 0:
            return []
        containers: List[Dict] = []
        for line in result.stdout.strip().split("\n"):
            if not line.strip():
                continue
            parts = line.split("\t")
            if len(parts) >= 4:
                containers.append({
                    "name": parts[0],
                    "status": parts[1],
                    "image": parts[2],
                    "ports": parts[3],
                    "id": parts[4] if len(parts) > 4 else "",
                    "running": parts[1].startswith("Up"),
                })
        return containers
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return []


def docker_action(container: str, action: str) -> Dict:
    """Start, stop, or restart a Docker container."""
    if action not in ("start", "stop", "restart"):
        return {"success": False, "error": f"invalid action: {action}"}
    try:
        result = subprocess.run(
            ["docker", action, container],
            capture_output=True, text=True, timeout=30,
        )
        return {
            "success": result.returncode == 0,
            "container": container,
            "action": action,
            "output": (result.stdout.strip() or result.stderr.strip())[:500],
        }
    except FileNotFoundError:
        return {"success": False, "error": "docker not installed"}
    except subprocess.TimeoutExpired:
        return {"success": False, "error": "timeout"}


def docker_logs(container: str, lines: int = 100) -> Dict:
    """Fetch recent logs from a container."""
    try:
        result = subprocess.run(
            ["docker", "logs", "--tail", str(min(lines, 500)), container],
            capture_output=True, text=True, timeout=10,
        )
        output = result.stdout or result.stderr
        return {"container": container, "lines": output.strip().split("\n") if output.strip() else []}
    except FileNotFoundError:
        return {"container": container, "lines": [], "error": "docker not installed"}
    except subprocess.TimeoutExpired:
        return {"container": container, "lines": [], "error": "timeout"}


# ═══════════════════════════════════════════════════════════════════════════
# SYSTEMD SERVICE DISCOVERY
# ═══════════════════════════════════════════════════════════════════════════


def discover_nexus_services() -> List[Dict]:
    """Find all nexus-* systemd services and their status."""
    try:
        result = subprocess.run(
            ["systemctl", "list-units", "--type=service", "--all",
             "--no-pager", "--plain", "--no-legend"],
            capture_output=True, text=True, timeout=10,
        )
        if result.returncode != 0:
            return []
        services: List[Dict] = []
        for line in result.stdout.strip().split("\n"):
            cols = line.split()
            if len(cols) >= 4 and NEXUS_SYSTEMD_PREFIX in cols[0]:
                services.append({
                    "unit": cols[0],
                    "load": cols[1],
                    "active": cols[2],
                    "sub": cols[3],
                    "description": " ".join(cols[4:]) if len(cols) > 4 else "",
                })
        return services
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return []


def systemd_service_status(unit: str) -> Dict:
    """Get detailed status of a specific systemd unit."""
    try:
        result = subprocess.run(
            ["systemctl", "show", unit,
             "--property=ActiveState,SubState,MainPID,MemoryCurrent,"
             "CPUUsageNSec,ExecMainStartTimestamp,Description"],
            capture_output=True, text=True, timeout=10,
        )
        if result.returncode != 0:
            return {"unit": unit, "error": result.stderr.strip()[:200]}
        props: Dict[str, str] = {}
        for line in result.stdout.strip().split("\n"):
            if "=" in line:
                key, _, val = line.partition("=")
                props[key] = val
        mem_raw = props.get("MemoryCurrent", "")
        if mem_raw.isdigit():
            mem_mb = round(int(mem_raw) / (1024 ** 2), 1)
        else:
            mem_mb = 0
        cpu_raw = props.get("CPUUsageNSec", "")
        cpu_sec = round(int(cpu_raw) / 1_000_000_000, 2) if cpu_raw.isdigit() else 0
        return {
            "unit": unit,
            "active": props.get("ActiveState", "unknown"),
            "sub": props.get("SubState", "unknown"),
            "pid": props.get("MainPID", "0"),
            "memory_mb": mem_mb,
            "cpu_seconds": cpu_sec,
            "started": props.get("ExecMainStartTimestamp", ""),
            "description": props.get("Description", ""),
        }
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return {"unit": unit, "error": "systemctl unavailable"}


# ═══════════════════════════════════════════════════════════════════════════
# API ROUTES
# ═══════════════════════════════════════════════════════════════════════════


@app.get("/api/health")
async def health():
    return {
        "service": "orchestrator",
        "status": "running",
        "version": "2025.1",
        "timestamp": datetime.now().isoformat(),
        "uptime_seconds": int((datetime.now() - _start_time).total_seconds()),
    }


@app.get("/api/dashboard")
async def dashboard():
    """Combined dashboard: child service health, Docker, systemd, quick metrics."""
    stella_health, maxjr_health, stella_scan, maxjr_metrics = await asyncio.gather(
        _service_health("stella"),
        _service_health("maxjr"),
        _fetch(f"{SERVICE_ENDPOINTS['stella']}/api/scan"),
        _fetch(f"{SERVICE_ENDPOINTS['maxjr']}/api/metrics"),
    )
    return {
        "timestamp": datetime.now().isoformat(),
        "services": {
            "stella": stella_health,
            "maxjr": maxjr_health,
        },
        "security": stella_scan,
        "performance": maxjr_metrics,
        "docker": docker_list(),
        "systemd": discover_nexus_services(),
    }


@app.get("/api/services")
async def services():
    """Health check for all child services."""
    stella, maxjr = await asyncio.gather(
        _service_health("stella"),
        _service_health("maxjr"),
    )
    return {"services": [stella, maxjr]}


@app.get("/api/security")
async def security():
    """Proxy to Stella full scan."""
    data = await _fetch(f"{SERVICE_ENDPOINTS['stella']}/api/scan")
    return data if data else {"error": "stella unreachable"}


@app.get("/api/performance")
async def performance():
    """Proxy to Max Jr all metrics."""
    data = await _fetch(f"{SERVICE_ENDPOINTS['maxjr']}/api/metrics")
    return data if data else {"error": "maxjr unreachable"}


@app.get("/api/docker")
async def get_docker():
    return {"containers": docker_list()}


@app.post("/api/docker/{container}/{action}")
async def post_docker_action(container: str, action: str):
    return docker_action(container, action)


@app.get("/api/docker/{container}/logs")
async def get_docker_logs(container: str, lines: int = 100):
    return docker_logs(container, lines)


@app.get("/api/systemd")
async def get_systemd():
    return {"services": discover_nexus_services()}


@app.get("/api/systemd/{unit}")
async def get_systemd_unit(unit: str):
    return systemd_service_status(unit)


@app.get("/api/recommendations")
async def recommendations():
    """Aggregate recommendations from both Stella and Max Jr."""
    stella_recs, maxjr_recs = await asyncio.gather(
        _fetch(f"{SERVICE_ENDPOINTS['stella']}/api/recommendations"),
        _fetch(f"{SERVICE_ENDPOINTS['maxjr']}/api/recommendations"),
    )
    combined: List[Dict] = []
    if stella_recs and "recommendations" in stella_recs:
        for r in stella_recs["recommendations"]:
            r["source"] = "stella"
            combined.append(r)
    if maxjr_recs and "recommendations" in maxjr_recs:
        for r in maxjr_recs["recommendations"]:
            r["source"] = "maxjr"
            combined.append(r)
    severity_order = {"critical": 0, "high": 1, "warning": 2, "info": 3}
    combined.sort(key=lambda x: severity_order.get(x.get("severity", "info"), 99))
    return {"recommendations": combined}


if __name__ == "__main__":
    import uvicorn

    logger.info("NexusOS Orchestrator starting on port 8600")
    uvicorn.run(app, host="0.0.0.0", port=8600, log_level="info")
