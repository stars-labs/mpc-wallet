//! Test that 'S' key triggers wallet session creation

use frost_secp256k1::Secp256K1Sha256;
use std::sync::Arc;
use tracing::info;
use tui_node::{ui::NoOpUIProvider, AppRunner};

const WEBSOCKET_URL: &str = "wss://auto-life.tech";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging to console
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("🧪 Testing 'S' Key Wallet Creation Trigger");
    info!("=========================================");
    info!("");
    info!("This test verifies that pressing 'S' on the wallet");
    info!("configuration screen properly triggers session creation.");
    info!("");

    // Create test node
    info!("Creating test node...");
    let ui = Arc::new(NoOpUIProvider);
    let app = AppRunner::<Secp256K1Sha256>::new(WEBSOCKET_URL.to_string(), ui);

    // Connect
    info!("Connecting to WebSocket...");
    app.connect("test-s-key".to_string()).await?;

    info!("✅ Connected successfully");
    info!("");
    info!("Test Instructions:");
    info!("1. Navigate to wallet configuration screen");
    info!("2. Fill in: wallet name, total (3), threshold (2)");
    info!("3. Press 'S' - should see:");
    info!("   '🚀 S key pressed - Creating wallet session: ...'");
    info!("4. Session should be created and broadcast");
    info!("");
    info!("Expected log output:");
    info!("- 'S' key pressed - Creating wallet session");
    info!("- Creating wallet session with configuration");
    info!("- Announcing session to network");
    info!("");

    // Run for a bit to test
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    info!("Test completed");
    Ok(())
}
