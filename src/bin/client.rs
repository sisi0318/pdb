//! PDB Client binary - command line tool for local and remote operations

use pdb::{Client, Device, KeyCode, WindowController};
use std::env;

/// Check if running in local mode
fn is_local_mode(args: &[String]) -> bool {
    args.iter().any(|a| a == "--local" || a == "-l")
}

/// Filter out --local flag from args
fn filter_args(args: &[String]) -> Vec<String> {
    args.iter()
        .filter(|a| *a != "--local" && *a != "-l")
        .cloned()
        .collect()
}

#[tokio::main]
async fn main() -> pdb::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();
    let local_mode = is_local_mode(&args);
    let args = filter_args(&args);

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    if local_mode {
        run_local_command(command, &args).await
    } else {
        run_remote_command(command, &args).await
    }
}

/// Run command in local mode (no server required)
async fn run_local_command(command: &str, args: &[String]) -> pdb::Result<()> {
    let controller = WindowController::new();

    match command {
        "devices" | "list" => {
            let windows = controller.list_windows()?;
            
            println!("List of Windows (Local):");
            println!("{:<20} {:<60} {}", "HWND", "Title", "Class");
            println!("{}", "-".repeat(100));
            for window in windows {
                println!("{:<20} {:<60} {}", 
                    format!("0x{:X}", window.hwnd),
                    truncate_unicode(&window.title, 58),
                    window.class_name
                );
            }
        }
        
        "connect" => {
            if args.len() < 3 {
                println!("Usage: pdb-client --local connect <window_title>");
                return Ok(());
            }
            let title = &args[2];
            let info = controller.find_window(title)?;
            println!("Connected to: {} (HWND: 0x{:X})", info.title, info.hwnd);
        }
        
        "click" => {
            if args.len() < 5 {
                println!("Usage: pdb-client --local click <hwnd> <x> <y>");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let x: i32 = args[3].parse().expect("Invalid x coordinate");
            let y: i32 = args[4].parse().expect("Invalid y coordinate");
            
            let info = controller.get_window_by_hwnd(hwnd)?;
            let device = Device::new(info);
            device.click(x, y)?;
            println!("Clicked at ({}, {})", x, y);
        }
        
        "swipe" => {
            if args.len() < 7 {
                println!("Usage: pdb-client --local swipe <hwnd> <x1> <y1> <x2> <y2> [duration_ms]");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let x1: i32 = args[3].parse().expect("Invalid x1");
            let y1: i32 = args[4].parse().expect("Invalid y1");
            let x2: i32 = args[5].parse().expect("Invalid x2");
            let y2: i32 = args[6].parse().expect("Invalid y2");
            let duration_ms: u32 = args.get(7).and_then(|s| s.parse().ok()).unwrap_or(500);
            
            let info = controller.get_window_by_hwnd(hwnd)?;
            let device = Device::new(info);
            device.swipe(x1, y1, x2, y2, duration_ms)?;
            println!("Swiped from ({}, {}) to ({}, {})", x1, y1, x2, y2);
        }
        
        "text" => {
            if args.len() < 4 {
                println!("Usage: pdb-client --local text <hwnd> <text>");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let text = &args[3];
            
            let info = controller.get_window_by_hwnd(hwnd)?;
            let device = Device::new(info);
            device.input_text(text)?;
            println!("Input text: {}", text);
        }
        
        "key" => {
            if args.len() < 4 {
                println!("Usage: pdb-client --local key <hwnd> <keycode>");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let key = parse_keycode(&args[3])?;
            
            let info = controller.get_window_by_hwnd(hwnd)?;
            let device = Device::new(info);
            device.key_event(key)?;
            println!("Sent key event: {:?}", key);
        }
        
        "screenshot" => {
            if args.len() < 4 {
                println!("Usage: pdb-client --local screenshot <hwnd> <output_path>");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let output_path = &args[3];
            
            let info = controller.get_window_by_hwnd(hwnd)?;
            let device = Device::new(info);
            let screenshot = device.screenshot()?;
            screenshot.save(output_path)?;
            println!("Screenshot saved to: {}", output_path);
        }
        
        "coord" | "mouse" => {
            if args.len() < 3 {
                println!("Usage: pdb-client --local coord <hwnd>");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            
            let info = controller.get_window_by_hwnd(hwnd)?;
            let device = Device::new(info.clone());
            let (width, height) = device.get_size()?;
            
            println!("Tracking mouse position for window: {} (HWND: 0x{:X})", info.title, hwnd);
            println!("Window size: {}x{}", width, height);
            println!("Press Ctrl+C to stop\n");
            println!("{:>8}  {:>8}  {:>10}", "X", "Y", "Status");
            println!("{}", "-".repeat(45));
            
            let mut last_inside = false;
            loop {
                match device.get_cursor_pos() {
                    Ok((x, y)) => {
                        let inside = x >= 0 && y >= 0 && x < width && y < height;
                        if inside {
                            print!("\r{:>8}  {:>8}  {:>10}", x, y, "IN WINDOW");
                            last_inside = true;
                        } else if last_inside {
                            print!("\r{:>8}  {:>8}  {:>10}", "-", "-", "OUTSIDE  ");
                            last_inside = false;
                        }
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                    }
                    Err(_) => {}
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        
        _ => {
            print_usage();
        }
    }

    Ok(())
}

/// Run command in remote mode (requires server)
async fn run_remote_command(command: &str, args: &[String]) -> pdb::Result<()> {
    match command {
        "devices" | "list" => {
            let addr = get_addr(args, 2);
            let client = Client::connect(&addr).await?;
            let windows = client.list_windows().await?;
            
            println!("List of Windows (Remote: {}):", addr);
            println!("{:<20} {:<60} {}", "HWND", "Title", "Class");
            println!("{}", "-".repeat(100));
            for window in windows {
                println!("{:<20} {:<60} {}", 
                    format!("0x{:X}", window.hwnd),
                    truncate_unicode(&window.title, 58),
                    window.class_name
                );
            }
        }
        
        "connect" => {
            if args.len() < 3 {
                println!("Usage: pdb-client connect <window_title> [server_addr]");
                return Ok(());
            }
            let title = &args[2];
            let addr = get_addr(args, 3);
            let client = Client::connect(&addr).await?;
            let device = client.connect_window(title).await?;
            println!("Connected to: {} (HWND: 0x{:X})", device.info().title, device.hwnd());
        }
        
        "click" => {
            if args.len() < 5 {
                println!("Usage: pdb-client click <hwnd> <x> <y> [server_addr]");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let x: i32 = args[3].parse().expect("Invalid x coordinate");
            let y: i32 = args[4].parse().expect("Invalid y coordinate");
            let addr = get_addr(args, 5);
            
            let client = Client::connect(&addr).await?;
            let device = client.connect_window_by_hwnd(hwnd).await?;
            device.click(x, y).await?;
            println!("Clicked at ({}, {})", x, y);
        }
        
        "swipe" => {
            if args.len() < 7 {
                println!("Usage: pdb-client swipe <hwnd> <x1> <y1> <x2> <y2> [duration_ms] [server_addr]");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let x1: i32 = args[3].parse().expect("Invalid x1");
            let y1: i32 = args[4].parse().expect("Invalid y1");
            let x2: i32 = args[5].parse().expect("Invalid x2");
            let y2: i32 = args[6].parse().expect("Invalid y2");
            let duration_ms: u32 = args.get(7).and_then(|s| s.parse().ok()).unwrap_or(500);
            let addr = get_addr(args, 8);
            
            let client = Client::connect(&addr).await?;
            let device = client.connect_window_by_hwnd(hwnd).await?;
            device.swipe(x1, y1, x2, y2, duration_ms).await?;
            println!("Swiped from ({}, {}) to ({}, {})", x1, y1, x2, y2);
        }
        
        "text" => {
            if args.len() < 4 {
                println!("Usage: pdb-client text <hwnd> <text> [server_addr]");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let text = &args[3];
            let addr = get_addr(args, 4);
            
            let client = Client::connect(&addr).await?;
            let device = client.connect_window_by_hwnd(hwnd).await?;
            device.input_text(text).await?;
            println!("Input text: {}", text);
        }
        
        "key" => {
            if args.len() < 4 {
                println!("Usage: pdb-client key <hwnd> <keycode> [server_addr]");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let key = parse_keycode(&args[3])?;
            let addr = get_addr(args, 4);
            
            let client = Client::connect(&addr).await?;
            let device = client.connect_window_by_hwnd(hwnd).await?;
            device.key_event(key).await?;
            println!("Sent key event: {:?}", key);
        }
        
        "screenshot" => {
            if args.len() < 4 {
                println!("Usage: pdb-client screenshot <hwnd> <output_path> [server_addr]");
                return Ok(());
            }
            let hwnd = parse_hwnd(&args[2])?;
            let output_path = &args[3];
            let addr = get_addr(args, 4);
            
            let client = Client::connect(&addr).await?;
            let device = client.connect_window_by_hwnd(hwnd).await?;
            let screenshot = device.screenshot().await?;
            screenshot.save(output_path)?;
            println!("Screenshot saved to: {}", output_path);
        }
        
        "ping" => {
            let addr = get_addr(args, 2);
            let client = Client::connect(&addr).await?;
            if client.ping().await? {
                println!("Server is alive");
            } else {
                println!("No response from server");
            }
        }
        
        _ => {
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("PDB Client - PC Window Controller");
    println!();
    println!("Usage: pdb-client [--local|-l] <command> [args...]");
    println!();
    println!("Modes:");
    println!("  --local, -l                             Run in local mode (no server required)");
    println!("  (default)                               Connect to remote server");
    println!();
    println!("Commands:");
    println!("  devices|list [server_addr]              List all windows");
    println!("  connect <title> [server_addr]           Connect to a window by title");
    println!("  click <hwnd> <x> <y> [server_addr]      Click at position");
    println!("  swipe <hwnd> <x1> <y1> <x2> <y2> [duration_ms] [server_addr]");
    println!("                                          Swipe from one position to another");
    println!("  text <hwnd> <text> [server_addr]        Input text");
    println!("  key <hwnd> <keycode> [server_addr]      Send key event");
    println!("  screenshot <hwnd> <path> [server_addr]  Take screenshot");
    println!("  coord|mouse <hwnd>                      Track mouse position (local only)");
    println!("  ping [server_addr]                      Ping server (remote only)");
    println!();
    println!("Examples:");
    println!("  pdb-client --local devices              List windows locally");
    println!("  pdb-client --local click 0x12345 100 200");
    println!("  pdb-client --local coord 0x12345        Track mouse in window");
    println!("  pdb-client devices                      List windows via server");
    println!("  pdb-client devices 192.168.1.100:5037   List windows on remote machine");
    println!();
    println!("Default server address: 127.0.0.1:5037");
    println!();
    println!("HWND can be specified as decimal or hex (0x prefix)");
    println!();
    println!("Keycodes: enter, backspace, escape, tab, space, up, down, left, right,");
    println!("          a-z, 0-9, f1-f12");
}

fn get_addr(args: &[String], index: usize) -> String {
    args.get(index)
        .cloned()
        .unwrap_or_else(|| format!("127.0.0.1:{}", pdb::DEFAULT_PORT))
}

fn parse_hwnd(s: &str) -> pdb::Result<usize> {
    if s.starts_with("0x") || s.starts_with("0X") {
        usize::from_str_radix(&s[2..], 16)
            .map_err(|_| pdb::PdbError::HandleError("Invalid HWND".into()))
    } else {
        s.parse()
            .map_err(|_| pdb::PdbError::HandleError("Invalid HWND".into()))
    }
}

fn parse_keycode(s: &str) -> pdb::Result<KeyCode> {
    let key = match s.to_lowercase().as_str() {
        "enter" | "return" => KeyCode::Enter,
        "backspace" | "back" => KeyCode::Backspace,
        "escape" | "esc" => KeyCode::Escape,
        "tab" => KeyCode::Tab,
        "space" => KeyCode::Space,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "insert" => KeyCode::Insert,
        "delete" => KeyCode::Delete,
        "f1" => KeyCode::F1,
        "f2" => KeyCode::F2,
        "f3" => KeyCode::F3,
        "f4" => KeyCode::F4,
        "f5" => KeyCode::F5,
        "f6" => KeyCode::F6,
        "f7" => KeyCode::F7,
        "f8" => KeyCode::F8,
        "f9" => KeyCode::F9,
        "f10" => KeyCode::F10,
        "f11" => KeyCode::F11,
        "f12" => KeyCode::F12,
        "a" => KeyCode::A,
        "b" => KeyCode::B,
        "c" => KeyCode::C,
        "d" => KeyCode::D,
        "e" => KeyCode::E,
        "f" => KeyCode::F,
        "g" => KeyCode::G,
        "h" => KeyCode::H,
        "i" => KeyCode::I,
        "j" => KeyCode::J,
        "k" => KeyCode::K,
        "l" => KeyCode::L,
        "m" => KeyCode::M,
        "n" => KeyCode::N,
        "o" => KeyCode::O,
        "p" => KeyCode::P,
        "q" => KeyCode::Q,
        "r" => KeyCode::R,
        "s" => KeyCode::S,
        "t" => KeyCode::T,
        "u" => KeyCode::U,
        "v" => KeyCode::V,
        "w" => KeyCode::W,
        "x" => KeyCode::X,
        "y" => KeyCode::Y,
        "z" => KeyCode::Z,
        "0" => KeyCode::Num0,
        "1" => KeyCode::Num1,
        "2" => KeyCode::Num2,
        "3" => KeyCode::Num3,
        "4" => KeyCode::Num4,
        "5" => KeyCode::Num5,
        "6" => KeyCode::Num6,
        "7" => KeyCode::Num7,
        "8" => KeyCode::Num8,
        "9" => KeyCode::Num9,
        _ => return Err(pdb::PdbError::InputError(format!("Unknown keycode: {}", s))),
    };
    Ok(key)
}

/// Truncate string safely for Unicode characters
fn truncate_unicode(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max_chars {
        let truncated: String = s.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}
