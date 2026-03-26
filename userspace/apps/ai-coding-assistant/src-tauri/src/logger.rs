use tracing_subscriber;
use tracing::{info, warn, error};

// Initialize logging for the application
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
    
    info!("Logging initialized");
}

// Helper to log errors with context
pub fn log_error_with_context(context: crate::errors::ErrorContext) {
    error!(target: "error_context", "
        Error Code: {}
        Message: {}
        Timestamp: {}
        Recoverable: {}
        Should Retry: {}
        Context: {:?}",
        context.error_code,
        context.message,
        context.timestamp,
        context.recoverable,
        context.should_retry,
        context.context
    );
}

// Log informational messages
pub fn log_info(message: &str) {
    info!(message);
}

// Log warnings
pub fn log_warning(message: &str) {
    warn!(message);
}
