use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};
use crate::ai::{SystemState, WorkloadType, natural_language::Intent, natural_language::IntentCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub action: String,
    pub parameters: HashMap<String, String>,
    pub confidence: f64,
    pub reasoning: String,
    pub expected_outcome: ExpectedOutcome,
    pub risk_level: RiskLevel,
    pub priority: u8, // 1-10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutcome {
    pub description: String,
    pub estimated_duration: u32, // seconds
    pub system_impact: SystemImpact,
    pub user_benefits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemImpact {
    Minimal,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
struct DecisionRule {
    condition: Box<dyn Fn(&SystemState, &Intent) -> bool + Send + Sync>,
    action: String,
    confidence_modifier: f64,
    risk_level: RiskLevel,
}

pub struct DecisionEngine {
    decision_rules: Vec<DecisionRule>,
    action_history: Vec<(Decision, bool)>, // (decision, was_successful)
    system_constraints: SystemConstraints,
    user_preferences: UserPreferences,
}

#[derive(Debug, Clone)]
struct SystemConstraints {
    max_cpu_usage_threshold: f64,
    max_memory_usage_threshold: f64,
    max_temperature_threshold: f64,
    critical_processes: Vec<String>,
    protected_directories: Vec<String>,
    maintenance_window: (u8, u8), // (start_hour, end_hour)
}

#[derive(Debug, Clone)]
struct UserPreferences {
    performance_over_stability: bool,
    auto_update_packages: bool,
    aggressive_cleanup: bool,
    backup_before_changes: bool,
    preferred_cpu_governor: String,
    notification_level: NotificationLevel,
}

#[derive(Debug, Clone)]
enum NotificationLevel {
    Silent,
    Minimal,
    Normal,
    Verbose,
    Debug,
}

impl DecisionEngine {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🧭 Initializing Decision Engine for system administration...");
        
        let mut engine = Self {
            decision_rules: Vec::new(),
            action_history: Vec::new(),
            system_constraints: SystemConstraints {
                max_cpu_usage_threshold: 85.0,
                max_memory_usage_threshold: 90.0,
                max_temperature_threshold: 85.0,
                critical_processes: vec![
                    "systemd".to_string(),
                    "kernel".to_string(),
                    "init".to_string(),
                    "dbus".to_string(),
                ],
                protected_directories: vec![
                    "/".to_string(),
                    "/boot".to_string(),
                    "/sys".to_string(),
                    "/proc".to_string(),
                ],
                maintenance_window: (2, 6), // 2 AM to 6 AM
            },
            user_preferences: UserPreferences {
                performance_over_stability: true, // Lou's gaming/dev setup
                auto_update_packages: false, // Manual control preferred
                aggressive_cleanup: true,
                backup_before_changes: true,
                preferred_cpu_governor: "performance".to_string(),
                notification_level: NotificationLevel::Normal,
            },
        };
        
        engine.initialize_decision_rules().await?;
        
        Ok(engine)
    }
    
    async fn initialize_decision_rules(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // CPU Optimization Rules
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                matches!(intent.category, IntentCategory::SystemOptimization) &&
                intent.action == "optimize_cpu" &&
                state.cpu_usage > 70.0
            }),
            action: "optimize_cpu_high_usage".to_string(),
            confidence_modifier: 0.9,
            risk_level: RiskLevel::Low,
        });
        
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                matches!(intent.category, IntentCategory::SystemOptimization) &&
                intent.action == "set_cpu_governor"
            }),
            action: "set_cpu_governor".to_string(),
            confidence_modifier: 0.95,
            risk_level: RiskLevel::Medium,
        });
        
        // Memory Optimization Rules
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                matches!(intent.category, IntentCategory::SystemOptimization) &&
                intent.action == "optimize_memory" &&
                state.memory_usage > 80.0
            }),
            action: "optimize_memory_aggressive".to_string(),
            confidence_modifier: 0.9,
            risk_level: RiskLevel::Low,
        });
        
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                matches!(intent.category, IntentCategory::SystemOptimization) &&
                intent.action == "optimize_memory" &&
                state.memory_usage <= 80.0
            }),
            action: "optimize_memory_gentle".to_string(),
            confidence_modifier: 0.7,
            risk_level: RiskLevel::Safe,
        });
        
        // Temperature Management Rules
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, _intent| {
                state.temperature > 85.0
            }),
            action: "emergency_cooling".to_string(),
            confidence_modifier: 1.0,
            risk_level: RiskLevel::Safe,
        });
        
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                intent.action == "reduce_temperature" ||
                (state.temperature > 80.0 && state.cpu_usage > 80.0)
            }),
            action: "thermal_optimization".to_string(),
            confidence_modifier: 0.9,
            risk_level: RiskLevel::Low,
        });
        
        // Package Management Rules
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|_state, intent| {
                matches!(intent.category, IntentCategory::PackageManagement) &&
                intent.action == "update_packages"
            }),
            action: "update_packages_safe".to_string(),
            confidence_modifier: 0.8,
            risk_level: RiskLevel::Medium,
        });
        
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|_state, intent| {
                matches!(intent.category, IntentCategory::PackageManagement) &&
                intent.action == "install_package"
            }),
            action: "install_package_with_deps".to_string(),
            confidence_modifier: 0.9,
            risk_level: RiskLevel::Medium,
        });
        
        // File Management Rules
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                matches!(intent.category, IntentCategory::FileManagement) &&
                intent.action == "clean_temp_files" &&
                state.disk_usage > 80.0
            }),
            action: "aggressive_cleanup".to_string(),
            confidence_modifier: 0.9,
            risk_level: RiskLevel::Low,
        });
        
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                matches!(intent.category, IntentCategory::FileManagement) &&
                intent.action == "organize_files"
            }),
            action: "smart_file_organization".to_string(),
            confidence_modifier: 0.8,
            risk_level: RiskLevel::Safe,
        });
        
        // Workload-Specific Rules
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                matches!(state.current_workload, WorkloadType::Gaming) &&
                matches!(intent.category, IntentCategory::SystemOptimization)
            }),
            action: "gaming_optimization".to_string(),
            confidence_modifier: 1.0,
            risk_level: RiskLevel::Low,
        });
        
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, intent| {
                matches!(state.current_workload, WorkloadType::Development) &&
                matches!(intent.category, IntentCategory::SystemOptimization)
            }),
            action: "development_optimization".to_string(),
            confidence_modifier: 0.9,
            risk_level: RiskLevel::Safe,
        });
        
        // Emergency Rules (Highest Priority)
        self.decision_rules.push(DecisionRule {
            condition: Box::new(|state, _intent| {
                state.temperature > 90.0 || state.memory_usage > 95.0
            }),
            action: "emergency_system_protection".to_string(),
            confidence_modifier: 1.0,
            risk_level: RiskLevel::Safe,
        });
        
        Ok(())
    }
    
    pub async fn decide_action(&mut self, intent: &Intent, system_state: &SystemState) -> Result<Decision, Box<dyn std::error::Error>> {
        debug!("🤔 Making decision for intent: {} in current system state", intent.action);
        
        // Check for emergency conditions first
        if let Some(emergency_decision) = self.check_emergency_conditions(system_state).await? {
            return Ok(emergency_decision);
        }
        
        // Find matching decision rules
        let mut matching_decisions = Vec::new();
        
        for rule in &self.decision_rules {
            if (rule.condition)(system_state, intent) {
                let base_confidence = intent.confidence * rule.confidence_modifier;
                let adjusted_confidence = self.adjust_confidence_based_on_history(&rule.action, base_confidence);
                
                let decision = self.create_decision(
                    &rule.action,
                    &intent.parameters,
                    adjusted_confidence,
                    &rule.risk_level,
                    system_state,
                    intent,
                ).await?;
                
                matching_decisions.push(decision);
            }
        }
        
        // If no rules matched, create a default decision
        if matching_decisions.is_empty() {
            return self.create_default_decision(intent, system_state).await;
        }
        
        // Select the best decision (highest confidence and appropriate risk)
        matching_decisions.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let best_decision = matching_decisions.into_iter().next().unwrap();
        
        // Validate the decision before returning
        self.validate_decision(&best_decision, system_state).await?;
        
        Ok(best_decision)
    }
    
    async fn check_emergency_conditions(&self, system_state: &SystemState) -> Result<Option<Decision>, Box<dyn std::error::Error>> {
        // Critical temperature
        if system_state.temperature > 90.0 {
            return Ok(Some(Decision {
                action: "emergency_cooling".to_string(),
                parameters: HashMap::new(),
                confidence: 1.0,
                reasoning: format!("Critical temperature detected: {:.1}°C. Immediate cooling required.", system_state.temperature),
                expected_outcome: ExpectedOutcome {
                    description: "Reduce CPU temperature to safe levels".to_string(),
                    estimated_duration: 30,
                    system_impact: SystemImpact::Medium,
                    user_benefits: vec!["Prevent thermal throttling".to_string(), "Protect hardware".to_string()],
                },
                risk_level: RiskLevel::Safe,
                priority: 10,
            }));
        }
        
        // Critical memory usage
        if system_state.memory_usage > 95.0 {
            return Ok(Some(Decision {
                action: "emergency_memory_cleanup".to_string(),
                parameters: HashMap::new(),
                confidence: 1.0,
                reasoning: format!("Critical memory usage: {:.1}%. Immediate cleanup required.", system_state.memory_usage),
                expected_outcome: ExpectedOutcome {
                    description: "Free up memory to prevent system instability".to_string(),
                    estimated_duration: 15,
                    system_impact: SystemImpact::Low,
                    user_benefits: vec!["Prevent system freeze".to_string(), "Maintain responsiveness".to_string()],
                },
                risk_level: RiskLevel::Safe,
                priority: 10,
            }));
        }
        
        Ok(None)
    }
    
    async fn create_decision(
        &self,
        action: &str,
        parameters: &HashMap<String, String>,
        confidence: f64,
        risk_level: &RiskLevel,
        system_state: &SystemState,
        intent: &Intent,
    ) -> Result<Decision, Box<dyn std::error::Error>> {
        
        let reasoning = self.generate_reasoning(action, system_state, intent);
        let expected_outcome = self.predict_outcome(action, system_state).await?;
        let priority = self.calculate_priority(action, system_state, risk_level);
        
        Ok(Decision {
            action: action.to_string(),
            parameters: parameters.clone(),
            confidence,
            reasoning,
            expected_outcome,
            risk_level: risk_level.clone(),
            priority,
        })
    }
    
    async fn create_default_decision(&self, intent: &Intent, system_state: &SystemState) -> Result<Decision, Box<dyn std::error::Error>> {
        let action = match &intent.category {
            IntentCategory::SystemOptimization => "general_optimization",
            IntentCategory::FileManagement => "basic_file_management",
            IntentCategory::PackageManagement => "check_packages",
            IntentCategory::Monitoring => "show_system_status",
            IntentCategory::Backup => "check_backup_status",
            IntentCategory::Hardware => "check_hardware_status",
            IntentCategory::Maintenance => "basic_maintenance",
            IntentCategory::Query => "provide_information",
            IntentCategory::Conversation => "conversational_response",
        };
        
        self.create_decision(
            action,
            &intent.parameters,
            0.5,
            &RiskLevel::Safe,
            system_state,
            intent,
        ).await
    }
    
    fn generate_reasoning(&self, action: &str, system_state: &SystemState, intent: &Intent) -> String {
        let mut reasoning = String::new();
        
        match action {
            "optimize_cpu_high_usage" => {
                reasoning = format!(
                    "High CPU usage detected ({:.1}%). Applying i9-13900HX specific optimizations.",
                    system_state.cpu_usage
                );
            },
            "optimize_memory_aggressive" => {
                reasoning = format!(
                    "Memory usage is high ({:.1}% of 64GB). Applying aggressive cleanup.",
                    system_state.memory_usage
                );
            },
            "emergency_cooling" => {
                reasoning = format!(
                    "Temperature is critical ({:.1}°C). Immediate thermal management required.",
                    system_state.temperature
                );
            },
            "gaming_optimization" => {
                reasoning = "Gaming workload detected. Optimizing for performance over power efficiency.".to_string();
            },
            "development_optimization" => {
                reasoning = "Development workload detected. Balancing performance with stability.".to_string();
            },
            _ => {
                reasoning = format!(
                    "Responding to user intent: {} with confidence {:.2}",
                    intent.action, intent.confidence
                );
            }
        }
        
        // Add workload context
        match system_state.current_workload {
            WorkloadType::Gaming => reasoning.push_str(" Gaming mode optimizations applied."),
            WorkloadType::Development => reasoning.push_str(" Development-friendly approach."),
            WorkloadType::Media => reasoning.push_str(" Media processing optimizations."),
            WorkloadType::SystemMaintenance => reasoning.push_str(" Maintenance window detected."),
            WorkloadType::Idle => reasoning.push_str(" System idle - safe to perform maintenance."),
        }
        
        reasoning
    }
    
    async fn predict_outcome(&self, action: &str, system_state: &SystemState) -> Result<ExpectedOutcome, Box<dyn std::error::Error>> {
        let outcome = match action {
            "optimize_cpu_high_usage" => ExpectedOutcome {
                description: "Reduce CPU usage and improve responsiveness".to_string(),
                estimated_duration: 10,
                system_impact: SystemImpact::Low,
                user_benefits: vec![
                    "Improved system responsiveness".to_string(),
                    "Reduced CPU temperature".to_string(),
                    "Better multitasking performance".to_string(),
                ],
            },
            
            "optimize_memory_aggressive" => ExpectedOutcome {
                description: format!("Free up memory from current {:.1}% usage", system_state.memory_usage),
                estimated_duration: 15,
                system_impact: SystemImpact::Low,
                user_benefits: vec![
                    "More available RAM".to_string(),
                    "Faster application launches".to_string(),
                    "Reduced swap usage".to_string(),
                ],
            },
            
            "emergency_cooling" => ExpectedOutcome {
                description: "Rapidly reduce CPU temperature".to_string(),
                estimated_duration: 30,
                system_impact: SystemImpact::Medium,
                user_benefits: vec![
                    "Prevent thermal throttling".to_string(),
                    "Protect hardware longevity".to_string(),
                    "Maintain performance".to_string(),
                ],
            },
            
            "gaming_optimization" => ExpectedOutcome {
                description: "Optimize system for gaming performance".to_string(),
                estimated_duration: 20,
                system_impact: SystemImpact::Medium,
                user_benefits: vec![
                    "Higher FPS in games".to_string(),
                    "Reduced input latency".to_string(),
                    "Smoother gameplay".to_string(),
                ],
            },
            
            "update_packages_safe" => ExpectedOutcome {
                description: "Update system packages with safety checks".to_string(),
                estimated_duration: 300, // 5 minutes
                system_impact: SystemImpact::Medium,
                user_benefits: vec![
                    "Latest security updates".to_string(),
                    "Bug fixes and improvements".to_string(),
                    "Better hardware support".to_string(),
                ],
            },
            
            _ => ExpectedOutcome {
                description: "Perform requested system operation".to_string(),
                estimated_duration: 30,
                system_impact: SystemImpact::Low,
                user_benefits: vec!["System improvement".to_string()],
            },
        };
        
        Ok(outcome)
    }
    
    fn calculate_priority(&self, action: &str, system_state: &SystemState, risk_level: &RiskLevel) -> u8 {
        let mut priority = 5; // Default priority
        
        // Emergency actions get highest priority
        if action.starts_with("emergency_") {
            priority = 10;
        }
        
        // Adjust based on system state
        if system_state.temperature > 85.0 {
            priority += 2;
        }
        
        if system_state.cpu_usage > 90.0 {
            priority += 1;
        }
        
        if system_state.memory_usage > 90.0 {
            priority += 1;
        }
        
        // Adjust based on risk level
        match risk_level {
            RiskLevel::Safe => priority += 1,
            RiskLevel::Low => priority += 0,
            RiskLevel::Medium => priority -= 1,
            RiskLevel::High => priority -= 2,
            RiskLevel::Critical => priority -= 3,
        }
        
        // Ensure priority stays in valid range
        priority.max(1).min(10)
    }
    
    async fn validate_decision(&self, decision: &Decision, system_state: &SystemState) -> Result<(), Box<dyn std::error::Error>> {
        // Check if action is safe to perform
        match decision.risk_level {
            RiskLevel::Critical | RiskLevel::High => {
                if !self.is_in_maintenance_window() && system_state.cpu_usage > 80.0 {
                    return Err("High-risk action not allowed during high system load outside maintenance window".into());
                }
            },
            _ => {} // Safe to proceed
        }
        
        // Check system constraints
        if system_state.memory_usage > self.system_constraints.max_memory_usage_threshold &&
           decision.action.contains("memory") {
            warn!("Memory usage is critically high, but proceeding with memory optimization");
        }
        
        Ok(())
    }
    
    fn adjust_confidence_based_on_history(&self, action: &str, base_confidence: f64) -> f64 {
        let successful_attempts = self.action_history.iter()
            .filter(|(decision, success)| decision.action == action && *success)
            .count();
        
        let total_attempts = self.action_history.iter()
            .filter(|(decision, _)| decision.action == action)
            .count();
        
        if total_attempts == 0 {
            return base_confidence;
        }
        
        let success_rate = successful_attempts as f64 / total_attempts as f64;
        
        // Adjust confidence based on historical success rate
        let adjusted_confidence = base_confidence * (0.5 + success_rate * 0.5);
        adjusted_confidence.max(0.1).min(1.0)
    }
    
    fn is_in_maintenance_window(&self) -> bool {
        let current_hour = chrono::Utc::now().hour() as u8;
        current_hour >= self.system_constraints.maintenance_window.0 &&
        current_hour <= self.system_constraints.maintenance_window.1
    }
    
    pub async fn record_decision_outcome(&mut self, decision: Decision, success: bool) -> Result<(), Box<dyn std::error::Error>> {
        debug!("📝 Recording decision outcome: {} -> {}", decision.action, success);
        
        self.action_history.push((decision, success));
        
        // Keep history manageable
        if self.action_history.len() > 1000 {
            self.action_history.remove(0);
        }
        
        Ok(())
    }
    
    pub fn get_decision_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        stats.insert("total_decisions".to_string(), self.action_history.len() as f64);
        
        let successful_decisions = self.action_history.iter()
            .filter(|(_, success)| *success)
            .count();
        
        let success_rate = if !self.action_history.is_empty() {
            successful_decisions as f64 / self.action_history.len() as f64
        } else {
            0.0
        };
        
        stats.insert("success_rate".to_string(), success_rate);
        stats.insert("total_rules".to_string(), self.decision_rules.len() as f64);
        
        // Action frequency statistics
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        for (decision, _) in &self.action_history {
            *action_counts.entry(decision.action.clone()).or_insert(0) += 1;
        }
        
        for (action, count) in action_counts {
            stats.insert(format!("action_{}", action), count as f64);
        }
        
        stats
    }
}
