//! Network protocol for remote operations

use crate::types::{KeyCode, Screenshot, WindowInfo};
use serde::{Deserialize, Serialize};

/// Command sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// List all windows
    ListWindows,
    
    /// Connect to window by title
    Connect { title: String },
    
    /// Connect to window by hwnd
    ConnectByHwnd { hwnd: usize },
    
    /// Click at position
    Click { hwnd: usize, x: i32, y: i32 },
    
    /// Swipe from one position to another
    Swipe {
        hwnd: usize,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        duration_ms: u32,
    },
    
    /// Take screenshot
    Screenshot { hwnd: usize },
    
    /// Input text
    InputText { hwnd: usize, text: String },
    
    /// Send key event
    KeyEvent { hwnd: usize, key: KeyCode },
    
    /// Get window size
    GetSize { hwnd: usize },
    
    /// Focus window
    Focus { hwnd: usize },
    
    /// Ping to check connection
    Ping,
    
    /// Disconnect
    Disconnect,
}

/// Response sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Success with no data
    Ok,
    
    /// Window list
    Windows(Vec<WindowInfo>),
    
    /// Single window info
    Window(WindowInfo),
    
    /// Screenshot data
    Screenshot(Screenshot),
    
    /// Window size
    Size { width: i32, height: i32 },
    
    /// Error message
    Error(String),
    
    /// Pong response
    Pong,
}

/// Default server port
pub const DEFAULT_PORT: u16 = 5037; // Same as ADB

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Message header for framing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub version: u32,
    pub length: u32,
}

impl MessageHeader {
    pub fn new(length: u32) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            length,
        }
    }
}
