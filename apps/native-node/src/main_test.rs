// Test version without UI to verify core functionality
mod app;
mod config;
mod mpc_manager;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

use app::MpcWalletApp;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting MPC Wallet Native Node (Test Mode - No UI)");

    // Create the MPC wallet application
    let app = MpcWalletApp::new().await?;
    
    info!("Device ID: {}", app.get_device_id());
    info!("Application initialized successfully");
    
    // Test basic functionality
    info!("Testing configuration...");
    let config = app.get_config().await;
    info!("WebSocket URL: {}", config.websocket_url);
    info!("Data directory: {}", config.data_dir.display());
    
    info!("MPC Wallet Native Node test completed successfully");
    Ok(())
}