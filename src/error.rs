//! Error types for the PDB library

use thiserror::Error;

/// Result type alias for PDB operations
pub type Result<T> = std::result::Result<T, PdbError>;

/// Error types for PDB operations
#[derive(Error, Debug)]
pub enum PdbError {
    /// Window not found
    #[error("Window not found: {0}")]
    WindowNotFound(String),

    /// Failed to get window handle
    #[error("Failed to get window handle: {0}")]
    HandleError(String),

    /// Input simulation failed
    #[error("Input simulation failed: {0}")]
    InputError(String),

    /// Screenshot capture failed
    #[error("Screenshot capture failed: {0}")]
    CaptureError(String),

    /// Windows API error
    #[error("Windows API error: {0}")]
    WindowsError(#[from] windows::core::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Network connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Image error
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}
