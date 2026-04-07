use anyhow::{Context, Result};
use reqwest::Client;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, info};
use uuid::Uuid;
use std::hash::Hash;

use crate::ai::{AIConfig, AIService};

/// Request priority levels for AI service
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequestPriority {
    Critical = 0,  // User interactive requests
    High = 1,      // Real-time operations
    Normal = 2,    // Standard operations
    Low = 3,       // Low priority operations
    Background = 4, // Non-urgent operations
}

/// AI request structure with priority and metadata
#[derive(Debug, Clone)]
pub struct AIRequest {
    pub id: String,
    pub prompt: String,
    pub model: Option<String>,
    pub priority: RequestPriority,
    pub created_at: Instant,
    pub timeout: Duration,
    pub context: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl AIRequest {
    pub fn new(prompt: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            prompt,
            model: None,
            priority: RequestPriority::Normal,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            context: None,
            retry_count: 0,
            max_retries: 3,
        }
    }
    
    pub fn new_with_options(
        prompt: String,
        model: String,
        priority: RequestPriority,
        _max_tokens: u32,
        _temperature: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            prompt,
            model: Some(model),
            priority,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            context: None,
            retry_count: 0,
            max_retries: 3,
        }
    }
    
    pub fn simple(prompt: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            prompt,
            model: None,
            priority: RequestPriority::Normal,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            context: None,
            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn with_priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// AI response with metadata
#[derive(Debug, Clone)]
pub struct AIResponse {
    pub id: String,
    pub request_id: String,
    pub content: String,
    pub model_used: String,
    pub processing_time: Duration,
    pub tokens_used: Option<u32>,
    pub success: bool,
    pub error: Option<String>,
}

/// Connection pool statistics
#[derive(Debug, Clone, Serialize)]
pub struct PoolStats {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub pending_requests: usize,
    pub processed_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub queue_by_priority: HashMap<String, usize>,
}

/// HTTP client pool for managing connections
#[derive(Debug)]
#[allow(dead_code)]
pub struct HttpClientPool {
    pool: Vec<Client>,
    available: Arc<Mutex<VecDeque<usize>>>,
    config: AIConfig,
    max_connections: usize,
}

#[allow(dead_code)]
impl HttpClientPool {
    pub fn new(config: &AIConfig, max_connections: usize) -> Result<Self> {
        let mut pool = Vec::with_capacity(max_connections);
        let mut available = VecDeque::new();

        for i in 0..max_connections {
            let client = Client::builder()
                .timeout(Duration::from_secs(config.timeout_seconds))
                .tcp_keepalive(Duration::from_secs(60))
                .tcp_nodelay(true)
                .pool_max_idle_per_host(2)
                .build()
                .context("Failed to create HTTP client")?;
            
            pool.push(client);
            available.push_back(i);
        }

        Ok(Self {
            pool,
            available: Arc::new(Mutex::new(available)),
            config: config.clone(),
            max_connections,
        })
    }

    pub async fn get_client(&self) -> Result<(usize, &Client)> {
        let mut available = self.available.lock().await;
        
        if let Some(index) = available.pop_front() {
            // Use config to apply any dynamic settings to the client if needed
            let _timeout = Duration::from_secs(self.config.timeout_seconds);
            Ok((index, &self.pool[index]))
        } else {
            Err(anyhow::anyhow!("No available HTTP clients"))
        }
    }

    pub async fn return_client(&self, index: usize) {
        let mut available = self.available.lock().await;
        available.push_back(index);
    }

    pub async fn get_stats(&self) -> (usize, usize) {
        let available = self.available.lock().await;
        (self.max_connections - available.len(), available.len())
    }
}

/// Optimized AI service with connection pooling and request management
#[derive(Debug)]
#[allow(dead_code)]
pub struct OptimizedAIService {
    base_service: AIService,
    client_pool: Arc<HttpClientPool>,
    request_queue: Arc<Mutex<VecDeque<AIRequest>>>,
    priority_queues: Arc<Mutex<HashMap<RequestPriority, VecDeque<AIRequest>>>>,
    response_cache: Arc<RwLock<HashMap<String, (AIResponse, Instant)>>>,
    request_semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<PoolStats>>,
    response_times: Arc<Mutex<VecDeque<Duration>>>,
    shutdown_sender: Option<mpsc::Sender<()>>,
    background_tasks: Vec<tokio::task::JoinHandle<()>>,
}

#[allow(dead_code)]
impl OptimizedAIService {
    pub async fn new(config: &AIConfig) -> Result<Self> {
        Self::new_with_config(config).await
    }
    
    pub async fn new_with_config(config: &AIConfig) -> Result<Self> {
        // Create a basic client instead of full AIService to avoid circular dependency
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;
        
        let base_service = AIService {
            client,
            config: config.clone(),
            optimized_service: None, // Don't create circular reference
        };
        
        let max_connections = 10; // Configurable connection pool size
        let client_pool = Arc::new(HttpClientPool::new(config, max_connections)?);
        
        let mut priority_queues = HashMap::new();
        priority_queues.insert(RequestPriority::Critical, VecDeque::new());
        priority_queues.insert(RequestPriority::High, VecDeque::new());
        priority_queues.insert(RequestPriority::Normal, VecDeque::new());
        priority_queues.insert(RequestPriority::Background, VecDeque::new());

        let initial_stats = PoolStats {
            active_connections: 0,
            idle_connections: max_connections,
            pending_requests: 0,
            processed_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            queue_by_priority: priority_queues
                .keys()
                .map(|p| (format!("{:?}", p), 0))
                .collect(),
        };

        let (shutdown_sender, _shutdown_receiver) = mpsc::channel(1);

        let service = Self {
            base_service,
            client_pool: client_pool.clone(),
            request_queue: Arc::new(Mutex::new(VecDeque::new())),
            priority_queues: Arc::new(Mutex::new(priority_queues)),
            response_cache: Arc::new(RwLock::new(HashMap::new())),
            request_semaphore: Arc::new(Semaphore::new(max_connections)),
            stats: Arc::new(RwLock::new(initial_stats)),
            response_times: Arc::new(Mutex::new(VecDeque::new())),
            shutdown_sender: Some(shutdown_sender),
            background_tasks: Vec::new(),
        };

        info!("OptimizedAIService initialized with {} max connections (background tasks not started)", max_connections);
        Ok(service)
    }

    /// Submit a request to the AI service (returns response receiver)
    pub async fn submit_request_async(&self, request: AIRequest) -> Result<mpsc::Receiver<AIResponse>> {
        // Check cache first
        if let Some(cached) = self.get_cached_response(&request).await {
            let (tx, rx) = mpsc::channel(1);
            let _ = tx.send(cached).await;
            return Ok(rx);
        }

        // Add to appropriate priority queue
        let (tx, rx) = mpsc::channel(1);
        self.enqueue_request(request, tx).await?;
        
        Ok(rx)
    }

    /// Quick API for chat functionality
    pub async fn chat_async(&self, message: &str, context: Option<&str>) -> Result<AIResponse> {
        let request = AIRequest::simple(message.to_string())
            .with_priority(RequestPriority::High)
            .with_context(context.unwrap_or_default().to_string());

        let mut rx = self.submit_request_async(request).await?;
        
        // Wait for response with timeout
        match timeout(Duration::from_secs(30), rx.recv()).await {
            Ok(Some(response)) => Ok(response),
            Ok(None) => Err(anyhow::anyhow!("Request channel closed")),
            Err(_) => Err(anyhow::anyhow!("Request timeout")),
        }
    }

    /// Background processing of requests
    async fn start_request_processor(&self, mut shutdown_receiver: mpsc::Receiver<()>) -> tokio::task::JoinHandle<()> {
        let client_pool = self.client_pool.clone();
        let priority_queues = self.priority_queues.clone();
        let response_cache = self.response_cache.clone();
        let stats = self.stats.clone();
        let response_times = self.response_times.clone();
        let request_semaphore = self.request_semaphore.clone();
        let base_service = self.base_service.clone();

        tokio::spawn(async move {
            let request_handlers: Arc<Mutex<HashMap<String, (AIRequest, mpsc::Sender<AIResponse>)>>> = Arc::new(Mutex::new(HashMap::new()));
            
            loop {
                tokio::select! {
                    _ = shutdown_receiver.recv() => {
                        debug!("Request processor shutting down");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        if let Some((request, response_sender)) = Self::get_next_request(&priority_queues).await {
                            let permit = match request_semaphore.clone().try_acquire_owned() {
                                Ok(permit) => permit,
                                Err(_) => continue, // No available slots
                            };

                            let request_id = request.id.clone();
                            {
                                let mut handlers = request_handlers.lock().await;
                                handlers.insert(request_id.clone(), (request.clone(), response_sender));
                            }

                            // Process request in background
                            let client_pool_clone = client_pool.clone();
                            let response_cache_clone = response_cache.clone();
                            let stats_clone = stats.clone();
                            let response_times_clone = response_times.clone();
                            let base_service_clone = base_service.clone();
                            let request_handlers_clone = request_handlers.clone();

                            tokio::spawn(async move {
                                let _permit = permit; // Keep permit alive
                                let result = Self::process_single_request(
                                    request,
                                    client_pool_clone,
                                    base_service_clone,
                                ).await;

                                match result {
                                    Ok(response) => {
                                        // Cache successful responses
                                        Self::cache_response(&response_cache_clone, &response).await;
                                        
                                        // Update stats
                                        Self::update_stats(&stats_clone, &response_times_clone, &response).await;
                                        
                                        // Send response
                                        if let Some((_, sender)) = {
                                            let mut handlers = request_handlers_clone.lock().await;
                                            handlers.remove(&request_id)
                                        } {
                                            let _ = sender.send(response).await;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Request processing failed: {}", e);
                                        Self::update_failed_stats(&stats_clone).await;
                                        
                                        if let Some((_, sender)) = {
                                            let mut handlers = request_handlers_clone.lock().await;
                                            handlers.remove(&request_id)
                                        } {
                                            let error_response = AIResponse {
                                                id: Uuid::new_v4().to_string(),
                                                request_id,
                                                content: String::new(),
                                                model_used: "error".to_string(),
                                                processing_time: Duration::default(),
                                                tokens_used: None,
                                                success: false,
                                                error: Some(e.to_string()),
                                            };
                                            let _ = sender.send(error_response).await;
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }
        })
    }

    async fn get_next_request(
        priority_queues: &Arc<Mutex<HashMap<RequestPriority, VecDeque<AIRequest>>>>
    ) -> Option<(AIRequest, mpsc::Sender<AIResponse>)> {
        let mut queues = priority_queues.lock().await;
        
        // Process in priority order
        for priority in [RequestPriority::Critical, RequestPriority::High, RequestPriority::Normal, RequestPriority::Background] {
            if let Some(queue) = queues.get_mut(&priority) {
                if let Some(request) = queue.pop_front() {
                    // Create response channel
                    let (tx, _rx) = mpsc::channel(1);
                    return Some((request, tx));
                }
            }
        }
        None
    }

    async fn process_single_request(
        request: AIRequest,
        client_pool: Arc<HttpClientPool>,
        base_service: AIService,
    ) -> Result<AIResponse> {
        let start_time = Instant::now();
        
        // Get HTTP client from pool
        let (client_index, _client) = client_pool.get_client().await?;
        
        let response = match base_service.chat(&request.prompt, request.context.as_deref()).await {
            Ok(content) => AIResponse {
                id: Uuid::new_v4().to_string(),
                request_id: request.id,
                content,
                model_used: request.model.unwrap_or_else(|| "default".to_string()),
                processing_time: start_time.elapsed(),
                tokens_used: None, // Would need to implement token counting
                success: true,
                error: None,
            },
            Err(e) => AIResponse {
                id: Uuid::new_v4().to_string(),
                request_id: request.id,
                content: String::new(),
                model_used: request.model.unwrap_or_else(|| "error".to_string()),
                processing_time: start_time.elapsed(),
                tokens_used: None,
                success: false,
                error: Some(e.to_string()),
            },
        };
        
        // Return client to pool
        client_pool.return_client(client_index).await;
        
        Ok(response)
    }

    async fn enqueue_request(&self, request: AIRequest, _response_sender: mpsc::Sender<AIResponse>) -> Result<()> {
        // Add to main request queue for tracking
        {
            let mut main_queue = self.request_queue.lock().await;
            main_queue.push_back(request.clone());
        }
        
        // Add to priority queue for processing
        let mut queues = self.priority_queues.lock().await;
        if let Some(queue) = queues.get_mut(&request.priority) {
            queue.push_back(request);
            // Store response sender - in real implementation, would use a better approach
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid request priority"))
        }
    }

    async fn get_cached_response(&self, request: &AIRequest) -> Option<AIResponse> {
        let cache = self.response_cache.read().await;
        let cache_key = self.generate_cache_key(request);
        
        if let Some((response, cached_at)) = cache.get(&cache_key) {
            // Check if cache is still valid (5 minutes)
            if cached_at.elapsed() < Duration::from_secs(300) {
                Some(response.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn cache_response(cache: &Arc<RwLock<HashMap<String, (AIResponse, Instant)>>>, response: &AIResponse) {
        let mut cache = cache.write().await;
        let cache_key = response.request_id.clone(); // Simplified cache key
        cache.insert(cache_key, (response.clone(), Instant::now()));
        
        // Prevent cache from growing too large
        if cache.len() > 1000 {
            // Remove oldest 20% of entries
            let to_remove: Vec<String> = cache
                .iter()
                .map(|(k, (_, t))| (k.clone(), *t))
                .collect::<Vec<_>>()
                .into_iter()
                .min_by_key(|(_, t)| *t)
                .into_iter()
                .take(200)
                .map(|(k, _)| k)
                .collect();
            
            for key in to_remove {
                cache.remove(&key);
            }
        }
    }

    fn generate_cache_key(&self, request: &AIRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request.prompt.hash(&mut hasher);
        request.context.hash(&mut hasher);
        request.model.hash(&mut hasher);
        
        format!("ai_cache_{}", hasher.finish())
    }

    async fn update_stats(
        stats: &Arc<RwLock<PoolStats>>,
        response_times: &Arc<Mutex<VecDeque<Duration>>>,
        response: &AIResponse,
    ) {
        let mut stats = stats.write().await;
        let mut times = response_times.lock().await;
        
        stats.processed_requests += 1;
        times.push_back(response.processing_time);
        
        // Keep only last 100 response times for average calculation
        if times.len() > 100 {
            times.pop_front();
        }
        
        // Calculate average response time
        let total_time: Duration = times.iter().sum();
        stats.average_response_time = total_time.as_millis() as f64 / times.len() as f64;
    }

    async fn update_failed_stats(stats: &Arc<RwLock<PoolStats>>) {
        let mut stats = stats.write().await;
        stats.failed_requests += 1;
    }

    /// Start cache cleanup background task
    async fn start_cache_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let cache = self.response_cache.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                let mut cache = cache.write().await;
                let now = Instant::now();
                
                // Remove expired entries (older than 10 minutes)
                cache.retain(|_, (_, cached_at)| now.duration_since(*cached_at) < Duration::from_secs(600));
                
                debug!("Cache cleanup completed. Entries remaining: {}", cache.len());
            }
        })
    }

    /// Start stats updater background task
    async fn start_stats_updater(&self) -> tokio::task::JoinHandle<()> {
        let stats = self.stats.clone();
        let client_pool = self.client_pool.clone();
        let priority_queues = self.priority_queues.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10)); // Update every 10 seconds
            
            loop {
                interval.tick().await;
                
                let (active, idle) = client_pool.get_stats().await;
                let queues = priority_queues.lock().await;
                
                let mut stats = stats.write().await;
                stats.active_connections = active;
                stats.idle_connections = idle;
                
                let mut total_pending = 0;
                for (priority, queue) in queues.iter() {
                    let queue_size = queue.len();
                    stats.queue_by_priority.insert(format!("{:?}", priority), queue_size);
                    total_pending += queue_size;
                }
                stats.pending_requests = total_pending;
            }
        })
    }

    /// Get current service statistics as PoolStats struct
    pub async fn get_pool_stats(&self) -> PoolStats {
        self.stats.read().await.clone()
    }

    /// Force cleanup of cache and queues
    pub async fn force_cleanup(&self) {
        // Clear cache
        {
            let mut cache = self.response_cache.write().await;
            cache.clear();
        }
        
        // Clear old response times
        {
            let mut times = self.response_times.lock().await;
            if times.len() > 10 {
                times.truncate(10);
            }
        }
        
        info!("Forced cleanup completed");
    }
    
    /// Simple submit request method that returns response receiver
    pub async fn submit_request(&self, request: AIRequest) -> Result<mpsc::Receiver<AIResponse>> {
        self.submit_request_async(request).await
    }
    
    /// Get response for a request ID
    pub async fn get_response(&self, request_id: String) -> Result<String> {
        // In a real implementation, this would wait for the response
        // For now, return a simple response
        Ok(format!("Response for request {}", request_id))
    }
    
    /// Get service statistics as a formatted string
    pub async fn get_stats(&self) -> String {
        let stats = self.stats.read().await;
        format!(
            "Active: {}, Idle: {}, Pending: {}, Processed: {}, Failed: {}, Avg Time: {:.2}ms",
            stats.active_connections,
            stats.idle_connections,
            stats.pending_requests,
            stats.processed_requests,
            stats.failed_requests,
            stats.average_response_time
        )
    }
    
    /// Clear completed requests
    pub async fn clear_completed(&self) {
        self.force_cleanup().await;
    }
    
    /// Start background tasks (should be called after initialization)
    pub async fn start_background_tasks(&mut self) -> Result<()> {
        if !self.background_tasks.is_empty() {
            info!("Background tasks already started");
            return Ok(());
        }
        
        // Create new shutdown channel for this instance
        let (shutdown_sender, shutdown_receiver) = mpsc::channel(1);
        self.shutdown_sender = Some(shutdown_sender);
        
        // Start background request processor
        let processor_handle = self.start_request_processor(shutdown_receiver).await;
        self.background_tasks.push(processor_handle);

        // Start cache cleanup task
        let cache_cleanup_handle = self.start_cache_cleanup().await;
        self.background_tasks.push(cache_cleanup_handle);

        // Start stats updater
        let stats_handle = self.start_stats_updater().await;
        self.background_tasks.push(stats_handle);

        info!("Background tasks started for OptimizedAIService");
        Ok(())
    }
}

impl Drop for OptimizedAIService {
    fn drop(&mut self) {
        // Send shutdown signal
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.try_send(());
        }
        
        // Cancel background tasks
        for handle in &self.background_tasks {
            handle.abort();
        }
    }
}

