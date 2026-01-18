//! Common types for PDB library

use serde::{Deserialize, Serialize};

/// Window information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    /// Window handle (as usize for serialization)
    pub hwnd: usize,
    /// Window title
    pub title: String,
    /// Window class name
    pub class_name: String,
    /// Window rectangle
    pub rect: Rect,
    /// Is window visible
    pub visible: bool,
}

/// Point structure
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Rectangle structure
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self { left, top, right, bottom }
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// Key codes similar to Android KeyEvent
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u16)]
pub enum KeyCode {
    // Numbers
    Num0 = 0x30,
    Num1 = 0x31,
    Num2 = 0x32,
    Num3 = 0x33,
    Num4 = 0x34,
    Num5 = 0x35,
    Num6 = 0x36,
    Num7 = 0x37,
    Num8 = 0x38,
    Num9 = 0x39,

    // Letters
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,

    // Function keys
    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,

    // Special keys
    Backspace = 0x08,
    Tab = 0x09,
    Enter = 0x0D,
    Shift = 0x10,
    Ctrl = 0x11,
    Alt = 0x12,
    Pause = 0x13,
    CapsLock = 0x14,
    Escape = 0x1B,
    Space = 0x20,
    PageUp = 0x21,
    PageDown = 0x22,
    End = 0x23,
    Home = 0x24,
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,
    Insert = 0x2D,
    Delete = 0x2E,

    // Windows key
    LWin = 0x5B,
    RWin = 0x5C,
}

impl KeyCode {
    /// Get virtual key code
    pub fn vk_code(&self) -> u16 {
        *self as u16
    }
}

/// Screenshot data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Raw RGBA pixel data (base64 encoded for network transfer)
    pub data: Vec<u8>,
}

impl Screenshot {
    /// Save screenshot to file
    pub fn save(&self, path: &str) -> crate::error::Result<()> {
        // win-screenshot returns data in correct format for image crate
        let img = image::RgbaImage::from_raw(self.width, self.height, self.data.clone())
            .ok_or_else(|| crate::error::PdbError::CaptureError("Failed to create image".into()))?;
        img.save(path)?;
        Ok(())
    }
    
    /// Get raw pixel data
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }
    
    /// Get pixel data as RGBA
    pub fn rgba_data(&self) -> &[u8] {
        &self.data
    }
}
