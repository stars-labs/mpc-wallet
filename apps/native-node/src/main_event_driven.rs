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
use tokio::sync::mpsc;

slint::include_modules!();

/// Event types for UI updates
#[derive(Clone, Debug)]
enum UIEvent {
    ConnectionStatusChanged(bool),
    DeviceIdChanged(String),
    SessionStatusChanged(String),
    AddressGenerated(String),
    LogAdded(String),
    SigningRequestAdded(PendingSigningRequest),
    Error(String),
}

/// Event-driven UI provider that sends events directly to UI
struct EventDrivenUIProvider {
    event_tx: mpsc::UnboundedSender<UIEvent>,
}

impl EventDrivenUIProvider {
    fn new() -> (Self, mpsc::UnboundedReceiver<UIEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        (Self { event_tx }, event_rx)
    }
    
    fn send_event(&self, event: UIEvent) {
        let _ = self.event_tx.send(event);
    }
}

#[async_trait]
impl UIProvider for EventDrivenUIProvider {
    async fn set_connection_status(&self, connected: bool) {
        info!("UIProvider: Setting connection status to {}", connected);
        self.send_event(UIEvent::ConnectionStatusChanged(connected));
    }
    
    async fn set_device_id(&self, device_id: String) {
        self.send_event(UIEvent::DeviceIdChanged(device_id));
    }
    
    async fn update_device_list(&self, _devices: Vec<String>) {
        // Could add DeviceListUpdated event
    }
    
    async fn update_device_status(&self, _device_id: String, _status: String) {
        // Could add DeviceStatusUpdated event
    }
    
    async fn update_session_status(&self, status: String) {
        self.send_event(UIEvent::SessionStatusChanged(status));
    }
    
    async fn add_session_invite(&self, _invite: SessionInfo) {
        // Could add SessionInviteAdded event
    }
    
    async fn remove_session_invite(&self, _session_id: String) {
        // Could add SessionInviteRemoved event
    }
    
    async fn set_active_session(&self, session: Option<SessionInfo>) {
        let status = session
            .map(|s| format!("{} ({}/{})", s.session_id, s.participants.len(), s.total))
            .unwrap_or_else(|| "No active session".to_string());
        self.send_event(UIEvent::SessionStatusChanged(status));
    }
    
    async fn update_dkg_status(&self, status: String) {
        self.send_event(UIEvent::SessionStatusChanged(format!("DKG: {}", status)));
    }
    
    async fn set_generated_address(&self, address: Option<String>) {
        if let Some(addr) = address {
            self.send_event(UIEvent::AddressGenerated(addr));
        }
    }
    
    async fn set_group_public_key(&self, _key: Option<String>) {
        // Could add GroupPublicKeySet event
    }
    
    async fn add_signing_request(&self, request: PendingSigningRequest) {
        self.send_event(UIEvent::SigningRequestAdded(request));
    }
    
    async fn remove_signing_request(&self, _signing_id: String) {
        // Could add SigningRequestRemoved event
    }
    
    async fn update_signing_status(&self, status: String) {
        self.send_event(UIEvent::SessionStatusChanged(format!("Signing: {}", status)));
    }
    
    async fn set_signature_result(&self, _signing_id: String, _signature: Vec<u8>) {
        // Could add SignatureResult event
    }
    
    async fn update_wallet_list(&self, _wallets: Vec<String>) {
        // Could add WalletListUpdated event
    }
    
    async fn set_selected_wallet(&self, _wallet_id: Option<String>) {
        // Could add SelectedWalletChanged event
    }
    
    async fn add_log(&self, message: String) {
        self.send_event(UIEvent::LogAdded(message));
    }
    
    async fn set_logs(&self, logs: Vec<String>) {
        for log in logs {
            self.send_event(UIEvent::LogAdded(log));
        }
    }
    
    async fn update_mesh_status(&self, ready_devices: usize, total_devices: usize) {
        self.send_event(UIEvent::LogAdded(format!("Mesh: {}/{} ready", ready_devices, total_devices)));
    }
    
    async fn show_error(&self, error: String) {
        self.send_event(UIEvent::Error(error));
    }
    
    async fn show_success(&self, message: String) {
        self.send_event(UIEvent::LogAdded(format!("SUCCESS: {}", message)));
    }
    
    async fn set_busy(&self, _busy: bool) {
        // Could add BusyStateChanged event
    }
    
    async fn set_progress(&self, _progress: Option<f32>) {
        // Could add ProgressUpdated event
    }
}

/// Process UI events and update UI directly
async fn process_ui_events(
    ui_handle: slint::Weak<MainWindow>,
    mut event_rx: mpsc::UnboundedReceiver<UIEvent>,
) {
    let mut logs: Vec<String> = Vec::new();
    let mut signing_requests: Vec<PendingSigningRequest> = Vec::new();
    
    while let Some(event) = event_rx.recv().await {
        if let Some(ui) = ui_handle.upgrade() {
            match event {
                UIEvent::ConnectionStatusChanged(connected) => {
                    info!("UI Event: Connection status changed to {}", connected);
                    // Use invoke_from_event_loop to ensure we're on the UI thread
                    slint::invoke_from_event_loop(move || {
                        ui.set_websocket_connected(connected);
                        ui.window().request_redraw();
                    });
                }
                
                UIEvent::DeviceIdChanged(device_id) => {
                    slint::invoke_from_event_loop(move || {
                        ui.set_device_id(device_id.into());
                    });
                }
                
                UIEvent::SessionStatusChanged(status) => {
                    slint::invoke_from_event_loop(move || {
                        ui.set_session_status(status.into());
                    });
                }
                
                UIEvent::AddressGenerated(address) => {
                    slint::invoke_from_event_loop(move || {
                        ui.set_generated_address(address.into());
                    });
                }
                
                UIEvent::LogAdded(message) => {
                    logs.push(message);
                    if logs.len() > 1000 {
                        logs.drain(0..logs.len() - 1000);
                    }
                    
                    let log_messages: Vec<slint::SharedString> = logs
                        .iter()
                        .rev()
                        .take(100)
                        .map(|s| s.as_str().into())
                        .collect();
                    
                    slint::invoke_from_event_loop(move || {
                        ui.set_log_messages(ModelRc::new(slint::VecModel::from(log_messages)));
                    });
                }
                
                UIEvent::SigningRequestAdded(request) => {
                    signing_requests.push(request);
                    
                    let signing_model: Vec<slint::SharedString> = signing_requests
                        .iter()
                        .map(|req| format!("ID: {} from {}", req.signing_id, req.from_device).into())
                        .collect();
                    
                    slint::invoke_from_event_loop(move || {
                        ui.set_pending_signing_requests(ModelRc::new(slint::VecModel::from(signing_model)));
                    });
                }
                
                UIEvent::Error(error) => {
                    logs.push(format!("ERROR: {}", error));
                    let log_messages: Vec<slint::SharedString> = logs
                        .iter()
                        .rev()
                        .take(100)
                        .map(|s| s.as_str().into())
                        .collect();
                    
                    slint::invoke_from_event_loop(move || {
                        ui.set_log_messages(ModelRc::new(slint::VecModel::from(log_messages)));
                    });
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("Starting MPC Wallet Native Node (Event-Driven)");
    
    // Create UI
    let ui = MainWindow::new()?;
    let ui_weak = ui.as_weak();
    
    // Create event-driven UI provider
    let (ui_provider, event_rx) = EventDrivenUIProvider::new();
    let ui_provider = Arc::new(ui_provider);
    
    // Start event processor
    tokio::spawn(process_ui_events(ui_weak.clone(), event_rx));
    
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
            let device_id = device_id.to_string();
            let tx = tx.clone();
            let ui_provider = ui_provider.clone();
            
            info!("UI: Connect button clicked with device ID: {}", device_id);
            
            // Immediately update device ID
            let provider = ui_provider.clone();
            let id = device_id.clone();
            tokio::spawn(async move {
                provider.set_device_id(id).await;
            });
            
            // Send connect command
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
    
    {
        let tx = cmd_sender.clone();
        ui.on_accept_signing(move |request_info| {
            let tx = tx.clone();
            let request_str = request_info.to_string();
            if let Some(id_start) = request_str.find("ID: ") {
                if let Some(from_start) = request_str.find(" from ") {
                    let signing_id = request_str[id_start + 4..from_start].to_string();
                    tokio::spawn(async move {
                        let _ = tx.send(InternalCommand::AcceptSigning { signing_id });
                    });
                }
            }
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

// This event-driven implementation:
//
// 1. **Direct Event Flow**: State changes → Events → UI updates
// 2. **No Polling**: Removes timer-based approach completely
// 3. **Thread Safety**: Uses invoke_from_event_loop for UI updates
// 4. **Immediate Updates**: Events trigger UI changes immediately
// 5. **Explicit Logging**: Logs all connection status changes
// 6. **Force Redraw**: Calls request_redraw() after critical updates