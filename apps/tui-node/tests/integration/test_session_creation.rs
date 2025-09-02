//! Test session creation and WebSocket broadcasting

use frost_secp256k1::Secp256K1Sha256;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::info;
use tui_node::{ui::NoOpUIProvider, AppRunner};

const WEBSOCKET_URL: &str = "wss://auto-life.tech";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("🧪 Testing Session Creation and Broadcasting");
    info!("============================================");

    // Create test node
    info!("Creating test node...");
    let ui = Arc::new(NoOpUIProvider);
    let app = AppRunner::<Secp256K1Sha256>::new(WEBSOCKET_URL.to_string(), ui);

    // Connect to WebSocket
    info!("Connecting to WebSocket server...");
    app.connect("test-session-creator".to_string()).await?;

    // Wait for connection
    sleep(Duration::from_secs(2)).await;

    // Create a wallet session
    info!("Creating wallet session...");
    let config = tui_node::handlers::session_handler::WalletSessionConfig {
        wallet_name: "TestWallet".to_string(),
        description: Some("Test wallet for debugging".to_string()),
        total: 3,
        threshold: 2,
        curve_type: "secp256k1".to_string(),
        mode: tui_node::handlers::session_handler::WalletCreationMode::Online,
        timeout_hours: 24,
        auto_discovery: true,
        blockchain_config: vec![],
    };

    // Send create wallet session command
    let cmd_tx = app.get_command_sender();
    let _ = cmd_tx.send(
        tui_node::utils::state::InternalCommand::<Secp256K1Sha256>::CreateWalletSession { config },
    );

    info!("Waiting for session to be created and broadcast...");
    sleep(Duration::from_secs(5)).await;

    info!("✅ Test complete - check logs for:");
    info!("  1. 'Creating wallet session' message");
    info!("  2. 'Announcing session' message");
    info!("  3. 'Session announcement sent to network' message");
    info!("");
    info!("If #3 is missing, the WebSocket sending is broken!");

    Ok(())
}
