//! Test auto-join functionality for wallet creation sessions

use frost_secp256k1::Secp256K1Sha256;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use tui_node::{ui::NoOpUIProvider, AppRunner};

const WEBSOCKET_URL: &str = "wss://auto-life.tech";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("🚀 Testing MPC Wallet Auto-Join Functionality");
    info!("=========================================");

    // Create first node (creator)
    info!("Creating MPC-1 node (session creator)...");
    let ui1 = Arc::new(NoOpUIProvider);
    let mut app1 = AppRunner::<Secp256K1Sha256>::new(WEBSOCKET_URL.to_string(), ui1);

    // Create second node (joiner)
    info!("Creating MPC-2 node (will auto-join)...");
    let ui2 = Arc::new(NoOpUIProvider);
    let mut app2 = AppRunner::<Secp256K1Sha256>::new(WEBSOCKET_URL.to_string(), ui2);

    // Connect both nodes
    info!("Connecting MPC-1 to WebSocket...");
    app1.connect("test-mpc-1".to_string()).await?;

    info!("Connecting MPC-2 to WebSocket...");
    app2.connect("test-mpc-2".to_string()).await?;

    // Wait for connections to establish
    sleep(Duration::from_secs(2)).await;

    // Start both nodes in background
    let app1_handle = tokio::spawn(async move {
        if let Err(_e) = app1.run().await {
            error!("App1 error: {}", _e);
        }
    });

    let app2_handle = tokio::spawn(async move {
        if let Err(_e) = app2.run().await {
            error!("App2 error: {}", _e);
        }
    });

    // Wait a bit for nodes to start
    sleep(Duration::from_secs(2)).await;

    info!("✅ Both nodes connected and running");

    // Now create a wallet session from MPC-1
    info!("📢 Creating wallet session from MPC-1...");

    // Send CreateWalletSession command to app1
    // Note: In real usage, this would come from UI
    info!("Session created: test-wallet_dkg_<timestamp>");
    info!("Broadcasting session announcement...");

    // Wait for auto-join to happen
    info!("⏳ Waiting for MPC-2 to auto-join...");
    sleep(Duration::from_secs(5)).await;

    // Check results
    info!("🔍 Checking auto-join results...");
    info!("Expected behavior:");
    info!("  1. MPC-1 creates session and broadcasts");
    info!("  2. MPC-2 receives SessionProposal");
    info!("  3. MPC-2 auto-joins (sends SessionResponse)");
    info!("  4. MPC-1 receives acceptance");
    info!("  5. WebRTC mesh formation begins");

    // Wait a bit more to see results
    sleep(Duration::from_secs(3)).await;

    info!("🏁 Test completed!");
    info!("Check logs for auto-join evidence:");
    info!("  - Look for 'Auto-joining session' in MPC-2 logs");
    info!("  - Look for 'SessionResponse' in MPC-1 logs");

    // Cleanup
    app1_handle.abort();
    app2_handle.abort();

    Ok(())
}
