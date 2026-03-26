use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use chrono::{DateTime, Utc};

// Missing types expected by main.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowAnalysis {
    pub flow_id: String,
    pub command_count: usize,
    pub dependency_count: usize,
    pub complexity_score: f64,
    pub execution_paths: Vec<Vec<String>>,
    pub potential_bottlenecks: Vec<String>,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub command: String,
    pub weight: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub source: String,
    pub target: String,
    pub weight: f64,
    pub edge_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDependency {
    pub command: String,
    pub depends_on: Vec<String>,
    pub required_by: Vec<String>,
    pub dependency_type: String,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionVisualization {
    pub execution_id: String,
    pub timeline: Vec<TimelineEvent>,
    pub resource_usage: HashMap<String, f64>,
    pub performance_metrics: HashMap<String, f64>,
    pub bottlenecks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub event_type: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub command: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub output_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandNode {
    pub id: String,
    pub command: String,
    pub description: String,
    pub category: CommandCategory,
    pub execution_time: Option<f64>,
    pub success_rate: f64,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub last_executed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandCategory {
    System,
    Git,
    Development,
    Network,
    FileSystem,
    Process,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEdge {
    pub from: String,
    pub to: String,
    pub relationship: EdgeType,
    pub weight: f64,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    RequiresBefore,   // Command must run before another
    RequiresAfter,    // Command must run after another
    ConditionalOn,    // Command runs conditionally based on another
    DataFlow,         // Data flows from one command to another
    ErrorHandling,    // Error handling relationship
    Pipeline,         // Part of a pipeline
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFlow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<CommandNode>,
    pub edges: Vec<CommandEdge>,
    pub entry_points: Vec<String>,
    pub exit_points: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowExecution {
    pub id: String,
    pub flow_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub current_node: Option<String>,
    pub executed_nodes: Vec<String>,
    pub failed_nodes: Vec<String>,
    pub execution_log: Vec<ExecutionLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogEntry {
    pub timestamp: DateTime<Utc>,
    pub node_id: String,
    pub command: String,
    pub duration: u64,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub event: ExecutionEvent,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    Started,
    Completed,
    Failed,
    Skipped,
    Retry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowVisualization {
    pub layout: FlowLayout,
    pub positions: HashMap<String, NodePosition>,
    pub clusters: Vec<NodeCluster>,
    pub critical_path: Vec<String>,
    pub bottlenecks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowLayout {
    Hierarchical,
    Circular,
    ForceDirected,
    Grid,
    Timeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCluster {
    pub id: String,
    pub name: String,
    pub nodes: Vec<String>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDependencyAnalysis {
    pub direct_dependencies: HashMap<String, Vec<String>>,
    pub transitive_dependencies: HashMap<String, Vec<String>>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub dependency_depth: HashMap<String, u32>,
    pub fan_in: HashMap<String, u32>,
    pub fan_out: HashMap<String, u32>,
}

#[derive(Debug)]
pub struct CommandFlowEngine {
    flows: HashMap<String, CommandFlow>,
    executions: HashMap<String, FlowExecution>,
    command_registry: HashMap<String, CommandNode>,
}

#[allow(dead_code)]
impl CommandFlowEngine {
    pub fn new() -> Self {
        Self {
            flows: HashMap::new(),
            executions: HashMap::new(),
            command_registry: HashMap::new(),
        }
    }

    pub fn register_command(&mut self, command: CommandNode) {
        self.command_registry.insert(command.id.clone(), command);
    }

    pub fn create_flow(&mut self, name: String, description: String) -> String {
        let flow_id = uuid::Uuid::new_v4().to_string();
        let flow = CommandFlow {
            id: flow_id.clone(),
            name,
            description,
            nodes: Vec::new(),
            edges: Vec::new(),
            entry_points: Vec::new(),
            exit_points: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.flows.insert(flow_id.clone(), flow);
        flow_id
    }

    pub fn add_command_to_flow(&mut self, flow_id: &str, command_id: &str) -> Result<()> {
        if let Some(flow) = self.flows.get_mut(flow_id) {
            if let Some(command) = self.command_registry.get(command_id) {
                flow.nodes.push(command.clone());
                flow.updated_at = Utc::now();
                Ok(())
            } else {
                Err(anyhow::anyhow!("Command not found: {}", command_id))
            }
        } else {
            Err(anyhow::anyhow!("Flow not found: {}", flow_id))
        }
    }

    pub fn add_dependency(&mut self, flow_id: &str, from: &str, to: &str, edge_type: EdgeType) -> Result<()> {
        if let Some(flow) = self.flows.get_mut(flow_id) {
            let edge = CommandEdge {
                from: from.to_string(),
                to: to.to_string(),
                relationship: edge_type,
                weight: 1.0,
                condition: None,
            };
            flow.edges.push(edge);
            flow.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Flow not found: {}", flow_id))
        }
    }

    pub fn analyze_dependencies(&self, flow_id: &str) -> Result<CommandDependencyAnalysis> {
        if let Some(flow) = self.flows.get(flow_id) {
            let mut analysis = CommandDependencyAnalysis {
                direct_dependencies: HashMap::new(),
                transitive_dependencies: HashMap::new(),
                circular_dependencies: Vec::new(),
                dependency_depth: HashMap::new(),
                fan_in: HashMap::new(),
                fan_out: HashMap::new(),
            };

            // Build direct dependencies
            for edge in &flow.edges {
                analysis.direct_dependencies
                    .entry(edge.to.clone())
                    .or_insert_with(Vec::new)
                    .push(edge.from.clone());
            }

            // Calculate fan-in and fan-out
            for node in &flow.nodes {
                let fan_in = flow.edges.iter().filter(|e| e.to == node.id).count();
                let fan_out = flow.edges.iter().filter(|e| e.from == node.id).count();
                analysis.fan_in.insert(node.id.clone(), fan_in as u32);
                analysis.fan_out.insert(node.id.clone(), fan_out as u32);
            }

            // Find circular dependencies
            analysis.circular_dependencies = self.find_circular_dependencies(flow);

            // Calculate transitive dependencies
            for node in &flow.nodes {
                analysis.transitive_dependencies.insert(
                    node.id.clone(),
                    self.get_transitive_dependencies(&analysis.direct_dependencies, &node.id)
                );
            }

            // Calculate dependency depth
            analysis.dependency_depth = self.calculate_dependency_depth(flow);

            Ok(analysis)
        } else {
            Err(anyhow::anyhow!("Flow not found: {}", flow_id))
        }
    }

    fn find_circular_dependencies(&self, flow: &CommandFlow) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in &flow.nodes {
            if !visited.contains(&node.id) {
                self.dfs_find_cycles(
                    &node.id,
                    flow,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_find_cycles(
        &self,
        node: &str,
        flow: &CommandFlow,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        for edge in &flow.edges {
            if edge.from == node {
                if rec_stack.contains(&edge.to) {
                    // Found a cycle
                    if let Some(start_idx) = path.iter().position(|n| n == &edge.to) {
                        cycles.push(path[start_idx..].to_vec());
                    }
                } else if !visited.contains(&edge.to) {
                    self.dfs_find_cycles(&edge.to, flow, visited, rec_stack, path, cycles);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    fn get_transitive_dependencies(
        &self,
        direct_deps: &HashMap<String, Vec<String>>,
        node: &str,
    ) -> Vec<String> {
        let mut all_deps = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(deps) = direct_deps.get(node) {
            for dep in deps {
                queue.push_back(dep.clone());
                all_deps.insert(dep.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(deps) = direct_deps.get(&current) {
                for dep in deps {
                    if !all_deps.contains(dep) {
                        all_deps.insert(dep.clone());
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        all_deps.into_iter().collect()
    }

    fn calculate_dependency_depth(&self, flow: &CommandFlow) -> HashMap<String, u32> {
        let mut depths = HashMap::new();
        let mut visited = HashSet::new();

        for node in &flow.nodes {
            if !visited.contains(&node.id) {
                self.calculate_depth_dfs(&node.id, flow, &mut depths, &mut visited, 0);
            }
        }

        depths
    }

    fn calculate_depth_dfs(
        &self,
        node: &str,
        flow: &CommandFlow,
        depths: &mut HashMap<String, u32>,
        visited: &mut HashSet<String>,
        current_depth: u32,
    ) {
        if visited.contains(node) {
            return;
        }

        visited.insert(node.to_string());
        depths.insert(node.to_string(), current_depth);

        for edge in &flow.edges {
            if edge.from == node {
                self.calculate_depth_dfs(&edge.to, flow, depths, visited, current_depth + 1);
            }
        }
    }

    pub fn generate_visualization(&self, flow_id: &str, layout: FlowLayout) -> Result<FlowVisualization> {
        if let Some(flow) = self.flows.get(flow_id) {
            let positions = self.calculate_layout(flow, &layout);
            let clusters = self.identify_clusters(flow);
            let critical_path = self.find_critical_path(flow);
            let bottlenecks = self.identify_bottlenecks(flow);

            Ok(FlowVisualization {
                layout,
                positions,
                clusters,
                critical_path,
                bottlenecks,
            })
        } else {
            Err(anyhow::anyhow!("Flow not found: {}", flow_id))
        }
    }

    fn calculate_layout(&self, flow: &CommandFlow, layout: &FlowLayout) -> HashMap<String, NodePosition> {
        let mut positions = HashMap::new();

        match layout {
            FlowLayout::Hierarchical => {
                let levels = self.calculate_levels(flow);
                let mut level_counts = HashMap::new();
                
                for (node_id, level) in &levels {
                    let count = level_counts.entry(*level).or_insert(0);
                    let x = *count as f64 * 150.0;
                    let y = *level as f64 * 100.0;
                    positions.insert(node_id.clone(), NodePosition { x, y, z: None });
                    *count += 1;
                }
            }
            FlowLayout::Circular => {
                let node_count = flow.nodes.len();
                let radius = 200.0;
                
                for (i, node) in flow.nodes.iter().enumerate() {
                    let angle = 2.0 * std::f64::consts::PI * i as f64 / node_count as f64;
                    let x = radius * angle.cos();
                    let y = radius * angle.sin();
                    positions.insert(node.id.clone(), NodePosition { x, y, z: None });
                }
            }
            FlowLayout::Grid => {
                let cols = (flow.nodes.len() as f64).sqrt().ceil() as usize;
                
                for (i, node) in flow.nodes.iter().enumerate() {
                    let x = (i % cols) as f64 * 150.0;
                    let y = (i / cols) as f64 * 100.0;
                    positions.insert(node.id.clone(), NodePosition { x, y, z: None });
                }
            }
            _ => {
                // Default simple layout
                for (i, node) in flow.nodes.iter().enumerate() {
                    let x = i as f64 * 100.0;
                    let y = 0.0;
                    positions.insert(node.id.clone(), NodePosition { x, y, z: None });
                }
            }
        }

        positions
    }

    fn calculate_levels(&self, flow: &CommandFlow) -> HashMap<String, usize> {
        let mut levels = HashMap::new();
        let mut visited = HashSet::new();

        // Find entry points (nodes with no incoming edges)
        for node in &flow.nodes {
            let has_incoming = flow.edges.iter().any(|e| e.to == node.id);
            if !has_incoming {
                self.assign_levels(&node.id, flow, &mut levels, &mut visited, 0);
            }
        }

        levels
    }

    fn assign_levels(
        &self,
        node: &str,
        flow: &CommandFlow,
        levels: &mut HashMap<String, usize>,
        visited: &mut HashSet<String>,
        level: usize,
    ) {
        if visited.contains(node) {
            return;
        }

        visited.insert(node.to_string());
        levels.insert(node.to_string(), level);

        for edge in &flow.edges {
            if edge.from == node {
                self.assign_levels(&edge.to, flow, levels, visited, level + 1);
            }
        }
    }

    fn identify_clusters(&self, flow: &CommandFlow) -> Vec<NodeCluster> {
        let mut clusters = Vec::new();
        let mut category_groups: HashMap<String, Vec<String>> = HashMap::new();

        // Group by category
        for node in &flow.nodes {
            let category = format!("{:?}", node.category);
            category_groups.entry(category.clone()).or_insert_with(Vec::new).push(node.id.clone());
        }

        for (category, nodes) in category_groups {
            if nodes.len() > 1 {
                clusters.push(NodeCluster {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: category.clone(),
                    nodes,
                    color: self.get_category_color(&category),
                });
            }
        }

        clusters
    }

    fn get_category_color(&self, category: &str) -> String {
        match category {
            "System" => "#FF6B6B".to_string(),
            "Git" => "#4ECDC4".to_string(),
            "Development" => "#45B7D1".to_string(),
            "Network" => "#FFA07A".to_string(),
            "FileSystem" => "#98D8C8".to_string(),
            "Process" => "#F7DC6F".to_string(),
            _ => "#DDA0DD".to_string(),
        }
    }

    fn find_critical_path(&self, flow: &CommandFlow) -> Vec<String> {
        // Simplified critical path calculation
        // In a real implementation, this would use proper scheduling algorithms
        let mut path = Vec::new();
        
        // Find the longest path through the graph
        if let Some(entry) = flow.entry_points.first() {
            path.push(entry.clone());
            let mut current = entry.clone();
            
            while let Some(next) = self.find_longest_successor(&current, flow) {
                path.push(next.clone());
                current = next;
            }
        }

        path
    }

    fn find_longest_successor(&self, node: &str, flow: &CommandFlow) -> Option<String> {
        let mut max_time = 0.0;
        let mut best_successor = None;

        for edge in &flow.edges {
            if edge.from == node {
                if let Some(successor_node) = flow.nodes.iter().find(|n| n.id == edge.to) {
                    let time = successor_node.execution_time.unwrap_or(0.0);
                    if time > max_time {
                        max_time = time;
                        best_successor = Some(edge.to.clone());
                    }
                }
            }
        }

        best_successor
    }

    fn identify_bottlenecks(&self, flow: &CommandFlow) -> Vec<String> {
        let mut bottlenecks = Vec::new();

        for node in &flow.nodes {
            let fan_in = flow.edges.iter().filter(|e| e.to == node.id).count();
            let _fan_out = flow.edges.iter().filter(|e| e.from == node.id).count();
            
            // Consider high fan-in or high execution time as bottlenecks
            if fan_in > 3 || node.execution_time.unwrap_or(0.0) > 10.0 {
                bottlenecks.push(node.id.clone());
            }
        }

        bottlenecks
    }

    pub async fn execute_flow(&mut self, flow_id: &str) -> Result<String> {
        if let Some(_flow) = self.flows.get(flow_id) {
            let execution_id = uuid::Uuid::new_v4().to_string();
            let execution = FlowExecution {
                id: execution_id.clone(),
                flow_id: flow_id.to_string(),
                started_at: Utc::now(),
                completed_at: None,
                status: ExecutionStatus::Running,
                current_node: None,
                executed_nodes: Vec::new(),
                failed_nodes: Vec::new(),
                execution_log: Vec::new(),
            };

            self.executions.insert(execution_id.clone(), execution);
            
            // Implement actual execution logic with topological sort
            let flow = _flow.clone();
            
            // Perform topological sort to determine execution order
            let execution_order = self.topological_sort(&flow)?;
            
            // Execute commands in order
            for node_id in execution_order {
                if let Some(node) = flow.nodes.iter().find(|n| n.id == node_id) {
                    // Update current node
                    if let Some(execution) = self.executions.get_mut(&execution_id) {
                        execution.current_node = Some(node_id.clone());
                        execution.status = ExecutionStatus::Running;
                    }
                    
                    let start_time = std::time::Instant::now();
                    match self.execute_node(&node, &flow).await {
                        Ok(output) => {
                            if let Some(execution) = self.executions.get_mut(&execution_id) {
                                execution.executed_nodes.push(node_id.clone());
                                execution.execution_log.push(ExecutionLogEntry {
                                    node_id: node_id.clone(),
                                    command: node.command.clone(),
                                    timestamp: Utc::now(),
                                    duration: start_time.elapsed().as_millis() as u64,
                                    success: true,
                                    output: Some(output),
                                    error: None,
                                    event: ExecutionEvent::Completed,
                                    details: "Command executed successfully".to_string(),
                                });
                            }
                        },
                        Err(error) => {
                            if let Some(execution) = self.executions.get_mut(&execution_id) {
                                execution.failed_nodes.push(node_id.clone());
                                execution.status = ExecutionStatus::Failed;
                                execution.execution_log.push(ExecutionLogEntry {
                                    node_id: node_id.clone(),
                                    command: node.command.clone(),
                                    timestamp: Utc::now(),
                                    duration: start_time.elapsed().as_millis() as u64,
                                    success: false,
                                    output: None,
                                    error: Some(error.to_string()),
                                    event: ExecutionEvent::Failed,
                                    details: "Command execution failed".to_string(),
                                });
                            }
                            break;
                        }
                    }
                }
            }
            
            // Update execution status
            if let Some(execution) = self.executions.get_mut(&execution_id) {
                if execution.status == ExecutionStatus::Running {
                    execution.status = ExecutionStatus::Completed;
                }
                execution.completed_at = Some(Utc::now());
            }
            
            Ok(execution_id)
        } else {
            Err(anyhow::anyhow!("Flow not found: {}", flow_id))
        }
    }

    pub fn get_flow(&self, flow_id: &str) -> Option<&CommandFlow> {
        self.flows.get(flow_id)
    }

    pub fn get_execution(&self, execution_id: &str) -> Option<&FlowExecution> {
        self.executions.get(execution_id)
    }

    pub fn list_flows(&self) -> Vec<&CommandFlow> {
        self.flows.values().collect()
    }

    pub fn export_flow(&self, flow_id: &str) -> Result<String> {
        if let Some(flow) = self.flows.get(flow_id) {
            serde_json::to_string_pretty(flow).map_err(Into::into)
        } else {
            Err(anyhow::anyhow!("Flow not found: {}", flow_id))
        }
    }

    pub fn import_flow(&mut self, flow_json: &str) -> Result<String> {
        let flow: CommandFlow = serde_json::from_str(flow_json)?;
        let flow_id = flow.id.clone();
        self.flows.insert(flow_id.clone(), flow);
        Ok(flow_id)
    }

    // Methods expected by main.rs
    pub async fn analyze_command(&self, command: &str, _context: &serde_json::Value) -> Result<FlowAnalysis> {
        // Mock implementation - analyze command complexity and dependencies
        let flow_id = uuid::Uuid::new_v4().to_string();
        
        Ok(FlowAnalysis {
            flow_id,
            command_count: 1,
            dependency_count: 0,
            complexity_score: self.calculate_command_complexity(command),
            execution_paths: vec![vec![command.to_string()]],
            potential_bottlenecks: Vec::new(),
            optimization_suggestions: vec![
                "Consider breaking complex commands into smaller parts".to_string(),
                "Add error handling for better reliability".to_string(),
            ],
        })
    }

    pub async fn create_dependency_graph(&self, commands: &[String]) -> Result<DependencyGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for (i, command) in commands.iter().enumerate() {
            nodes.push(DependencyNode {
                id: format!("node_{}", i),
                command: command.clone(),
                weight: 1.0,
                category: "unknown".to_string(),
            });

            // Create simple sequential dependencies
            if i > 0 {
                edges.push(DependencyEdge {
                    source: format!("node_{}", i - 1),
                    target: format!("node_{}", i),
                    weight: 1.0,
                    edge_type: "sequential".to_string(),
                });
            }
        }

        Ok(DependencyGraph {
            nodes,
            edges,
            metadata: HashMap::new(),
        })
    }

    pub async fn get_command_dependencies(&self, command: &str) -> Result<Vec<CommandDependency>> {
        // Mock implementation - in real app would analyze command dependencies
        let dependency = CommandDependency {
            command: command.to_string(),
            depends_on: self.extract_dependencies(command),
            required_by: Vec::new(),
            dependency_type: "runtime".to_string(),
            strength: 0.8,
        };
        
        Ok(vec![dependency])
    }

    pub async fn visualize_execution(&self, execution_id: &str) -> Result<ExecutionVisualization> {
        if let Some(execution) = self.executions.get(execution_id) {
            let mut timeline = Vec::new();
            let mut resource_usage = HashMap::new();
            let mut performance_metrics = HashMap::new();

            // Build timeline from execution log
            for entry in &execution.execution_log {
                timeline.push(TimelineEvent {
                    timestamp: entry.timestamp,
                    command: entry.node_id.clone(),
                    event_type: format!("{:?}", entry.event),
                    duration_ms: Some(100), // Mock duration
                });
            }

            // Mock resource usage
            resource_usage.insert("cpu".to_string(), 0.65);
            resource_usage.insert("memory".to_string(), 0.42);
            resource_usage.insert("disk".to_string(), 0.18);

            // Mock performance metrics
            performance_metrics.insert("total_duration_ms".to_string(), 2500.0);
            performance_metrics.insert("commands_per_second".to_string(), 0.4);
            performance_metrics.insert("success_rate".to_string(), 0.95);

            Ok(ExecutionVisualization {
                execution_id: execution_id.to_string(),
                timeline,
                resource_usage,
                performance_metrics,
                bottlenecks: vec!["slow_command_1".to_string()],
            })
        } else {
            Err(anyhow::anyhow!("Execution not found: {}", execution_id))
        }
    }

    pub async fn track_command_execution(&mut self, command: &str, start_time: &str) -> Result<String> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        
        // Parse start time or use current time
        let started_at = chrono::DateTime::parse_from_rfc3339(start_time)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| Utc::now());

        let execution = FlowExecution {
            id: execution_id.clone(),
            flow_id: "single_command".to_string(),
            started_at,
            completed_at: None,
            status: ExecutionStatus::Running,
            current_node: Some(command.to_string()),
            executed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
            execution_log: vec![
                ExecutionLogEntry {
                    timestamp: started_at,
                    node_id: command.to_string(),
                    command: command.to_string(),
                    duration: 0,
                    success: true,
                    output: None,
                    error: None,
                    event: ExecutionEvent::Started,
                    details: "Command execution started".to_string(),
                }
            ],
        };

        self.executions.insert(execution_id.clone(), execution);
        Ok(execution_id)
    }

    pub async fn get_execution_history(&self, limit: Option<u32>) -> Result<Vec<ExecutionRecord>> {
        let mut records = Vec::new();
        let limit = limit.unwrap_or(100) as usize;

        for (_, execution) in self.executions.iter().take(limit) {
            let duration_ms = execution.completed_at
                .map(|end| (end - execution.started_at).num_milliseconds() as u64);

            records.push(ExecutionRecord {
                id: execution.id.clone(),
                command: execution.current_node.clone().unwrap_or("unknown".to_string()),
                started_at: execution.started_at,
                completed_at: execution.completed_at,
                status: format!("{:?}", execution.status),
                duration_ms,
                exit_code: Some(0), // Mock exit code
                output_size: 1024,  // Mock output size
            });
        }

        Ok(records)
    }

    // Helper methods
    fn calculate_command_complexity(&self, command: &str) -> f64 {
        let mut complexity: f32 = 1.0;
        
        // Increase complexity based on command characteristics
        if command.contains("|") {
            complexity += 0.5; // Pipes increase complexity
        }
        if command.contains("&&") || command.contains("||") {
            complexity += 0.3; // Logic operators
        }
        if command.contains("$(") || command.contains("`") {
            complexity += 0.4; // Command substitution
        }
        if command.split_whitespace().count() > 5 {
            complexity += 0.2; // Long commands
        }
        
        complexity.min(5.0) as f64 // Cap at 5.0
    }

    fn extract_dependencies(&self, command: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Extract common dependencies based on command patterns
        if command.starts_with("git") {
            dependencies.push("git".to_string());
        }
        if command.contains("docker") {
            dependencies.push("docker".to_string());
        }
        if command.contains("npm") || command.contains("yarn") {
            dependencies.push("node".to_string());
        }
        if command.contains("cargo") {
            dependencies.push("rust".to_string());
        }
        
        dependencies
    }
    
    // Implement topological sort for flow execution
    fn topological_sort(&self, flow: &CommandFlow) -> Result<Vec<String>> {
        let mut in_degree = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize in-degree count for all nodes
        for node in &flow.nodes {
            in_degree.insert(node.id.clone(), 0);
            adj_list.insert(node.id.clone(), Vec::new());
        }
        
        // Build adjacency list and calculate in-degrees
        for edge in &flow.edges {
            adj_list.get_mut(&edge.from).unwrap().push(edge.to.clone());
            *in_degree.get_mut(&edge.to).unwrap() += 1;
        }
        
        // Start with nodes that have no dependencies
        let mut queue = VecDeque::new();
        for (node_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(node_id.clone());
            }
        }
        
        let mut result = Vec::new();
        
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            
            // Process all neighbors
            if let Some(neighbors) = adj_list.get(&current) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }
        
        // Check for cycles
        if result.len() != flow.nodes.len() {
            return Err(anyhow::anyhow!("Cycle detected in command flow"));
        }
        
        Ok(result)
    }
    
    // Execute a single node in the flow
    async fn execute_node(&self, node: &CommandNode, _flow: &CommandFlow) -> Result<String> {
        use tokio::process::Command;
        
        // Parse command into program and arguments
        let parts: Vec<&str> = node.command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }
        
        let program = parts[0];
        let args = &parts[1..];
        
        // Execute the command
        let output = Command::new(program)
            .args(args)
            .output()
            .await?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Command failed: {}", error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_flow_engine_creation() {
        let engine = CommandFlowEngine::new();
        assert!(engine.flows.is_empty());
        assert!(engine.command_registry.is_empty());
    }

    #[test]
    fn test_create_flow() {
        let mut engine = CommandFlowEngine::new();
        let flow_id = engine.create_flow("Test Flow".to_string(), "A test flow".to_string());
        assert!(engine.flows.contains_key(&flow_id));
    }

    #[test]
    fn test_register_command() {
        let mut engine = CommandFlowEngine::new();
        let command = CommandNode {
            id: "test-cmd".to_string(),
            command: "echo test".to_string(),
            description: "Test command".to_string(),
            category: CommandCategory::System,
            execution_time: Some(0.5),
            success_rate: 0.95,
            dependencies: vec![],
            dependents: vec![],
            metadata: HashMap::new(),
            last_executed: None,
        };
        
        engine.register_command(command);
        assert!(engine.command_registry.contains_key("test-cmd"));
    }
}
