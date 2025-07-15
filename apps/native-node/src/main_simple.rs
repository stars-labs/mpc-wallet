use anyhow::Result;
use async_trait::async_trait;
use cli_node::{AppRunner, UIProvider};
use cli_node::protocal::signal::SessionInfo;
use cli_node::utils::state::{PendingSigningRequest, InternalCommand};
use frost_secp256k1::Secp256K1Sha256;
use slint::{ModelRc, ComponentHandle};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

slint::include_modules!();

/// Simple UI provider that updates global state
struct SimpleUIProvider;

impl SimpleUIProvider {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UIProvider for SimpleUIProvider {
    async fn set_connection_status(&self, connected: bool) {
        info!("Setting connection status to: {}", connected);
        
        // Use run_in_event_loop to ensure we're on the UI thread
        let _ = slint::invoke_from_event_loop(move || {
            let app_state = AppState::get(&slint::Window::new().unwrap());
            app_state.set_websocket_connected(connected);
            info!("Updated websocket_connected in AppState to: {}", connected);
        });
    }
    
    async fn set_device_id(&self, device_id: String) {
        let _ = slint::invoke_from_event_loop(move || {
            let app_state = AppState::get(&slint::Window::new().unwrap());
            app_state.set_device_id(device_id.into());
        });
    }
    
    async fn update_device_list(&self, _devices: Vec<String>) {}
    
    async fn update_device_status(&self, _device_id: String, _status: String) {}
    
    async fn update_session_status(&self, status: String) {
        let _ = slint::invoke_from_event_loop(move || {
            let app_state = AppState::get(&slint::Window::new().unwrap());
            app_state.set_session_status(status.into());
        });
    }
    
    async fn add_session_invite(&self, _invite: SessionInfo) {}
    
    async fn remove_session_invite(&self, _session_id: String) {}
    
    async fn set_active_session(&self, session: Option<SessionInfo>) {
        if let Some(session) = session {
            let status = format!("{} ({}/{})", session.session_id, session.participants.len(), session.total);
            self.update_session_status(status).await;
        }
    }
    
    async fn update_dkg_status(&self, status: String) {
        self.update_session_status(format!("DKG: {}", status)).await;
    }
    
    async fn set_generated_address(&self, address: Option<String>) {
        if let Some(address) = address {
            let _ = slint::invoke_from_event_loop(move || {
                let app_state = AppState::get(&slint::Window::new().unwrap());
                app_state.set_generated_address(address.into());
            });
        }
    }
    
    async fn set_group_public_key(&self, _key: Option<String>) {}
    
    async fn add_signing_request(&self, request: PendingSigningRequest) {
        let _ = slint::invoke_from_event_loop(move || {
            let app_state = AppState::get(&slint::Window::new().unwrap());
            let mut requests = app_state.get_pending_signing_requests().to_vec();
            requests.push(format!("ID: {} from {}", request.signing_id, request.from_device).into());
            app_state.set_pending_signing_requests(ModelRc::new(slint::VecModel::from(requests)));
        });
    }
    
    async fn remove_signing_request(&self, _signing_id: String) {}
    
    async fn update_signing_status(&self, status: String) {
        self.update_session_status(format!("Signing: {}", status)).await;
    }
    
    async fn set_signature_result(&self, _signing_id: String, _signature: Vec<u8>) {}
    
    async fn update_wallet_list(&self, _wallets: Vec<String>) {}
    
    async fn set_selected_wallet(&self, _wallet_id: Option<String>) {}
    
    async fn add_log(&self, message: String) {
        let _ = slint::invoke_from_event_loop(move || {
            let app_state = AppState::get(&slint::Window::new().unwrap());
            let mut logs = app_state.get_log_messages().to_vec();
            logs.push(message.into());
            if logs.len() > 100 {
                logs.drain(0..logs.len() - 100);
            }
            app_state.set_log_messages(ModelRc::new(slint::VecModel::from(logs)));
        });
    }
    
    async fn set_logs(&self, logs: Vec<String>) {
        let _ = slint::invoke_from_event_loop(move || {
            let app_state = AppState::get(&slint::Window::new().unwrap());
            let messages: Vec<slint::SharedString> = logs.into_iter()
                .map(|s| s.into())
                .collect();
            app_state.set_log_messages(ModelRc::new(slint::VecModel::from(messages)));
        });
    }
    
    async fn update_mesh_status(&self, ready_devices: usize, total_devices: usize) {
        self.add_log(format!("Mesh: {}/{} ready", ready_devices, total_devices)).await;
    }
    
    async fn show_error(&self, error: String) {
        self.add_log(format!("ERROR: {}", error)).await;
    }
    
    async fn show_success(&self, message: String) {
        self.add_log(format!("SUCCESS: {}", message)).await;
    }
    
    async fn set_busy(&self, _busy: bool) {}
    
    async fn set_progress(&self, _progress: Option<f32>) {}
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("Starting MPC Wallet Native Node (Simple Reactive)");
    
    // Create UI
    let ui = MainWindow::new()?;
    
    // Get AppState handle for initial setup
    let app_state = ui.global::<AppState>();
    app_state.set_websocket_connected(false);
    app_state.set_device_id("".into());
    
    // Create simple UI provider
    let ui_provider = Arc::new(SimpleUIProvider::new());
    
    // Create app runner
    let app_runner = AppRunner::<Secp256K1Sha256>::new(
        "wss://auto-life.tech".to_string(),
        ui_provider.clone(),
    );
    
    // Get command sender
    let cmd_sender = app_runner.get_command_sender();
    
    // Setup UI callbacks
    {
        let tx = cmd_sender.clone();
        ui.on_connect_websocket(move |device_id| {
            info!("Connect button clicked with device ID: {}", device_id);
            let device_id = device_id.to_string();
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::SendToServer(
                    webrtc_signal_server::ClientMsg::Register { device_id }
                ));
            });
        });
    }
    
    {
        let tx = cmd_sender.clone();
        ui.on_create_session(move |session_id, total, threshold| {
            let session_id = session_id.to_string();
            let total = total as u16;
            let threshold = threshold as u16;
            let tx = tx.clone();
            tokio::spawn(async move {
                let participants: Vec<String> = (1..=total).map(|i| format!("device-{}", i)).collect();
                let _ = tx.send(InternalCommand::ProposeSession {
                    session_id,
                    total,
                    threshold,
                    participants,
                });
            });
        });
    }
    
    {
        let tx = cmd_sender.clone();
        ui.on_start_dkg(move || {
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::TriggerDkgRound1);
            });
        });
    }
    
    {
        let tx = cmd_sender.clone();
        ui.on_initiate_signing(move |tx_data, blockchain| {
            let tx_data = tx_data.to_string();
            let blockchain = blockchain.to_string();
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::InitiateSigning {
                    transaction_data: tx_data,
                    blockchain,
                    chain_id: None,
                });
            });
        });
    }
    
    // Run the app logic
    tokio::spawn(async move {
        if let Err(e) = app_runner.run().await {
            eprintln!("App runner error: {}", e);
        }
    });
    
    info!("MPC Wallet Native Node started");
    
    // Run UI event loop
    ui.run()?;
    
    Ok(())
}