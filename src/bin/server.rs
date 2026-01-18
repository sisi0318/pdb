//! PDB Server binary - runs the remote control server

use pdb::Server;
use log::info;

#[tokio::main]
async fn main() -> pdb::Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().collect();
    
    let addr = if args.len() > 1 {
        args[1].clone()
    } else {
        format!("0.0.0.0:{}", pdb::DEFAULT_PORT)
    };

    info!("Starting PDB Server on {}", addr);
    let server = Server::new(&addr);
    server.start().await
}
