//! Demo example showing local usage of PDB

use pdb::{Device, WindowController};

/// Truncate a string to a maximum number of characters (Unicode-safe)
fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max_chars {
        let truncated: String = s.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

fn main() -> pdb::Result<()> {
    println!("=== PDB Demo - Local Usage ===\n");

    // Create controller
    let controller = WindowController::new();

    // List all windows
    println!("Listing all visible windows:\n");
    let windows = controller.list_windows()?;
    
    println!("{:<20} {:<50} {}", "HWND", "Title", "Class");
    println!("{}", "-".repeat(90));
    
    for window in &windows {
        if !window.title.is_empty() {
            let title = truncate_str(&window.title, 45);
            println!("{:<20} {:<50} {}", 
                format!("0x{:X}", window.hwnd),
                title,
                window.class_name
            );
        }
    }

    println!("\nTotal: {} windows\n", windows.len());

    // Try to find Notepad (if running)
    match controller.find_window("Notepad") {
        Ok(info) => {
            println!("Found Notepad: {} (HWND: 0x{:X})", info.title, info.hwnd);
            
            let device = Device::new(info);
            
            // Get window size
            let (width, height) = device.get_size()?;
            println!("Window size: {}x{}", width, height);
            
            // Uncomment to test operations:
            // device.click(100, 100)?;
            // device.input_text("Hello from PDB!")?;
            // device.key_event(KeyCode::Enter)?;
            // 
            // let screenshot = device.screenshot()?;
            // screenshot.save("notepad_screenshot.png")?;
            // println!("Screenshot saved to notepad_screenshot.png");
        }
        Err(_) => {
            println!("Notepad not found. Open Notepad to test device operations.");
        }
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
