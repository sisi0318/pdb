//! Server implementation for remote connections

use crate::controller::WindowController;
use crate::device::Device;
use crate::error::Result;
use crate::protocol::{Command, MessageHeader, Response, DEFAULT_PORT};
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// PDB Server - listens for remote connections (like ADB daemon)
pub struct Server {
    /// Server address
    addr: String,
    /// Connected devices (hwnd -> Device)
    devices: Arc<Mutex<HashMap<usize, Device>>>,
}

impl Server {
    /// Create a new server
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
            devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create server with default port
    pub fn default_addr() -> Self {
        Self::new(&format!("0.0.0.0:{}", DEFAULT_PORT))
    }

    /// Start the server
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        info!("PDB Server listening on {}", self.addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
                    let devices = self.devices.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, devices).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }
}

/// Handle a single client connection
async fn handle_connection(
    mut stream: TcpStream,
    devices: Arc<Mutex<HashMap<usize, Device>>>,
) -> Result<()> {
    let controller = WindowController::new();

    loop {
        // Read message header (8 bytes: version u32 + length u32)
        let mut header_buf = [0u8; 8];
        match stream.read_exact(&mut header_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                info!("Client disconnected");
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        }

        let header: MessageHeader = {
            let version = u32::from_le_bytes(header_buf[0..4].try_into().unwrap());
            let length = u32::from_le_bytes(header_buf[4..8].try_into().unwrap());
            MessageHeader { version, length }
        };

        // Read message body
        let mut body_buf = vec![0u8; header.length as usize];
        stream.read_exact(&mut body_buf).await?;

        // Parse command
        let command: Command = serde_json::from_slice(&body_buf)?;
        
        // Handle command
        let response = handle_command(command, &controller, &devices).await;

        // Send response
        let response_json = serde_json::to_vec(&response)?;
        let resp_header = MessageHeader::new(response_json.len() as u32);
        
        let mut resp_header_buf = [0u8; 8];
        resp_header_buf[0..4].copy_from_slice(&resp_header.version.to_le_bytes());
        resp_header_buf[4..8].copy_from_slice(&resp_header.length.to_le_bytes());
        
        stream.write_all(&resp_header_buf).await?;
        stream.write_all(&response_json).await?;
        stream.flush().await?;
    }
}

/// Handle a command and return response
async fn handle_command(
    command: Command,
    controller: &WindowController,
    devices: &Arc<Mutex<HashMap<usize, Device>>>,
) -> Response {
    match command {
        Command::Ping => Response::Pong,
        
        Command::Disconnect => Response::Ok,
        
        Command::ListWindows => {
            match controller.list_windows() {
                Ok(windows) => Response::Windows(windows),
                Err(e) => Response::Error(e.to_string()),
            }
        }
        
        Command::Connect { title } => {
            match controller.find_window(&title) {
                Ok(info) => {
                    let device = Device::new(info.clone());
                    devices.lock().await.insert(info.hwnd, device);
                    Response::Window(info)
                }
                Err(e) => Response::Error(e.to_string()),
            }
        }
        
        Command::ConnectByHwnd { hwnd } => {
            match controller.get_window_by_hwnd(hwnd) {
                Ok(info) => {
                    let device = Device::new(info.clone());
                    devices.lock().await.insert(info.hwnd, device);
                    Response::Window(info)
                }
                Err(e) => Response::Error(e.to_string()),
            }
        }
        
        Command::Click { hwnd, x, y } => {
            let devices = devices.lock().await;
            if let Some(device) = devices.get(&hwnd) {
                match device.click(x, y) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e.to_string()),
                }
            } else {
                Response::Error("Device not connected".to_string())
            }
        }
        
        Command::Swipe { hwnd, x1, y1, x2, y2, duration_ms } => {
            let devices = devices.lock().await;
            if let Some(device) = devices.get(&hwnd) {
                match device.swipe(x1, y1, x2, y2, duration_ms) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e.to_string()),
                }
            } else {
                Response::Error("Device not connected".to_string())
            }
        }
        
        Command::Screenshot { hwnd } => {
            let devices = devices.lock().await;
            if let Some(device) = devices.get(&hwnd) {
                match device.screenshot() {
                    Ok(screenshot) => Response::Screenshot(screenshot),
                    Err(e) => Response::Error(e.to_string()),
                }
            } else {
                Response::Error("Device not connected".to_string())
            }
        }
        
        Command::InputText { hwnd, text } => {
            let devices = devices.lock().await;
            if let Some(device) = devices.get(&hwnd) {
                match device.input_text(&text) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e.to_string()),
                }
            } else {
                Response::Error("Device not connected".to_string())
            }
        }
        
        Command::KeyEvent { hwnd, key } => {
            let devices = devices.lock().await;
            if let Some(device) = devices.get(&hwnd) {
                match device.key_event(key) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e.to_string()),
                }
            } else {
                Response::Error("Device not connected".to_string())
            }
        }
        
        Command::GetSize { hwnd } => {
            let devices = devices.lock().await;
            if let Some(device) = devices.get(&hwnd) {
                match device.get_size() {
                    Ok((width, height)) => Response::Size { width, height },
                    Err(e) => Response::Error(e.to_string()),
                }
            } else {
                Response::Error("Device not connected".to_string())
            }
        }
        
        Command::Focus { hwnd } => {
            let devices = devices.lock().await;
            if let Some(device) = devices.get(&hwnd) {
                match device.focus() {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e.to_string()),
                }
            } else {
                Response::Error("Device not connected".to_string())
            }
        }
    }
}
