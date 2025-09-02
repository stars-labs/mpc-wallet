#!/usr/bin/env -S cargo +stable run --bin

//! Test binary to validate session discovery fixes
//! Creates sessions automatically for testing

use clap::Parser;
use frost_secp256k1::Secp256K1Sha256;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use tui_node::handlers::session_handler::WalletSessionConfig;
use tui_node::{ui::NoOpUIProvider, AppRunner};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Device ID for this instance
    #[arg(long = "device-id")]
    device_id: String,

    /// Mode: creator or joiner
    #[arg(long)]
    mode: String,

    /// Test duration in seconds
    #[arg(long, default_value = "20")]
    duration: u64,
}

const WEBSOCKET_URL: &str = "ws://localhost:9000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let args = Args::parse();
    info!(
        "🧪 Starting session discovery test - Device: {}, Mode: {}",
        args.device_id, args.mode
    );

    let ui = Arc::new(NoOpUIProvider);
    let mut app = AppRunner::<Secp256K1Sha256>::new(WEBSOCKET_URL.to_string(), ui.clone());

    // Connect to WebSocket
    app.connect(args.device_id.clone()).await?;
    info!("✅ Connected to WebSocket server");

    // Wait for connection to establish
    sleep(Duration::from_secs(2)).await;

    if args.mode == "creator" {
        run_creator(&mut app).await?;
    } else if args.mode == "joiner" {
        run_joiner(&mut app).await?;
    } else {
        error!("Invalid mode. Use 'creator' or 'joiner'");
        return Ok(());
    }

    // Keep running for the specified duration
    info!("⏰ Running test for {} seconds...", args.duration);
    let _start_time = std::time::Instant::now();

    // Run the app runner to handle messages
    tokio::select! {
        _ = app.run() => {
            info!("AppRunner completed");
        }
        _ = sleep(Duration::from_secs(args.duration)) => {
            info!("Test duration completed");
            app.shutdown();
        }
    }

    info!("🏁 Test completed");
    Ok(())
}

async fn run_creator(app: &mut AppRunner<Secp256K1Sha256>) -> anyhow::Result<()> {
    info!("🎯 Running as session creator");

    // Wait a bit more for proper connection
    sleep(Duration::from_secs(1)).await;

    let config = WalletSessionConfig {
        wallet_name: "Test-Horizon-Session".to_string(),
        description: Some("Test session for discovery validation".to_string()),
        total: 2,
        threshold: 2,
        curve_type: "secp256k1".to_string(),
        mode: tui_node::handlers::session_handler::WalletCreationMode::Online,
        timeout_hours: 1,
        auto_discovery: true,
        blockchain_config: vec![],
    };

    info!("🚀 Creating wallet session: {}", config.wallet_name);
    let cmd_tx = app.get_command_sender();
    let _ = cmd_tx.send(tui_node::utils::state::InternalCommand::CreateWalletSession { config });

    // Periodically trigger session discovery
    tokio::spawn(async move {
        for i in 0..5 {
            sleep(Duration::from_secs(3)).await;
            info!("📡 Triggering session discovery round {}", i + 1);
        }
    });

    Ok(())
}

async fn run_joiner(app: &mut AppRunner<Secp256K1Sha256>) -> anyhow::Result<()> {
    info!("🔍 Running as session joiner");

    // Wait for creator to establish session
    sleep(Duration::from_secs(3)).await;

    // Continuously discover sessions
    let internal_tx = app.get_command_sender();
    tokio::spawn(async move {
        for i in 0..10 {
            sleep(Duration::from_secs(2)).await;
            info!("🔍 Discovering sessions - attempt {}", i + 1);
            let _ = internal_tx.send(tui_node::utils::state::InternalCommand::DiscoverSessions);
        }
    });

    Ok(())
}
