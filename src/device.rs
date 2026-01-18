//! Device abstraction - represents a connected window (similar to ADB device)

use crate::capture;
use crate::error::Result;
use crate::input;
use crate::types::{KeyCode, Rect, Screenshot, WindowInfo};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetWindowRect, SetForegroundWindow, IsIconic, ShowWindow,
    SW_SHOWNOACTIVATE, SW_MINIMIZE,
};

/// Device represents a connected window, similar to an ADB device
#[derive(Debug, Clone)]
pub struct Device {
    /// Window handle
    hwnd: HWND,
    /// Window info
    info: WindowInfo,
}

impl Device {
    /// Create a new device from window info
    pub fn new(info: WindowInfo) -> Self {
        Self {
            hwnd: HWND(info.hwnd as *mut _),
            info,
        }
    }

    /// Get window info
    pub fn info(&self) -> &WindowInfo {
        &self.info
    }

    /// Get window handle
    pub fn hwnd(&self) -> usize {
        self.info.hwnd
    }

    /// Check if window is minimized
    pub fn is_minimized(&self) -> bool {
        unsafe { IsIconic(self.hwnd).as_bool() }
    }

    /// Restore window if minimized (without activating)
    /// Returns true if window was minimized
    fn ensure_visible(&self) -> bool {
        unsafe {
            let was_minimized = IsIconic(self.hwnd).as_bool();
            if was_minimized {
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            was_minimized
        }
    }

    /// Re-minimize window if it was minimized before
    fn restore_minimized(&self, was_minimized: bool) {
        if was_minimized {
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_MINIMIZE);
            }
        }
    }

    /// Bring window to foreground
    pub fn focus(&self) -> Result<()> {
        unsafe {
            let _ = SetForegroundWindow(self.hwnd);
        }
        Ok(())
    }

    /// Get window size (client area)
    pub fn get_size(&self) -> Result<(i32, i32)> {
        unsafe {
            let mut rect = windows::Win32::Foundation::RECT::default();
            let _ = GetClientRect(self.hwnd, &mut rect);
            Ok((rect.right - rect.left, rect.bottom - rect.top))
        }
    }

    /// Get window rectangle
    pub fn get_rect(&self) -> Result<Rect> {
        unsafe {
            let mut rect = windows::Win32::Foundation::RECT::default();
            let _ = GetWindowRect(self.hwnd, &mut rect);
            Ok(Rect::new(rect.left, rect.top, rect.right, rect.bottom))
        }
    }

    /// Click at position (relative to window client area)
    /// If window is minimized, it will be temporarily restored
    pub fn click(&self, x: i32, y: i32) -> Result<()> {
        let was_minimized = self.ensure_visible();
        self.focus()?;
        let (screen_x, screen_y) = self.client_to_screen(x, y)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        let result = input::mouse_click(screen_x, screen_y);
        self.restore_minimized(was_minimized);
        result
    }

    /// Tap at position (alias for click)
    pub fn tap(&self, x: i32, y: i32) -> Result<()> {
        self.click(x, y)
    }

    /// Swipe from (x1, y1) to (x2, y2) over duration_ms milliseconds
    /// If window is minimized, it will be temporarily restored
    pub fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<()> {
        let was_minimized = self.ensure_visible();
        self.focus()?;
        let (screen_x1, screen_y1) = self.client_to_screen(x1, y1)?;
        let (screen_x2, screen_y2) = self.client_to_screen(x2, y2)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        let result = input::mouse_swipe(screen_x1, screen_y1, screen_x2, screen_y2, duration_ms);
        self.restore_minimized(was_minimized);
        result
    }

    /// Take screenshot of window
    pub fn screenshot(&self) -> Result<Screenshot> {
        capture::capture_window(self.hwnd)
    }

    /// Take screenshot of window client area
    pub fn screenshot_client(&self) -> Result<Screenshot> {
        capture::capture_window_client(self.hwnd)
    }

    /// Input text
    /// If window is minimized, it will be temporarily restored
    pub fn input_text(&self, text: &str) -> Result<()> {
        let was_minimized = self.ensure_visible();
        self.focus()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        let result = input::input_text(text);
        self.restore_minimized(was_minimized);
        result
    }

    /// Send key event
    /// If window is minimized, it will be temporarily restored
    pub fn key_event(&self, key: KeyCode) -> Result<()> {
        let was_minimized = self.ensure_visible();
        self.focus()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        let result = input::key_event(key);
        self.restore_minimized(was_minimized);
        result
    }

    /// Press Enter key
    pub fn press_enter(&self) -> Result<()> {
        self.key_event(KeyCode::Enter)
    }

    /// Press Backspace key
    pub fn press_backspace(&self) -> Result<()> {
        self.key_event(KeyCode::Backspace)
    }

    /// Press Escape key
    pub fn press_escape(&self) -> Result<()> {
        self.key_event(KeyCode::Escape)
    }

    /// Get current cursor position relative to window client area
    pub fn get_cursor_pos(&self) -> Result<(i32, i32)> {
        let (screen_x, screen_y) = input::get_cursor_pos()?;
        self.screen_to_client(screen_x, screen_y)
    }

    /// Convert client coordinates to screen coordinates
    fn client_to_screen(&self, x: i32, y: i32) -> Result<(i32, i32)> {
        unsafe {
            let mut point = windows::Win32::Foundation::POINT { x, y };
            let _ = windows::Win32::Graphics::Gdi::ClientToScreen(self.hwnd, &mut point);
            Ok((point.x, point.y))
        }
    }

    /// Convert screen coordinates to client coordinates
    fn screen_to_client(&self, x: i32, y: i32) -> Result<(i32, i32)> {
        unsafe {
            let mut point = windows::Win32::Foundation::POINT { x, y };
            let _ = windows::Win32::Graphics::Gdi::ScreenToClient(self.hwnd, &mut point);
            Ok((point.x, point.y))
        }
    }
}

// Make Device Send + Sync for async usage
unsafe impl Send for Device {}
unsafe impl Sync for Device {}
