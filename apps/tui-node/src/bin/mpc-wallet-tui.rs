//! MPC Wallet TUI - Uses AppRunner for consistent architecture
//! This ensures TUI and tests use the same code paths

#![allow(deprecated)]

use clap::Parser;
use frost_secp256k1::Secp256K1Sha256;
use std::io::IsTerminal;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::info;
use tui_node::{
    ui::{tui::UIMode, NoOpUIProvider, UIProvider},
    AppRunner,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Log file location
    #[arg(long, default_value = "~/.frost_keystore/logs/mpc-wallet.log")]
    log_location: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Device ID for this instance (must be unique)
    /// Example: --device-id alice (creates .mpc-wallet-alice/)
    /// If not provided, uses hostname
    #[arg(long = "device-id")]
    device_id: Option<String>,

    /// Run in headless mode (no TUI)
    #[arg(long)]
    headless: bool,

    /// Signal server URL
    /// Example: --signal-server ws://localhost:9000
    #[arg(long, default_value = "wss://auto-life.tech")]
    signal_server: String,
}

// Signal server URL is now configurable via CLI argument

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Determine device ID
    let device_id = args.device_id.unwrap_or_else(|| {
        gethostname::gethostname()
            .into_string()
            .unwrap_or_else(|_| "default-node".to_string())
    });

    if args.headless {
        // Setup logging only for headless mode
        tracing_subscriber::fmt()
            .with_env_filter(args.log_level)
            .init();

        info!("Starting MPC Wallet TUI with device ID: {}", device_id);
        info!("Signal server: {}", args.signal_server);
        run_headless(device_id, args.signal_server).await
    } else {
        // For TUI mode, log to a file in the current directory
        let log_filename = format!("mpc-wallet-{}.log", device_id);
        println!("Logging to: {}", log_filename);
        println!(
            "Current directory: {:?}",
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        );

        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true) // Start fresh each run
            .open(&log_filename)
            .unwrap_or_else(|e| {
                eprintln!("Failed to create log file {}: {}", log_filename, e);
                std::fs::File::create("/dev/null").unwrap()
            });

        tracing_subscriber::fmt()
            .with_writer(log_file)
            .with_env_filter(args.log_level)
            .with_ansi(false)
            .init();

        info!("=== TUI Node Started ===");
        info!("Device ID: {}", device_id);
        info!(
            "Working directory: {:?}",
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        );
        info!("Log file: {}", log_filename);
        info!("Signal server: {}", args.signal_server);

        run_with_tui(device_id, args.signal_server).await
    }
}

/// Run in headless mode (for testing or automation)
async fn run_headless(device_id: String, signal_server: String) -> anyhow::Result<()> {
    info!("Running in headless mode");

    let ui = Arc::new(NoOpUIProvider);
    let mut app = AppRunner::<Secp256K1Sha256>::new(signal_server, ui);

    // Connect to WebSocket
    app.connect(device_id.clone()).await?;
    info!("Connected with device ID: {}", device_id);

    // Run the app (blocks until shutdown)
    app.run().await?;

    Ok(())
}

/// Run with TUI
async fn run_with_tui(device_id: String, signal_server: String) -> anyhow::Result<()> {
    use crossterm::{
        cursor,
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;
    use tokio::time::sleep;
    use tui_node::ui::tui_provider::TuiProvider;

    // Setup terminal
    // Check if we're in a TTY environment
    if !std::io::stdout().is_terminal() {
        // Don't print to console in TUI mode as it corrupts the display
        return Err(anyhow::anyhow!(
            "TUI requires a TTY environment. Use --headless mode or run in a proper terminal."
        ));
    }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Create TUI provider
    let (tui_provider, mut redraw_rx) = TuiProvider::new(terminal);
    let ui_provider = Arc::new(tui_provider.clone());

    // Create AppRunner (not wrapped yet, so we can call run())
    let mut app_runner_instance =
        AppRunner::<Secp256K1Sha256>::new(signal_server, ui_provider.clone());

    // Get command sender and app state before moving the runner
    let cmd_tx = app_runner_instance.get_command_sender();
    let app_state = app_runner_instance.get_app_state();

    // Connect to WebSocket
    app_runner_instance.connect(device_id.clone()).await?;
    tui_provider.set_device_id(device_id.clone()).await;

    // Start AppRunner in background task
    let app_runner_handle = tokio::spawn(async move {
        tracing::info!("🚀 Starting AppRunner event loop");
        if let Err(_e) = app_runner_instance.run().await {
            tracing::error!("❌ AppRunner error: {}", _e);
        }
        tracing::info!("AppRunner stopped");
    });

    // Store command sender and app state for access
    let cmd_tx = Arc::new(cmd_tx);
    let app_state = Arc::new(app_state);

    // Command sender and app state are already available

    // AppRunner is already running in background (started above)

    // TUI event loop
    let mut should_quit = false;

    loop {
        // Handle UI updates
        tokio::select! {
            _ = redraw_rx.recv() => {
                // Redraw requested
                let state = app_state.lock().await;
                tui_provider.render(&*state).await;
            }
            _ = sleep(Duration::from_millis(100)) => {
                // Check for keyboard events
                if event::poll(Duration::from_millis(10))? {
                    if let Event::Key(key) = event::read()? {
                        // Handle key events
                        if let Some(command) = tui_provider.handle_key_event(key).await {
                            match command.as_str() {
                                "quit" => {
                                    should_quit = true;
                                }
                                "start_dkg" => {
                                    let _ = cmd_tx.send(tui_node::utils::state::InternalCommand::<frost_secp256k1::Secp256K1Sha256>::TriggerDkgRound1);
                                }
                                cmd if cmd.starts_with("create_wallet_session:") => {
                                    tracing::info!("🚀 Processing wallet creation command: {}", cmd);

                                    // Parse the wallet creation command
                                    let parts: Vec<&str> = cmd.split(':').collect();
                                    if parts.len() == 6 {
                                        let wallet_name = parts[1].to_string();
                                        let total = parts[2].parse::<u16>().unwrap_or(3);
                                        let threshold = parts[3].parse::<u16>().unwrap_or(2);
                                        let mode = parts[4].parse::<usize>().unwrap_or(0);
                                        let curve = parts[5].parse::<usize>().unwrap_or(0);

                                        // Map mode and curve to actual values
                                        let curve_type = if curve == 0 { "secp256k1" } else { "ed25519" };
                                        let creation_mode = match mode {
                                            1 => tui_node::handlers::session_handler::WalletCreationMode::Offline,
                                            2 => tui_node::handlers::session_handler::WalletCreationMode::Hybrid,
                                            _ => tui_node::handlers::session_handler::WalletCreationMode::Online,
                                        };

                                        tracing::info!("Creating wallet session: {} ({}/{}) {:?} {}",
                                            wallet_name, threshold, total, creation_mode, curve_type);

                                        // Create wallet session configuration
                                        let config = tui_node::handlers::session_handler::WalletSessionConfig {
                                            wallet_name: wallet_name.clone(),
                                            description: Some(format!("{}/{} {} wallet", threshold, total, curve_type)),
                                            total,
                                            threshold,
                                            curve_type: curve_type.to_string(),
                                            mode: creation_mode,
                                            timeout_hours: 24,
                                            auto_discovery: true,
                                            blockchain_config: vec![
                                                tui_node::handlers::session_handler::BlockchainConfig {
                                                    blockchain: "ethereum".to_string(),
                                                    network: "mainnet".to_string(),
                                                    enabled: true,
                                                    chain_id: Some(1),
                                                }
                                            ],
                                        };

                                        // Send the proper command to create wallet session
                                        tracing::info!("📤 Sending wallet creation command: {}", wallet_name);

                                        let send_result = cmd_tx.send(tui_node::utils::state::InternalCommand::<frost_secp256k1::Secp256K1Sha256>::CreateWalletSession {
                                            config: config.clone(),
                                        });

                                        match send_result {
                                            Ok(_) => tracing::info!("✅ Wallet creation command queued successfully"),
                                            Err(_e) => tracing::error!("❌ Failed to queue wallet creation command: {}", _e),
                                        }

                                        // Show progress screen
                                        tui_provider.set_ui_mode(UIMode::DkgProgress {
                                            allow_cancel: true
                                        }).await;
                                    }
                                }
                                cmd if cmd.starts_with("propose_session:") => {
                                    // Parse the session proposal command
                                    let parts: Vec<&str> = cmd.split(':').collect();
                                    if parts.len() == 5 {
                                        let session_name = parts[1].to_string();
                                        let total_str = parts[2];
                                        let threshold_str = parts[3];
                                        let participants_str = parts[4];

                                        if let (Ok(total), Ok(threshold)) = (total_str.parse::<u16>(), threshold_str.parse::<u16>()) {
                                            let participants: Vec<String> = participants_str
                                                .split(',')
                                                .map(|s| s.trim().to_string())
                                                .filter(|s| !s.is_empty())
                                                .collect();


                                            // Send propose session command
                                            let _ = cmd_tx.send(tui_node::utils::state::InternalCommand::<frost_secp256k1::Secp256K1Sha256>::ProposeSession {
                                                session_id: session_name,
                                                total,
                                                threshold,
                                                participants,
                                            });

                                            // For now, just go back to main menu
                                            tui_provider.set_ui_mode(UIMode::MainMenu { selected_index: 0 }).await;
                                        }
                                    }
                                }
                                cmd if cmd.starts_with("join_session:") => {
                                    // Parse the join session command
                                    let parts: Vec<&str> = cmd.split(':').collect();
                                    if parts.len() == 2 {
                                        let session_index = parts[1].parse::<usize>().unwrap_or(0);

                                        // Get available sessions from state and join the selected one
                                        let state = app_state.lock().await;

                                        if session_index < state.available_sessions.len() {
                                            let session = &state.available_sessions[session_index];
                                            let session_id = session.session_code.clone();
                                            drop(state);

                                            tracing::info!("Joining session: {}", session_id);
                                            // Use JoinSession which properly handles both initial join and rejoin scenarios
                                            let _ = cmd_tx.send(tui_node::utils::state::InternalCommand::<frost_secp256k1::Secp256K1Sha256>::JoinSession(
                                                session_id
                                            ));

                                            // Show DKG progress
                                            tui_provider.set_ui_mode(UIMode::DkgProgress {
                                                allow_cancel: true
                                            }).await;
                                        }
                                    }
                                }
                                "discover_sessions" => {
                                    // Trigger session discovery
                                    let _ = cmd_tx.send(tui_node::utils::state::InternalCommand::<frost_secp256k1::Secp256K1Sha256>::DiscoverSessions);

                                    // Show session discovery screen
                                    tui_provider.set_ui_mode(UIMode::SessionDiscovery {
                                        selected_index: 0,
                                        filter_text: String::new(),
                                        input_mode: false,
                                    }).await;
                                }
                                _ => {
                                    // Log to file, not console in TUI mode
                                    tracing::info!("Unhandled command: {}", command);
                                }
                            }
                        }

                        // Handle Esc directly
                        if key.code == KeyCode::Esc {
                            should_quit = true;
                        }

                        // Handle Ctrl-C
                        if key.code == KeyCode::Char('c') &&
                           key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            should_quit = true;
                        }
                    }
                }

                // Regular render
                let state = app_state.lock().await;
                tui_provider.render(&*state).await;
            }
        }

        if should_quit {
            break;
        }
    }

    // Restore terminal FIRST (before any async operations that might hang)
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)?;

    // Abort the runner handle
    app_runner_handle.abort();

    // Force clean exit
    std::process::exit(0);
}
