use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("AI service error: {0}")]
    AIService(String),
    
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("System error: {0}")]
    System(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Update error: {0}")]
    Update(String),
    
    #[error("Backup error: {0}")]
    Backup(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    #[error("Telemetry error: {0}")]
    Telemetry(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Parsing error: {0}")]
    Parsing(String),
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Network(_) => "NETWORK_ERROR",
            AppError::AIService(_) => "AI_SERVICE_ERROR",
            AppError::FileSystem(_) => "FILE_SYSTEM_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Permission(_) => "PERMISSION_ERROR",
            AppError::RateLimit(_) => "RATE_LIMIT_ERROR",
            AppError::Configuration(_) => "CONFIG_ERROR",
            AppError::Security(_) => "SECURITY_ERROR",
            AppError::System(_) => "SYSTEM_ERROR",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Cache(_) => "CACHE_ERROR",
            AppError::Update(_) => "UPDATE_ERROR",
            AppError::Backup(_) => "BACKUP_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Plugin(_) => "PLUGIN_ERROR",
            AppError::Telemetry(_) => "TELEMETRY_ERROR",
            AppError::NotFound(_) => "NOT_FOUND_ERROR",
            AppError::Parsing(_) => "PARSING_ERROR",
        }
    }
    
    pub fn is_recoverable(&self) -> bool {
        match self {
            AppError::Network(_) => true,
            AppError::RateLimit(_) => true,
            AppError::Cache(_) => true,
            AppError::AIService(_) => true,
            _ => false,
        }
    }
    
    pub fn should_retry(&self) -> bool {
        match self {
            AppError::Network(_) => true,
            AppError::RateLimit(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_code: String,
    pub message: String,
    pub timestamp: String,
    pub recoverable: bool,
    pub should_retry: bool,
    pub context: Option<serde_json::Value>,
}

impl From<AppError> for ErrorContext {
    fn from(error: AppError) -> Self {
        ErrorContext {
            error_code: error.error_code().to_string(),
            message: error.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            recoverable: error.is_recoverable(),
            should_retry: error.should_retry(),
            context: None,
        }
    }
}

// Convert standard IO errors to AppError
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::FileSystem(error.to_string())
    }
}

// Convert serde errors to AppError
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Validation(format!("JSON error: {}", error))
    }
}

// Convert reqwest errors to AppError
impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError::Network(error.to_string())
    }
}

// Convert rusqlite errors to AppError
impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        AppError::Database(error.to_string())
    }
}

// Convert toml errors to AppError
impl From<toml::de::Error> for AppError {
    fn from(error: toml::de::Error) -> Self {
        AppError::Configuration(format!("TOML parsing error: {}", error))
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(error: toml::ser::Error) -> Self {
        AppError::Configuration(format!("TOML serialization error: {}", error))
    }
}

// Convert git2 errors to AppError
impl From<git2::Error> for AppError {
    fn from(error: git2::Error) -> Self {
        AppError::System(format!("Git error: {}", error))
    }
}

pub type AppResult<T> = Result<T, AppError>;

// Helper macro for error handling
#[macro_export]
macro_rules! app_error {
    ($variant:ident, $msg:expr) => {
        crate::errors::AppError::$variant($msg.to_string())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        crate::errors::AppError::$variant(format!($fmt, $($arg)*))
    };
}

// Retry mechanism for recoverable errors
pub async fn retry_on_error<F, Fut, T>(
    operation: F,
    max_retries: usize,
    base_delay: std::time::Duration,
) -> AppResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if !error.should_retry() || attempt == max_retries {
                    return Err(error);
                }
                
                last_error = Some(error);
                
                // Exponential backoff
                let delay = base_delay * 2_u32.pow(attempt as u32);
                tokio::time::sleep(delay).await;
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| AppError::Internal("Retry logic failed".to_string())))
}
