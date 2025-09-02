use anyhow::Result;
use async_trait::async_trait;
use tui_node::{AppRunner, UIProvider};
use tui_node::protocal::signal::SessionInfo;
use tui_node::utils::state::{PendingSigningRequest, InternalCommand};
use frost_secp256k1::Secp256K1Sha256;
use slint::{ModelRc, ComponentHandle};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;
use tokio::sync::mpsc;

slint::include_modules!();

/// Debug UI provider that logs everything
struct DebugUIProvider {
    ui_handle: slint::Weak<MainWindow>,
}

impl DebugUIProvider {
    fn new(ui_handle: slint::Weak<MainWindow>) -> Self {
        Self { ui_handle }
    }
}

#[async_trait]
impl UIProvider for DebugUIProvider {
    async fn set_connection_status(&self, connected: bool) {
        info!("=== UIProvider::set_connection_status called with: {} ===", connected);
        
        if let Some(ui) = self.ui_handle.upgrade() {
            info!("UI handle upgraded successfully");
            
            // Try multiple approaches to ensure update
            slint::invoke_from_event_loop(move || {
                info!("Inside event loop - setting websocket_connected to {}", connected);
                
                // Get current value first
                let current = ui.get_websocket_connected();
                info!("Current websocket_connected value: {}", current);
                
                // Set new value
                ui.set_websocket_connected(connected);
                
                // Verify it was set
                let new_value = ui.get_websocket_connected();
                info!("New websocket_connected value after set: {}", new_value);
                
                // Force redraw
                ui.window().request_redraw();
                info!("Requested redraw");
            });
        } else {
            info!("Failed to upgrade UI handle!");
        }
    }
    
    async fn set_device_id(&self, device_id: String) {
        info!("UIProvider::set_device_id called with: {}", device_id);
        if let Some(ui) = self.ui_handle.upgrade() {
            slint::invoke_from_event_loop(move || {
                ui.set_device_id(device_id.into());
            });
        }
    }
    
    async fn update_device_list(&self, devices: Vec<String>) {
        info!("UIProvider::update_device_list called with {} devices", devices.len());
    }
    
    async fn update_device_status(&self, device_id: String, status: String) {
        info!("UIProvider::update_device_status: {} -> {}", device_id, status);
    }
    
    async fn update_session_status(&self, status: String) {
        info!("UIProvider::update_session_status: {}", status);
        if let Some(ui) = self.ui_handle.upgrade() {
            slint::invoke_from_event_loop(move || {
                ui.set_session_status(status.into());
            });
        }
    }
    
    async fn add_session_invite(&self, invite: SessionInfo) {
        info!("UIProvider::add_session_invite: {}", invite.session_id);
    }
    
    async fn remove_session_invite(&self, session_id: String) {
        info!("UIProvider::remove_session_invite: {}", session_id);
    }
    
    async fn set_active_session(&self, session: Option<SessionInfo>) {
        info!("UIProvider::set_active_session: {:?}", session.as_ref().map(|s| &s.session_id));
    }
    
    async fn update_dkg_status(&self, status: String) {
        info!("UIProvider::update_dkg_status: {}", status);
    }
    
    async fn set_generated_address(&self, address: Option<String>) {
        info!("UIProvider::set_generated_address: {:?}", address);
        if let Some(addr) = address {
            if let Some(ui) = self.ui_handle.upgrade() {
                slint::invoke_from_event_loop(move || {
                    ui.set_generated_address(addr.into());
                });
            }
        }
    }
    
    async fn set_group_public_key(&self, key: Option<String>) {
        info!("UIProvider::set_group_public_key: {:?}", key);
    }
    
    async fn add_signing_request(&self, request: PendingSigningRequest) {
        info!("UIProvider::add_signing_request: {}", request.signing_id);
    }
    
    async fn remove_signing_request(&self, signing_id: String) {
        info!("UIProvider::remove_signing_request: {}", signing_id);
    }
    
    async fn update_signing_status(&self, status: String) {
        info!("UIProvider::update_signing_status: {}", status);
    }
    
    async fn set_signature_result(&self, signing_id: String, signature: Vec<u8>) {
        info!("UIProvider::set_signature_result: {} - {} bytes", signing_id, signature.len());
    }
    
    async fn update_wallet_list(&self, wallets: Vec<String>) {
        info!("UIProvider::update_wallet_list: {} wallets", wallets.len());
    }
    
    async fn set_selected_wallet(&self, wallet_id: Option<String>) {
        info!("UIProvider::set_selected_wallet: {:?}", wallet_id);
    }
    
    async fn add_log(&self, message: String) {
        info!("UIProvider::add_log: {}", message);
    }
    
    async fn set_logs(&self, logs: Vec<String>) {
        info!("UIProvider::set_logs: {} logs", logs.len());
    }
    
    async fn update_mesh_status(&self, ready_devices: usize, total_devices: usize) {
        info!("UIProvider::update_mesh_status: {}/{}", ready_devices, total_devices);
    }
    
    async fn show_error(&self, error: String) {
        info!("UIProvider::show_error: {}", error);
    }
    
    async fn show_success(&self, message: String) {
        info!("UIProvider::show_success: {}", message);
    }
    
    async fn set_busy(&self, busy: bool) {
        info!("UIProvider::set_busy: {}", busy);
    }
    
    async fn set_progress(&self, progress: Option<f32>) {
        info!("UIProvider::set_progress: {:?}", progress);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with more detail
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .init();
    
    info!("Starting MPC Wallet Native Node (Debug Version)");
    
    // Create UI
    let ui = MainWindow::new()?;
    info!("UI created successfully");
    
    // Test direct UI update
    info!("Testing direct UI update...");
    ui.set_websocket_connected(false);
    info!("Initial websocket_connected: {}", ui.get_websocket_connected());
    
    let ui_weak = ui.as_weak();
    
    // Create debug UI provider
    let ui_provider = Arc::new(DebugUIProvider::new(ui_weak.clone()));
    
    // Test UI provider update before starting app runner
    info!("Testing UI provider update...");
    ui_provider.set_connection_status(true).await;
    
    // Create app runner
    let app_runner = AppRunner::<Secp256K1Sha256>::new(
        "wss://auto-life.tech".to_string(),
        ui_provider.clone(),
    );
    
    // Get command sender
    let cmd_sender = app_runner.get_command_sender();
    
    // Setup connect button with extensive logging
    {
        let tx = cmd_sender.clone();
        let ui_provider = ui_provider.clone();
        ui.on_connect_websocket(move |device_id| {
            info!("=== CONNECT BUTTON CLICKED ===");
            info!("Device ID: {}", device_id);
            
            let device_id = device_id.to_string();
            let tx = tx.clone();
            
            tokio::spawn(async move {
                info!("Sending Register command...");
                let _ = tx.send(InternalCommand::SendToServer(
                    webrtc_signal_server::ClientMsg::Register { device_id }
                ));
            });
        });
    }
    
    // Add debug button to manually test connection status
    {
        let ui_provider = ui_provider.clone();
        ui.on_start_dkg(move || {
            info!("=== DKG BUTTON USED AS DEBUG TEST ===");
            let provider = ui_provider.clone();
            tokio::spawn(async move {
                info!("Manually setting connection status to true...");
                provider.set_connection_status(true).await;
            });
        });
    }
    
    // Run the app logic
    tokio::spawn(async move {
        info!("Starting app runner...");
        if let Err(e) = app_runner.run().await {
            eprintln!("App runner error: {}", e);
        }
    });
    
    info!("MPC Wallet Native Node started - entering UI event loop");
    
    // Run UI event loop
    ui.run()?;
    
    Ok(())
}