use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

use crate::types::{Config, Report, ScanState, ScanStatus};

// ─── Scan Entry ───────────────────────────────────────────────────────────────

/// Live record for a single scan run
pub struct ScanEntry {
    pub status: ScanStatus,
    /// Tokio task handle — aborted on cancel_scan
    pub handle: Option<JoinHandle<()>>,
    /// Populated when state == Complete
    pub report: Option<Report>,
}

// ─── Scan Registry ────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct ScanRegistry {
    pub scans: HashMap<String, ScanEntry>,
}

pub type SharedScanRegistry = Arc<Mutex<ScanRegistry>>;

pub fn new_registry() -> SharedScanRegistry {
    Arc::new(Mutex::new(ScanRegistry::default()))
}

// ─── Config State ─────────────────────────────────────────────────────────────

pub type SharedConfig = Arc<Mutex<Config>>;

pub fn new_config() -> SharedConfig {
    Arc::new(Mutex::new(Config::default()))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

impl ScanRegistry {
    /// Register a new scan as Running.
    pub fn register(&mut self, scan_id: String, handle: Option<JoinHandle<()>>) {
        let status = ScanStatus {
            scan_id: scan_id.clone(),
            state: ScanState::Running,
            total: 0,
            processed: 0,
            current_file: None,
            stage: "initializing".to_string(),
            error: None,
            results_so_far: vec![],
        };
        self.scans.insert(
            scan_id,
            ScanEntry {
                status,
                handle,
                report: None,
            },
        );
    }
}
