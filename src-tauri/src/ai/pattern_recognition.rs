use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Timelike, Weekday, Datelike};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};
use statrs::statistics::{Statistics, Data};
use crate::ai::{UserAction, SystemState, AIRecommendation, WorkloadType, ActionOutcome};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub frequency: f64,
    pub confidence: f64,
    pub last_seen: DateTime<Utc>,
    pub context: PatternContext,
    pub triggers: Vec<PatternTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    TimeBasedUsage,
    WorkloadTransition,
    PerformanceOptimization,
    MaintenanceSchedule,
    ResourceUsage,
    ApplicationUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContext {
    pub time_range: (u8, u8), // (start_hour, end_hour)
    pub days_of_week: Vec<Weekday>,
    pub system_conditions: Vec<SystemCondition>,
    pub user_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternTrigger {
    pub condition: String,
    pub threshold: f64,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemCondition {
    CpuUsageAbove(f64),
    CpuUsageBelow(f64),
    MemoryUsageAbove(f64),
    MemoryUsageBelow(f64),
    TemperatureAbove(f64),
    TemperatureBelow(f64),
    ProcessRunning(String),
    ProcessNotRunning(String),
    WorkloadType(WorkloadType),
}

pub struct PatternRecognizer {
    patterns: Vec<UsagePattern>,
    action_history: VecDeque<UserAction>,
    system_history: VecDeque<(DateTime<Utc>, SystemState)>,
    pattern_weights: HashMap<String, f64>,
    min_pattern_occurrences: usize,
    max_history_size: usize,
}

impl PatternRecognizer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔍 Initializing Pattern Recognition system...");
        
        Ok(Self {
            patterns: Vec::new(),
            action_history: VecDeque::new(),
            system_history: VecDeque::new(),
            pattern_weights: HashMap::new(),
            min_pattern_occurrences: 3,
            max_history_size: 1000,
        })
    }
    
    pub async fn analyze_action(&mut self, action: &UserAction) -> Result<(), Box<dyn std::error::Error>> {
        debug!("🔍 Analyzing user action for patterns: {}", action.action_type);
        
        // Add to history
        self.action_history.push_back(action.clone());
        if self.action_history.len() > self.max_history_size {
            self.action_history.pop_front();
        }
        
        // Detect new patterns
        self.detect_time_based_patterns().await?;
        self.detect_workload_patterns().await?;
        self.detect_performance_patterns().await?;
        self.detect_maintenance_patterns().await?;
        
        // Update existing pattern weights
        self.update_pattern_weights(action).await?;
        
        Ok(())
    }
    
    pub async fn record_system_state(&mut self, state: SystemState) -> Result<(), Box<dyn std::error::Error>> {
        self.system_history.push_back((Utc::now(), state));
        if self.system_history.len() > self.max_history_size {
            self.system_history.pop_front();
        }
        
        // Continuously analyze system state patterns
        self.analyze_system_state_patterns().await?;
        
        Ok(())
    }
    
    async fn detect_time_based_patterns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut time_action_map: HashMap<(u8, Weekday), Vec<&UserAction>> = HashMap::new();
        
        // Group actions by time and day
        for action in &self.action_history {
            let hour = action.timestamp.hour() as u8;
            let weekday = action.timestamp.weekday();
            let key = (hour, weekday);
            
            time_action_map.entry(key).or_insert_with(Vec::new).push(action);
        }
        
        // Find patterns with sufficient frequency
        for ((hour, weekday), actions) in time_action_map {
            if actions.len() >= self.min_pattern_occurrences {
                let most_common_action = self.find_most_common_action(&actions);
                let success_rate = self.calculate_success_rate(&actions);
                
                if success_rate > 0.7 {
                    let pattern = UsagePattern {
                        pattern_id: format!("time_{}_{:?}", hour, weekday),
                        pattern_type: PatternType::TimeBasedUsage,
                        frequency: actions.len() as f64,
                        confidence: success_rate,
                        last_seen: Utc::now(),
                        context: PatternContext {
                            time_range: (hour, hour + 1),
                            days_of_week: vec![weekday],
                            system_conditions: Vec::new(),
                            user_actions: vec![most_common_action.clone()],
                        },
                        triggers: vec![PatternTrigger {
                            condition: format!("time_{}_{:?}", hour, weekday),
                            threshold: 0.8,
                            action: most_common_action,
                        }],
                    };
                    
                    self.add_or_update_pattern(pattern).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn detect_workload_patterns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Analyze workload transitions and associated actions
        let mut workload_transitions: HashMap<(WorkloadType, WorkloadType), Vec<&UserAction>> = HashMap::new();
        
        if self.system_history.len() > 1 {
            for i in 1..self.system_history.len() {
                let prev_workload = &self.system_history[i-1].1.current_workload;
                let curr_workload = &self.system_history[i].1.current_workload;
                
                if prev_workload != curr_workload {
                    let transition_key = (prev_workload.clone(), curr_workload.clone());
                    let transition_time = self.system_history[i].0;
                    
                    // Find actions around this transition
                    let relevant_actions: Vec<&UserAction> = self.action_history.iter()
                        .filter(|action| {
                            let time_diff = (action.timestamp - transition_time).num_seconds().abs();
                            time_diff < 300 // Within 5 minutes
                        })
                        .collect();
                    
                    if !relevant_actions.is_empty() {
                        workload_transitions.entry(transition_key).or_insert_with(Vec::new)
                            .extend(relevant_actions);
                    }
                }
            }
        }
        
        // Create patterns for significant workload transitions
        for ((from_workload, to_workload), actions) in workload_transitions {
            if actions.len() >= self.min_pattern_occurrences {
                let most_common_action = self.find_most_common_action(&actions);
                let success_rate = self.calculate_success_rate(&actions);
                
                if success_rate > 0.6 {
                    let pattern = UsagePattern {
                        pattern_id: format!("workload_{:?}_to_{:?}", from_workload, to_workload),
                        pattern_type: PatternType::WorkloadTransition,
                        frequency: actions.len() as f64,
                        confidence: success_rate,
                        last_seen: Utc::now(),
                        context: PatternContext {
                            time_range: (0, 24),
                            days_of_week: vec![
                                Weekday::Mon, Weekday::Tue, Weekday::Wed, 
                                Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
                            ],
                            system_conditions: vec![
                                SystemCondition::WorkloadType(from_workload.clone()),
                                SystemCondition::WorkloadType(to_workload.clone()),
                            ],
                            user_actions: vec![most_common_action.clone()],
                        },
                        triggers: vec![PatternTrigger {
                            condition: format!("workload_transition_{:?}_to_{:?}", from_workload, to_workload),
                            threshold: 0.7,
                            action: most_common_action,
                        }],
                    };
                    
                    self.add_or_update_pattern(pattern).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn detect_performance_patterns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Analyze performance-related actions
        let performance_actions: Vec<&UserAction> = self.action_history.iter()
            .filter(|action| {
                matches!(action.action_type.as_str(), 
                    "optimize_cpu" | "optimize_memory" | "optimize_disk" | 
                    "manage_processes" | "clean_system" | "set_cpu_governor"
                )
            })
            .collect();
        
        if performance_actions.len() >= self.min_pattern_occurrences {
            // Group by system conditions that triggered the actions
            let mut condition_groups: HashMap<String, Vec<&UserAction>> = HashMap::new();
            
            for action in performance_actions {
                // Extract system conditions from action context
                if action.context.contains("high_cpu") {
                    condition_groups.entry("high_cpu_usage".to_string())
                        .or_insert_with(Vec::new).push(action);
                }
                if action.context.contains("high_memory") {
                    condition_groups.entry("high_memory_usage".to_string())
                        .or_insert_with(Vec::new).push(action);
                }
                if action.context.contains("high_temp") {
                    condition_groups.entry("high_temperature".to_string())
                        .or_insert_with(Vec::new).push(action);
                }
            }
            
            for (condition, actions) in condition_groups {
                if actions.len() >= self.min_pattern_occurrences {
                    let most_common_action = self.find_most_common_action(&actions);
                    let success_rate = self.calculate_success_rate(&actions);
                    
                    let pattern = UsagePattern {
                        pattern_id: format!("performance_{}", condition),
                        pattern_type: PatternType::PerformanceOptimization,
                        frequency: actions.len() as f64,
                        confidence: success_rate,
                        last_seen: Utc::now(),
                        context: PatternContext {
                            time_range: (0, 24),
                            days_of_week: vec![
                                Weekday::Mon, Weekday::Tue, Weekday::Wed, 
                                Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
                            ],
                            system_conditions: self.parse_system_conditions(&condition),
                            user_actions: vec![most_common_action.clone()],
                        },
                        triggers: vec![PatternTrigger {
                            condition: condition.clone(),
                            threshold: 0.75,
                            action: most_common_action,
                        }],
                    };
                    
                    self.add_or_update_pattern(pattern).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn detect_maintenance_patterns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Analyze maintenance scheduling patterns
        let maintenance_actions: Vec<&UserAction> = self.action_history.iter()
            .filter(|action| {
                matches!(action.action_type.as_str(),
                    "update_packages" | "clean_system" | "backup_data" | 
                    "optimize_system" | "clean_temp_files"
                )
            })
            .collect();
        
        if maintenance_actions.len() >= self.min_pattern_occurrences {
            // Analyze timing patterns for maintenance
            let mut daily_maintenance: HashMap<Weekday, Vec<&UserAction>> = HashMap::new();
            let mut hourly_maintenance: HashMap<u8, Vec<&UserAction>> = HashMap::new();
            
            for action in maintenance_actions {
                let weekday = action.timestamp.weekday();
                let hour = action.timestamp.hour() as u8;
                
                daily_maintenance.entry(weekday).or_insert_with(Vec::new).push(action);
                hourly_maintenance.entry(hour).or_insert_with(Vec::new).push(action);
            }
            
            // Find preferred maintenance days
            for (weekday, actions) in daily_maintenance {
                if actions.len() >= 2 {
                    let most_common_action = self.find_most_common_action(&actions);
                    
                    let pattern = UsagePattern {
                        pattern_id: format!("maintenance_{:?}", weekday),
                        pattern_type: PatternType::MaintenanceSchedule,
                        frequency: actions.len() as f64,
                        confidence: 0.8,
                        last_seen: Utc::now(),
                        context: PatternContext {
                            time_range: (0, 24),
                            days_of_week: vec![weekday],
                            system_conditions: Vec::new(),
                            user_actions: vec![most_common_action.clone()],
                        },
                        triggers: vec![PatternTrigger {
                            condition: format!("maintenance_day_{:?}", weekday),
                            threshold: 0.6,
                            action: most_common_action,
                        }],
                    };
                    
                    self.add_or_update_pattern(pattern).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn analyze_system_state_patterns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.system_history.len() < 10 {
            return Ok(());
        }
        
        // Analyze resource usage patterns
        let recent_states: Vec<&SystemState> = self.system_history.iter()
            .rev()
            .take(50)
            .map(|(_, state)| state)
            .collect();
        
        // CPU usage patterns
        let cpu_usages: Vec<f64> = recent_states.iter().map(|s| s.cpu_usage).collect();
        if let Ok(cpu_data) = Data::new(cpu_usages.clone()) {
            let cpu_mean = cpu_data.mean().unwrap_or(0.0);
            let cpu_std = cpu_data.std_dev().unwrap_or(0.0);
            
            if cpu_mean > 70.0 && cpu_std < 10.0 {
                // Consistently high CPU usage pattern
                let pattern = UsagePattern {
                    pattern_id: "high_cpu_usage_pattern".to_string(),
                    pattern_type: PatternType::ResourceUsage,
                    frequency: recent_states.len() as f64,
                    confidence: 0.85,
                    last_seen: Utc::now(),
                    context: PatternContext {
                        time_range: (0, 24),
                        days_of_week: vec![
                            Weekday::Mon, Weekday::Tue, Weekday::Wed, 
                            Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
                        ],
                        system_conditions: vec![SystemCondition::CpuUsageAbove(70.0)],
                        user_actions: vec!["optimize_cpu".to_string()],
                    },
                    triggers: vec![PatternTrigger {
                        condition: "sustained_high_cpu".to_string(),
                        threshold: 70.0,
                        action: "optimize_cpu".to_string(),
                    }],
                };
                
                self.add_or_update_pattern(pattern).await?;
            }
        }
        
        // Memory usage patterns
        let memory_usages: Vec<f64> = recent_states.iter().map(|s| s.memory_usage).collect();
        if let Ok(memory_data) = Data::new(memory_usages.clone()) {
            let memory_mean = memory_data.mean().unwrap_or(0.0);
            
            if memory_mean > 80.0 {
                let pattern = UsagePattern {
                    pattern_id: "high_memory_usage_pattern".to_string(),
                    pattern_type: PatternType::ResourceUsage,
                    frequency: recent_states.len() as f64,
                    confidence: 0.80,
                    last_seen: Utc::now(),
                    context: PatternContext {
                        time_range: (0, 24),
                        days_of_week: vec![
                            Weekday::Mon, Weekday::Tue, Weekday::Wed, 
                            Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
                        ],
                        system_conditions: vec![SystemCondition::MemoryUsageAbove(80.0)],
                        user_actions: vec!["optimize_memory".to_string()],
                    },
                    triggers: vec![PatternTrigger {
                        condition: "high_memory_usage".to_string(),
                        threshold: 80.0,
                        action: "optimize_memory".to_string(),
                    }],
                };
                
                self.add_or_update_pattern(pattern).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn generate_pattern_based_recommendations(&self, current_state: &SystemState) -> Result<Vec<AIRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();
        let current_time = Utc::now();
        let current_hour = current_time.hour() as u8;
        let current_weekday = current_time.weekday();
        
        for pattern in &self.patterns {
            // Check if current conditions match the pattern
            let mut matches = true;
            
            // Check time conditions
            if !pattern.context.time_range.0 <= current_hour && current_hour <= pattern.context.time_range.1 {
                continue;
            }
            
            if !pattern.context.days_of_week.contains(&current_weekday) {
                continue;
            }
            
            // Check system conditions
            for condition in &pattern.context.system_conditions {
                match condition {
                    SystemCondition::CpuUsageAbove(threshold) => {
                        if current_state.cpu_usage <= *threshold {
                            matches = false;
                            break;
                        }
                    },
                    SystemCondition::CpuUsageBelow(threshold) => {
                        if current_state.cpu_usage >= *threshold {
                            matches = false;
                            break;
                        }
                    },
                    SystemCondition::MemoryUsageAbove(threshold) => {
                        if current_state.memory_usage <= *threshold {
                            matches = false;
                            break;
                        }
                    },
                    SystemCondition::MemoryUsageBelow(threshold) => {
                        if current_state.memory_usage >= *threshold {
                            matches = false;
                            break;
                        }
                    },
                    SystemCondition::TemperatureAbove(threshold) => {
                        if current_state.temperature <= *threshold {
                            matches = false;
                            break;
                        }
                    },
                    SystemCondition::TemperatureBelow(threshold) => {
                        if current_state.temperature >= *threshold {
                            matches = false;
                            break;
                        }
                    },
                    SystemCondition::ProcessRunning(process_name) => {
                        if !current_state.active_processes.contains(process_name) {
                            matches = false;
                            break;
                        }
                    },
                    SystemCondition::ProcessNotRunning(process_name) => {
                        if current_state.active_processes.contains(process_name) {
                            matches = false;
                            break;
                        }
                    },
                    SystemCondition::WorkloadType(workload) => {
                        if current_state.current_workload != *workload {
                            matches = false;
                            break;
                        }
                    },
                }
            }
            
            if matches && pattern.confidence > 0.6 {
                for action in &pattern.context.user_actions {
                    recommendations.push(AIRecommendation {
                        id: uuid::Uuid::new_v4().to_string(),
                        priority: (pattern.confidence * 10.0) as u8,
                        title: format!("Pattern-based suggestion: {}", action),
                        description: format!("Based on your usage pattern '{}', you typically {} at this time.", 
                                           pattern.pattern_id, action),
                        action: action.clone(),
                        confidence: pattern.confidence,
                        reasoning: format!("This recommendation is based on {} previous occurrences with {:.1}% success rate.",
                                         pattern.frequency as u32, pattern.confidence * 100.0),
                        estimated_impact: self.estimate_impact(action),
                    });
                }
            }
        }
        
        Ok(recommendations)
    }
    
    fn estimate_impact(&self, action: &str) -> String {
        match action {
            "optimize_cpu" => "Reduce CPU usage by 10-20%".to_string(),
            "optimize_memory" => "Free up 2-5GB of RAM".to_string(),
            "clean_system" => "Free up 1-10GB of disk space".to_string(),
            "update_packages" => "Improve security and stability".to_string(),
            "backup_data" => "Ensure data safety".to_string(),
            "optimize_disk" => "Improve disk performance by 15-30%".to_string(),
            "manage_processes" => "Improve system responsiveness".to_string(),
            _ => "Improve system performance".to_string(),
        }
    }
    
    async fn add_or_update_pattern(&mut self, new_pattern: UsagePattern) -> Result<(), Box<dyn std::error::Error>> {
        // Check if pattern already exists
        if let Some(existing_index) = self.patterns.iter().position(|p| p.pattern_id == new_pattern.pattern_id) {
            // Update existing pattern
            let existing = &mut self.patterns[existing_index];
            existing.frequency += 1.0;
            existing.confidence = (existing.confidence + new_pattern.confidence) / 2.0;
            existing.last_seen = new_pattern.last_seen;
            
            debug!("Updated pattern: {} (frequency: {}, confidence: {:.2})", 
                   existing.pattern_id, existing.frequency, existing.confidence);
        } else {
            // Add new pattern
            debug!("Added new pattern: {} (confidence: {:.2})", 
                   new_pattern.pattern_id, new_pattern.confidence);
            self.patterns.push(new_pattern);
        }
        
        Ok(())
    }
    
    async fn update_pattern_weights(&mut self, action: &UserAction) -> Result<(), Box<dyn std::error::Error>> {
        let action_success_weight = match &action.outcome {
            ActionOutcome::Success => 1.0,
            ActionOutcome::Partial(_) => 0.5,
            ActionOutcome::Failed(_) => -0.2,
        };
        
        // Update weights for patterns that suggested this action
        for pattern in &mut self.patterns {
            if pattern.context.user_actions.contains(&action.action_type) {
                let current_weight = self.pattern_weights.get(&pattern.pattern_id).unwrap_or(&1.0);
                let new_weight = (current_weight + action_success_weight * 0.1).max(0.1).min(2.0);
                self.pattern_weights.insert(pattern.pattern_id.clone(), new_weight);
                
                // Adjust pattern confidence based on weight
                pattern.confidence = (pattern.confidence * new_weight).min(1.0);
            }
        }
        
        Ok(())
    }
    
    fn find_most_common_action(&self, actions: &[&UserAction]) -> String {
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        
        for action in actions {
            *action_counts.entry(action.action_type.clone()).or_insert(0) += 1;
        }
        
        action_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(action, _)| action)
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    fn calculate_success_rate(&self, actions: &[&UserAction]) -> f64 {
        if actions.is_empty() {
            return 0.0;
        }
        
        let success_count = actions.iter().filter(|action| {
            matches!(action.outcome, ActionOutcome::Success)
        }).count();
        
        success_count as f64 / actions.len() as f64
    }
    
    fn parse_system_conditions(&self, condition: &str) -> Vec<SystemCondition> {
        match condition {
            "high_cpu_usage" => vec![SystemCondition::CpuUsageAbove(80.0)],
            "high_memory_usage" => vec![SystemCondition::MemoryUsageAbove(85.0)],
            "high_temperature" => vec![SystemCondition::TemperatureAbove(85.0)],
            _ => Vec::new(),
        }
    }
    
    pub fn get_pattern_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        stats.insert("total_patterns".to_string(), self.patterns.len() as f64);
        stats.insert("action_history_size".to_string(), self.action_history.len() as f64);
        stats.insert("system_history_size".to_string(), self.system_history.len() as f64);
        
        let avg_confidence = self.patterns.iter()
            .map(|p| p.confidence)
            .sum::<f64>() / self.patterns.len().max(1) as f64;
        stats.insert("average_confidence".to_string(), avg_confidence);
        
        // Pattern type distribution
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for pattern in &self.patterns {
            let type_name = format!("{:?}", pattern.pattern_type);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
        
        for (pattern_type, count) in type_counts {
            stats.insert(format!("patterns_{}", pattern_type.to_lowercase()), count as f64);
        }
        
        stats
    }
}
