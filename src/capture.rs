//! Screenshot capture module using Windows Graphics Capture API

use crate::error::{PdbError, Result};
use crate::types::Screenshot;
use win_screenshot::capture::capture_window as wgc_capture;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
    GetDIBits, GetWindowDC, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, 
    BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetWindowRect, IsIconic, ShowWindow,
    SW_SHOWNOACTIVATE, SW_MINIMIZE,
};

/// Capture screenshot of entire screen using GDI
pub fn capture_screen() -> Result<Screenshot> {
    unsafe {
        let hwnd = HWND(std::ptr::null_mut());
        let hdc_screen = GetDC(hwnd);
        if hdc_screen.is_invalid() {
            return Err(PdbError::CaptureError("Failed to get screen DC".into()));
        }

        let width = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
        );
        let height = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
        );

        let result = capture_from_dc(hdc_screen, 0, 0, width, height);
        let _ = ReleaseDC(hwnd, hdc_screen);
        result
    }
}

/// Capture screenshot of a specific window using Windows Graphics Capture API
/// This works even if the window is occluded or uses hardware acceleration
/// If the window is minimized, it will be temporarily restored (without activation)
pub fn capture_window(hwnd: HWND) -> Result<Screenshot> {
    unsafe {
        // Check if window is minimized
        let was_minimized = IsIconic(hwnd).as_bool();
        
        if was_minimized {
            // Restore window without activating it
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            // Give window time to render
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        let hwnd_isize = hwnd.0 as isize;
        
        // Try Windows Graphics Capture first
        let result = match capture_window_wgc(hwnd_isize) {
            Ok(screenshot) => Ok(screenshot),
            Err(_) => {
                // Fall back to GDI
                capture_window_gdi(hwnd)
            }
        };
        
        // Re-minimize if it was minimized before
        if was_minimized {
            let _ = ShowWindow(hwnd, SW_MINIMIZE);
        }
        
        result
    }
}

/// Capture using Windows Graphics Capture API via win-screenshot crate
fn capture_window_wgc(hwnd: isize) -> Result<Screenshot> {
    // Use capture_window from win-screenshot crate
    let buf = wgc_capture(hwnd)
        .map_err(|e| PdbError::CaptureError(format!("WGC capture failed: {:?}", e)))?;

    let width = buf.width;
    let height = buf.height;
    let data = buf.pixels;

    // win-screenshot returns BGRA format, image crate expects RGBA
    // But when saving with image crate from RgbaImage, it handles this correctly
    // The data from win-screenshot is already in the right byte order

    Ok(Screenshot {
        width,
        height,
        data,
    })
}

/// Fallback: Capture window using GDI
fn capture_window_gdi(hwnd: HWND) -> Result<Screenshot> {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return Err(PdbError::CaptureError("Failed to get window rect".into()));
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        let hdc_window = GetWindowDC(hwnd);
        if hdc_window.is_invalid() {
            return Err(PdbError::CaptureError("Failed to get window DC".into()));
        }

        let result = capture_from_dc(hdc_window, 0, 0, width, height);
        let _ = ReleaseDC(hwnd, hdc_window);
        result
    }
}

/// Capture screenshot of window client area
pub fn capture_window_client(hwnd: HWND) -> Result<Screenshot> {
    // WGC captures the whole window, so we use GDI for client area
    capture_window_client_gdi(hwnd)
}

/// GDI for client area
fn capture_window_client_gdi(hwnd: HWND) -> Result<Screenshot> {
    unsafe {
        let mut rect = RECT::default();
        if GetClientRect(hwnd, &mut rect).is_err() {
            return Err(PdbError::CaptureError("Failed to get client rect".into()));
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        let hdc_window = GetDC(hwnd);
        if hdc_window.is_invalid() {
            return Err(PdbError::CaptureError("Failed to get window DC".into()));
        }

        let result = capture_from_dc(hdc_window, 0, 0, width, height);
        let _ = ReleaseDC(hwnd, hdc_window);
        result
    }
}

/// Capture from a device context using BitBlt (GDI)
unsafe fn capture_from_dc(
    hdc_src: windows::Win32::Graphics::Gdi::HDC,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<Screenshot> {
    let hdc_mem = CreateCompatibleDC(hdc_src);
    if hdc_mem.is_invalid() {
        return Err(PdbError::CaptureError("Failed to create compatible DC".into()));
    }

    let hbitmap = CreateCompatibleBitmap(hdc_src, width, height);
    if hbitmap.is_invalid() {
        let _ = DeleteDC(hdc_mem);
        return Err(PdbError::CaptureError("Failed to create bitmap".into()));
    }

    let old_bitmap = SelectObject(hdc_mem, hbitmap);

    if BitBlt(hdc_mem, 0, 0, width, height, hdc_src, x, y, SRCCOPY).is_err() {
        SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(hdc_mem);
        return Err(PdbError::CaptureError("BitBlt failed".into()));
    }

    let mut bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default()],
    };

    let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

    let result = GetDIBits(
        hdc_mem,
        hbitmap,
        0,
        height as u32,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut bi,
        DIB_RGB_COLORS,
    );

    SelectObject(hdc_mem, old_bitmap);
    let _ = DeleteObject(hbitmap);
    let _ = DeleteDC(hdc_mem);

    if result == 0 {
        return Err(PdbError::CaptureError("GetDIBits failed".into()));
    }

    // GDI returns BGRA, convert to RGBA to match win-screenshot format
    for chunk in buffer.chunks_exact_mut(4) {
        chunk.swap(0, 2); // Swap B and R
    }

    Ok(Screenshot {
        width: width as u32,
        height: height as u32,
        data: buffer,
    })
}
