//! Input simulation module

use crate::error::{PdbError, Result};
use crate::types::KeyCode;
use std::thread;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT,
    VIRTUAL_KEY,
};

/// Send mouse click at screen coordinates
pub fn mouse_click(x: i32, y: i32) -> Result<()> {
    let (abs_x, abs_y) = screen_to_absolute(x, y);

    let inputs = [
        // Move to position
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: abs_x,
                    dy: abs_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        // Mouse down
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: abs_x,
                    dy: abs_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        // Mouse up
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: abs_x,
                    dy: abs_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_LEFTUP | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];

    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(PdbError::InputError(format!(
            "SendInput failed, sent {} of {} inputs",
            sent,
            inputs.len()
        )));
    }

    Ok(())
}

/// Send mouse swipe from (x1, y1) to (x2, y2) over duration_ms milliseconds
pub fn mouse_swipe(x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<()> {
    // Use more steps for smoother movement
    let steps = 50u32.max(duration_ms / 10);
    let step_delay = Duration::from_millis((duration_ms / steps).max(5) as u64);

    let (abs_x1, abs_y1) = screen_to_absolute(x1, y1);

    // Move to start position first
    let move_to_start = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: abs_x1,
                dy: abs_y1,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];
    unsafe { SendInput(&move_to_start, std::mem::size_of::<INPUT>() as i32) };
    thread::sleep(Duration::from_millis(30));

    // Press mouse button
    let mouse_down = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: abs_x1,
                dy: abs_y1,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];
    unsafe { SendInput(&mouse_down, std::mem::size_of::<INPUT>() as i32) };
    
    // Wait a bit after pressing (important for games to register the press)
    thread::sleep(Duration::from_millis(50));

    // Move in steps
    for i in 1..=steps {
        let progress = i as f64 / steps as f64;
        // Use easing for more natural movement
        let eased_progress = ease_out_quad(progress);
        let current_x = x1 + ((x2 - x1) as f64 * eased_progress) as i32;
        let current_y = y1 + ((y2 - y1) as f64 * eased_progress) as i32;
        let (abs_x, abs_y) = screen_to_absolute(current_x, current_y);

        let move_input = [INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: abs_x,
                    dy: abs_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }];

        unsafe { SendInput(&move_input, std::mem::size_of::<INPUT>() as i32) };
        thread::sleep(step_delay);
    }

    // Small delay before releasing
    thread::sleep(Duration::from_millis(30));

    // Release mouse
    let (abs_x2, abs_y2) = screen_to_absolute(x2, y2);
    let end_input = [INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: abs_x2,
                dy: abs_y2,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_LEFTUP | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];

    unsafe { SendInput(&end_input, std::mem::size_of::<INPUT>() as i32) };

    Ok(())
}

/// Quadratic ease-out function for smoother movement
fn ease_out_quad(t: f64) -> f64 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Send key event
pub fn key_event(key: KeyCode) -> Result<()> {
    let inputs = [
        // Key down
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(key.vk_code()),
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        // Key up
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(key.vk_code()),
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];

    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(PdbError::InputError("SendInput failed for key event".into()));
    }

    Ok(())
}

/// Send text input using unicode
pub fn input_text(text: &str) -> Result<()> {
    for ch in text.chars() {
        let inputs = [
            // Key down
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // Key up
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

/// Convert screen coordinates to absolute coordinates for SendInput
fn screen_to_absolute(x: i32, y: i32) -> (i32, i32) {
    // Get screen dimensions
    let screen_width = unsafe { windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
        windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN
    ) };
    let screen_height = unsafe { windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
        windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN
    ) };

    // Convert to absolute coordinates (0-65535 range)
    let abs_x = (x * 65535) / screen_width;
    let abs_y = (y * 65535) / screen_height;

    (abs_x, abs_y)
}

/// Get current cursor position (screen coordinates)
pub fn get_cursor_pos() -> Result<(i32, i32)> {
    unsafe {
        let mut point = windows::Win32::Foundation::POINT::default();
        if windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point).is_ok() {
            Ok((point.x, point.y))
        } else {
            Err(PdbError::InputError("Failed to get cursor position".into()))
        }
    }
}

