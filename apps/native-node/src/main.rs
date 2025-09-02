use anyhow::Result;
use async_trait::async_trait;
use tui_node::{AppRunner, UIProvider};
use tui_node::protocal::signal::SessionInfo;
use tui_node::utils::state::{PendingSigningRequest, InternalCommand};
use frost_secp256k1::Secp256K1Sha256;
use slint::{ModelRc, ComponentHandle, Model, VecModel};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

slint::include_modules!();

/// Simple UI provider that updates global state
struct SimpleUIProvider {
    window: slint::Weak<MainWindow>,
}

impl SimpleUIProvider {
    fn new(window: slint::Weak<MainWindow>) -> Self {
        Self { window }
    }
}

#[async_trait]
impl UIProvider for SimpleUIProvider {
    async fn set_connection_status(&self, connected: bool) {
        info!("UIProvider::set_connection_status called with: {}", connected);
        
        let window = self.window.clone();
        let result = slint::invoke_from_event_loop(move || {
            if let Some(window) = window.upgrade() {
                let app_state = window.global::<AppState>();
                app_state.set_websocket_connected(connected);
                info!("Successfully updated websocket_connected in AppState to: {}", connected);
                
                // Also add a log message
                let current_logs = app_state.get_log_messages();
                let mut logs: Vec<slint::SharedString> = Vec::new();
                
                for i in 0..current_logs.row_count() {
                    if let Some(log) = current_logs.row_data(i) {
                        logs.push(log);
                    }
                }
                
                let status_msg = if connected {
                    "✓ Connected to WebSocket server"
                } else {
                    "✗ Disconnected from WebSocket server"
                };
                logs.push(status_msg.into());
                
                if logs.len() > 100 {
                    logs.drain(0..logs.len() - 100);
                }
                
                app_state.set_log_messages(ModelRc::new(slint::VecModel::from(logs)));
            } else {
                info!("Failed to upgrade window weak reference");
            }
        });
        
        if result.is_err() {
            info!("Failed to invoke from event loop");
        }
    }
    
    async fn set_device_id(&self, device_id: String) {
        let window = self.window.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window.upgrade() {
                let app_state = window.global::<AppState>();
                app_state.set_device_id(device_id.into());
            }
        });
    }
    
    async fn update_device_list(&self, _devices: Vec<String>) {}
    
    async fn update_device_status(&self, _device_id: String, _status: String) {}
    
    async fn update_session_status(&self, status: String) {
        let window = self.window.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window.upgrade() {
                let app_state = window.global::<AppState>();
                app_state.set_session_status(status.into());
            }
        });
    }
    
    async fn add_session_invite(&self, invite: SessionInfo) {
        let window = self.window.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window.upgrade() {
                let app_state = window.global::<AppState>();
                let mut invites: Vec<SessionInvite> = app_state.get_session_invites().iter().collect();
                invites.push(SessionInvite {
                    session_id: invite.session_id.into(),
                    from_device: invite.proposer_id.into(),
                });
                app_state.set_session_invites(ModelRc::new(VecModel::from(invites)));
            }
        });
    }
    
    async fn remove_session_invite(&self, session_id: String) {
        let window = self.window.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window.upgrade() {
                let app_state = window.global::<AppState>();
                let mut invites: Vec<SessionInvite> = app_state.get_session_invites().iter().collect();
                invites.retain(|i| i.session_id != session_id);
                app_state.set_session_invites(ModelRc::new(VecModel::from(invites)));
            }
        });
    }
    
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
            let window = self.window.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(window) = window.upgrade() {
                    let app_state = window.global::<AppState>();
                    app_state.set_generated_address(address.into());
                }
            });
        }
    }
    
    async fn set_group_public_key(&self, _key: Option<String>) {}
    
    async fn add_signing_request(&self, request: PendingSigningRequest) {
        let window = self.window.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window.upgrade() {
                let _app_state = window.global::<AppState>();
                // Note: This UI doesn't have pending_signing_requests property
                // Just log it for now
                let _ = request;
            }
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
        let window = self.window.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window.upgrade() {
                let app_state = window.global::<AppState>();
                
                // Get current logs
                let current_logs = app_state.get_log_messages();
                let mut logs: Vec<slint::SharedString> = Vec::new();
                
                // Convert ModelRc to Vec
                for i in 0..current_logs.row_count() {
                    if let Some(log) = current_logs.row_data(i) {
                        logs.push(log);
                    }
                }
                
                // Add new message
                logs.push(message.into());
                
                // Keep only last 100
                if logs.len() > 100 {
                    logs.drain(0..logs.len() - 100);
                }
                
                // Update
                app_state.set_log_messages(ModelRc::new(slint::VecModel::from(logs)));
            }
        });
    }
    
    async fn set_logs(&self, logs: Vec<String>) {
        let window = self.window.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window.upgrade() {
                let app_state = window.global::<AppState>();
                let messages: Vec<slint::SharedString> = logs.into_iter()
                    .map(|s| s.into())
                    .collect();
                app_state.set_log_messages(ModelRc::new(slint::VecModel::from(messages)));
            }
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
    app_state.set_log_messages(ModelRc::new(slint::VecModel::from(Vec::<slint::SharedString>::new())));
    app_state.set_session_invites(ModelRc::new(VecModel::from(Vec::<SessionInvite>::new())));
    
    let ui_weak = ui.as_weak();
    
    // Create simple UI provider with window reference
    let ui_provider = Arc::new(SimpleUIProvider::new(ui_weak.clone()));
    
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
        let ui_provider = ui_provider.clone();
        ui.on_connect_websocket(move |device_id| {
            info!("Connect button clicked with device ID: {}", device_id);
            let device_id_str = device_id.to_string();
            let tx = tx.clone();
            let provider = ui_provider.clone();
            
            tokio::spawn(async move {
                // Update UI immediately
                provider.set_device_id(device_id_str.clone()).await;
                provider.add_log(format!("Connecting with device ID: {}", device_id_str)).await;
                
                // Send command
                let _ = tx.send(InternalCommand::SendToServer(
                    webrtc_signal_server::ClientMsg::Register { device_id: device_id_str }
                ));
            });
        });
    }
    
    {
        let tx = cmd_sender.clone();
        ui.on_create_session(move |session_id, participants, threshold| {
            info!("Create session button clicked");
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::ProposeSession {
                    session_id: session_id.to_string(),
                    total: participants as u16,
                    threshold: threshold as u16,
                    participants: vec![], // This will be populated by the server
                });
            });
        });
    }

    {
        let tx = cmd_sender.clone();
        ui.on_accept_session(move |session_id| {
            info!("Accept session button clicked");
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::AcceptSessionProposal(session_id.to_string()));
            });
        });
    }

    {
        let tx = cmd_sender.clone();
        ui.on_reject_session(move |session_id| {
            info!("Reject session button clicked");
            let tx = tx.clone();
            // For now, just remove the invite from the UI
            // In the future, this could send a SessionResponse with accepted: false
            let _ = tx.send(InternalCommand::ProcessSessionResponse {
                from_device_id: "".to_string(), // Not needed for our own rejection
                response: tui_node::protocal::signal::SessionResponse {
                    session_id: session_id.to_string(),
                    accepted: false,
                    wallet_status: None,
                },
            });
        });
    }
    
    {
        let tx = cmd_sender.clone();
        ui.on_start_dkg(move || {
            info!("Start DKG button clicked");
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(InternalCommand::TriggerDkgRound1);
            });
        });
    }
    
    // Note: main_simple.slint doesn't have initiate_signing callback
    
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
