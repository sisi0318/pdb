//! Window controller module

use crate::error::{PdbError, Result};
use crate::types::{Rect, WindowInfo};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    IsWindowVisible,
};

/// Window controller - entry point similar to ADB
pub struct WindowController;

impl WindowController {
    /// Create a new window controller
    pub fn new() -> Self {
        Self
    }

    /// List all visible windows
    pub fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        let mut windows: Vec<WindowInfo> = Vec::new();

        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
            );
        }

        Ok(windows)
    }

    /// Find window by title (partial match)
    pub fn find_window(&self, title: &str) -> Result<WindowInfo> {
        let windows = self.list_windows()?;
        windows
            .into_iter()
            .find(|w| w.title.contains(title))
            .ok_or_else(|| PdbError::WindowNotFound(title.to_string()))
    }

    /// Find window by exact title
    pub fn find_window_exact(&self, title: &str) -> Result<WindowInfo> {
        let windows = self.list_windows()?;
        windows
            .into_iter()
            .find(|w| w.title == title)
            .ok_or_else(|| PdbError::WindowNotFound(title.to_string()))
    }

    /// Find window by class name
    pub fn find_window_by_class(&self, class_name: &str) -> Result<WindowInfo> {
        let windows = self.list_windows()?;
        windows
            .into_iter()
            .find(|w| w.class_name.contains(class_name))
            .ok_or_else(|| PdbError::WindowNotFound(class_name.to_string()))
    }

    /// Get window info by handle
    pub fn get_window_by_hwnd(&self, hwnd: usize) -> Result<WindowInfo> {
        get_window_info(HWND(hwnd as *mut _))
    }
}

impl Default for WindowController {
    fn default() -> Self {
        Self::new()
    }
}

/// Callback for EnumWindows
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    // Only include visible windows
    if IsWindowVisible(hwnd).as_bool() {
        if let Ok(info) = get_window_info(hwnd) {
            // Filter out empty titles
            if !info.title.is_empty() {
                windows.push(info);
            }
        }
    }

    BOOL(1) // Continue enumeration
}

/// Get window information
fn get_window_info(hwnd: HWND) -> Result<WindowInfo> {
    unsafe {
        // Get window title
        let title_len = GetWindowTextLengthW(hwnd);
        let title = if title_len > 0 {
            let mut buffer: Vec<u16> = vec![0; (title_len + 1) as usize];
            GetWindowTextW(hwnd, &mut buffer);
            OsString::from_wide(&buffer[..title_len as usize])
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        };

        // Get class name
        let mut class_buffer: Vec<u16> = vec![0; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buffer);
        let class_name = if class_len > 0 {
            OsString::from_wide(&class_buffer[..class_len as usize])
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        };

        // Get window rect
        let mut rect = windows::Win32::Foundation::RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        Ok(WindowInfo {
            hwnd: hwnd.0 as usize,
            title,
            class_name,
            rect: Rect::new(rect.left, rect.top, rect.right, rect.bottom),
            visible: IsWindowVisible(hwnd).as_bool(),
        })
    }
}
