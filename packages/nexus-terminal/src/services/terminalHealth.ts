import { invoke } from '@tauri-apps/api/core';

export interface TerminalHealthMetrics {
  // Performance Metrics
  memoryUsage: {
    rss: number; // Resident Set Size
    heapTotal: number;
    heapUsed: number;
    external: number;
    arrayBuffers: number;
  };
  
  cpuUsage: {
    user: number;
    system: number;
    percent: number;
  };
  
  // Terminal Specific Metrics
  commandStats: {
    totalCommands: number;
    commandsPerMinute: number;
    averageExecutionTime: number;
    failureRate: number;
    mostUsedCommands: Array<{ command: string; count: number }>;
  };
  
  sessionStats: {
    uptime: number;
    totalSessions: number;
    activeSessions: number;
    averageSessionLength: number;
  };
  
  // System Health
  systemHealth: {
    diskSpace: {
      total: number;
      used: number;
      available: number;
      percentUsed: number;
    };
    networkStats: {
      bytesReceived: number;
      bytesSent: number;
      packetsReceived: number;
      packetsSent: number;
    };
    loadAverage: number[];
  };
  
  // Terminal UI Health
  renderingStats: {
    fps: number;
    frameTime: number;
    droppedFrames: number;
    scrollbackBufferSize: number;
  };
  
  timestamp: Date;
}

export interface HealthAlert {
  id: string;
  type: 'warning' | 'error' | 'critical';
  category: 'performance' | 'memory' | 'disk' | 'network' | 'system';
  message: string;
  details: string;
  timestamp: Date;
  acknowledged: boolean;
  resolved: boolean;
  resolvedAt?: Date;
  actions?: string[];
}

export interface HealthThresholds {
  memory: {
    warningMB: number;
    criticalMB: number;
  };
  cpu: {
    warningPercent: number;
    criticalPercent: number;
  };
  disk: {
    warningPercent: number;
    criticalPercent: number;
  };
  commandFailureRate: {
    warningPercent: number;
    criticalPercent: number;
  };
  fps: {
    warningFPS: number;
    criticalFPS: number;
  };
  responseTime: {
    warningMS: number;
    criticalMS: number;
  };
}

export interface HealthTrend {
  metric: string;
  values: Array<{ timestamp: Date; value: number }>;
  trend: 'improving' | 'stable' | 'degrading';
  prediction?: number; // Predicted value for next measurement
}

export class TerminalHealthMonitor {
  private metrics: TerminalHealthMetrics[] = [];
  private alerts: HealthAlert[] = [];
  private isMonitoring = false;
  private monitoringInterval?: number;
  private thresholds: HealthThresholds;
  
  constructor() {
    this.thresholds = this.getDefaultThresholds();
  }
  
  async startMonitoring(intervalSeconds: number = 30): Promise<void> {
    if (this.isMonitoring) return;
    
    this.isMonitoring = true;
    
    // Initial collection
    await this.collectMetrics();
    
    // Set up periodic collection
    this.monitoringInterval = window.setInterval(async () => {
      await this.collectMetrics();
      this.checkHealth();
    }, intervalSeconds * 1000);
  }
  
  stopMonitoring(): void {
    if (!this.isMonitoring) return;
    
    this.isMonitoring = false;
    
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = undefined;
    }
  }
  
  async collectMetrics(): Promise<TerminalHealthMetrics> {
    try {
      const metrics = await invoke<TerminalHealthMetrics>('get_health_metrics');
      
      // Store metrics (keep last 1000 entries)
      this.metrics.push(metrics);
      if (this.metrics.length > 1000) {
        this.metrics.shift();
      }
      
      return metrics;
    } catch (error) {
      throw new Error(`Failed to collect metrics: ${error}`);
    }
  }
  
  getLatestMetrics(): TerminalHealthMetrics | null {
    return this.metrics[this.metrics.length - 1] || null;
  }
  
  getMetricsHistory(hours: number = 1): TerminalHealthMetrics[] {
    const cutoff = new Date(Date.now() - hours * 60 * 60 * 1000);
    return this.metrics.filter(m => m.timestamp > cutoff);
  }
  
  private checkHealth(): void {
    const latest = this.getLatestMetrics();
    if (!latest) return;
    
    this.checkMemoryHealth(latest);
    this.checkCPUHealth(latest);
    this.checkDiskHealth(latest);
    this.checkPerformanceHealth(latest);
    this.checkRenderingHealth(latest);
    // this.checkCommandHealth(latest);
  }
  
  private checkMemoryHealth(metrics: TerminalHealthMetrics): void {
    const memoryMB = metrics.memoryUsage.rss / 1024 / 1024;
    
    if (memoryMB > this.thresholds.memory.criticalMB) {
      this.createAlert('critical', 'memory', 
        `Critical memory usage: ${memoryMB.toFixed(1)}MB`,
        'Terminal is using excessive memory and may become unstable',
        ['Restart terminal', 'Clear command history', 'Check for memory leaks']
      );
    } else if (memoryMB > this.thresholds.memory.warningMB) {
      this.createAlert('warning', 'memory',
        `High memory usage: ${memoryMB.toFixed(1)}MB`,
        'Terminal memory usage is elevated',
        ['Monitor closely', 'Clear unnecessary buffers']
      );
    }
  }
  
  private checkCPUHealth(metrics: TerminalHealthMetrics): void {
    const cpuPercent = metrics.cpuUsage.percent;
    
    if (cpuPercent > this.thresholds.cpu.criticalPercent) {
      this.createAlert('critical', 'performance',
        `Critical CPU usage: ${cpuPercent.toFixed(1)}%`,
        'Terminal is consuming excessive CPU resources',
        ['Check for runaway processes', 'Restart terminal', 'Review recent commands']
      );
    } else if (cpuPercent > this.thresholds.cpu.warningPercent) {
      this.createAlert('warning', 'performance',
        `High CPU usage: ${cpuPercent.toFixed(1)}%`,
        'Terminal CPU usage is elevated',
        ['Monitor system processes', 'Check for background tasks']
      );
    }
  }
  
  private checkDiskHealth(metrics: TerminalHealthMetrics): void {
    const diskPercent = metrics.systemHealth.diskSpace.percentUsed;
    
    if (diskPercent > this.thresholds.disk.criticalPercent) {
      this.createAlert('critical', 'disk',
        `Critical disk usage: ${diskPercent.toFixed(1)}%`,
        'Disk space is critically low',
        ['Free disk space', 'Clean temporary files', 'Archive old logs']
      );
    } else if (diskPercent > this.thresholds.disk.warningPercent) {
      this.createAlert('warning', 'disk',
        `High disk usage: ${diskPercent.toFixed(1)}%`,
        'Available disk space is running low',
        ['Monitor disk usage', 'Clean unnecessary files']
      );
    }
  }
  
  private checkPerformanceHealth(metrics: TerminalHealthMetrics): void {
    const avgExecTime = metrics.commandStats.averageExecutionTime;
    
    if (avgExecTime > this.thresholds.responseTime.criticalMS) {
      this.createAlert('error', 'performance',
        `Slow command execution: ${avgExecTime.toFixed(0)}ms average`,
        'Commands are taking unusually long to execute',
        ['Check system load', 'Review recent commands', 'Restart terminal']
      );
    } else if (avgExecTime > this.thresholds.responseTime.warningMS) {
      this.createAlert('warning', 'performance',
        `Elevated command execution time: ${avgExecTime.toFixed(0)}ms average`,
        'Command execution is slower than normal',
        ['Monitor system performance']
      );
    }
    
    const failureRate = metrics.commandStats.failureRate * 100;
    if (failureRate > this.thresholds.commandFailureRate.criticalPercent) {
      this.createAlert('error', 'system',
        `High command failure rate: ${failureRate.toFixed(1)}%`,
        'Many commands are failing to execute properly',
        ['Check system health', 'Review command syntax', 'Check permissions']
      );
    }
  }
  
  private checkRenderingHealth(metrics: TerminalHealthMetrics): void {
    const fps = metrics.renderingStats.fps;
    const droppedFrames = metrics.renderingStats.droppedFrames;
    
    if (fps < this.thresholds.fps.criticalFPS) {
      this.createAlert('error', 'performance',
        `Low rendering performance: ${fps.toFixed(1)} FPS`,
        'Terminal rendering is severely degraded',
        ['Reduce scrollback buffer', 'Check GPU acceleration', 'Restart terminal']
      );
    } else if (fps < this.thresholds.fps.warningFPS) {
      this.createAlert('warning', 'performance',
        `Reduced rendering performance: ${fps.toFixed(1)} FPS`,
        'Terminal rendering performance is below optimal',
        ['Monitor GPU usage', 'Check for background processes']
      );
    }
    
    if (droppedFrames > 10) {
      this.createAlert('warning', 'performance',
        `Dropped frames detected: ${droppedFrames}`,
        'Terminal is dropping rendering frames',
        ['Reduce terminal load', 'Check system resources']
      );
    }
  }
  
  private createAlert(type: HealthAlert['type'], category: HealthAlert['category'], 
                     message: string, details: string, actions?: string[]): void {
    // Check if similar alert already exists and is not resolved
    const existingAlert = this.alerts.find(a => 
      !a.resolved && a.category === category && a.message === message
    );
    
    if (existingAlert) return;
    
    const alert: HealthAlert = {
      id: this.generateAlertId(),
      type,
      category,
      message,
      details,
      timestamp: new Date(),
      acknowledged: false,
      resolved: false,
      actions
    };
    
    this.alerts.push(alert);
    
    // Keep only last 100 alerts
    if (this.alerts.length > 100) {
      this.alerts.shift();
    }
  }
  
  private generateAlertId(): string {
    return `alert_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  getActiveAlerts(): HealthAlert[] {
    return this.alerts.filter(a => !a.resolved);
  }
  
  getAllAlerts(): HealthAlert[] {
    return [...this.alerts];
  }
  
  acknowledgeAlert(alertId: string): void {
    const alert = this.alerts.find(a => a.id === alertId);
    if (alert) {
      alert.acknowledged = true;
    }
  }
  
  resolveAlert(alertId: string): void {
    const alert = this.alerts.find(a => a.id === alertId);
    if (alert) {
      alert.resolved = true;
      alert.resolvedAt = new Date();
    }
  }
  
  getHealthScore(): number {
    const latest = this.getLatestMetrics();
    if (!latest) return 100;
    
    let score = 100;
    
    // Memory score (0-25 points)
    const memoryMB = latest.memoryUsage.rss / 1024 / 1024;
    const memoryScore = Math.max(0, 25 - (memoryMB / this.thresholds.memory.criticalMB) * 25);
    
    // CPU score (0-25 points)
    const cpuScore = Math.max(0, 25 - (latest.cpuUsage.percent / 100) * 25);
    
    // Performance score (0-25 points)
    const perfScore = Math.max(0, 25 - (latest.commandStats.failureRate * 25));
    
    // Rendering score (0-25 points)
    const renderScore = Math.max(0, (latest.renderingStats.fps / 60) * 25);
    
    score = memoryScore + cpuScore + perfScore + renderScore;
    
    // Penalize for active alerts
    const activeAlerts = this.getActiveAlerts();
    const criticalAlerts = activeAlerts.filter(a => a.type === 'critical').length;
    const errorAlerts = activeAlerts.filter(a => a.type === 'error').length;
    const warningAlerts = activeAlerts.filter(a => a.type === 'warning').length;
    
    score -= criticalAlerts * 15;
    score -= errorAlerts * 10;
    score -= warningAlerts * 5;
    
    return Math.max(0, Math.min(100, score));
  }
  
  getTrends(hours: number = 1): HealthTrend[] {
    const history = this.getMetricsHistory(hours);
    if (history.length < 2) return [];
    
    const trends: HealthTrend[] = [];
    
    // Memory trend
    trends.push(this.calculateTrend('memory', history.map(h => ({
      timestamp: h.timestamp,
      value: h.memoryUsage.rss / 1024 / 1024
    }))));
    
    // CPU trend
    trends.push(this.calculateTrend('cpu', history.map(h => ({
      timestamp: h.timestamp,
      value: h.cpuUsage.percent
    }))));
    
    // FPS trend
    trends.push(this.calculateTrend('fps', history.map(h => ({
      timestamp: h.timestamp,
      value: h.renderingStats.fps
    }))));
    
    return trends;
  }
  
  private calculateTrend(metric: string, values: Array<{ timestamp: Date; value: number }>): HealthTrend {
    if (values.length < 2) {
      return { metric, values, trend: 'stable' };
    }
    
    // Simple linear regression for trend
    const n = values.length;
    const sumX = values.reduce((sum, _, i) => sum + i, 0);
    const sumY = values.reduce((sum, v) => sum + v.value, 0);
    const sumXY = values.reduce((sum, v, i) => sum + i * v.value, 0);
    const sumX2 = values.reduce((sum, _, i) => sum + i * i, 0);
    
    const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
    
    let trend: 'improving' | 'stable' | 'degrading';
    if (Math.abs(slope) < 0.1) {
      trend = 'stable';
    } else if ((metric === 'fps' && slope > 0) || (metric !== 'fps' && slope < 0)) {
      trend = 'improving';
    } else {
      trend = 'degrading';
    }
    
    return { metric, values, trend };
  }
  
  async runHealthCheck(): Promise<HealthCheckReport> {
    const metrics = await this.collectMetrics();
    const healthScore = this.getHealthScore();
    const activeAlerts = this.getActiveAlerts();
    const trends = this.getTrends();
    
    return {
      healthScore,
      status: this.getHealthStatus(healthScore),
      metrics,
      activeAlerts,
      trends,
      recommendations: this.getRecommendations(metrics, activeAlerts),
      timestamp: new Date()
    };
  }
  
  private getHealthStatus(score: number): 'excellent' | 'good' | 'fair' | 'poor' | 'critical' {
    if (score >= 90) return 'excellent';
    if (score >= 75) return 'good';
    if (score >= 50) return 'fair';
    if (score >= 25) return 'poor';
    return 'critical';
  }
  
  private getRecommendations(metrics: TerminalHealthMetrics, alerts: HealthAlert[]): string[] {
    const recommendations: string[] = [];
    
    if (metrics.memoryUsage.rss / 1024 / 1024 > 500) {
      recommendations.push('Consider reducing scrollback buffer size to free memory');
    }
    
    if (metrics.commandStats.failureRate > 0.1) {
      recommendations.push('Review recent command failures and syntax');
    }
    
    if (metrics.renderingStats.fps < 30) {
      recommendations.push('Enable GPU acceleration if available');
    }
    
    if (alerts.filter(a => a.type === 'critical').length > 0) {
      recommendations.push('Address critical alerts immediately');
    }
    
    if (metrics.systemHealth.diskSpace.percentUsed > 90) {
      recommendations.push('Clean up disk space to prevent system issues');
    }
    
    return recommendations;
  }
  
  private getDefaultThresholds(): HealthThresholds {
    return {
      memory: {
        warningMB: 512,
        criticalMB: 1024
      },
      cpu: {
        warningPercent: 70,
        criticalPercent: 90
      },
      disk: {
        warningPercent: 80,
        criticalPercent: 95
      },
      commandFailureRate: {
        warningPercent: 10,
        criticalPercent: 25
      },
      fps: {
        warningFPS: 30,
        criticalFPS: 15
      },
      responseTime: {
        warningMS: 1000,
        criticalMS: 3000
      }
    };
  }
  
  updateThresholds(thresholds: Partial<HealthThresholds>): void {
    this.thresholds = { ...this.thresholds, ...thresholds };
  }
  
  exportHealthData(): string {
    return JSON.stringify({
      metrics: this.metrics,
      alerts: this.alerts,
      thresholds: this.thresholds
    }, null, 2);
  }
  
  async generateHealthReport(format: 'json' | 'html' | 'pdf' = 'json'): Promise<string> {
    const healthCheck = await this.runHealthCheck();
    
    if (format === 'json') {
      return JSON.stringify(healthCheck, null, 2);
    }
    
    try {
      return await invoke<string>('generate_health_report', { healthCheck, format });
    } catch (error) {
      throw new Error(`Failed to generate health report: ${error}`);
    }
  }
}

export interface HealthCheckReport {
  healthScore: number;
  status: 'excellent' | 'good' | 'fair' | 'poor' | 'critical';
  metrics: TerminalHealthMetrics;
  activeAlerts: HealthAlert[];
  trends: HealthTrend[];
  recommendations: string[];
  timestamp: Date;
}

export const terminalHealthMonitor = new TerminalHealthMonitor();
