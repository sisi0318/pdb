//! PDB - PC Window Controller with ADB-like interface
//!
//! This library provides a way to control Windows applications similar to how
//! ADB controls Android devices. It supports both local and remote operations.
//!
//! # Example - Local Operation
//!
//! ```rust,no_run
//! use pdb::{WindowController, Device};
//!
//! fn main() -> pdb::Result<()> {
//!     let controller = WindowController::new();
//!     
//!     // List all windows
//!     for window in controller.list_windows()? {
//!         println!("{}: {}", window.hwnd, window.title);
//!     }
//!     
//!     // Connect to a window
//!     let info = controller.find_window("Notepad")?;
//!     let device = Device::new(info);
//!     
//!     // Perform operations
//!     device.click(100, 100)?;
//!     device.input_text("Hello, World!")?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Example - Remote Operation
//!
//! ```rust,no_run
//! use pdb::Client;
//!
//! #[tokio::main]
//! async fn main() -> pdb::Result<()> {
//!     // Connect to remote PDB server
//!     let client = Client::connect("192.168.1.100:5037").await?;
//!     
//!     // List windows on remote machine
//!     let windows = client.list_windows().await?;
//!     
//!     // Connect to a window
//!     let device = client.connect_window("Notepad").await?;
//!     
//!     // Perform remote operations
//!     device.click(100, 100).await?;
//!     device.input_text("Hello from remote!").await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod capture;
pub mod client;
pub mod controller;
pub mod device;
pub mod error;
pub mod input;
pub mod protocol;
pub mod server;
pub mod types;

// Re-export commonly used types
pub use client::{Client, RemoteDevice};
pub use controller::WindowController;
pub use device::Device;
pub use error::{PdbError, Result};
pub use protocol::{Command, Response, DEFAULT_PORT};
pub use server::Server;
pub use types::{KeyCode, Point, Rect, Screenshot, WindowInfo};
