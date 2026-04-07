use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

// Missing types expected by main.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_latency: f64,
    pub command_execution_time: f64,
    pub response_time: f64,
    pub throughput: f64,
    pub error_rate: f64,
    pub uptime_percentage: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub total_commands: u64,
    pub unique_commands: u64,
    pub most_used_commands: Vec<CommandUsage>,
    pub session_duration: f64,
    pub active_hours: Vec<u8>,
    pub peak_usage_hour: u8,
    pub command_success_rate: f64,
    pub average_command_length: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandUsage {
    pub command: String,
    pub count: u64,
    pub success_rate: f64,
    pub average_execution_time: f64,
    pub last_used: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    pub pattern: String,
    pub frequency: u64,
    pub confidence: f64,
    pub context: String,
    pub optimization_potential: f64,
    pub related_patterns: Vec<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AnalyticsEngine {
    metrics: HashMap<String, MetricSeries>,
    insights: Vec<Insight>,
    reports: HashMap<String, AnalyticsReport>,
    performance_profiles: HashMap<String, PerformanceProfile>,
    optimization_suggestions: Vec<OptimizationSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    pub name: String,
    pub metric_type: MetricType,
    pub data_points: Vec<DataPoint>,
    pub aggregation: AggregationType,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
    Rate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Min,
    Max,
    Count,
    Percentile(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: String,
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub severity: InsightSeverity,
    pub confidence: f64,
    pub metric_names: Vec<String>,
    pub time_range: TimeRange,
    pub created_at: DateTime<Utc>,
    pub actions: Vec<RecommendedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    Performance,
    Usage,
    Error,
    Anomaly,
    Trend,
    Optimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: ActionType,
    pub description: String,
    pub priority: u32,
    pub estimated_impact: EstimatedImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Configuration,
    Optimization,
    Scaling,
    Investigation,
    Monitoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedImpact {
    pub performance_improvement: Option<f64>,
    pub resource_savings: Option<f64>,
    pub user_experience_improvement: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub id: String,
    pub name: String,
    pub report_type: ReportType,
    pub time_range: TimeRange,
    pub sections: Vec<ReportSection>,
    pub generated_at: DateTime<Utc>,
    pub summary: ReportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    Performance,
    Usage,
    Security,
    Trends,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub content_type: ContentType,
    pub data: serde_json::Value,
    pub visualization: Option<VisualizationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Chart,
    Table,
    Text,
    Metrics,
    Timeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub chart_type: ChartType,
    pub x_axis: String,
    pub y_axis: String,
    pub series: Vec<String>,
    pub colors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Area,
    Scatter,
    Heatmap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_metrics: u32,
    pub insights_found: u32,
    pub performance_score: Option<f64>,
    pub key_findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub id: String,
    pub name: String,
    pub component: String,
    pub metrics: HashMap<String, PerformanceMetric>,
    pub baseline: HashMap<String, f64>,
    pub thresholds: HashMap<String, Threshold>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub current_value: f64,
    pub trend: TrendDirection,
    pub change_percentage: f64,
    pub time_period: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    pub warning: f64,
    pub critical: f64,
    pub comparison: ComparisonType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonType {
    GreaterThan,
    LessThan,
    Equals,
    Range(f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub id: String,
    pub category: OptimizationCategory,
    pub title: String,
    pub description: String,
    pub impact_rating: f64,
    pub complexity: ComplexityLevel,
    pub estimated_savings: EstimatedSavings,
    pub implementation_steps: Vec<String>,
    pub prerequisites: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationCategory {
    Performance,
    Memory,
    CPU,
    Network,
    Storage,
    Configuration,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedSavings {
    pub cpu_percentage: Option<f64>,
    pub memory_mb: Option<u64>,
    pub network_bandwidth: Option<f64>,
    pub execution_time_ms: Option<f64>,
    pub cost_savings: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub pattern_type: PatternType,
    pub frequency: f64,
    pub time_distribution: HashMap<u8, f64>, // Hour of day -> usage percentage
    pub user_segments: HashMap<String, f64>,
    pub feature_correlation: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Daily,
    Weekly,
    Monthly,
    Seasonal,
    EventDriven,
}

#[allow(dead_code)]
impl AnalyticsEngine {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            insights: Vec::new(),
            reports: HashMap::new(),
            performance_profiles: HashMap::new(),
            optimization_suggestions: Vec::new(),
        }
    }

    pub fn record_metric(&mut self, name: String, value: f64, tags: HashMap<String, String>) {
        let data_point = DataPoint {
            timestamp: Utc::now(),
            value,
            tags,
        };

        if let Some(series) = self.metrics.get_mut(&name) {
            series.data_points.push(data_point);
            
            // Apply retention policy
            let cutoff = Utc::now() - Duration::days(series.retention_days as i64);
            series.data_points.retain(|dp| dp.timestamp > cutoff);
        } else {
            // Create new metric series
            let series = MetricSeries {
                name: name.clone(),
                metric_type: MetricType::Gauge,
                data_points: vec![data_point],
                aggregation: AggregationType::Average,
                retention_days: 30,
            };
            self.metrics.insert(name, series);
        }
    }

    pub fn get_metric_value(&self, name: &str, time_range: Option<TimeRange>) -> Option<f64> {
        if let Some(series) = self.metrics.get(name) {
            let data_points = if let Some(range) = time_range {
                series.data_points.iter()
                    .filter(|dp| dp.timestamp >= range.start && dp.timestamp <= range.end)
                    .collect::<Vec<_>>()
            } else {
                series.data_points.iter().collect()
            };

            if data_points.is_empty() {
                return None;
            }

            match series.aggregation {
                AggregationType::Average => {
                    let sum: f64 = data_points.iter().map(|dp| dp.value).sum();
                    Some(sum / data_points.len() as f64)
                }
                AggregationType::Sum => {
                    Some(data_points.iter().map(|dp| dp.value).sum())
                }
                AggregationType::Min => {
                    data_points.iter().map(|dp| dp.value).fold(f64::INFINITY, f64::min).into()
                }
                AggregationType::Max => {
                    data_points.iter().map(|dp| dp.value).fold(f64::NEG_INFINITY, f64::max).into()
                }
                AggregationType::Count => {
                    Some(data_points.len() as f64)
                }
                AggregationType::Percentile(p) => {
                    let mut values: Vec<f64> = data_points.iter().map(|dp| dp.value).collect();
                    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let index = (p / 100.0 * (values.len() - 1) as f64) as usize;
                    values.get(index).copied()
                }
            }
        } else {
            None
        }
    }

    pub fn analyze_performance(&mut self) -> Vec<Insight> {
        let mut insights = Vec::new();
        
        // CPU usage analysis
        if let Some(cpu_avg) = self.get_metric_value("cpu_usage", None) {
            if cpu_avg > 80.0 {
                insights.push(Insight {
                    id: uuid::Uuid::new_v4().to_string(),
                    insight_type: InsightType::Performance,
                    title: "High CPU Usage Detected".to_string(),
                    description: format!("Average CPU usage is {:.1}%, which exceeds the recommended threshold", cpu_avg),
                    severity: InsightSeverity::High,
                    confidence: 0.9,
                    metric_names: vec!["cpu_usage".to_string()],
                    time_range: TimeRange {
                        start: Utc::now() - Duration::hours(1),
                        end: Utc::now(),
                    },
                    created_at: Utc::now(),
                    actions: vec![
                        RecommendedAction {
                            action_type: ActionType::Investigation,
                            description: "Investigate processes consuming high CPU".to_string(),
                            priority: 1,
                            estimated_impact: EstimatedImpact {
                                performance_improvement: Some(25.0),
                                resource_savings: Some(20.0),
                                user_experience_improvement: Some(15.0),
                            },
                        }
                    ],
                });
            }
        }

        // Memory usage analysis
        if let Some(memory_avg) = self.get_metric_value("memory_usage", None) {
            if memory_avg > 85.0 {
                insights.push(Insight {
                    id: uuid::Uuid::new_v4().to_string(),
                    insight_type: InsightType::Performance,
                    title: "High Memory Usage".to_string(),
                    description: format!("Memory usage is at {:.1}%", memory_avg),
                    severity: InsightSeverity::Medium,
                    confidence: 0.85,
                    metric_names: vec!["memory_usage".to_string()],
                    time_range: TimeRange {
                        start: Utc::now() - Duration::hours(1),
                        end: Utc::now(),
                    },
                    created_at: Utc::now(),
                    actions: vec![
                        RecommendedAction {
                            action_type: ActionType::Optimization,
                            description: "Consider increasing available memory or optimizing memory usage".to_string(),
                            priority: 2,
                            estimated_impact: EstimatedImpact {
                                performance_improvement: Some(20.0),
                                resource_savings: None,
                                user_experience_improvement: Some(10.0),
                            },
                        }
                    ],
                });
            }
        }

        // Command execution time analysis
        if let Some(exec_time_avg) = self.get_metric_value("command_execution_time", None) {
            if exec_time_avg > 5000.0 { // 5 seconds
                insights.push(Insight {
                    id: uuid::Uuid::new_v4().to_string(),
                    insight_type: InsightType::Performance,
                    title: "Slow Command Execution".to_string(),
                    description: format!("Average command execution time is {:.1}ms", exec_time_avg),
                    severity: InsightSeverity::Medium,
                    confidence: 0.8,
                    metric_names: vec!["command_execution_time".to_string()],
                    time_range: TimeRange {
                        start: Utc::now() - Duration::hours(1),
                        end: Utc::now(),
                    },
                    created_at: Utc::now(),
                    actions: vec![
                        RecommendedAction {
                            action_type: ActionType::Optimization,
                            description: "Analyze slow commands and consider optimization or caching".to_string(),
                            priority: 3,
                            estimated_impact: EstimatedImpact {
                                performance_improvement: Some(30.0),
                                resource_savings: Some(15.0),
                                user_experience_improvement: Some(35.0),
                            },
                        }
                    ],
                });
            }
        }

        self.insights.extend(insights.clone());
        insights
    }

    pub fn detect_anomalies(&mut self) -> Vec<Insight> {
        let mut anomalies = Vec::new();

        for (metric_name, series) in &self.metrics {
            if series.data_points.len() < 10 {
                continue; // Need sufficient data for anomaly detection
            }

            let recent_values: Vec<f64> = series.data_points.iter()
                .rev()
                .take(10)
                .map(|dp| dp.value)
                .collect();

            let historical_values: Vec<f64> = series.data_points.iter()
                .rev()
                .skip(10)
                .take(50)
                .map(|dp| dp.value)
                .collect();

            if historical_values.is_empty() {
                continue;
            }

            let historical_avg = historical_values.iter().sum::<f64>() / historical_values.len() as f64;
            let recent_avg = recent_values.iter().sum::<f64>() / recent_values.len() as f64;

            let deviation = ((recent_avg - historical_avg) / historical_avg).abs();

            if deviation > 0.5 { // 50% deviation threshold
                let severity = if deviation > 1.0 {
                    InsightSeverity::High
                } else {
                    InsightSeverity::Medium
                };

                anomalies.push(Insight {
                    id: uuid::Uuid::new_v4().to_string(),
                    insight_type: InsightType::Anomaly,
                    title: format!("Anomaly Detected in {}", metric_name),
                    description: format!("Metric {} shows {:.1}% deviation from baseline", metric_name, deviation * 100.0),
                    severity,
                    confidence: 0.7,
                    metric_names: vec![metric_name.clone()],
                    time_range: TimeRange {
                        start: Utc::now() - Duration::minutes(30),
                        end: Utc::now(),
                    },
                    created_at: Utc::now(),
                    actions: vec![
                        RecommendedAction {
                            action_type: ActionType::Investigation,
                            description: "Investigate the cause of this anomaly".to_string(),
                            priority: 1,
                            estimated_impact: EstimatedImpact {
                                performance_improvement: None,
                                resource_savings: None,
                                user_experience_improvement: None,
                            },
                        }
                    ],
                });
            }
        }

        self.insights.extend(anomalies.clone());
        anomalies
    }

    pub fn generate_optimization_suggestions(&mut self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // CPU optimization suggestions
        if let Some(cpu_usage) = self.get_metric_value("cpu_usage", None) {
            if cpu_usage > 70.0 {
                suggestions.push(OptimizationSuggestion {
                    id: uuid::Uuid::new_v4().to_string(),
                    category: OptimizationCategory::CPU,
                    title: "Optimize CPU Usage".to_string(),
                    description: "High CPU usage detected. Consider implementing CPU optimization techniques.".to_string(),
                    impact_rating: 8.5,
                    complexity: ComplexityLevel::Medium,
                    estimated_savings: EstimatedSavings {
                        cpu_percentage: Some(25.0),
                        memory_mb: None,
                        network_bandwidth: None,
                        execution_time_ms: Some(500.0),
                        cost_savings: Some(100.0),
                    },
                    implementation_steps: vec![
                        "Profile CPU-intensive operations".to_string(),
                        "Implement asynchronous processing where applicable".to_string(),
                        "Consider caching frequently computed results".to_string(),
                        "Optimize algorithms and data structures".to_string(),
                    ],
                    prerequisites: vec![
                        "CPU profiling tools".to_string(),
                        "Performance testing environment".to_string(),
                    ],
                    created_at: Utc::now(),
                });
            }
        }

        // Memory optimization suggestions
        if let Some(memory_usage) = self.get_metric_value("memory_usage", None) {
            if memory_usage > 80.0 {
                suggestions.push(OptimizationSuggestion {
                    id: uuid::Uuid::new_v4().to_string(),
                    category: OptimizationCategory::Memory,
                    title: "Optimize Memory Usage".to_string(),
                    description: "High memory usage detected. Implement memory optimization strategies.".to_string(),
                    impact_rating: 7.8,
                    complexity: ComplexityLevel::Medium,
                    estimated_savings: EstimatedSavings {
                        cpu_percentage: None,
                        memory_mb: Some(512),
                        network_bandwidth: None,
                        execution_time_ms: None,
                        cost_savings: Some(75.0),
                    },
                    implementation_steps: vec![
                        "Analyze memory usage patterns".to_string(),
                        "Implement object pooling for frequently used objects".to_string(),
                        "Optimize data structures for memory efficiency".to_string(),
                        "Implement garbage collection tuning".to_string(),
                    ],
                    prerequisites: vec![
                        "Memory profiling tools".to_string(),
                        "Understanding of application memory patterns".to_string(),
                    ],
                    created_at: Utc::now(),
                });
            }
        }

        // Network optimization suggestions
        if let Some(network_latency) = self.get_metric_value("network_latency", None) {
            if network_latency > 100.0 { // 100ms
                suggestions.push(OptimizationSuggestion {
                    id: uuid::Uuid::new_v4().to_string(),
                    category: OptimizationCategory::Network,
                    title: "Optimize Network Performance".to_string(),
                    description: "High network latency detected. Consider network optimization techniques.".to_string(),
                    impact_rating: 6.5,
                    complexity: ComplexityLevel::Low,
                    estimated_savings: EstimatedSavings {
                        cpu_percentage: None,
                        memory_mb: None,
                        network_bandwidth: Some(20.0),
                        execution_time_ms: Some(200.0),
                        cost_savings: Some(50.0),
                    },
                    implementation_steps: vec![
                        "Implement connection pooling".to_string(),
                        "Use compression for data transfer".to_string(),
                        "Implement caching for frequently accessed data".to_string(),
                        "Consider CDN for static content".to_string(),
                    ],
                    prerequisites: vec![
                        "Network monitoring tools".to_string(),
                        "Understanding of network topology".to_string(),
                    ],
                    created_at: Utc::now(),
                });
            }
        }

        self.optimization_suggestions.extend(suggestions.clone());
        suggestions
    }

    pub fn generate_report(&mut self, report_type: ReportType, time_range: TimeRange) -> Result<AnalyticsReport> {
        let report_id = uuid::Uuid::new_v4().to_string();
        let mut sections = Vec::new();

        match report_type {
            ReportType::Performance => {
                // CPU usage section
                if let Some(cpu_data) = self.get_time_series_data("cpu_usage", &time_range) {
                    sections.push(ReportSection {
                        title: "CPU Usage".to_string(),
                        content_type: ContentType::Chart,
                        data: serde_json::to_value(&cpu_data)?,
                        visualization: Some(VisualizationConfig {
                            chart_type: ChartType::Line,
                            x_axis: "timestamp".to_string(),
                            y_axis: "cpu_percentage".to_string(),
                            series: vec!["CPU Usage".to_string()],
                            colors: vec!["#FF6B6B".to_string()],
                        }),
                    });
                }

                // Memory usage section
                if let Some(memory_data) = self.get_time_series_data("memory_usage", &time_range) {
                    sections.push(ReportSection {
                        title: "Memory Usage".to_string(),
                        content_type: ContentType::Chart,
                        data: serde_json::to_value(&memory_data)?,
                        visualization: Some(VisualizationConfig {
                            chart_type: ChartType::Area,
                            x_axis: "timestamp".to_string(),
                            y_axis: "memory_mb".to_string(),
                            series: vec!["Memory Usage".to_string()],
                            colors: vec!["#4ECDC4".to_string()],
                        }),
                    });
                }

                // Performance summary
                let performance_summary = self.calculate_performance_summary(&time_range);
                sections.push(ReportSection {
                    title: "Performance Summary".to_string(),
                    content_type: ContentType::Metrics,
                    data: serde_json::to_value(&performance_summary)?,
                    visualization: None,
                });
            }
            ReportType::Usage => {
                // Command usage patterns
                let usage_patterns = self.analyze_usage_patterns(&time_range);
                sections.push(ReportSection {
                    title: "Usage Patterns".to_string(),
                    content_type: ContentType::Chart,
                    data: serde_json::to_value(&usage_patterns)?,
                    visualization: Some(VisualizationConfig {
                        chart_type: ChartType::Bar,
                        x_axis: "command".to_string(),
                        y_axis: "usage_count".to_string(),
                        series: vec!["Command Usage".to_string()],
                        colors: vec!["#45B7D1".to_string()],
                    }),
                });
            }
            _ => {
                // Default sections for other report types
                sections.push(ReportSection {
                    title: "Overview".to_string(),
                    content_type: ContentType::Text,
                    data: serde_json::Value::String("Report data not available".to_string()),
                    visualization: None,
                });
            }
        }

        let insights_count = self.insights.len() as u32;
        let performance_score = self.calculate_overall_performance_score();

        let report = AnalyticsReport {
            id: report_id,
            name: format!("{:?} Report", report_type),
            report_type,
            time_range,
            sections,
            generated_at: Utc::now(),
            summary: ReportSummary {
                total_metrics: self.metrics.len() as u32,
                insights_found: insights_count,
                performance_score: Some(performance_score),
                key_findings: self.get_key_findings(),
            },
        };

        self.reports.insert(report.id.clone(), report.clone());
        Ok(report)
    }

    fn get_time_series_data(&self, metric_name: &str, time_range: &TimeRange) -> Option<Vec<DataPoint>> {
        self.metrics.get(metric_name).map(|series| {
            series.data_points.iter()
                .filter(|dp| dp.timestamp >= time_range.start && dp.timestamp <= time_range.end)
                .cloned()
                .collect()
        })
    }

    fn calculate_performance_summary(&self, time_range: &TimeRange) -> HashMap<String, f64> {
        let mut summary = HashMap::new();

        if let Some(cpu_avg) = self.get_metric_value("cpu_usage", Some(time_range.clone())) {
            summary.insert("avg_cpu_usage".to_string(), cpu_avg);
        }

        if let Some(memory_avg) = self.get_metric_value("memory_usage", Some(time_range.clone())) {
            summary.insert("avg_memory_usage".to_string(), memory_avg);
        }

        if let Some(exec_time_avg) = self.get_metric_value("command_execution_time", Some(time_range.clone())) {
            summary.insert("avg_execution_time_ms".to_string(), exec_time_avg);
        }

        summary
    }

    fn analyze_usage_patterns(&self, _time_range: &TimeRange) -> HashMap<String, u64> {
        let mut patterns = HashMap::new();

        // This would analyze command usage patterns from metrics
        // For now, return mock data
        patterns.insert("git".to_string(), 150);
        patterns.insert("ls".to_string(), 120);
        patterns.insert("cd".to_string(), 100);
        patterns.insert("npm".to_string(), 80);
        patterns.insert("cargo".to_string(), 60);

        patterns
    }

    fn calculate_overall_performance_score(&self) -> f64 {
        let mut score = 100.0;
        let mut factors = 0;

        // Factor in CPU usage
        if let Some(cpu_usage) = self.get_metric_value("cpu_usage", None) {
            score -= (cpu_usage.max(0.0) - 50.0).max(0.0) / 50.0 * 30.0;
            factors += 1;
        }

        // Factor in memory usage
        if let Some(memory_usage) = self.get_metric_value("memory_usage", None) {
            score -= (memory_usage.max(0.0) - 70.0).max(0.0) / 30.0 * 25.0;
            factors += 1;
        }

        // Factor in execution time
        if let Some(exec_time) = self.get_metric_value("command_execution_time", None) {
            let normalized_time = (exec_time / 1000.0).min(10.0); // Normalize to seconds, cap at 10s
            score -= normalized_time * 4.5; // Up to 45 points deduction
            factors += 1;
        }

        if factors == 0 {
            50.0 // Default score if no metrics available
        } else {
            score.max(0.0).min(100.0)
        }
    }

    fn get_key_findings(&self) -> Vec<String> {
        let mut findings = Vec::new();

        // Get top insights
        let mut sorted_insights = self.insights.clone();
        sorted_insights.sort_by(|a, b| {
            let severity_order = |s: &InsightSeverity| match s {
                InsightSeverity::Critical => 0,
                InsightSeverity::High => 1,
                InsightSeverity::Medium => 2,
                InsightSeverity::Low => 3,
                InsightSeverity::Info => 4,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        for insight in sorted_insights.iter().take(5) {
            findings.push(insight.title.clone());
        }

        if findings.is_empty() {
            findings.push("System performance is within normal parameters".to_string());
        }

        findings
    }

    pub async fn get_insights(&self) -> Result<Vec<Insight>> {
        Ok(self.insights.clone())
    }

    pub async fn get_optimization_suggestions(&self) -> Result<Vec<OptimizationSuggestion>> {
        Ok(self.optimization_suggestions.clone())
    }

    pub fn get_report(&self, report_id: &str) -> Option<&AnalyticsReport> {
        self.reports.get(report_id)
    }

    pub fn list_reports(&self) -> Vec<&AnalyticsReport> {
        self.reports.values().collect()
    }

    // Methods expected by main.rs
    pub async fn get_performance_metrics(&self, time_range: &str) -> Result<PerformanceMetrics> {
        let range = self.parse_time_range(time_range)?;
        
        let cpu_usage = self.get_metric_value("cpu_usage", Some(range.clone())).unwrap_or(0.0);
        let memory_usage = self.get_metric_value("memory_usage", Some(range.clone())).unwrap_or(0.0);
        let disk_usage = self.get_metric_value("disk_usage", Some(range.clone())).unwrap_or(0.0);
        let network_latency = self.get_metric_value("network_latency", Some(range.clone())).unwrap_or(0.0);
        let command_execution_time = self.get_metric_value("command_execution_time", Some(range.clone())).unwrap_or(0.0);
        let response_time = self.get_metric_value("response_time", Some(range.clone())).unwrap_or(0.0);
        let throughput = self.get_metric_value("throughput", Some(range.clone())).unwrap_or(0.0);
        let error_rate = self.get_metric_value("error_rate", Some(range.clone())).unwrap_or(0.0);
        let uptime_percentage = self.get_metric_value("uptime", Some(range)).unwrap_or(100.0);
        
        Ok(PerformanceMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_latency,
            command_execution_time,
            response_time,
            throughput,
            error_rate,
            uptime_percentage,
            timestamp: Utc::now(),
        })
    }

    pub async fn get_usage_statistics(&self, period: &str) -> Result<UsageStatistics> {
        let range = self.parse_time_range(period)?;
        
        // Calculate usage statistics from recorded metrics
        let total_commands = self.get_metric_value("command_count", Some(range.clone())).unwrap_or(0.0) as u64;
        let unique_commands = self.get_metric_value("unique_command_count", Some(range.clone())).unwrap_or(0.0) as u64;
        let session_duration = self.get_metric_value("session_duration", Some(range.clone())).unwrap_or(0.0);
        let command_success_rate = self.get_metric_value("command_success_rate", Some(range.clone())).unwrap_or(100.0);
        let average_command_length = self.get_metric_value("average_command_length", Some(range.clone())).unwrap_or(0.0);
        
        // Mock most used commands for now
        let most_used_commands = vec![
            CommandUsage {
                command: "git".to_string(),
                count: 150,
                success_rate: 98.5,
                average_execution_time: 250.0,
                last_used: Utc::now(),
            },
            CommandUsage {
                command: "ls".to_string(),
                count: 120,
                success_rate: 99.8,
                average_execution_time: 50.0,
                last_used: Utc::now(),
            },
            CommandUsage {
                command: "cd".to_string(),
                count: 100,
                success_rate: 100.0,
                average_execution_time: 10.0,
                last_used: Utc::now(),
            },
        ];
        
        // Mock active hours distribution
        let active_hours = vec![9, 10, 11, 14, 15, 16, 17];
        let peak_usage_hour = 15; // 3 PM
        
        Ok(UsageStatistics {
            total_commands,
            unique_commands,
            most_used_commands,
            session_duration,
            active_hours,
            peak_usage_hour,
            command_success_rate,
            average_command_length,
            period_start: range.start,
            period_end: range.end,
        })
    }


    pub async fn track_command(&mut self, command: &str, execution_time: u64, success: bool, _context: &serde_json::Value) -> Result<()> {
        let mut tags = HashMap::new();
        tags.insert("command".to_string(), command.to_string());
        tags.insert("success".to_string(), success.to_string());
        
        // Record execution time
        self.record_metric("command_execution_time".to_string(), execution_time as f64, tags.clone());
        
        // Record command count
        self.record_metric("command_count".to_string(), 1.0, tags.clone());
        
        // Record success/failure
        let success_value = if success { 1.0 } else { 0.0 };
        self.record_metric("command_success".to_string(), success_value, tags.clone());
        
        // Record command length
        self.record_metric("command_length".to_string(), command.len() as f64, tags);
        
        // Track unique commands
        let mut unique_tags = HashMap::new();
        unique_tags.insert("command_name".to_string(), command.to_string());
        self.record_metric(format!("unique_command_{}", command), 1.0, unique_tags);
        
        Ok(())
    }

    pub async fn get_command_patterns(&self) -> Result<Vec<CommandPattern>> {
        let mut patterns = Vec::new();
        
        // Analyze command patterns from metrics
        let mut command_counts = HashMap::new();
        
        for (metric_name, series) in &self.metrics {
            if metric_name.starts_with("unique_command_") {
                let command = metric_name.strip_prefix("unique_command_").unwrap_or(metric_name);
                let count = series.data_points.len() as u64;
                command_counts.insert(command.to_string(), count);
            }
        }
        
        // Generate patterns from most frequent commands
        for (command, frequency) in command_counts {
            if frequency > 5 { // Only include patterns with significant frequency
                patterns.push(CommandPattern {
                    pattern: command.clone(),
                    frequency,
                    confidence: (frequency as f64 / 100.0).min(1.0),
                    context: "Frequent command usage".to_string(),
                    optimization_potential: if frequency > 20 { 0.8 } else { 0.4 },
                    related_patterns: vec![], // Would be populated by pattern analysis
                    first_seen: Utc::now() - Duration::days(30),
                    last_seen: Utc::now(),
                });
            }
        }
        
        // Add some common patterns
        if patterns.is_empty() {
            patterns.push(CommandPattern {
                pattern: "git commit -m".to_string(),
                frequency: 50,
                confidence: 0.9,
                context: "Git commit pattern".to_string(),
                optimization_potential: 0.7,
                related_patterns: vec!["git add".to_string(), "git push".to_string()],
                first_seen: Utc::now() - Duration::days(7),
                last_seen: Utc::now(),
            });
            
            patterns.push(CommandPattern {
                pattern: "npm install".to_string(),
                frequency: 25,
                confidence: 0.8,
                context: "Node.js development".to_string(),
                optimization_potential: 0.6,
                related_patterns: vec!["npm run".to_string(), "npm test".to_string()],
                first_seen: Utc::now() - Duration::days(14),
                last_seen: Utc::now(),
            });
        }
        
        Ok(patterns)
    }


    fn parse_time_range(&self, time_range_str: &str) -> Result<TimeRange> {
        let end = Utc::now();
        let start = match time_range_str {
            "1h" | "hour" => end - Duration::hours(1),
            "24h" | "day" => end - Duration::days(1),
            "7d" | "week" => end - Duration::days(7),
            "30d" | "month" => end - Duration::days(30),
            "90d" | "quarter" => end - Duration::days(90),
            "365d" | "year" => end - Duration::days(365),
            _ => {
                // Try to parse as duration in hours
                if let Ok(hours) = time_range_str.parse::<i64>() {
                    end - Duration::hours(hours)
                } else {
                    return Err(anyhow!("Invalid time range format: {}", time_range_str));
                }
            }
        };
        
        Ok(TimeRange { start, end })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_engine_creation() {
        let engine = AnalyticsEngine::new();
        assert!(engine.metrics.is_empty());
        assert!(engine.insights.is_empty());
    }

    #[test]
    fn test_record_metric() {
        let mut engine = AnalyticsEngine::new();
        let mut tags = HashMap::new();
        tags.insert("source".to_string(), "test".to_string());

        engine.record_metric("cpu_usage".to_string(), 75.5, tags);
        
        assert!(engine.metrics.contains_key("cpu_usage"));
        let series = engine.metrics.get("cpu_usage").unwrap();
        assert_eq!(series.data_points.len(), 1);
        assert_eq!(series.data_points[0].value, 75.5);
    }

    #[test]
    fn test_metric_aggregation() {
        let mut engine = AnalyticsEngine::new();
        
        // Record multiple values
        engine.record_metric("test_metric".to_string(), 10.0, HashMap::new());
        engine.record_metric("test_metric".to_string(), 20.0, HashMap::new());
        engine.record_metric("test_metric".to_string(), 30.0, HashMap::new());

        let avg_value = engine.get_metric_value("test_metric", None);
        assert_eq!(avg_value, Some(20.0));
    }

    #[test]
    fn test_performance_analysis() {
        let mut engine = AnalyticsEngine::new();
        
        // Record high CPU usage
        engine.record_metric("cpu_usage".to_string(), 85.0, HashMap::new());
        
        let insights = engine.analyze_performance();
        assert!(!insights.is_empty());
        
        let cpu_insight = insights.iter().find(|i| i.title.contains("CPU"));
        assert!(cpu_insight.is_some());
    }

    #[tokio::test]
    async fn test_optimization_suggestions() {
        let mut engine = AnalyticsEngine::new();
        
        // Record metrics that should trigger suggestions
        engine.record_metric("cpu_usage".to_string(), 75.0, HashMap::new());
        engine.record_metric("memory_usage".to_string(), 85.0, HashMap::new());
        
        let suggestions = engine.generate_optimization_suggestions();
        assert!(!suggestions.is_empty());
        
        let cpu_suggestion = suggestions.iter().find(|s| matches!(s.category, OptimizationCategory::CPU));
        assert!(cpu_suggestion.is_some());
        
        let memory_suggestion = suggestions.iter().find(|s| matches!(s.category, OptimizationCategory::Memory));
        assert!(memory_suggestion.is_some());
    }
}
