use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub ai_processing_time: Duration,
    pub file_operations_count: u64,
    pub cache_hit_rate: f64,
    pub active_connections: u32,
    pub request_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AIProcessingMetrics {
    pub model: String,
    pub provider: String,
    pub processing_time: Duration,
    pub tokens_processed: u32,
    pub success: bool,
    pub error_type: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileOperationMetrics {
    pub operation_type: String, // "read", "write", "search", "watch"
    pub file_path: String,
    pub duration: Duration,
    pub file_size: u64,
    pub success: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheMetrics {
    pub cache_type: String, // "ai_response", "file_content", "config"
    pub hit: bool,
    pub key_hash: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInteractionMetrics {
    pub action: String, // "code_analysis", "file_search", "config_update"
    pub duration: Duration,
    pub success: bool,
    pub user_id: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemHealthMetrics {
    pub component: String, // "database", "ai_service", "file_watcher"
    pub status: String,    // "healthy", "degraded", "unhealthy"
    pub response_time: Option<Duration>,
    pub error_message: Option<String>,
    pub timestamp: u64,
}

pub struct TelemetryManager {
    metrics_buffer: Arc<RwLock<Vec<PerformanceMetrics>>>,
    ai_metrics_buffer: Arc<RwLock<Vec<AIProcessingMetrics>>>,
    file_metrics_buffer: Arc<RwLock<Vec<FileOperationMetrics>>>,
    cache_metrics_buffer: Arc<RwLock<Vec<CacheMetrics>>>,
    user_metrics_buffer: Arc<RwLock<Vec<UserInteractionMetrics>>>,
    health_metrics_buffer: Arc<RwLock<Vec<SystemHealthMetrics>>>,
    aggregated_stats: Arc<RwLock<AggregatedStats>>,
    metrics_tx: mpsc::UnboundedSender<MetricEvent>,
    enabled: bool,
    retention_duration: Duration,
}

#[derive(Debug)]
pub enum MetricEvent {
    AIProcessing(AIProcessingMetrics),
    FileOperation(FileOperationMetrics),
    CacheAccess(CacheMetrics),
    UserInteraction(UserInteractionMetrics),
    SystemHealth(SystemHealthMetrics),
    FlushMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub total_requests: u64,
    pub avg_processing_time: f64,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
    pub peak_memory_usage: u64,
    pub uptime_seconds: u64,
    pub error_breakdown: HashMap<String, u64>,
    pub most_used_models: HashMap<String, u64>,
    pub file_operation_stats: HashMap<String, u64>,
}

impl Default for AggregatedStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            avg_processing_time: 0.0,
            success_rate: 100.0,
            cache_hit_rate: 0.0,
            peak_memory_usage: 0,
            uptime_seconds: 0,
            error_breakdown: HashMap::new(),
            most_used_models: HashMap::new(),
            file_operation_stats: HashMap::new(),
        }
    }
}

impl TelemetryManager {
    pub fn new(enabled: bool, retention_hours: u64) -> Self {
        let (metrics_tx, metrics_rx) = mpsc::unbounded_channel();
        let retention_duration = Duration::from_secs(retention_hours * 3600);

        let manager = Self {
            metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            ai_metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            file_metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            cache_metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            user_metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            health_metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            aggregated_stats: Arc::new(RwLock::new(AggregatedStats::default())),
            metrics_tx,
            enabled,
            retention_duration,
        };

        if enabled {
            manager.start_background_tasks(metrics_rx);
        }

        manager
    }

    pub fn record_ai_processing(&self, metrics: AIProcessingMetrics) {
        if !self.enabled {
            return;
        }

        if let Err(e) = self.metrics_tx.send(MetricEvent::AIProcessing(metrics)) {
            error!("Failed to send AI processing metrics: {}", e);
        }
    }

    pub fn record_file_operation(&self, metrics: FileOperationMetrics) {
        if !self.enabled {
            return;
        }

        if let Err(e) = self.metrics_tx.send(MetricEvent::FileOperation(metrics)) {
            error!("Failed to send file operation metrics: {}", e);
        }
    }

    pub fn record_cache_access(&self, metrics: CacheMetrics) {
        if !self.enabled {
            return;
        }

        if let Err(e) = self.metrics_tx.send(MetricEvent::CacheAccess(metrics)) {
            error!("Failed to send cache metrics: {}", e);
        }
    }

    pub fn record_user_interaction(&self, metrics: UserInteractionMetrics) {
        if !self.enabled {
            return;
        }

        if let Err(e) = self.metrics_tx.send(MetricEvent::UserInteraction(metrics)) {
            error!("Failed to send user interaction metrics: {}", e);
        }
    }

    pub fn record_system_health(&self, metrics: SystemHealthMetrics) {
        if !self.enabled {
            return;
        }

        if let Err(e) = self.metrics_tx.send(MetricEvent::SystemHealth(metrics)) {
            error!("Failed to send system health metrics: {}", e);
        }
    }

    pub fn get_current_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            cpu_usage: self.get_cpu_usage(),
            memory_usage: self.get_memory_usage(),
            memory_total: self.get_total_memory(),
            ai_processing_time: Duration::from_millis(0),
            file_operations_count: 0,
            cache_hit_rate: 0.0,
            active_connections: 0,
            request_count: 0,
            error_count: 0,
        }
    }

    pub async fn get_aggregated_stats(&self) -> AggregatedStats {
        self.aggregated_stats.read().await.clone()
    }

    pub async fn get_ai_metrics(&self, limit: Option<usize>) -> Vec<AIProcessingMetrics> {
        let buffer = self.ai_metrics_buffer.read().await;
        if let Some(limit) = limit {
            buffer.iter().rev().take(limit).cloned().collect()
        } else {
            buffer.clone()
        }
    }

    pub async fn get_file_metrics(&self, limit: Option<usize>) -> Vec<FileOperationMetrics> {
        let buffer = self.file_metrics_buffer.read().await;
        if let Some(limit) = limit {
            buffer.iter().rev().take(limit).cloned().collect()
        } else {
            buffer.clone()
        }
    }

    pub async fn get_health_status(&self) -> HashMap<String, String> {
        let mut status = HashMap::new();
        let health_buffer = self.health_metrics_buffer.read().await;

        // Get latest status for each component
        let mut component_status: HashMap<String, &SystemHealthMetrics> = HashMap::new();
        for metric in health_buffer.iter() {
            if let Some(existing) = component_status.get(&metric.component) {
                if metric.timestamp > existing.timestamp {
                    component_status.insert(metric.component.clone(), metric);
                }
            } else {
                component_status.insert(metric.component.clone(), metric);
            }
        }

        for (component, metric) in component_status {
            status.insert(component, metric.status.clone());
        }

        status
    }

    fn start_background_tasks(&self, mut metrics_rx: mpsc::UnboundedReceiver<MetricEvent>) {
        let metrics_buffer = Arc::clone(&self.metrics_buffer);
        let ai_metrics_buffer = Arc::clone(&self.ai_metrics_buffer);
        let file_metrics_buffer = Arc::clone(&self.file_metrics_buffer);
        let cache_metrics_buffer = Arc::clone(&self.cache_metrics_buffer);
        let user_metrics_buffer = Arc::clone(&self.user_metrics_buffer);
        let health_metrics_buffer = Arc::clone(&self.health_metrics_buffer);
        let aggregated_stats = Arc::clone(&self.aggregated_stats);
        let retention_duration = self.retention_duration;

        // Metrics collection task
        tokio::spawn(async move {
            while let Some(event) = metrics_rx.recv().await {
                match event {
                    MetricEvent::AIProcessing(metrics) => {
                        let mut buffer = ai_metrics_buffer.write().await;
                        buffer.push(metrics.clone());

                        // Update aggregated stats
                        let mut stats = aggregated_stats.write().await;
                        stats.total_requests += 1;
                        *stats.most_used_models.entry(metrics.model).or_insert(0) += 1;

                        if !metrics.success {
                            if let Some(error_type) = metrics.error_type {
                                *stats.error_breakdown.entry(error_type).or_insert(0) += 1;
                            }
                        }
                    }
                    MetricEvent::FileOperation(metrics) => {
                        let mut buffer = file_metrics_buffer.write().await;
                        buffer.push(metrics.clone());

                        let mut stats = aggregated_stats.write().await;
                        *stats.file_operation_stats.entry(metrics.operation_type).or_insert(0) += 1;
                    }
                    MetricEvent::CacheAccess(metrics) => {
                        let mut buffer = cache_metrics_buffer.write().await;
                        buffer.push(metrics);
                    }
                    MetricEvent::UserInteraction(metrics) => {
                        let mut buffer = user_metrics_buffer.write().await;
                        buffer.push(metrics);
                    }
                    MetricEvent::SystemHealth(metrics) => {
                        let mut buffer = health_metrics_buffer.write().await;
                        buffer.push(metrics);
                    }
                    MetricEvent::FlushMetrics => {
                        // Cleanup old metrics
                        Self::cleanup_old_metrics(&ai_metrics_buffer, retention_duration).await;
                        Self::cleanup_old_metrics(&file_metrics_buffer, retention_duration).await;
                        Self::cleanup_old_metrics(&cache_metrics_buffer, retention_duration).await;
                        Self::cleanup_old_metrics(&user_metrics_buffer, retention_duration).await;
                        Self::cleanup_old_metrics(&health_metrics_buffer, retention_duration).await;
                    }
                }
            }
        });

        // Periodic metrics collection task
        let metrics_buffer_clone = Arc::clone(&metrics_buffer);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Collect every minute
            loop {
                interval.tick().await;
                
                let current_metrics = PerformanceMetrics {
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    cpu_usage: 0.0, // Would use sysinfo crate in real implementation
                    memory_usage: 0,
                    memory_total: 0,
                    ai_processing_time: Duration::from_millis(0),
                    file_operations_count: 0,
                    cache_hit_rate: 0.0,
                    active_connections: 0,
                    request_count: 0,
                    error_count: 0,
                };

                let mut buffer = metrics_buffer_clone.write().await;
                buffer.push(current_metrics);
            }
        });

        // Periodic cleanup task
        let metrics_tx_clone = self.metrics_tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Cleanup every hour
            loop {
                interval.tick().await;
                if let Err(e) = metrics_tx_clone.send(MetricEvent::FlushMetrics) {
                    error!("Failed to send flush metrics event: {}", e);
                }
            }
        });
    }

    async fn cleanup_old_metrics<T>(buffer: &Arc<RwLock<Vec<T>>>, retention_duration: Duration) 
    where
        T: HasTimestamp,
    {
        let mut buffer = buffer.write().await;
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .saturating_sub(retention_duration)
            .as_secs();

        buffer.retain(|metric| metric.timestamp() > cutoff_time);
    }

    fn get_cpu_usage(&self) -> f64 {
        // In real implementation, use sysinfo crate
        0.0
    }

    fn get_memory_usage(&self) -> u64 {
        // In real implementation, use sysinfo crate
        0
    }

    fn get_total_memory(&self) -> u64 {
        // In real implementation, use sysinfo crate
        0
    }

    pub async fn health_check(&self) -> HashMap<String, bool> {
        let mut health = HashMap::new();
        
        // Check database connectivity
        health.insert("database".to_string(), true);
        
        // Check AI service availability
        health.insert("ai_service".to_string(), true);
        
        // Check file system access
        health.insert("file_system".to_string(), true);
        
        // Check memory usage
        let memory_usage = self.get_memory_usage();
        let memory_total = self.get_total_memory();
        let memory_percentage = if memory_total > 0 {
            (memory_usage as f64 / memory_total as f64) * 100.0
        } else {
            0.0
        };
        health.insert("memory".to_string(), memory_percentage < 90.0);
        
        health
    }

    pub async fn generate_report(&self) -> TelemetryReport {
        let stats = self.get_aggregated_stats().await;
        let health = self.health_check().await;
        
        TelemetryReport {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            stats,
            health_status: health,
            recommendations: self.generate_recommendations().await,
        }
    }

    async fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let stats = self.get_aggregated_stats().await;
        
        if stats.success_rate < 95.0 {
            recommendations.push("Consider investigating frequent errors to improve success rate".to_string());
        }
        
        if stats.cache_hit_rate < 50.0 {
            recommendations.push("Low cache hit rate - consider adjusting cache policies".to_string());
        }
        
        if stats.avg_processing_time > 5000.0 {
            recommendations.push("High average processing time - consider optimizing AI requests".to_string());
        }
        
        recommendations
    }
}

trait HasTimestamp {
    fn timestamp(&self) -> u64;
}

impl HasTimestamp for AIProcessingMetrics {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl HasTimestamp for FileOperationMetrics {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl HasTimestamp for CacheMetrics {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl HasTimestamp for UserInteractionMetrics {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl HasTimestamp for SystemHealthMetrics {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

#[derive(Debug, Serialize)]
pub struct TelemetryReport {
    pub timestamp: u64,
    pub stats: AggregatedStats,
    pub health_status: HashMap<String, bool>,
    pub recommendations: Vec<String>,
}

// Helper functions for creating metrics
pub fn create_ai_processing_metrics(
    model: String,
    provider: String,
    processing_time: Duration,
    tokens_processed: u32,
    success: bool,
    error_type: Option<String>,
) -> AIProcessingMetrics {
    AIProcessingMetrics {
        model,
        provider,
        processing_time,
        tokens_processed,
        success,
        error_type,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

pub fn create_file_operation_metrics(
    operation_type: String,
    file_path: String,
    duration: Duration,
    file_size: u64,
    success: bool,
) -> FileOperationMetrics {
    FileOperationMetrics {
        operation_type,
        file_path,
        duration,
        file_size,
        success,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

pub fn create_cache_metrics(
    cache_type: String,
    hit: bool,
    key_hash: String,
) -> CacheMetrics {
    CacheMetrics {
        cache_type,
        hit,
        key_hash,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_manager_creation() {
        let manager = TelemetryManager::new(true, 24);
        assert!(manager.enabled);
        
        let stats = manager.get_aggregated_stats().await;
        assert_eq!(stats.total_requests, 0);
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let manager = TelemetryManager::new(true, 24);
        
        let ai_metrics = create_ai_processing_metrics(
            "gpt-3.5-turbo".to_string(),
            "openai".to_string(),
            Duration::from_millis(1500),
            100,
            true,
            None,
        );
        
        manager.record_ai_processing(ai_metrics);
        
        // Give some time for background processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let recorded_metrics = manager.get_ai_metrics(Some(10)).await;
        assert!(!recorded_metrics.is_empty());
    }
}
