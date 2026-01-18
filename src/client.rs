//! Client implementation for remote connections

use crate::error::{PdbError, Result};
use crate::protocol::{Command, MessageHeader, Response, DEFAULT_PORT};
use crate::types::{KeyCode, Screenshot, WindowInfo};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Remote client - connects to PDB server (like ADB client)
pub struct Client {
    stream: Arc<Mutex<TcpStream>>,
}

impl Client {
    /// Connect to remote server
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    /// Connect to localhost with default port
    pub async fn connect_local() -> Result<Self> {
        Self::connect(&format!("127.0.0.1:{}", DEFAULT_PORT)).await
    }

    /// Send command and receive response
    async fn send_command(&self, command: Command) -> Result<Response> {
        let mut stream = self.stream.lock().await;

        // Serialize command
        let command_json = serde_json::to_vec(&command)?;
        let header = MessageHeader::new(command_json.len() as u32);

        // Send header
        let mut header_buf = [0u8; 8];
        header_buf[0..4].copy_from_slice(&header.version.to_le_bytes());
        header_buf[4..8].copy_from_slice(&header.length.to_le_bytes());
        stream.write_all(&header_buf).await?;

        // Send body
        stream.write_all(&command_json).await?;
        stream.flush().await?;

        // Read response header
        let mut resp_header_buf = [0u8; 8];
        stream.read_exact(&mut resp_header_buf).await?;
        let resp_length = u32::from_le_bytes(resp_header_buf[4..8].try_into().unwrap());

        // Read response body
        let mut resp_body = vec![0u8; resp_length as usize];
        stream.read_exact(&mut resp_body).await?;

        // Parse response
        let response: Response = serde_json::from_slice(&resp_body)?;
        Ok(response)
    }

    /// Ping server
    pub async fn ping(&self) -> Result<bool> {
        match self.send_command(Command::Ping).await? {
            Response::Pong => Ok(true),
            _ => Ok(false),
        }
    }

    /// List all windows on remote machine
    pub async fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        match self.send_command(Command::ListWindows).await? {
            Response::Windows(windows) => Ok(windows),
            Response::Error(e) => Err(PdbError::ConnectionError(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }

    /// Connect to a window by title
    pub async fn connect_window(&self, title: &str) -> Result<RemoteDevice> {
        match self.send_command(Command::Connect { title: title.to_string() }).await? {
            Response::Window(info) => Ok(RemoteDevice {
                client: self.stream.clone(),
                info,
            }),
            Response::Error(e) => Err(PdbError::WindowNotFound(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }

    /// Connect to a window by hwnd
    pub async fn connect_window_by_hwnd(&self, hwnd: usize) -> Result<RemoteDevice> {
        match self.send_command(Command::ConnectByHwnd { hwnd }).await? {
            Response::Window(info) => Ok(RemoteDevice {
                client: self.stream.clone(),
                info,
            }),
            Response::Error(e) => Err(PdbError::WindowNotFound(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }
}

/// Remote device - represents a window on the remote machine
pub struct RemoteDevice {
    client: Arc<Mutex<TcpStream>>,
    info: WindowInfo,
}

impl RemoteDevice {
    /// Get window info
    pub fn info(&self) -> &WindowInfo {
        &self.info
    }

    /// Get window handle
    pub fn hwnd(&self) -> usize {
        self.info.hwnd
    }

    /// Send command helper
    async fn send_command(&self, command: Command) -> Result<Response> {
        let mut stream = self.client.lock().await;

        let command_json = serde_json::to_vec(&command)?;
        let header = MessageHeader::new(command_json.len() as u32);

        let mut header_buf = [0u8; 8];
        header_buf[0..4].copy_from_slice(&header.version.to_le_bytes());
        header_buf[4..8].copy_from_slice(&header.length.to_le_bytes());
        stream.write_all(&header_buf).await?;
        stream.write_all(&command_json).await?;
        stream.flush().await?;

        let mut resp_header_buf = [0u8; 8];
        stream.read_exact(&mut resp_header_buf).await?;
        let resp_length = u32::from_le_bytes(resp_header_buf[4..8].try_into().unwrap());

        let mut resp_body = vec![0u8; resp_length as usize];
        stream.read_exact(&mut resp_body).await?;

        let response: Response = serde_json::from_slice(&resp_body)?;
        Ok(response)
    }

    /// Click at position
    pub async fn click(&self, x: i32, y: i32) -> Result<()> {
        match self.send_command(Command::Click { hwnd: self.info.hwnd, x, y }).await? {
            Response::Ok => Ok(()),
            Response::Error(e) => Err(PdbError::InputError(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }

    /// Tap at position (alias for click)
    pub async fn tap(&self, x: i32, y: i32) -> Result<()> {
        self.click(x, y).await
    }

    /// Swipe from one position to another
    pub async fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<()> {
        match self.send_command(Command::Swipe {
            hwnd: self.info.hwnd,
            x1, y1, x2, y2,
            duration_ms,
        }).await? {
            Response::Ok => Ok(()),
            Response::Error(e) => Err(PdbError::InputError(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }

    /// Take screenshot
    pub async fn screenshot(&self) -> Result<Screenshot> {
        match self.send_command(Command::Screenshot { hwnd: self.info.hwnd }).await? {
            Response::Screenshot(s) => Ok(s),
            Response::Error(e) => Err(PdbError::CaptureError(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }

    /// Input text
    pub async fn input_text(&self, text: &str) -> Result<()> {
        match self.send_command(Command::InputText {
            hwnd: self.info.hwnd,
            text: text.to_string(),
        }).await? {
            Response::Ok => Ok(()),
            Response::Error(e) => Err(PdbError::InputError(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }

    /// Send key event
    pub async fn key_event(&self, key: KeyCode) -> Result<()> {
        match self.send_command(Command::KeyEvent { hwnd: self.info.hwnd, key }).await? {
            Response::Ok => Ok(()),
            Response::Error(e) => Err(PdbError::InputError(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }

    /// Get window size
    pub async fn get_size(&self) -> Result<(i32, i32)> {
        match self.send_command(Command::GetSize { hwnd: self.info.hwnd }).await? {
            Response::Size { width, height } => Ok((width, height)),
            Response::Error(e) => Err(PdbError::HandleError(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }

    /// Focus window
    pub async fn focus(&self) -> Result<()> {
        match self.send_command(Command::Focus { hwnd: self.info.hwnd }).await? {
            Response::Ok => Ok(()),
            Response::Error(e) => Err(PdbError::HandleError(e)),
            _ => Err(PdbError::ProtocolError("Unexpected response".into())),
        }
    }
}
