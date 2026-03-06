#!/usr/bin/env python3
"""
Max Jr. — NexusOS Performance Optimizer
Real-time performance monitoring: CPU, GPU, memory, temperatures, gaming detection
FastAPI service on port 8602
"""

import logging
import os
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

import psutil
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

_log_handlers = [logging.StreamHandler()]
if Path("/var/log/nexus-os").exists():
    _log_handlers.append(logging.FileHandler("/var/log/nexus-os/maxjr.log", mode="a"))

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [MAX JR] %(levelname)s %(message)s",
    handlers=_log_handlers,
)
logger = logging.getLogger("maxjr")

app = FastAPI(
    title="Max Jr. — NexusOS Performance Optimizer",
    version="2025.1",
    description="Real-time performance monitoring for NexusOS",
)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

_start_time = datetime.now()

GAMING_PROCESSES = frozenset({
    "steam", "lutris", "wine", "wine64", "wineserver",
    "proton", "gamescope", "mangohud", "gamemode",
    "bottles", "heroic", "legendary",
})

# ═══════════════════════════════════════════════════════════════════════════
# PERFORMANCE MONITORING FUNCTIONS
# ═══════════════════════════════════════════════════════════════════════════


def get_cpu_stats() -> Dict:
    """Get CPU usage, frequency, and per-core stats."""
    freq = psutil.cpu_freq()
    return {
        "usage_percent": psutil.cpu_percent(interval=0.5),
        "per_core": psutil.cpu_percent(interval=0.5, percpu=True),
        "core_count_physical": psutil.cpu_count(logical=False),
        "core_count_logical": psutil.cpu_count(logical=True),
        "frequency_mhz": round(freq.current, 0) if freq else 0,
        "frequency_max_mhz": round(freq.max, 0) if freq else 0,
        "load_avg_1m": round(os.getloadavg()[0], 2),
        "load_avg_5m": round(os.getloadavg()[1], 2),
        "load_avg_15m": round(os.getloadavg()[2], 2),
    }


def get_memory_stats() -> Dict:
    """Get memory and swap usage."""
    mem = psutil.virtual_memory()
    swap = psutil.swap_memory()
    return {
        "total_gb": round(mem.total / (1024 ** 3), 1),
        "used_gb": round(mem.used / (1024 ** 3), 1),
        "available_gb": round(mem.available / (1024 ** 3), 1),
        "percent": mem.percent,
        "swap_total_gb": round(swap.total / (1024 ** 3), 1),
        "swap_used_gb": round(swap.used / (1024 ** 3), 1),
        "swap_percent": swap.percent,
    }


def get_gpu_stats() -> Optional[Dict]:
    """Get NVIDIA GPU stats via nvidia-smi."""
    try:
        result = subprocess.run(
            [
                "nvidia-smi",
                "--query-gpu=name,utilization.gpu,memory.used,memory.total,"
                "temperature.gpu,power.draw,fan.speed,clocks.current.graphics",
                "--format=csv,noheader,nounits",
            ],
            capture_output=True, text=True, timeout=5,
        )
        if result.returncode == 0:
            parts = [p.strip() for p in result.stdout.strip().split(",")]
            if len(parts) >= 8:
                def safe_float(val: str) -> float:
                    try:
                        return float(val)
                    except (ValueError, TypeError):
                        return 0.0

                return {
                    "name": parts[0],
                    "utilization_percent": safe_float(parts[1]),
                    "memory_used_mb": safe_float(parts[2]),
                    "memory_total_mb": safe_float(parts[3]),
                    "temperature_c": safe_float(parts[4]),
                    "power_draw_w": safe_float(parts[5]),
                    "fan_speed_percent": safe_float(parts[6]),
                    "clock_mhz": safe_float(parts[7]),
                }
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    return None


def get_temperatures() -> Dict:
    """Get CPU and other sensor temperatures."""
    temps: Dict = {}
    try:
        sensor_temps = psutil.sensors_temperatures()
        for sensor_name, entries in sensor_temps.items():
            temps[sensor_name] = [
                {
                    "label": entry.label or sensor_name,
                    "current": entry.current,
                    "high": entry.high,
                    "critical": entry.critical,
                }
                for entry in entries
            ]
    except (AttributeError, RuntimeError):
        pass
    return temps


def get_disk_io() -> Dict:
    """Get disk I/O statistics."""
    io = psutil.disk_io_counters()
    if io:
        return {
            "read_mb": round(io.read_bytes / (1024 ** 2), 1),
            "write_mb": round(io.write_bytes / (1024 ** 2), 1),
            "read_count": io.read_count,
            "write_count": io.write_count,
        }
    return {}


def get_network_stats() -> Dict:
    """Get network I/O statistics."""
    io = psutil.net_io_counters()
    return {
        "bytes_sent_mb": round(io.bytes_sent / (1024 ** 2), 1),
        "bytes_recv_mb": round(io.bytes_recv / (1024 ** 2), 1),
        "packets_sent": io.packets_sent,
        "packets_recv": io.packets_recv,
        "errors_in": io.errin,
        "errors_out": io.errout,
    }


def detect_gaming() -> Dict:
    """Detect running gaming-related processes."""
    gaming: List[Dict] = []
    for proc in psutil.process_iter(["pid", "name", "cpu_percent", "memory_percent"]):
        try:
            name_lower = proc.info["name"].lower()
            if any(g in name_lower for g in GAMING_PROCESSES):
                gaming.append({
                    "pid": proc.info["pid"],
                    "name": proc.info["name"],
                    "cpu_percent": proc.info["cpu_percent"] or 0,
                    "memory_percent": round(proc.info["memory_percent"] or 0, 1),
                })
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    return {
        "active": len(gaming) > 0,
        "process_count": len(gaming),
        "processes": gaming,
    }


def get_top_processes(n: int = 15) -> List[Dict]:
    """Get top N processes by CPU usage."""
    procs: List[Dict] = []
    for proc in psutil.process_iter(["pid", "name", "cpu_percent", "memory_percent", "username"]):
        try:
            procs.append({
                "pid": proc.info["pid"],
                "name": proc.info["name"],
                "cpu_percent": proc.info["cpu_percent"] or 0,
                "memory_percent": round(proc.info["memory_percent"] or 0, 1),
                "user": proc.info["username"],
            })
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    procs.sort(key=lambda x: x["cpu_percent"], reverse=True)
    return procs[:n]


def generate_recommendations() -> List[Dict]:
    """Generate performance recommendations based on current system state."""
    recs: List[Dict] = []

    mem = psutil.virtual_memory()
    if mem.percent > 90:
        recs.append({
            "severity": "critical",
            "category": "memory",
            "title": f"Critical memory usage: {mem.percent}%",
            "description": "Close unused applications or add more RAM",
        })
    elif mem.percent > 80:
        recs.append({
            "severity": "warning",
            "category": "memory",
            "title": f"High memory usage: {mem.percent}%",
            "description": "Monitor for memory leaks in top processes",
        })

    cpu = psutil.cpu_percent(interval=0.5)
    if cpu > 90:
        recs.append({
            "severity": "critical",
            "category": "cpu",
            "title": f"CPU usage at {cpu}%",
            "description": "Check for runaway processes",
        })

    gpu = get_gpu_stats()
    if gpu:
        if gpu["temperature_c"] > 85:
            recs.append({
                "severity": "warning",
                "category": "gpu",
                "title": f"GPU temperature: {gpu['temperature_c']}°C",
                "description": "Improve airflow or reduce GPU load",
            })
        if gpu["memory_used_mb"] > gpu["memory_total_mb"] * 0.9:
            recs.append({
                "severity": "warning",
                "category": "gpu",
                "title": "GPU VRAM nearly full",
                "description": f"{gpu['memory_used_mb']:.0f}/{gpu['memory_total_mb']:.0f} MB used",
            })

    gaming = detect_gaming()
    if gaming["active"]:
        recs.append({
            "severity": "info",
            "category": "gaming",
            "title": f"Gaming mode active ({gaming['process_count']} processes)",
            "description": "Performance profiles optimized for gaming",
        })

    load = os.getloadavg()
    cores = psutil.cpu_count() or 1
    if load[0] > cores * 2:
        recs.append({
            "severity": "warning",
            "category": "load",
            "title": f"High system load: {load[0]:.1f} (vs {cores} cores)",
            "description": "System is overloaded — check process list",
        })

    swap = psutil.swap_memory()
    if swap.percent > 50:
        recs.append({
            "severity": "warning",
            "category": "swap",
            "title": f"Heavy swap usage: {swap.percent}%",
            "description": "System is swapping — add RAM or reduce workload",
        })

    return recs


# ═══════════════════════════════════════════════════════════════════════════
# API ROUTES
# ═══════════════════════════════════════════════════════════════════════════


@app.get("/api/health")
async def health():
    return {
        "service": "maxjr",
        "status": "running",
        "version": "2025.1",
        "timestamp": datetime.now().isoformat(),
        "uptime_seconds": int((datetime.now() - _start_time).total_seconds()),
    }


@app.get("/api/metrics")
async def get_all_metrics():
    return {
        "timestamp": datetime.now().isoformat(),
        "cpu": get_cpu_stats(),
        "memory": get_memory_stats(),
        "gpu": get_gpu_stats(),
        "temperatures": get_temperatures(),
        "disk_io": get_disk_io(),
        "network": get_network_stats(),
        "gaming": detect_gaming(),
        "top_processes": get_top_processes(),
        "recommendations": generate_recommendations(),
    }


@app.get("/api/cpu")
async def cpu():
    return get_cpu_stats()


@app.get("/api/memory")
async def memory():
    return get_memory_stats()


@app.get("/api/gpu")
async def gpu():
    stats = get_gpu_stats()
    return stats if stats else {"available": False}


@app.get("/api/temperatures")
async def temperatures():
    return get_temperatures()


@app.get("/api/network")
async def network():
    return get_network_stats()


@app.get("/api/gaming")
async def gaming():
    return detect_gaming()


@app.get("/api/processes")
async def processes():
    return {"processes": get_top_processes(25)}


@app.get("/api/recommendations")
async def recommendations():
    return {"recommendations": generate_recommendations()}


if __name__ == "__main__":
    import uvicorn

    logger.info("Max Jr. Performance Optimizer starting on port 8602")
    uvicorn.run(app, host="0.0.0.0", port=8602, log_level="info")
