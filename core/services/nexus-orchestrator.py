#!/usr/bin/env python3

"""
NexusOS Service Orchestrator
AI-powered unified service management with Stella and Max Jr.
Coordinates gaming, media, system, and security services
Version: 2024.1 Stellar Edition
"""

import asyncio
import json
import logging
import os
import subprocess
import time
import yaml
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from enum import Enum

import psutil
import docker
from fastapi import FastAPI, WebSocket
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel


class ServiceType(Enum):
    GAMING = "gaming"
    MEDIA = "media"
    DESKTOP = "desktop"
    SECURITY = "security"
    DEVELOPMENT = "development"
    SYSTEM = "system"


class ServiceStatus(Enum):
    RUNNING = "running"
    STOPPED = "stopped"
    STARTING = "starting"
    STOPPING = "stopping"
    ERROR = "error"
    UNKNOWN = "unknown"


@dataclass
class SystemMetrics:
    timestamp: datetime
    cpu_usage: float
    memory_usage: float
    disk_usage: float
    gpu_usage: float
    temperature: float
    network_activity: float
    gaming_active: bool
    media_active: bool


@dataclass
class AIRecommendation:
    source: str  # 'stella' or 'maxjr'
    type: str
    title: str
    description: str
    confidence: float
    action: Dict[str, Any]
    priority: int


class ServiceInfo(BaseModel):
    name: str
    type: ServiceType
    status: ServiceStatus
    port: Optional[int] = None
    url: Optional[str] = None
    description: str
    ai_monitoring: str  # 'stella', 'maxjr', or 'both'
    health_score: float = 1.0
    last_seen: datetime


class NexusOrchestrator:
    def __init__(self):
        self.version = "2024.1-stellar"
        self.base_dir = Path("/opt/nexus-os")
        self.config_dir = Path("/etc/nexus-os")
        self.log_dir = Path("/var/log/nexus-os")
        
        # Setup logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(self.log_dir / "orchestrator.log"),
                logging.StreamHandler()
            ]
        )
        self.logger = logging.getLogger("NexusOrchestrator")
        
        # Initialize components
        self.docker_client = docker.from_env()
        self.services: Dict[str, ServiceInfo] = {}
        self.system_metrics: List[SystemMetrics] = []
        self.ai_recommendations: List[AIRecommendation] = []
        
        # AI Assistant states
        self.stella_active = True
        self.maxjr_active = True
        
        # Load configuration
        self.load_configuration()
        
        # FastAPI app for web interface
        self.app = FastAPI(title="NexusOS Orchestrator", version=self.version)
        self.setup_api_routes()
        
        self.logger.info("🚀 NexusOS Orchestrator initialized")
        self.logger.info("🛡️ Stella: Security & Media monitoring active")
        self.logger.info("⚡ Max Jr.: Gaming & Performance optimization active")

    def load_configuration(self):
        """Load NexusOS configuration from YAML files"""
        try:
            config_file = self.config_dir / "nexus-packages.yml"
            if config_file.exists():
                with open(config_file, 'r') as f:
                    self.config = yaml.safe_load(f)
                self.logger.info("Configuration loaded successfully")
            else:
                self.logger.warning("Configuration file not found, using defaults")
                self.config = {}
        except Exception as e:
            self.logger.error(f"Failed to load configuration: {e}")
            self.config = {}

    async def discover_services(self):
        """Discover all NexusOS services across different categories"""
        self.logger.info("🔍 Discovering NexusOS services...")
        
        # Discover Docker containers (Media Stack)
        await self._discover_docker_services()
        
        # Discover systemd services (Gaming & Desktop)
        await self._discover_systemd_services()
        
        # Discover NexusDE components
        await self._discover_nexusde_services()
        
        self.logger.info(f"Discovered {len(self.services)} services")

    async def _discover_docker_services(self):
        """Discover Docker-based media stack services"""
        try:
            containers = self.docker_client.containers.list(all=True)
            for container in containers:
                if container.name.startswith('nexus-') or 'nexus' in container.labels.get('nexus.service', ''):
                    service_type = ServiceType.MEDIA
                    if 'gaming' in container.name or container.labels.get('nexus.category') == 'gaming':
                        service_type = ServiceType.GAMING
                    elif 'security' in container.name:
                        service_type = ServiceType.SECURITY
                    
                    # Determine AI monitoring
                    ai_monitoring = container.labels.get('nexus.ai.monitoring', 'stella')
                    
                    # Get port mapping
                    port = None
                    url = None
                    if container.ports:
                        for container_port, host_bindings in container.ports.items():
                            if host_bindings:
                                port = int(host_bindings[0]['HostPort'])
                                url = f"http://localhost:{port}"
                                break
                    
                    status = ServiceStatus.RUNNING if container.status == 'running' else ServiceStatus.STOPPED
                    
                    self.services[container.name] = ServiceInfo(
                        name=container.name,
                        type=service_type,
                        status=status,
                        port=port,
                        url=url,
                        description=container.labels.get('description', f"{container.name} service"),
                        ai_monitoring=ai_monitoring,
                        last_seen=datetime.now()
                    )
                    
        except Exception as e:
            self.logger.error(f"Error discovering Docker services: {e}")

    async def _discover_systemd_services(self):
        """Discover systemd services for gaming and system components"""
        gaming_services = [
            'gamemode', 'irqbalance', 'thermald', 'steam', 'lutris'
        ]
        
        desktop_services = [
            'nexusde', 'sddm', 'NetworkManager', 'bluetooth'
        ]
        
        security_services = [
            'garuda-hello-daemon', 'ufw', 'fail2ban', 'clamav-daemon'
        ]
        
        for service_name in gaming_services:
            await self._check_systemd_service(service_name, ServiceType.GAMING, 'maxjr')
            
        for service_name in desktop_services:
            await self._check_systemd_service(service_name, ServiceType.DESKTOP, 'both')
            
        for service_name in security_services:
            await self._check_systemd_service(service_name, ServiceType.SECURITY, 'stella')

    async def _check_systemd_service(self, service_name: str, service_type: ServiceType, ai_monitoring: str):
        """Check status of a systemd service"""
        try:
            result = subprocess.run(
                ['systemctl', 'is-active', service_name],
                capture_output=True,
                text=True
            )
            
            status = ServiceStatus.RUNNING if result.stdout.strip() == 'active' else ServiceStatus.STOPPED
            
            self.services[service_name] = ServiceInfo(
                name=service_name,
                type=service_type,
                status=status,
                description=f"System service: {service_name}",
                ai_monitoring=ai_monitoring,
                last_seen=datetime.now()
            )
            
        except Exception as e:
            self.logger.error(f"Error checking service {service_name}: {e}")

    async def _discover_nexusde_services(self):
        """Discover NexusDE desktop environment components"""
        nexusde_services = [
            ('nexusde-compositor', 'NexusDE Compositor'),
            ('nexusde-window-manager', 'NexusDE Window Manager'),
            ('nexusde-shell', 'NexusDE Shell'),
            ('nexusde-session-manager', 'NexusDE Session Manager'),
            ('stella-security-ai', 'Stella Security AI'),
            ('maxjr-system-ai', 'Max Jr. System AI')
        ]
        
        for service_name, description in nexusde_services:
            # For now, assume they're running if NexusDE is active
            ai_monitoring = 'both'
            if 'stella' in service_name:
                ai_monitoring = 'stella'
            elif 'maxjr' in service_name:
                ai_monitoring = 'maxjr'
            
            self.services[service_name] = ServiceInfo(
                name=service_name,
                type=ServiceType.DESKTOP,
                status=ServiceStatus.RUNNING,  # Simplified for demo
                description=description,
                ai_monitoring=ai_monitoring,
                last_seen=datetime.now()
            )

    async def collect_system_metrics(self):
        """Collect comprehensive system metrics"""
        try:
            # CPU and Memory
            cpu_usage = psutil.cpu_percent(interval=1)
            memory = psutil.virtual_memory()
            memory_usage = memory.percent
            
            # Disk usage
            disk = psutil.disk_usage('/')
            disk_usage = (disk.used / disk.total) * 100
            
            # Network activity (simplified)
            network_io = psutil.net_io_counters()
            network_activity = (network_io.bytes_sent + network_io.bytes_recv) / 1024 / 1024  # MB
            
            # GPU usage (simplified - would need GPU-specific libraries)
            gpu_usage = self._get_gpu_usage()
            
            # Temperature (simplified)
            temperature = self._get_cpu_temperature()
            
            # Gaming/Media activity detection
            gaming_active = self._detect_gaming_activity()
            media_active = self._detect_media_activity()
            
            metrics = SystemMetrics(
                timestamp=datetime.now(),
                cpu_usage=cpu_usage,
                memory_usage=memory_usage,
                disk_usage=disk_usage,
                gpu_usage=gpu_usage,
                temperature=temperature,
                network_activity=network_activity,
                gaming_active=gaming_active,
                media_active=media_active
            )
            
            self.system_metrics.append(metrics)
            
            # Keep only last 1000 metrics
            if len(self.system_metrics) > 1000:
                self.system_metrics = self.system_metrics[-1000:]
                
            return metrics
            
        except Exception as e:
            self.logger.error(f"Error collecting system metrics: {e}")
            return None

    def _get_gpu_usage(self) -> float:
        """Get GPU usage percentage"""
        try:
            # Try nvidia-smi first
            result = subprocess.run(
                ['nvidia-smi', '--query-gpu=utilization.gpu', '--format=csv,noheader,nounits'],
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                return float(result.stdout.strip())
        except:
            pass
        
        # Fallback or AMD detection could go here
        return 0.0

    def _get_cpu_temperature(self) -> float:
        """Get CPU temperature"""
        try:
            temps = psutil.sensors_temperatures()
            if 'coretemp' in temps:
                return temps['coretemp'][0].current
            elif 'acpi' in temps:
                return temps['acpi'][0].current
        except:
            pass
        return 0.0

    def _detect_gaming_activity(self) -> bool:
        """Detect if gaming applications are running"""
        gaming_processes = ['steam', 'lutris', 'wine', 'proton', 'gamescope']
        
        for proc in psutil.process_iter(['name']):
            try:
                if any(game in proc.info['name'].lower() for game in gaming_processes):
                    return True
            except:
                continue
        return False

    def _detect_media_activity(self) -> bool:
        """Detect if media services are active"""
        try:
            # Check if media containers are running
            media_containers = [c for c in self.docker_client.containers.list() 
                             if 'media' in c.name or any(service in c.name for service in 
                             ['plex', 'jellyfin', 'sonarr', 'radarr'])]
            return len(media_containers) > 0
        except:
            return False

    async def stella_analysis(self, metrics: SystemMetrics) -> List[AIRecommendation]:
        """Stella's security and media analysis"""
        recommendations = []
        
        # Security analysis
        if metrics.cpu_usage > 80:
            recommendations.append(AIRecommendation(
                source="stella",
                type="security",
                title="High CPU Usage Detected",
                description="Monitoring for potential security threats or resource abuse",
                confidence=0.7,
                action={"type": "monitor", "service": "cpu"},
                priority=2
            ))
        
        # Media stack optimization
        if metrics.media_active and metrics.memory_usage > 85:
            recommendations.append(AIRecommendation(
                source="stella",
                type="media",
                title="Media Services Memory Optimization",
                description="Media services are consuming high memory. Consider transcoding optimization.",
                confidence=0.8,
                action={"type": "optimize", "service": "media", "action": "transcode_settings"},
                priority=1
            ))
        
        # Backup reminder
        current_hour = datetime.now().hour
        if current_hour == 2:  # 2 AM backup time
            recommendations.append(AIRecommendation(
                source="stella",
                type="security",
                title="Scheduled Backup Check",
                description="Ensuring Garuda Ultimate Restore System backups are current",
                confidence=1.0,
                action={"type": "backup", "service": "restore_system"},
                priority=1
            ))
        
        return recommendations

    async def maxjr_analysis(self, metrics: SystemMetrics) -> List[AIRecommendation]:
        """Max Jr.'s gaming and performance analysis"""
        recommendations = []
        
        # Gaming performance optimization
        if metrics.gaming_active:
            if metrics.gpu_usage < 50 and metrics.cpu_usage > 70:
                recommendations.append(AIRecommendation(
                    source="maxjr",
                    type="gaming",
                    title="GPU Underutilization During Gaming",
                    description="CPU bottleneck detected. Consider GPU workload shifting.",
                    confidence=0.9,
                    action={"type": "optimize", "service": "gpu", "action": "increase_gpu_load"},
                    priority=1
                ))
            
            if metrics.temperature > 80:
                recommendations.append(AIRecommendation(
                    source="maxjr",
                    type="performance",
                    title="Thermal Throttling Prevention",
                    description="High temperatures detected. Adjusting performance profiles.",
                    confidence=0.95,
                    action={"type": "thermal", "service": "cooling", "action": "increase_fan_speed"},
                    priority=1
                ))
        
        # System optimization
        if metrics.memory_usage > 90:
            recommendations.append(AIRecommendation(
                source="maxjr",
                type="performance",
                title="Memory Pressure Relief",
                description="High memory usage detected. Optimizing system processes.",
                confidence=0.8,
                action={"type": "optimize", "service": "memory", "action": "clear_cache"},
                priority=2
            ))
        
        return recommendations

    async def process_ai_recommendations(self):
        """Process and execute AI recommendations from Stella and Max Jr."""
        if not self.system_metrics:
            return
        
        latest_metrics = self.system_metrics[-1]
        
        # Get recommendations from both AI assistants
        if self.stella_active:
            stella_recs = await self.stella_analysis(latest_metrics)
            self.ai_recommendations.extend(stella_recs)
        
        if self.maxjr_active:
            maxjr_recs = await self.maxjr_analysis(latest_metrics)
            self.ai_recommendations.extend(maxjr_recs)
        
        # Sort by priority and execute top recommendations
        self.ai_recommendations.sort(key=lambda x: x.priority)
        
        # Execute high-priority recommendations
        for rec in self.ai_recommendations[:3]:  # Top 3 recommendations
            if rec.priority == 1:
                await self._execute_recommendation(rec)
        
        # Keep only last 100 recommendations
        if len(self.ai_recommendations) > 100:
            self.ai_recommendations = self.ai_recommendations[-100:]

    async def _execute_recommendation(self, rec: AIRecommendation):
        """Execute an AI recommendation"""
        try:
            self.logger.info(f"🤖 {rec.source.upper()}: Executing recommendation - {rec.title}")
            
            action_type = rec.action.get('type')
            service = rec.action.get('service')
            
            if action_type == 'optimize':
                await self._optimize_service(service, rec.action.get('action'))
            elif action_type == 'monitor':
                await self._monitor_service(service)
            elif action_type == 'backup':
                await self._trigger_backup()
            elif action_type == 'thermal':
                await self._adjust_thermal_settings(rec.action.get('action'))
                
        except Exception as e:
            self.logger.error(f"Error executing recommendation: {e}")

    async def _optimize_service(self, service: str, action: str):
        """Optimize a specific service"""
        self.logger.info(f"Optimizing {service} with action: {action}")
        # Implementation would depend on specific service
        
    async def _monitor_service(self, service: str):
        """Enhanced monitoring for a service"""
        self.logger.info(f"Enhanced monitoring activated for {service}")
        
    async def _trigger_backup(self):
        """Trigger backup system"""
        try:
            backup_script = Path("/opt/nexus-os/backup-system/scripts/daily-backup.sh")
            if backup_script.exists():
                subprocess.run([str(backup_script)], check=True)
                self.logger.info("🛡️ STELLA: Backup completed successfully")
        except Exception as e:
            self.logger.error(f"Backup failed: {e}")
            
    async def _adjust_thermal_settings(self, action: str):
        """Adjust thermal/performance settings"""
        self.logger.info(f"⚡ MAX JR.: Adjusting thermal settings - {action}")
        
    def setup_api_routes(self):
        """Setup FastAPI routes for web interface"""
        
        @self.app.get("/api/status")
        async def get_system_status():
            latest_metrics = self.system_metrics[-1] if self.system_metrics else None
            return {
                "version": self.version,
                "services_count": len(self.services),
                "stella_active": self.stella_active,
                "maxjr_active": self.maxjr_active,
                "latest_metrics": latest_metrics.__dict__ if latest_metrics else None,
                "recommendations_count": len(self.ai_recommendations)
            }
        
        @self.app.get("/api/services")
        async def get_services():
            return {name: service.__dict__ for name, service in self.services.items()}
        
        @self.app.get("/api/metrics")
        async def get_metrics():
            return [metrics.__dict__ for metrics in self.system_metrics[-50:]]
        
        @self.app.get("/api/recommendations")
        async def get_recommendations():
            return [rec.__dict__ for rec in self.ai_recommendations[-10:]]
        
        @self.app.post("/api/stella/toggle")
        async def toggle_stella():
            self.stella_active = not self.stella_active
            return {"stella_active": self.stella_active}
        
        @self.app.post("/api/maxjr/toggle")
        async def toggle_maxjr():
            self.maxjr_active = not self.maxjr_active
            return {"maxjr_active": self.maxjr_active}

    async def main_loop(self):
        """Main orchestration loop"""
        self.logger.info("🚀 Starting NexusOS orchestration loop")
        
        while True:
            try:
                # Discover services every 60 seconds
                await self.discover_services()
                
                # Collect metrics every 30 seconds
                metrics = await self.collect_system_metrics()
                
                if metrics:
                    # AI analysis every 60 seconds
                    await self.process_ai_recommendations()
                
                # Log status
                if self.stella_active or self.maxjr_active:
                    active_assistants = []
                    if self.stella_active:
                        active_assistants.append("🛡️ Stella")
                    if self.maxjr_active:
                        active_assistants.append("⚡ Max Jr.")
                    
                    self.logger.info(f"Active AI Assistants: {', '.join(active_assistants)}")
                
                await asyncio.sleep(30)  # 30-second cycle
                
            except Exception as e:
                self.logger.error(f"Error in main loop: {e}")
                await asyncio.sleep(10)


if __name__ == "__main__":
    import uvicorn
    
    orchestrator = NexusOrchestrator()
    
    # Start the orchestration loop in the background
    loop = asyncio.get_event_loop()
    loop.create_task(orchestrator.main_loop())
    
    # Start the web API
    uvicorn.run(
        orchestrator.app,
        host="0.0.0.0",
        port=8600,
        log_level="info"
    )