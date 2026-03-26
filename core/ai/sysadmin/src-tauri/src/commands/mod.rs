// Commands module - All Tauri command handlers
pub mod ai_extended;
pub mod hardware;
pub mod monitoring;
pub mod rgb;

// Re-export command functions for easy access
pub use ai_extended::*;
pub use hardware::*;
pub use monitoring::*;
pub use rgb::*;
