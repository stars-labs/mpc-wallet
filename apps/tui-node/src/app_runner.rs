use crate::ui::provider::UIProvider;
use crate::utils::appstate_compat::AppState;
use crate::utils::state::{InternalCommand, MeshStatus, DkgStateDisplay};
use crate::handlers::{dkg_commands, signing_commands};
use crate::protocal::signal::WebSocketMessage;
use webrtc_signal_server::{ClientMsg, ServerMsg};
use frost_core::Ciphersuite;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, Mutex};
use anyhow::Result;
use tracing::{info, error};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::SplitSink;
use webrtc::peer_connection::RTCPeerConnection;

/// Core application runner that handles all business logic
/// UI-agnostic - works with any UIProvider implementation
pub struct AppRunner<C: Ciphersuite> {
    app_state: Arc<Mutex<AppState<C>>>,
    ui_provider: Arc<dyn UIProvider>,
    websocket_url: String,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    internal_cmd_rx: Option<mpsc::UnboundedReceiver<InternalCommand<C>>>,
    #[allow(dead_code)]
    device_connections: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    ws_sink: Arc<Mutex<Option<SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, WsMessage>>>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    shutdown_rx: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl<C: Ciphersuite + Send + Sync + 'static> AppRunner<C> 
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    pub fn new(
        websocket_url: String,
        ui_provider: Arc<dyn UIProvider>,
    ) -> Self {
        let (internal_cmd_tx, internal_cmd_rx) = mpsc::unbounded_channel();
        let app_state = Arc::new(Mutex::new(AppState::new()));
        let device_connections = Arc::new(Mutex::new(HashMap::new()));
        let ws_sink = Arc::new(Mutex::new(None));
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        
        Self {
            app_state,
            ui_provider,
            websocket_url,
            internal_cmd_tx,
            internal_cmd_rx: Some(internal_cmd_rx),
            device_connections,
            ws_sink,
            shutdown_tx: Some(shutdown_tx),
            shutdown_rx: Some(shutdown_rx),
        }
    }
    
    /// Get a handle to send commands to the runner
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<InternalCommand<C>> {
        self.internal_cmd_tx.clone()
    }
    
    /// Get a reference to the application state (for testing and monitoring)
    pub fn get_app_state(&self) -> Arc<Mutex<AppState<C>>> {
        self.app_state.clone()
    }
    
    /// Shutdown the runner gracefully
    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
    
    /// Connect to WebSocket with given device ID
    pub async fn connect(&self, device_id: String) -> Result<()> {
        self.internal_cmd_tx.send(InternalCommand::SendToServer(
            ClientMsg::Register { device_id }
        )).map_err(|e| anyhow::anyhow!("Failed to send connect command: {}", e))?;
        Ok(())
    }
    
    /// Create a new session
    pub async fn create_session(
        &self,
        session_id: String,
        total: u16,
        threshold: u16,
        participants: Vec<String>,
    ) -> Result<()> {
        self.internal_cmd_tx.send(InternalCommand::ProposeSession {
            session_id,
            total,
            threshold,
            participants,
        }).map_err(|e| anyhow::anyhow!("Failed to send create session command: {}", e))?;
        Ok(())
    }
    
    /// Accept a session invitation
    pub async fn accept_session(&self, session_id: String) -> Result<()> {
        self.internal_cmd_tx.send(InternalCommand::AcceptSessionProposal(session_id))
            .map_err(|e| anyhow::anyhow!("Failed to send accept session command: {}", e))?;
        Ok(())
    }
    
    /// Start DKG process
    pub async fn start_dkg(&self) -> Result<()> {
        self.internal_cmd_tx.send(InternalCommand::TriggerDkgRound1)
            .map_err(|e| anyhow::anyhow!("Failed to send start DKG command: {}", e))?;
        Ok(())
    }
    
    /// Initiate signing
    pub async fn initiate_signing(
        &self,
        transaction_data: String,
        blockchain: String,
        chain_id: Option<u64>,
    ) -> Result<()> {
        self.internal_cmd_tx.send(InternalCommand::InitiateSigning {
            transaction_data,
            blockchain,
            chain_id,
        }).map_err(|e| anyhow::anyhow!("Failed to send initiate signing command: {}", e))?;
        Ok(())
    }
    
    /// Accept a signing request
    pub async fn accept_signing(&self, signing_id: String) -> Result<()> {
        let _ = self.internal_cmd_tx.send(InternalCommand::AcceptSigning { signing_id });
        Ok(())
    }
    
    /// Main run loop - handles all async operations
    pub async fn run(&mut self) -> Result<()> {
        let mut internal_cmd_rx = self.internal_cmd_rx.take()
            .ok_or_else(|| anyhow::anyhow!("Command receiver already taken"))?;
        
        let mut shutdown_rx = self.shutdown_rx.take()
            .ok_or_else(|| anyhow::anyhow!("Shutdown receiver already taken"))?;
        
        let _app_state = self.app_state.clone();
        let ui_provider = self.ui_provider.clone();
        let _websocket_url = self.websocket_url.clone();
        let _internal_cmd_tx = self.internal_cmd_tx.clone();
        
        // Main event loop
        let mut ws_stream: Option<futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>
        >> = None;
        let mut current_device_id = None;
        
        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = &mut shutdown_rx => {
                    info!("Shutdown signal received");
                    break;
                }
                // Handle internal commands
                Some(cmd) = internal_cmd_rx.recv() => {
                    self.handle_internal_command(
                        cmd,
                        &mut ws_stream,
                        &mut current_device_id,
                    ).await?;
                }
                
                // Handle WebSocket messages
                Some(msg) = async {
                    if let Some(stream) = ws_stream.as_mut() {
                        stream.next().await
                    } else {
                        None
                    }
                } => {
                    match msg {
                        Ok(WsMessage::Text(text)) => {
                            // Log incoming messages for debugging
                            tracing::debug!("📨 Received WebSocket message: {}", text);
                            if let Ok(server_msg) = serde_json::from_str::<ServerMsg>(&text) {
                                self.handle_server_message(server_msg).await?;
                            }
                        }
                        Ok(WsMessage::Close(_)) => {
                            info!("WebSocket closed");
                            ws_stream = None;
                            *self.ws_sink.lock().await = None;
                            ui_provider.set_connection_status(false).await;
                            ui_provider.add_log("WebSocket connection closed".to_string()).await;
                        }
                        Err(_e) => {
                            error!("WebSocket error: {}", _e);
                            ws_stream = None;
                            *self.ws_sink.lock().await = None;
                            ui_provider.set_connection_status(false).await;
                            ui_provider.add_log(format!("WebSocket error: {}", _e)).await;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Cleanup on shutdown
        if let Some(ref mut sink) = *self.ws_sink.lock().await {
            let _ = sink.close().await;
        }
        
        ui_provider.set_connection_status(false).await;
        ui_provider.add_log("AppRunner shutdown complete".to_string()).await;
        info!("AppRunner shutdown complete");
        
        Ok(())
    }
    
    /// Handle internal commands
    async fn handle_internal_command(
        &self,
        cmd: InternalCommand<C>,
        ws_stream: &mut Option<futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>
        >>,
        current_device_id: &mut Option<String>,
    ) -> Result<()> {
        match cmd {
            InternalCommand::SendToServer(msg) => {
                // Handle WebSocket connection for Register message
                if let ClientMsg::Register { device_id } = &msg {
                    // Update device ID
                    {
                        let mut state = self.app_state.lock().await;
                        state.device_id = device_id.clone();
                    }
                    self.ui_provider.set_device_id(device_id.clone()).await;
                    *current_device_id = Some(device_id.clone());
                    
                    // Connect to WebSocket
                    match connect_async(&self.websocket_url).await {
                        Ok((stream, _)) => {
                            let (mut sink, stream_part) = stream.split();
                            
                            // Send registration
                            if let Ok(msg_str) = serde_json::to_string(&msg) {
                                if sink.send(WsMessage::Text(msg_str.into())).await.is_ok() {
                                    // Store sink in shared field
                                    *self.ws_sink.lock().await = Some(sink);
                                    *ws_stream = Some(stream_part);
                                    
                                    self.ui_provider.set_connection_status(true).await;
                                    self.ui_provider.add_log("Connected to WebSocket server".to_string()).await;
                                    info!("Connected to WebSocket server");
                                    
                                    // Query server for any sessions we're part of (for rejoin after restart)
                                    let device_id = {
                                        let state = self.app_state.lock().await;
                                        state.device_id.clone()
                                    };
                                    
                                    tracing::info!("🔍 Querying server for active sessions for device '{}'", device_id);
                                    let query_msg = ClientMsg::QueryMyActiveSessions;
                                    let _ = self.internal_cmd_tx.send(InternalCommand::SendToServer(query_msg));
                                } else {
                                    self.ui_provider.show_error("Failed to send registration".to_string()).await;
                                }
                            }
                        }
                        Err(_e) => {
                            self.ui_provider.show_error(format!("Failed to connect: {}", _e)).await;
                            error!("Failed to connect: {}", _e);
                        }
                    }
                } else {
                    // Send other messages through stored sink
                    let mut sink_guard = self.ws_sink.lock().await;
                    if let Some(ref mut sink) = *sink_guard {
                        if let Ok(msg_str) = serde_json::to_string(&msg) {
                            match sink.send(WsMessage::Text(msg_str.into())).await {
                                Ok(_) => {
                                    // Only log session-related messages to reduce noise
                                    match &msg {
                                        ClientMsg::AnnounceSession { .. } => {
                                            tracing::info!("✅ Session announcement sent to network");
                                            self.ui_provider.add_log("Session announced to network".to_string()).await;
                                        }
                                        ClientMsg::Relay { .. } => {
                                            tracing::info!("📤 Relay message sent");
                                        }
                                        _ => {} // Less verbose for other messages
                                    }
                                }
                                Err(_e) => {
                                    tracing::error!("❌ Failed to send WebSocket message: {}", _e);
                                    self.ui_provider.show_error(format!("Failed to send message: {}", _e)).await;
                                }
                            }
                        }
                    } else {
                        tracing::error!("❌ No WebSocket connection available to send message!");
                        self.ui_provider.show_error("No WebSocket connection available".to_string()).await;
                    }
                }
            }
            
            InternalCommand::ProposeSession { session_id, total, threshold, participants } => {
                let device_id = {
                    let state = self.app_state.lock().await;
                    state.device_id.clone()
                };
                // Use the real session handler
                crate::handlers::session_handler::handle_propose_session(
                    session_id, total, threshold, participants,
                    self.app_state.clone(), self.internal_cmd_tx.clone(),
                    device_id
                ).await;
            }
            
            InternalCommand::AcceptSessionProposal(session_id) => {
                // Use the real session handler
                crate::handlers::session_handler::handle_accept_session_proposal(
                    session_id, self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::TriggerDkgRound1 => {
                let device_id = {
                    let state = self.app_state.lock().await;
                    state.device_id.clone()
                };
                dkg_commands::handle_trigger_dkg_round1(
                    self.app_state.clone(), 
                    device_id,
                    self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::InitiateSigning { transaction_data, blockchain, chain_id } => {
                signing_commands::handle_initiate_signing(
                    transaction_data, blockchain, chain_id,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::AcceptSigning { signing_id } => {
                signing_commands::handle_accept_signing(
                    signing_id, self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessSessionResponse { from_device_id, response } => {
                tracing::info!("🔄 APP_RUNNER: Received ProcessSessionResponse from {}", from_device_id);
                crate::handlers::session_handler::handle_process_session_response(
                    from_device_id, response,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::InitiateWebRTCConnections => {
                // Light debounce - just prevent rapid successive calls
                {
                    let state_guard = self.app_state.lock().await;
                    if state_guard.webrtc_initiation_in_progress {
                        if let Some(started_at) = state_guard.webrtc_initiation_started_at {
                            if started_at.elapsed() < std::time::Duration::from_millis(500) {
                                tracing::info!("⏱️ Debouncing InitiateWebRTCConnections - too soon ({}ms ago)", 
                                    started_at.elapsed().as_millis());
                                return Ok(());
                            }
                        }
                    }
                }
                
                // CRITICAL FIX: Use accepted_devices instead of participants
                // accepted_devices is kept up-to-date via SessionUpdate messages
                // participants list becomes stale when joiners create sessions from old invites
                let (accepted_devices, device_id) = {
                    let state_guard = self.app_state.lock().await;
                    if let Some(ref session) = state_guard.session {
                        (session.accepted_devices.clone(), state_guard.device_id.clone())
                    } else {
                        (Vec::new(), String::new())
                    }
                };
                
                // CRITICAL: Filter out self from accepted_devices to avoid self-connection attempts
                let other_devices: Vec<String> = accepted_devices
                    .into_iter()
                    .filter(|dev| *dev != device_id)
                    .collect();
                
                tracing::info!("🚀 InitiateWebRTCConnections: other_devices={:?}, device_id={}", 
                    other_devices, device_id);
                
                if !other_devices.is_empty() {
                    tracing::info!("📞 Calling initiate_webrtc_connections with {} OTHER devices (excluding self)", other_devices.len());
                    
                    crate::network::webrtc::initiate_webrtc_connections(
                        other_devices,
                        device_id,
                        self.app_state.clone(),
                        self.internal_cmd_tx.clone(),
                    ).await;
                    
                    tracing::info!("✅ Returned from initiate_webrtc_connections");
                } else {
                    tracing::warn!("⚠️  InitiateWebRTCConnections: No accepted devices found in session");
                }
            }
            
            InternalCommand::ProcessDkgRound1 { from_device_id, package } => {
                dkg_commands::handle_process_dkg_round1(
                    from_device_id, package,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessDkgRound2 { from_device_id, package } => {
                dkg_commands::handle_process_dkg_round2(
                    from_device_id, package,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessMeshReady { device_id } => {
                crate::handlers::mesh_commands::handle_process_mesh_ready(
                    device_id,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessSigningRequest { from_device_id, signing_id, transaction_data, timestamp, blockchain, chain_id } => {
                signing_commands::handle_process_signing_request(
                    from_device_id, signing_id, transaction_data, timestamp, blockchain, chain_id,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessSigningAcceptance { from_device_id, signing_id, timestamp } => {
                signing_commands::handle_process_signing_acceptance(
                    from_device_id, signing_id, timestamp,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessSignerSelection { from_device_id, signing_id, selected_signers } => {
                signing_commands::handle_process_signer_selection(
                    from_device_id, signing_id, selected_signers,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessSigningCommitment { from_device_id, signing_id, commitment } => {
                signing_commands::handle_process_signing_commitment(
                    from_device_id, signing_id, commitment,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessSignatureShare { from_device_id, signing_id, share } => {
                signing_commands::handle_process_signature_share(
                    from_device_id, signing_id, share,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ProcessAggregatedSignature { from_device_id, signing_id, signature } => {
                signing_commands::handle_process_aggregated_signature(
                    from_device_id, signing_id, signature,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::CheckAndTriggerDkg => {
                dkg_commands::handle_check_and_trigger_dkg(
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::TriggerDkgRound2 => {
                let device_id = {
                    let state = self.app_state.lock().await;
                    state.device_id.clone()
                };
                dkg_commands::handle_trigger_dkg_round2(
                    self.app_state.clone(),
                    device_id
                ).await;
            }
            
            InternalCommand::FinalizeDkg => {
                dkg_commands::handle_finalize_dkg(
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::InitiateFrostRound1 { signing_id, transaction_data, selected_signers } => {
                signing_commands::handle_initiate_frost_round1(
                    signing_id, transaction_data, selected_signers,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::SendOwnMeshReadySignal => {
                crate::handlers::mesh_commands::handle_send_own_mesh_ready_signal(
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::ReportChannelOpen { device_id } => {
                tracing::info!("🚨 APP_RUNNER: Processing ReportChannelOpen for {}", device_id);
                // Call the proper mesh signaling logic
                let self_device_id = {
                    let state = self.app_state.lock().await;
                    state.device_id.clone()
                };
                
                tracing::info!("🚨 APP_RUNNER: Calling handle_report_channel_open for {}", device_id);
                crate::handlers::mesh_commands::handle_report_channel_open(
                    device_id.clone(),
                    self.app_state.clone(),
                    self.internal_cmd_tx.clone(),
                    self_device_id,
                ).await;
                tracing::info!("🚨 APP_RUNNER: Finished handle_report_channel_open for {}", device_id);
                
                self.ui_provider.add_log(format!("Channel opened with {}", device_id)).await;
            }
            
            InternalCommand::SendDirect { to, message } => {
                crate::handlers::send_commands::handle_send_direct(
                    to, message,
                    self.app_state.clone()
                ).await;
            }
            
            // Keystore commands
            InternalCommand::InitKeystore { path, device_name } => {
                crate::handlers::keystore_commands::handle_init_keystore(
                    path, device_name,
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::ListWallets => {
                crate::handlers::keystore_commands::handle_list_wallets(
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::CreateWallet { name, description, password, tags } => {
                crate::handlers::keystore_commands::handle_create_wallet(
                    name, description, password, tags,
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::LocateWallet { wallet_id } => {
                crate::handlers::keystore_commands::handle_locate_wallet(
                    wallet_id,
                    self.app_state.clone()
                ).await;
            }
            
            // --- Offline Mode Commands ---
            InternalCommand::OfflineMode { enabled } => {
                crate::handlers::offline_commands::handle_offline_mode(
                    enabled,
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::CreateSigningRequest { wallet_id, message, transaction_hex } => {
                crate::handlers::offline_commands::handle_create_signing_request(
                    wallet_id, message, transaction_hex,
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::ExportSigningRequest { session_id, output_path } => {
                crate::handlers::offline_commands::handle_export_signing_request(
                    session_id, output_path,
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::ImportSigningRequest { input_path } => {
                crate::handlers::offline_commands::handle_import_signing_request(
                    input_path,
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::ReviewSigningRequest { session_id } => {
                crate::handlers::offline_commands::handle_review_signing_request(
                    session_id,
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::ListOfflineSessions => {
                crate::handlers::offline_commands::handle_list_offline_sessions(
                    self.app_state.clone()
                ).await;
            }
            
            InternalCommand::SetSession(session_info) => {
                // Update the shared AppState with the session
                let mut state_guard = self.app_state.lock().await;
                state_guard.session = Some(session_info.clone());
                // Session state updated
                self.ui_provider.add_log(format!(
                    "📋 Session state updated: {} ({}/{})",
                    session_info.session_id,
                    session_info.threshold,
                    session_info.total
                )).await;
            }
            
            // --- Wallet Creation Commands ---
            InternalCommand::CreateWalletSession { config } => {
                tracing::info!("🎯 Creating wallet session: {}", config.wallet_name);
                
                let device_id = {
                    let state = self.app_state.lock().await;
                    state.device_id.clone()
                };
                
                match crate::handlers::session_handler::handle_propose_wallet_session(
                    config.clone(),
                    self.app_state.clone(),
                    self.internal_cmd_tx.clone(),
                    device_id.clone(),
                ).await {
                    Ok(_) => {
                        tracing::info!("✅ Wallet session creation completed successfully");
                        self.ui_provider.add_log(format!("Wallet '{}' session created", config.wallet_name)).await;
                    }
                    Err(_e) => {
                        tracing::error!("❌ Wallet session creation failed: {}", _e);
                        let _ = self.ui_provider.show_error(format!("Failed to create wallet session: {}", _e));
                    }
                }
            }
            
            InternalCommand::StartParticipantDiscovery { session_id, required_participants } => {
                crate::handlers::session_handler::handle_start_participant_discovery(
                    session_id,
                    required_participants,
                    self.app_state.clone(),
                    self.internal_cmd_tx.clone(),
                ).await;
            }
            
            InternalCommand::UpdateProgress { progress } => {
                let mut state = self.app_state.lock().await;
                state.wallet_creation_progress = Some(progress.clone());
                drop(state);
                
                self.ui_provider.add_log(format!(
                    "Progress: {} - {}",
                    progress.stage.to_string(),
                    progress.message
                )).await;
            }
            
            InternalCommand::SetDkgMode(mode) => {
                let mut state = self.app_state.lock().await;
                state.dkg_mode = Some(mode);
                drop(state);
                
                self.ui_provider.add_log("DKG mode set".to_string()).await;
            }
            
            InternalCommand::DiscoverSessions => {
                crate::handlers::session_handler::handle_session_discovery(
                    self.app_state.clone(),
                    self.internal_cmd_tx.clone(),
                ).await.unwrap_or_else(|e| {
                    let _ = self.ui_provider.show_error(format!("Session discovery failed: {}", e));
                    Vec::new()
                });
            }
            
            InternalCommand::ProcessSessionAnnouncement { announcement } => {
                let mut state = self.app_state.lock().await;
                // Add to available sessions if not already present
                if !state.available_sessions.iter().any(|s| s.session_code == announcement.session_code) {
                    state.available_sessions.push(announcement.clone());
                    // Session available
                }
                drop(state);
                
                self.ui_provider.add_log(format!(
                    "📢 Session available: {}",
                    announcement.session_code
                )).await;
            }
            
            InternalCommand::CompleteWalletCreation { wallet_id, addresses } => {
                let mut state = self.app_state.lock().await;
                state.current_wallet_id = Some(wallet_id.clone());
                state.blockchain_addresses = addresses.clone();
                state.wallet_creation_progress = None;
                drop(state);
                
                self.ui_provider.add_log(format!(
                    "✅ Wallet created: {}",
                    wallet_id
                )).await;
                
                for addr in addresses {
                    self.ui_provider.add_log(format!(
                        "  {} address: {}",
                        addr.blockchain,
                        addr.address
                    )).await;
                }
            }
            
            InternalCommand::ProcessJoinRequest { from_device: _, session_id, device_id, is_rejoin } => {
                let state_guard = self.app_state.lock().await;
                
                // Check if we can handle this join request
                let can_handle = state_guard.session.as_ref()
                    .map(|s| {
                        s.session_id == session_id && 
                        s.accepted_devices.contains(&state_guard.device_id)
                    })
                    .unwrap_or(false);
                
                if can_handle {
                    let session = state_guard.session.as_ref().unwrap();
                    let is_original_creator = session.proposer_id == state_guard.device_id;
                    
                    tracing::info!("📨 {} handling join request from {} for session {}",
                        if is_original_creator { "Creator" } else { "Participant" },
                        device_id, session_id
                    );
                    
                    // Create session proposal
                    let proposal = crate::protocal::signal::SessionProposal {
                        session_id: session.session_id.clone(),
                        total: session.total,
                        threshold: session.threshold,
                        participants: session.participants.clone(),
                        session_type: session.session_type.clone(),
                        proposer_device_id: session.proposer_id.clone(),
                        curve_type: session.curve_type.clone(),
                        coordination_type: session.coordination_type.clone(),
                    };
                    
                    drop(state_guard);
                    
                    // Send proposal back to the requester
                    let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionProposal(proposal);
                    if let Ok(json_data) = serde_json::to_value(&websocket_msg) {
                        let _ = self.internal_cmd_tx.send(InternalCommand::SendToServer(
                            webrtc_signal_server::ClientMsg::Relay {
                                to: device_id.clone(),
                                data: json_data,
                            }
                        ));
                        
                        // Update session participants if it's a rejoin
                        if is_rejoin {
                            let mut state = self.app_state.lock().await;
                            if let Some(ref mut session) = state.session {
                                // Ensure device is in participants list
                                if !session.participants.contains(&device_id) {
                                    session.participants.push(device_id.clone());
                                }
                                // Remove from accepted_devices (they'll re-accept)
                                session.accepted_devices.retain(|d| d != &device_id);
                            }
                        }
                        
                        self.ui_provider.add_log(format!("📤 Sent session proposal to {}", device_id)).await;
                    } else {
                        tracing::error!("Failed to serialize session proposal");
                    }
                } else {
                    tracing::warn!("Cannot handle join request for session {} - not a participant", session_id);
                }
            }
            
            InternalCommand::JoinSession(session_id) => {
                let mut state = self.app_state.lock().await;
                state.joining_session_id = Some(session_id.clone());
                let device_id = state.device_id.clone();
                
                // Check if we already have the session (rejoin scenario)
                let is_rejoin = state.session.as_ref()
                    .map(|s| s.session_id == session_id)
                    .unwrap_or(false);
                
                // Debug logging to understand what's in available_sessions
                tracing::info!("🔍 JoinSession: Looking for session {} in {} available sessions and {} invites",
                    session_id, state.available_sessions.len(), state.invites.len());
                for (i, session) in state.available_sessions.iter().enumerate() {
                    tracing::info!("  [{i}] session_code: '{}', creator: '{}'", 
                        session.session_code, session.creator_device);
                }
                
                // Get the proposer ID from available sessions or invites
                let proposer_id = if let Some(announcement) = state.available_sessions.iter()
                    .find(|a| a.session_code == session_id) {
                    tracing::info!("✅ Found session in available_sessions, proposer: {}", 
                        announcement.creator_device);
                    Some(announcement.creator_device.clone())
                } else if let Some(invite) = state.invites.iter()
                    .find(|i| i.session_id == session_id) {
                    tracing::info!("✅ Found session in invites, proposer: {}", 
                        invite.proposer_id);
                    Some(invite.proposer_id.clone())
                } else {
                    tracing::warn!("❌ Session {} not found in available_sessions or invites!", session_id);
                    None
                };
                
                drop(state);
                
                // Send a join request to the session creator
                if let Some(proposer) = proposer_id {
                    tracing::info!("📤 Sending join request for session {} to proposer {}", 
                        session_id, proposer);
                    
                    // Create properly wrapped join request
                    let join_request = crate::protocal::signal::SessionJoinRequest {
                        session_id: session_id.clone(),
                        device_id: device_id,
                        is_rejoin: is_rejoin,
                    };
                    
                    // Wrap in WebSocketMessage
                    let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionJoinRequest(join_request);
                    
                    if let Ok(json_data) = serde_json::to_value(&websocket_msg) {
                        let _ = self.internal_cmd_tx.send(InternalCommand::SendToServer(
                            webrtc_signal_server::ClientMsg::Relay {
                                to: proposer,
                                data: json_data,
                            }
                        ));
                    } else {
                        tracing::error!("Failed to serialize join request");
                    }
                } else {
                    tracing::warn!("⚠️ Cannot join session {}: proposer not found", session_id);
                    
                    // If we can't find a proposer, try to get session info from ANY online participant
                    // This handles the case where the original creator is gone
                    let state = self.app_state.lock().await;
                    
                    // Get list of online devices (excluding ourselves)
                    let online_devices: Vec<String> = state.devices.iter()
                        .filter(|d| **d != state.device_id)
                        .cloned()
                        .collect();
                    
                    drop(state);
                    
                    if !online_devices.is_empty() {
                        tracing::info!("🔄 Attempting to join through any online participant: {:?}", online_devices);
                        
                        // Send join request to the first online device
                        // They will forward or handle if they're part of the session
                        let target_device = online_devices[0].clone();
                        
                        let join_request = crate::protocal::signal::SessionJoinRequest {
                            session_id: session_id.clone(),
                            device_id: device_id,
                            is_rejoin: is_rejoin,
                        };
                        
                        let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionJoinRequest(join_request);
                        
                        if let Ok(json_data) = serde_json::to_value(&websocket_msg) {
                            tracing::info!("📤 Sending join request to {} (fallback)", target_device);
                            let _ = self.internal_cmd_tx.send(InternalCommand::SendToServer(
                                webrtc_signal_server::ClientMsg::Relay {
                                    to: target_device,
                                    data: json_data,
                                }
                            ));
                        }
                    }
                }
                
                // Also delegate to AcceptSessionProposal for the case where we already have an invite
                let _ = self.internal_cmd_tx.send(InternalCommand::AcceptSessionProposal(session_id));
            }
            
            InternalCommand::RetryDkg => {
                // Reset DKG state and retry
                let mut state = self.app_state.lock().await;
                state.dkg_state = crate::utils::state::DkgState::Idle;
                state.received_dkg_packages.clear();
                state.received_dkg_round2_packages.clear();
                drop(state);
                
                let _ = self.internal_cmd_tx.send(InternalCommand::TriggerDkgRound1);
                self.ui_provider.add_log("Retrying DKG...".to_string()).await;
            }
            
            InternalCommand::CancelDkg => {
                // Cancel ongoing DKG
                let mut state = self.app_state.lock().await;
                state.dkg_state = crate::utils::state::DkgState::Idle;
                state.wallet_creation_progress = None;
                state.wallet_creation_config = None;
                drop(state);
                
                self.ui_provider.add_log("DKG cancelled".to_string()).await;
            }
        }
        
        // Update UI after command processing
        self.update_ui_state().await?;
        
        Ok(())
    }
    
    /// Handle messages from server
    async fn handle_server_message(&self, msg: ServerMsg) -> Result<()> {
        use crate::network::webrtc::handle_webrtc_signal;
        
        // Log the message type for debugging
        tracing::info!("📥 Processing server message type: {}", match &msg {
            ServerMsg::Devices { .. } => "Devices",
            ServerMsg::Error { .. } => "Error",
            ServerMsg::Relay { .. } => "Relay",
            ServerMsg::SessionAvailable { .. } => "SessionAvailable",
            ServerMsg::SessionsForDevice { .. } => "SessionsForDevice",
            ServerMsg::SessionRemoved { .. } => "SessionRemoved",
            _ => "Other",
        });
        
        match msg {
            ServerMsg::Devices { devices } => {
                let mut state = self.app_state.lock().await;
                state.devices = devices.clone();
                drop(state);
                self.ui_provider.update_device_list(devices).await;
            }
            ServerMsg::Error { error } => {
                self.ui_provider.show_error(error.clone()).await;
                self.ui_provider.add_log(format!("Error: {}", error)).await;
            }
            ServerMsg::Relay { from, data } => {
                // Log the raw relay data for debugging
                tracing::info!("📨 Relay from {}: {}", from, serde_json::to_string(&data).unwrap_or_else(|_| "invalid".to_string()));
                self.ui_provider.add_log(format!("Relay from {}: {:?}", from, data)).await;
                
                // First try to parse as WebSocketMessage
                match serde_json::from_value::<WebSocketMessage>(data.clone()) {
                    Ok(WebSocketMessage::WebRTCSignal(signal)) => {
                        let device_id = {
                            let state = self.app_state.lock().await;
                            state.device_id.clone()
                        };
                        
                        // Get device_connections from app_state to ensure consistency
                        let device_connections = {
                            let state = self.app_state.lock().await;
                            state.device_connections.clone()
                        };
                        
                        handle_webrtc_signal(
                            from,
                            signal,
                            self.app_state.clone(),
                            device_id,
                            self.internal_cmd_tx.clone(),
                            device_connections,  // Use the one from AppState
                        ).await;
                    }
                    Ok(WebSocketMessage::SessionProposal(proposal)) => {
                        tracing::info!("✅ Parsed SessionProposal from {}: {}", from, proposal.session_id);
                        let invite_info = crate::protocal::signal::SessionInfo {
                            session_id: proposal.session_id.clone(),
                            proposer_id: from.clone(),
                            total: proposal.total,
                            threshold: proposal.threshold,
                            participants: proposal.participants.clone(),
                            accepted_devices: Vec::new(),
                            session_type: proposal.session_type.clone(),
                            curve_type: proposal.curve_type.clone(),
                            coordination_type: proposal.coordination_type.clone(),
                        };
                        
                        let mut state = self.app_state.lock().await;
                        
                        // Check if this is a rejoin scenario (we already have this session)
                        let is_rejoin = state.session.as_ref()
                            .map(|s| s.session_id == proposal.session_id)
                            .unwrap_or(false);
                        
                        // Check if we're actively trying to join this session
                        let is_actively_joining = state.joining_session_id.as_ref()
                            .map(|id| *id == proposal.session_id)
                            .unwrap_or(false);
                        
                        // SECURITY FIX: Never auto-join sessions without explicit user consent
                        // Only allow joining if:
                        // 1. We're rejoining our existing session after disconnect, OR
                        // 2. User explicitly requested to join this specific session
                        let should_auto_join = is_rejoin || is_actively_joining;
                        
                        // Log security decision
                        if !should_auto_join {
                            tracing::warn!("🔒 Security: Rejecting auto-join for session {} - requires user consent", proposal.session_id);
                        }
                        
                        if is_rejoin {
                            tracing::info!("📍 REJOIN DETECTED: Received SessionProposal for our existing session {}", 
                                proposal.session_id);
                            
                            // CRITICAL: Clean up WebRTC state before rejoin
                            // Close and clear old connections
                            {
                                let mut conns = state.device_connections.lock().await;
                                for (peer_id, conn) in conns.iter() {
                                    tracing::info!("🔌 Closing stale connection to {} before rejoin", peer_id);
                                    let _ = conn.close().await;
                                }
                                conns.clear();
                            }
                            
                            // Clear all WebRTC-related state
                            state.data_channels.clear();
                            state.device_statuses.clear();
                            state.pending_ice_candidates.clear();
                            state.making_offer.clear();
                            state.webrtc_initiation_in_progress = false;
                            state.webrtc_initiation_started_at = None;
                            
                            // Clear the old session state to prepare for rejoin
                            state.session = None;
                        }
                        
                        state.invites.push(invite_info.clone());
                        
                        // Also add to available sessions for UI discovery
                        let announcement = crate::protocal::signal::SessionAnnouncement {
                            session_code: proposal.session_id.clone(),
                            wallet_type: format!("{}/{} Threshold", proposal.threshold, proposal.total),
                            creator_device: from.clone(),
                            total: proposal.total,
                            threshold: proposal.threshold,
                            participants_joined: proposal.participants.len() as u16,
                            curve_type: proposal.curve_type.clone(),
                            description: Some(format!("DKG session for {} wallet", proposal.curve_type)),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        };
                        
                        // Update or add the session to available list
                        if let Some(existing) = state.available_sessions.iter_mut()
                            .find(|s| s.session_code == announcement.session_code) {
                            *existing = announcement.clone();
                        } else {
                            state.available_sessions.push(announcement.clone());
                        }
                        
                        drop(state);
                        
                        self.ui_provider.add_session_invite(invite_info).await;
                        self.ui_provider.add_log(format!(
                            "Received SessionProposal from {}: ID={}, Total={}, Threshold={}",
                            from, proposal.session_id, proposal.total, proposal.threshold
                        )).await;
                        
                        // Only join if authorized (rejoin or explicit user request)
                        if should_auto_join {
                            let join_reason = if is_rejoin {
                                "Rejoining existing session after disconnect"
                            } else if is_actively_joining {
                                "User-requested join"
                            } else {
                                "Unknown" // This should never happen with new logic
                            };
                            
                            tracing::info!("✅ Authorized join: {} for session {}", join_reason, proposal.session_id);
                            
                            self.ui_provider.add_log(format!(
                                "🔐 {}: {}",
                                join_reason, proposal.session_id
                            )).await;
                            
                            let _ = self.internal_cmd_tx.send(InternalCommand::AcceptSessionProposal(
                                proposal.session_id.clone()
                            ));
                        } else {
                            // Log that we received but didn't auto-join for security
                            self.ui_provider.add_log(format!(
                                "🔒 Session proposal received from {}: {} (requires user consent to join)",
                                from, proposal.session_id
                            )).await;
                        }
                    }
                    Ok(WebSocketMessage::SessionResponse(response)) => {
                        self.ui_provider.add_log(format!(
                            "Received SessionResponse from {}: {:?}",
                            from, response
                        )).await;
                        
                        let _ = self.internal_cmd_tx.send(InternalCommand::ProcessSessionResponse {
                            from_device_id: from,
                            response,
                        });
                    }
                    Ok(WebSocketMessage::SessionJoinRequest(request)) => {
                        tracing::info!("📥 Received SessionJoinRequest from {} for session {}", 
                            from, request.session_id);
                        
                        // Forward to internal command for processing
                        let _ = self.internal_cmd_tx.send(InternalCommand::ProcessJoinRequest {
                            from_device: from,
                            session_id: request.session_id,
                            device_id: request.device_id,
                            is_rejoin: request.is_rejoin,
                        });
                    }
                    Ok(WebSocketMessage::SessionUpdate(update)) => {
                        self.ui_provider.add_log(format!(
                            "📢 Received SessionUpdate from {}: {:?}",
                            from, update.update_type
                        )).await;
                        
                        // Update our session state with the new participant list
                        let (should_trigger_webrtc, accepted_devices_for_check) = {
                            let mut state = self.app_state.lock().await;
                            let mut should_trigger = false;
                            let mut accepted_devices = Vec::new();
                            
                            if let Some(ref mut session) = state.session {
                                if session.session_id == update.session_id {
                                    session.accepted_devices = update.accepted_devices.clone();
                                    
                                    // CRITICAL FIX: Also sync participants with accepted_devices
                                    // This ensures both lists contain the complete mesh topology
                                    session.participants = update.accepted_devices.clone();
                                    
                                    tracing::info!("✅ SessionUpdate synced: accepted_devices={:?}, participants={:?}", 
                                        session.accepted_devices, session.participants);
                                    
                                    self.ui_provider.add_log(format!(
                                        "✅ Updated session: {}/{} participants: {:?}",
                                        session.accepted_devices.len(),
                                        session.total,
                                        session.accepted_devices
                                    )).await;
                                    
                                    accepted_devices = session.accepted_devices.clone();
                                    
                                    // Check if we have connections to all participants
                                    let device_connections = state.device_connections.clone();
                                    let connections = device_connections.lock().await;
                                    let device_id = state.device_id.clone();
                                    
                                    // Check if any accepted device (except self) lacks a connection
                                    let needs_connections = accepted_devices.iter().any(|dev| {
                                        *dev != device_id && !connections.contains_key(dev)
                                    });
                                    
                                    should_trigger = needs_connections && accepted_devices.len() > 1;
                                }
                            }
                            (should_trigger, accepted_devices)
                        }; // Release state lock here
                        
                        if should_trigger_webrtc {
                            tracing::info!("🔄 Triggering WebRTC connections for NEW participants after SessionUpdate");
                            let _ = self.internal_cmd_tx.send(InternalCommand::InitiateWebRTCConnections);
                        } else if !accepted_devices_for_check.is_empty() {
                            tracing::info!("✅ All participants already have connections, skipping WebRTC trigger");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("❌ Failed to parse WebSocketMessage from {}: {}", from, e);
                        
                        // Try parsing as raw SessionAnnouncement (for backwards compatibility)
                        match serde_json::from_value::<crate::protocal::signal::SessionAnnouncement>(data.clone()) {
                            Ok(announcement) => {
                                tracing::info!("✅ Parsed raw SessionAnnouncement from {}: {}", from, announcement.session_code);
                                
                                let mut state = self.app_state.lock().await;
                                
                                // Don't process our own announcements
                                if announcement.creator_device == state.device_id {
                                    drop(state);
                                    return Ok(());
                                }
                                
                                // Update or add the session to available list
                                if let Some(existing) = state.available_sessions.iter_mut()
                                    .find(|s| s.session_code == announcement.session_code) {
                                    *existing = announcement.clone();
                                } else {
                                    state.available_sessions.push(announcement.clone());
                                }
                                
                                // Also add to invites for compatibility
                                let invite_info = crate::protocal::signal::SessionInfo {
                                    session_id: announcement.session_code.clone(),
                                    proposer_id: announcement.creator_device.clone(),
                                    total: announcement.total,
                                    threshold: announcement.threshold,
                                    participants: vec![announcement.creator_device.clone()],
                                    accepted_devices: Vec::new(),
                                    session_type: crate::protocal::signal::SessionType::DKG,
                                    curve_type: announcement.curve_type.clone(),
                                    coordination_type: "network".to_string(),
                                };
                                
                                if !state.invites.iter().any(|i| i.session_id == invite_info.session_id) {
                                    state.invites.push(invite_info.clone());
                                }
                                
                                drop(state);
                                
                                self.ui_provider.add_session_invite(invite_info).await;
                                self.ui_provider.add_log(format!(
                                    "📢 Session discovered: {} ({}/{} participants)",
                                    announcement.session_code,
                                    announcement.participants_joined,
                                    announcement.total
                                )).await;
                            }
                            Err(e2) => {
                                tracing::warn!("❌ Also failed to parse as SessionAnnouncement: {}", e2);
                                self.ui_provider.add_log(format!("Failed to parse relay message: {}", e)).await;
                            }
                        }
                    }
                    Ok(WebSocketMessage::SessionOffer(session_info)) => {
                        tracing::info!("Received session offer: {:?}", session_info);
                        // Handle session offer - could add to available sessions or auto-join
                    }
                    Ok(WebSocketMessage::SessionAccepted { device_id, session_id }) => {
                        tracing::info!("Session accepted by device {} for session {}", device_id, session_id);
                        // Handle session acceptance
                    }
                }
            }
            ServerMsg::SessionAvailable { session_info } => {
                // Handle session discovery announcement
                match serde_json::from_value::<crate::protocal::signal::SessionAnnouncement>(session_info.clone()) {
                    Ok(announcement) => {
                        let mut state = self.app_state.lock().await;
                        let device_id = state.device_id.clone();
                        
                        // Don't process our own announcements
                        if announcement.creator_device == device_id {
                            drop(state);
                            return Ok(());
                        }
                        
                        // Check if we're already in a session
                        let already_in_session = state.session.is_some();
                        
                        // Update or add the session to available list
                        if let Some(existing) = state.available_sessions.iter_mut()
                            .find(|s| s.session_code == announcement.session_code) {
                            *existing = announcement.clone();
                        } else {
                            state.available_sessions.push(announcement.clone());
                        }
                        
                        // Check if we should auto-join this session
                        let should_auto_join = !already_in_session && 
                            state.wallet_creation_config.is_none() && // Not already creating a wallet
                            announcement.participants_joined < announcement.total; // Still has room
                        
                        drop(state);
                        
                        self.ui_provider.add_log(format!(
                            "📢 Session available: {} ({}/{} participants)", 
                            announcement.session_code,
                            announcement.participants_joined,
                            announcement.total
                        )).await;
                        
                        // SECURITY: Never auto-join announced sessions
                        // Sessions require explicit user consent to prevent unauthorized P2P connections
                        if should_auto_join {
                            tracing::warn!("🔒 Security: Blocked auto-join of announced session {} - requires user consent", 
                                announcement.session_code);
                        }
                        
                        // Only show available sessions to user, don't auto-join
                        if !already_in_session && announcement.participants_joined < announcement.total {
                            self.ui_provider.add_log(format!(
                                "🔔 New session available: {} - Press 'j' to view and join sessions",
                                announcement.session_code
                            )).await;
                        }
                    }
                    Err(_e) => {
                        self.ui_provider.add_log(format!("Error parsing session announcement: {}", _e)).await;
                    }
                }
            }
            ServerMsg::SessionListRequest { from } => {
                // If we're a creator with an active session, respond with our session info
                let state = self.app_state.lock().await;
                if let Some(session) = &state.session {
                    if session.proposer_id == state.device_id {
                        // We are the creator, send our session announcement
                        let announcement = crate::protocal::signal::SessionAnnouncement {
                            session_code: session.session_id.clone(),
                            wallet_type: match &session.session_type {
                                crate::protocal::signal::SessionType::DKG => {
                                    format!("Wallet {}/{}", session.threshold, session.total)
                                }
                                crate::protocal::signal::SessionType::Signing { .. } => {
                                    "Signing Session".to_string()
                                }
                            },
                            threshold: session.threshold,
                            total: session.total,
                            curve_type: session.curve_type.clone(),
                            creator_device: state.device_id.clone(),
                            participants_joined: session.accepted_devices.len() as u16,
                            description: None,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        };
                        
                        // Send back to the requester
                        if let Ok(announcement_json) = serde_json::to_value(&announcement) {
                            let _ = self.internal_cmd_tx.send(
                                crate::utils::state::InternalCommand::SendToServer(
                                    webrtc_signal_server::ClientMsg::Relay {
                                        to: from,
                                        data: announcement_json,
                                    }
                                )
                            );
                        }
                    }
                }
            }
            ServerMsg::SessionsForDevice { sessions } => {
                // KISS: Server returns all sessions where we're a participant
                tracing::info!("📥 Received {} sessions from server", sessions.len());
                
                let device_id = {
                    let state = self.app_state.lock().await;
                    state.device_id.clone()
                };
                
                for session_json in &sessions {
                    // Try to parse as SessionInfo or SessionAnnouncement
                    if let Ok(session_info) = serde_json::from_value::<crate::protocal::signal::SessionInfo>(session_json.clone()) {
                        let session_id = session_info.session_id.clone();
                        
                        // Client-side logic: Check if we're in accepted_devices
                        let is_accepted = session_info.accepted_devices.contains(&device_id);
                        
                        if is_accepted {
                            tracing::info!("🔄 Found session '{}' where we were already accepted - auto-rejoin", session_id);
                            self.ui_provider.add_log(format!(
                                "✅ Rejoining session: {}",
                                session_id
                            )).await;
                            
                            // Store session and trigger rejoin
                            let mut state = self.app_state.lock().await;
                            state.session = Some(session_info);
                            drop(state);
                            
                            // Trigger rejoin
                            let _ = self.internal_cmd_tx.send(InternalCommand::JoinSession(session_id));
                        } else {
                            tracing::info!("📨 Found session '{}' where we're invited but not accepted", session_id);
                            self.ui_provider.add_log(format!(
                                "📨 Pending invitation: {} - Press 'j' to join",
                                session_id
                            )).await;
                            
                            // Add to invites
                            let mut state = self.app_state.lock().await;
                            state.invites.push(session_info);
                        }
                    } else if let Ok(announcement) = serde_json::from_value::<crate::protocal::signal::SessionAnnouncement>(session_json.clone()) {
                        // Handle as announcement
                        tracing::info!("📋 Found session announcement: {}", announcement.session_code);
                        let mut state = self.app_state.lock().await;
                        state.available_sessions.push(announcement);
                    }
                }
                
                if sessions.is_empty() {
                    tracing::info!("No previous sessions found for device '{}'", device_id);
                }
            }
            ServerMsg::SessionRemoved { session_id, reason } => {
                // Handle session removal notification
                tracing::warn!("🗑️ Session '{}' was removed: {}", session_id, reason);
                
                let mut state = self.app_state.lock().await;
                
                // Check if we're in this session
                if let Some(ref current_session) = state.session {
                    if current_session.session_id == session_id {
                        tracing::error!("⚠️ Our current session was removed!");
                        state.session = None;
                        state.dkg_state = crate::utils::state::DkgState::Idle;
                        
                        // Clean up WebRTC connections
                        let connections = state.device_connections.clone();
                        drop(state);
                        
                        let mut conns = connections.lock().await;
                        for (_, conn) in conns.drain() {
                            let _ = conn.close().await;
                        }
                        
                        self.ui_provider.show_error(format!(
                            "Session terminated: {}",
                            reason
                        )).await;
                        
                        self.ui_provider.add_log(format!(
                            "❌ Session '{}' ended - {}",
                            session_id, reason
                        )).await;
                    }
                } else {
                    // Remove from invites or available sessions
                    state.invites.retain(|invite| invite.session_id != session_id);
                    state.available_sessions.retain(|session| {
                        session.session_code != session_id
                    });
                    
                    self.ui_provider.add_log(format!(
                        "📢 Session '{}' no longer available - {}",
                        session_id, reason
                    )).await;
                }
            }
        }
        
        // Update UI after message processing
        self.update_ui_state().await?;
        
        Ok(())
    }
    
    /// Update UI with current state
    async fn update_ui_state(&self) -> Result<()> {
        let state = self.app_state.lock().await;
        
        // Update device list
        self.ui_provider.update_device_list(state.devices.clone()).await;
        
        // Update session status
        if let Some(session) = &state.session {
            self.ui_provider.set_active_session(Some(session.clone())).await;
            self.ui_provider.update_session_status(
                format!("{} ({}/{})", session.session_id, session.participants.len(), session.total)
            ).await;
        } else {
            self.ui_provider.set_active_session(None).await;
            self.ui_provider.update_session_status("No active session".to_string()).await;
        }
        
        // Update DKG status
        self.ui_provider.update_dkg_status(state.dkg_state.display_status()).await;
        
        // Update generated address
        if !state.blockchain_addresses.is_empty() {
            if let Some(addr) = state.blockchain_addresses.first() {
                self.ui_provider.set_generated_address(Some(addr.address.clone())).await;
            }
        }
        
        // Update mesh status
        match &state.mesh_status {
            MeshStatus::Ready => {
                let total = state.session.as_ref().map(|s| s.participants.len()).unwrap_or(0);
                self.ui_provider.update_mesh_status(total, total).await;
            }
            MeshStatus::PartiallyReady { ready_devices, total_devices } => {
                self.ui_provider.update_mesh_status(ready_devices.len(), *total_devices).await;
            }
            MeshStatus::WebRTCInitiated => {
                // WebRTC initiated but connections not fully established yet
                let total = state.session.as_ref().map(|s| s.participants.len()).unwrap_or(0);
                self.ui_provider.update_mesh_status(0, total).await;
            }
            MeshStatus::Incomplete => {
                self.ui_provider.update_mesh_status(0, 0).await;
            }
        }
        
        // Update signing requests
        for request in &state.pending_signing_requests {
            self.ui_provider.add_signing_request(request.clone()).await;
        }
        
        // Update signing status
        self.ui_provider.update_signing_status(state.signing_state.display_status()).await;
        
        Ok(())
    }
}