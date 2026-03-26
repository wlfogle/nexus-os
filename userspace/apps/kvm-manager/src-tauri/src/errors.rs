use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvmError {
    #[error("Libvirt connection error: {0}")]
    LibvirtConnection(#[from] virt::error::Error),
    
    #[error("Virtual machine not found: {0}")]
    VmNotFound(String),
    
    #[error("Storage pool not found: {0}")]
    StoragePoolNotFound(String),
    
    #[error("Network not found: {0}")]
    NetworkNotFound(String),
    
    #[error("Invalid VM configuration: {0}")]
    InvalidVmConfig(String),
    
    #[error("VM operation failed: {0}")]
    VmOperationFailed(String),
    
    #[error("Storage operation failed: {0}")]
    StorageOperationFailed(String),
    
    #[error("Network operation failed: {0}")]
    NetworkOperationFailed(String),
    
    #[error("Snapshot operation failed: {0}")]
    SnapshotOperationFailed(String),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("XML parsing error: {0}")]
    XmlParsingError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<KvmError> for String {
    fn from(error: KvmError) -> Self {
        error.to_string()
    }
}

pub type Result<T> = std::result::Result<T, KvmError>;
