use std::collections::HashMap;
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use crate::ai::{UserAction, SystemState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkWeights {
    input_hidden: DMatrix<f64>,
    hidden_hidden: Vec<DMatrix<f64>>,
    hidden_output: DMatrix<f64>,
    hidden_bias: Vec<DVector<f64>>,
    output_bias: DVector<f64>,
}

#[derive(Debug)]
pub struct NeuralNetwork {
    weights: NetworkWeights,
    learning_rate: f64,
    hidden_layers: Vec<usize>,
    input_size: usize,
    output_size: usize,
    activation_history: Vec<Vec<DVector<f64>>>,
}

impl NeuralNetwork {
    pub async fn new_for_sysadmin() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🧠 Initializing Neural Network for system administration...");
        
        let input_size = 12; // System metrics: CPU, RAM, disk, temp, processes, etc.
        let output_size = 8;  // Actions: optimize, clean, update, backup, etc.
        let hidden_layers = vec![24, 16, 12]; // Deep network for complex patterns
        
        let weights = Self::initialize_weights(input_size, &hidden_layers, output_size);
        
        Ok(Self {
            weights,
            learning_rate: 0.001,
            hidden_layers,
            input_size,
            output_size,
            activation_history: Vec::new(),
        })
    }
    
    fn initialize_weights(input_size: usize, hidden_layers: &[usize], output_size: usize) -> NetworkWeights {
        let mut rng = rand::thread_rng();
        
        // Xavier initialization for better training
        let input_hidden = DMatrix::from_fn(hidden_layers[0], input_size, |_, _| {
            (rand::Rng::gen::<f64>(&mut rng) - 0.5) * 2.0 * (6.0 / (input_size + hidden_layers[0]) as f64).sqrt()
        });
        
        let mut hidden_hidden = Vec::new();
        for i in 1..hidden_layers.len() {
            let weight_matrix = DMatrix::from_fn(hidden_layers[i], hidden_layers[i-1], |_, _| {
                (rand::Rng::gen::<f64>(&mut rng) - 0.5) * 2.0 * (6.0 / (hidden_layers[i-1] + hidden_layers[i]) as f64).sqrt()
            });
            hidden_hidden.push(weight_matrix);
        }
        
        let hidden_output = DMatrix::from_fn(output_size, *hidden_layers.last().unwrap(), |_, _| {
            (rand::Rng::gen::<f64>(&mut rng) - 0.5) * 2.0 * (6.0 / (*hidden_layers.last().unwrap() + output_size) as f64).sqrt()
        });
        
        let mut hidden_bias = Vec::new();
        for &layer_size in hidden_layers {
            hidden_bias.push(DVector::zeros(layer_size));
        }
        
        let output_bias = DVector::zeros(output_size);
        
        NetworkWeights {
            input_hidden,
            hidden_hidden,
            hidden_output,
            hidden_bias,
            output_bias,
        }
    }
    
    pub async fn initialize_with_context(&mut self, system_state: &SystemState) -> Result<(), Box<dyn std::error::Error>> {
        debug!("🔧 Initializing neural network with system context");
        
        let input_vector = self.system_state_to_vector(system_state);
        
        // Warm up the network with current state
        self.forward_pass(&input_vector);
        
        // Adjust initial weights based on system characteristics
        if system_state.cpu_usage > 70.0 {
            // Increase weights for performance optimization actions
            self.adjust_output_bias(0, 0.1); // CPU optimization
        }
        
        if system_state.memory_usage > 80.0 {
            self.adjust_output_bias(1, 0.1); // Memory optimization
        }
        
        Ok(())
    }
    
    fn system_state_to_vector(&self, state: &SystemState) -> DVector<f64> {
        DVector::from_vec(vec![
            state.cpu_usage / 100.0,
            state.memory_usage / 100.0,
            state.disk_usage / 100.0,
            (state.temperature - 30.0) / 70.0, // Normalize 30-100°C to 0-1
            state.active_processes.len() as f64 / 500.0, // Normalize process count
            state.time_of_day as f64 / 24.0,
            state.day_of_week as f64 / 7.0,
            match state.current_workload {
                crate::ai::WorkloadType::Gaming => 1.0,
                crate::ai::WorkloadType::Development => 0.8,
                crate::ai::WorkloadType::Media => 0.6,
                crate::ai::WorkloadType::SystemMaintenance => 0.4,
                crate::ai::WorkloadType::Idle => 0.2,
            },
            // Additional features for Lou's specific patterns
            if state.active_processes.contains(&"firefox".to_string()) { 1.0 } else { 0.0 },
            if state.active_processes.contains(&"code".to_string()) { 1.0 } else { 0.0 },
            if state.active_processes.contains(&"steam".to_string()) { 1.0 } else { 0.0 },
            if state.cpu_usage > 90.0 && state.temperature > 85.0 { 1.0 } else { 0.0 }, // Thermal stress
        ])
    }
    
    pub fn forward_pass(&mut self, input: &DVector<f64>) -> DVector<f64> {
        let mut activations = Vec::new();
        
        // Input to first hidden layer
        let mut current_activation = self.relu(&(&self.weights.input_hidden * input + &self.weights.hidden_bias[0]));
        activations.push(current_activation.clone());
        
        // Hidden layers
        for i in 0..self.weights.hidden_hidden.len() {
            current_activation = self.relu(&(&self.weights.hidden_hidden[i] * &current_activation + &self.weights.hidden_bias[i + 1]));
            activations.push(current_activation.clone());
        }
        
        // Output layer with sigmoid for action probabilities
        let output = self.sigmoid(&(&self.weights.hidden_output * &current_activation + &self.weights.output_bias));
        
        // Store for backpropagation
        self.activation_history.push(activations);
        
        output
    }
    
    pub async fn train_on_action(&mut self, action: &UserAction) -> Result<(), Box<dyn std::error::Error>> {
        debug!("📚 Training neural network on user action: {}", action.action_type);
        
        // Create training data from the action
        let system_metrics = self.extract_system_metrics_from_context(&action.context);
        let input_vector = self.context_to_input_vector(&system_metrics);
        
        // Create target output based on action success/failure
        let target_output = self.action_to_target_output(&action.action_type, &action.outcome);
        
        // Forward pass
        let predicted_output = self.forward_pass(&input_vector);
        
        // Backpropagation
        self.backward_pass(&input_vector, &target_output, &predicted_output);
        
        Ok(())
    }
    
    fn extract_system_metrics_from_context(&self, context: &str) -> SystemState {
        // Parse context string to extract system metrics
        // In a real implementation, this would parse actual system data
        SystemState {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            disk_usage: 70.0,
            temperature: 65.0,
            active_processes: vec!["firefox".to_string(), "vscode".to_string()],
            current_workload: crate::ai::WorkloadType::Development,
            time_of_day: chrono::Utc::now().hour() as u8,
            day_of_week: chrono::Utc::now().weekday().number_from_monday() as u8 - 1,
        }
    }
    
    fn context_to_input_vector(&self, metrics: &SystemState) -> DVector<f64> {
        self.system_state_to_vector(metrics)
    }
    
    fn action_to_target_output(&self, action_type: &str, outcome: &crate::ai::ActionOutcome) -> DVector<f64> {
        let mut target = DVector::zeros(self.output_size);
        
        // Map actions to output indices
        let action_index = match action_type {
            "optimize_cpu" => 0,
            "optimize_memory" => 1,
            "clean_system" => 2,
            "update_packages" => 3,
            "backup_data" => 4,
            "optimize_disk" => 5,
            "manage_processes" => 6,
            _ => 7, // Other/custom actions
        };
        
        // Set target based on action outcome
        let target_value = match outcome {
            crate::ai::ActionOutcome::Success => 1.0,
            crate::ai::ActionOutcome::Partial(_) => 0.7,
            crate::ai::ActionOutcome::Failed(_) => 0.1,
        };
        
        target[action_index] = target_value;
        target
    }
    
    fn backward_pass(&mut self, input: &DVector<f64>, target: &DVector<f64>, predicted: &DVector<f64>) {
        let output_error = predicted - target;
        let output_delta = &output_error.component_mul(&self.sigmoid_derivative(predicted));
        
        // Update output weights
        if let Some(last_hidden) = self.activation_history.last().and_then(|h| h.last()) {
            let output_gradient = output_delta * last_hidden.transpose();
            self.weights.hidden_output -= self.learning_rate * output_gradient;
            self.weights.output_bias -= self.learning_rate * output_delta;
        }
        
        // Backpropagate through hidden layers
        let mut error = self.weights.hidden_output.transpose() * output_delta;
        
        if let Some(activations) = self.activation_history.last() {
            for i in (0..self.weights.hidden_hidden.len()).rev() {
                let delta = error.component_mul(&self.relu_derivative(&activations[i + 1]));
                
                let gradient = &delta * activations[i].transpose();
                self.weights.hidden_hidden[i] -= self.learning_rate * gradient;
                self.weights.hidden_bias[i + 1] -= self.learning_rate * &delta;
                
                if i > 0 {
                    error = self.weights.hidden_hidden[i].transpose() * &delta;
                } else {
                    error = self.weights.input_hidden.transpose() * &delta;
                }
            }
            
            // Update input to first hidden layer weights
            let first_delta = error.component_mul(&self.relu_derivative(&activations[0]));
            let input_gradient = &first_delta * input.transpose();
            self.weights.input_hidden -= self.learning_rate * input_gradient;
            self.weights.hidden_bias[0] -= self.learning_rate * &first_delta;
        }
    }
    
    pub fn predict_best_action(&mut self, system_state: &SystemState) -> (String, f64) {
        let input_vector = self.system_state_to_vector(system_state);
        let output = self.forward_pass(&input_vector);
        
        let mut best_action_idx = 0;
        let mut best_confidence = output[0];
        
        for i in 1..output.len() {
            if output[i] > best_confidence {
                best_confidence = output[i];
                best_action_idx = i;
            }
        }
        
        let action_name = match best_action_idx {
            0 => "optimize_cpu",
            1 => "optimize_memory", 
            2 => "clean_system",
            3 => "update_packages",
            4 => "backup_data",
            5 => "optimize_disk",
            6 => "manage_processes",
            _ => "monitor_system",
        };
        
        (action_name.to_string(), best_confidence)
    }
    
    fn adjust_output_bias(&mut self, output_idx: usize, adjustment: f64) {
        if output_idx < self.output_size {
            self.weights.output_bias[output_idx] += adjustment;
        }
    }
    
    fn relu(&self, x: &DVector<f64>) -> DVector<f64> {
        x.map(|val| val.max(0.0))
    }
    
    fn relu_derivative(&self, x: &DVector<f64>) -> DVector<f64> {
        x.map(|val| if val > 0.0 { 1.0 } else { 0.0 })
    }
    
    fn sigmoid(&self, x: &DVector<f64>) -> DVector<f64> {
        x.map(|val| 1.0 / (1.0 + (-val).exp()))
    }
    
    fn sigmoid_derivative(&self, x: &DVector<f64>) -> DVector<f64> {
        x.component_mul(&x.map(|val| 1.0 - val))
    }
    
    pub fn get_network_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        // Calculate network complexity metrics
        let total_weights = self.weights.input_hidden.len() +
            self.weights.hidden_hidden.iter().map(|m| m.len()).sum::<usize>() +
            self.weights.hidden_output.len();
            
        stats.insert("total_weights".to_string(), total_weights as f64);
        stats.insert("learning_rate".to_string(), self.learning_rate);
        stats.insert("hidden_layers".to_string(), self.hidden_layers.len() as f64);
        
        // Calculate average weight magnitudes
        let avg_input_weight = self.weights.input_hidden.iter().map(|w| w.abs()).sum::<f64>() / self.weights.input_hidden.len() as f64;
        stats.insert("avg_input_weight".to_string(), avg_input_weight);
        
        stats
    }
}
