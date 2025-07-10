mod app;
mod config;
mod mpc_manager;
mod state;
mod commands;
mod network;

use anyhow::Result;
use slint::{ModelRc, Timer, TimerMode};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, error, Level};
use tracing_subscriber;

slint::include_modules!();

use state::{AppState, SharedAppState};
use commands::InternalCommand;
use network::websocket::{connect_websocket, handle_websocket_message, WsStream};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting MPC Wallet Native Node");

    // Set display backend environment variables for better rendering
    std::env::set_var("SLINT_BACKEND", "winit");
    std::env::set_var("SLINT_SCALE_FACTOR", "1.0");
    std::env::set_var("SLINT_FONT_RENDERING", "subpixel");
    
    if std::env::var("DISPLAY").is_ok() && std::env::var("WAYLAND_DISPLAY").is_err() {
        std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    }

    // Create the Slint UI
    let ui = MainWindow::new()?;
    
    // Generate device ID
    let device_id = format!("Device-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    
    // Create shared application state
    let app_state: SharedAppState = Arc::new(Mutex::new(AppState::new(device_id.clone())));
    
    // Create command channel
    let (command_tx, command_rx) = mpsc::channel::<InternalCommand>(100);
    
    // Set initial UI state
    {
        let state = app_state.lock().await;
        ui.set_device_id(state.device_id.clone().into());
        ui.set_websocket_connected(false);
        let empty_logs: Vec<slint::SharedString> = vec![];
        ui.set_log_messages(ModelRc::new(slint::VecModel::from(empty_logs)));
    }

    // Setup WebSocket connection holder
    let ws_stream: Arc<Mutex<Option<WsStream>>> = Arc::new(Mutex::new(None));

    // Setup callbacks
    {
        let command_tx = command_tx.clone();
        ui.on_connect_websocket(move || {
            let tx = command_tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::ConnectWebSocket).await;
            });
        });
    }

    {
        let command_tx = command_tx.clone();
        ui.on_create_session(move |session_id, total, threshold| {
            let tx = command_tx.clone();
            let session_id = session_id.to_string();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::ProposeSession {
                    session_id,
                    total_participants: total as u16,
                    threshold: threshold as u16,
                    curve: "secp256k1".to_string(),
                }).await;
            });
        });
    }

    {
        let command_tx = command_tx.clone();
        ui.on_join_session(move |session_id| {
            let tx = command_tx.clone();
            let session_id = session_id.to_string();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::AcceptSessionProposal { session_id }).await;
            });
        });
    }

    {
        let command_tx = command_tx.clone();
        ui.on_start_dkg(move || {
            let tx = command_tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::CheckAndTriggerDkg).await;
            });
        });
    }

    {
        let command_tx = command_tx.clone();
        ui.on_export_keystore(move || {
            let tx = command_tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::ExportKeystore {
                    path: "keystore_export.json".to_string(),
                    password: "password".to_string(), // In real app, prompt for password
                }).await;
            });
        });
    }

    {
        let command_tx = command_tx.clone();
        ui.on_initiate_signing(move |tx_data, blockchain| {
            let tx = command_tx.clone();
            let tx_data = tx_data.to_string();
            let blockchain = blockchain.to_string();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::InitiateSigning {
                    transaction_data: tx_data,
                    blockchain,
                }).await;
            });
        });
    }

    // Setup periodic UI updates
    let ui_weak = ui.as_weak();
    let app_state_clone = app_state.clone();
    let timer = Timer::default();
    timer.start(
        TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                let state = app_state_clone.clone();
                slint::spawn_local(async move {
                    let state = state.lock().await;
                    
                    // Update connection status
                    ui.set_websocket_connected(state.websocket_connected);
                    
                    // Update log messages
                    let log_messages: Vec<slint::SharedString> = state.log_messages
                        .iter()
                        .map(|s| s.as_str().into())
                        .collect();
                    ui.set_log_messages(ModelRc::new(slint::VecModel::from(log_messages)));
                    
                    // Update connected devices count
                    let connected_count = state.connected_devices.len();
                    ui.set_current_participants(connected_count as i32 + 1); // +1 for self
                }).unwrap();
            }
        },
    );

    // Spawn the main event loop
    let app_state_clone = app_state.clone();
    let ws_stream_clone = ws_stream.clone();
    tokio::spawn(async move {
        handle_commands(
            command_rx,
            app_state_clone,
            ws_stream_clone,
            command_tx.clone(),
        ).await;
    });

    info!("MPC Wallet Native Node UI started");
    ui.run()?;

    Ok(())
}

async fn handle_commands(
    mut command_rx: mpsc::Receiver<InternalCommand>,
    app_state: SharedAppState,
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    command_tx: mpsc::Sender<InternalCommand>,
) {
    while let Some(command) = command_rx.recv().await {
        match command {
            InternalCommand::ConnectWebSocket => {
                let state = app_state.lock().await;
                let url = state.websocket_url.clone();
                let device_id = state.device_id.clone();
                drop(state);
                
                match connect_websocket(&url, &device_id).await {
                    Ok(stream) => {
                        let mut state = app_state.lock().await;
                        state.websocket_connected = true;
                        state.add_log("Connected to WebSocket server".to_string());
                        drop(state);
                        
                        // Store the stream
                        *ws_stream.lock().await = Some(stream);
                        
                        // Spawn WebSocket message handler
                        let ws_stream_clone = ws_stream.clone();
                        let app_state_clone = app_state.clone();
                        let command_tx_clone = command_tx.clone();
                        tokio::spawn(async move {
                            if let Some(mut stream) = ws_stream_clone.lock().await.take() {
                                use futures_util::StreamExt;
                                while let Some(msg) = stream.next().await {
                                    match msg {
                                        Ok(message) => {
                                            if let Err(e) = handle_websocket_message(
                                                message,
                                                app_state_clone.clone(),
                                                &command_tx_clone,
                                            ).await {
                                                error!("Error handling WebSocket message: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("WebSocket error: {}", e);
                                            break;
                                        }
                                    }
                                }
                                
                                // Update connection status
                                let mut state = app_state_clone.lock().await;
                                state.websocket_connected = false;
                                state.add_log("WebSocket connection lost".to_string());
                            }
                        });
                    }
                    Err(e) => {
                        let mut state = app_state.lock().await;
                        state.add_log(format!("Failed to connect: {}", e));
                        error!("Failed to connect to WebSocket: {}", e);
                    }
                }
            }
            
            InternalCommand::ProposeSession { session_id, total_participants, threshold, curve } => {
                let mut state = app_state.lock().await;
                state.add_log(format!(
                    "Creating session '{}' with {}/{} participants", 
                    session_id, threshold, total_participants
                ));
                
                // Create session info
                state.current_session = Some(crate::state::SessionInfo {
                    session_id: session_id.clone(),
                    total_participants,
                    threshold,
                    participants: vec![state.device_id.clone()],
                    is_creator: true,
                    curve: curve.clone(),
                });
                
                // TODO: Send session proposal to connected devices
                state.add_log("Session created, waiting for participants...".to_string());
            }
            
            InternalCommand::CheckAndTriggerDkg => {
                let mut state = app_state.lock().await;
                if state.is_mesh_ready() {
                    state.add_log("Mesh is ready, starting DKG...".to_string());
                    // TODO: Implement DKG
                } else {
                    state.add_log("Mesh not ready yet".to_string());
                }
            }
            
            InternalCommand::UpdateUI => {
                // UI updates are handled by the timer
            }
            
            _ => {
                info!("Unhandled command: {:?}", command);
            }
        }
    }
}