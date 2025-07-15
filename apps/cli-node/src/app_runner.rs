use crate::ui::provider::UIProvider;
use crate::utils::state::{AppState, InternalCommand, MeshStatus, DkgStateDisplay};
use crate::handlers::{dkg_commands, session_commands, signing_commands};
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
use webrtc::peer_connection::RTCPeerConnection;

/// Core application runner that handles all business logic
/// UI-agnostic - works with any UIProvider implementation
pub struct AppRunner<C: Ciphersuite> {
    app_state: Arc<Mutex<AppState<C>>>,
    ui_provider: Arc<dyn UIProvider>,
    websocket_url: String,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    internal_cmd_rx: Option<mpsc::UnboundedReceiver<InternalCommand<C>>>,
    device_connections: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
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
        
        Self {
            app_state,
            ui_provider,
            websocket_url,
            internal_cmd_tx,
            internal_cmd_rx: Some(internal_cmd_rx),
            device_connections,
        }
    }
    
    /// Get a handle to send commands to the runner
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<InternalCommand<C>> {
        self.internal_cmd_tx.clone()
    }
    
    /// Connect to WebSocket with given device ID
    pub async fn connect(&self, device_id: String) -> Result<()> {
        let _ = self.internal_cmd_tx.send(InternalCommand::SendToServer(
            ClientMsg::Register { device_id }
        ));
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
        let _ = self.internal_cmd_tx.send(InternalCommand::ProposeSession {
            session_id,
            total,
            threshold,
            participants,
        });
        Ok(())
    }
    
    /// Accept a session invitation
    pub async fn accept_session(&self, session_id: String) -> Result<()> {
        let _ = self.internal_cmd_tx.send(InternalCommand::AcceptSessionProposal(session_id));
        Ok(())
    }
    
    /// Start DKG process
    pub async fn start_dkg(&self) -> Result<()> {
        let _ = self.internal_cmd_tx.send(InternalCommand::TriggerDkgRound1);
        Ok(())
    }
    
    /// Initiate signing
    pub async fn initiate_signing(
        &self,
        transaction_data: String,
        blockchain: String,
        chain_id: Option<u64>,
    ) -> Result<()> {
        let _ = self.internal_cmd_tx.send(InternalCommand::InitiateSigning {
            transaction_data,
            blockchain,
            chain_id,
        });
        Ok(())
    }
    
    /// Accept a signing request
    pub async fn accept_signing(&self, signing_id: String) -> Result<()> {
        let _ = self.internal_cmd_tx.send(InternalCommand::AcceptSigning { signing_id });
        Ok(())
    }
    
    /// Main run loop - handles all async operations
    pub async fn run(mut self) -> Result<()> {
        let mut internal_cmd_rx = self.internal_cmd_rx.take()
            .ok_or_else(|| anyhow::anyhow!("Command receiver already taken"))?;
        
        let _app_state = self.app_state.clone();
        let ui_provider = self.ui_provider.clone();
        let _websocket_url = self.websocket_url.clone();
        let _internal_cmd_tx = self.internal_cmd_tx.clone();
        
        // Main event loop
        let mut ws_stream: Option<futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>
        >> = None;
        let mut ws_sink: Option<futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            WsMessage
        >> = None;
        let mut current_device_id = None;
        
        loop {
            tokio::select! {
                // Handle internal commands
                Some(cmd) = internal_cmd_rx.recv() => {
                    self.handle_internal_command(
                        cmd,
                        &mut ws_sink,
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
                            if let Ok(server_msg) = serde_json::from_str::<ServerMsg>(&text) {
                                self.handle_server_message(server_msg).await?;
                            }
                        }
                        Ok(WsMessage::Close(_)) => {
                            info!("WebSocket closed");
                            ws_stream = None;
                            ws_sink = None;
                            ui_provider.set_connection_status(false).await;
                            ui_provider.add_log("WebSocket connection closed".to_string()).await;
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            ws_stream = None;
                            ws_sink = None;
                            ui_provider.set_connection_status(false).await;
                            ui_provider.add_log(format!("WebSocket error: {}", e)).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    /// Handle internal commands
    async fn handle_internal_command(
        &self,
        cmd: InternalCommand<C>,
        ws_sink: &mut Option<futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            WsMessage
        >>,
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
                                    *ws_sink = Some(sink);
                                    *ws_stream = Some(stream_part);
                                    
                                    self.ui_provider.set_connection_status(true).await;
                                    self.ui_provider.add_log("Connected to WebSocket server".to_string()).await;
                                    info!("Connected to WebSocket server");
                                } else {
                                    self.ui_provider.show_error("Failed to send registration".to_string()).await;
                                }
                            }
                        }
                        Err(e) => {
                            self.ui_provider.show_error(format!("Failed to connect: {}", e)).await;
                            error!("Failed to connect: {}", e);
                        }
                    }
                } else if let Some(sink) = ws_sink {
                    // Send other messages
                    if let Ok(msg_str) = serde_json::to_string(&msg) {
                        if let Err(e) = sink.send(WsMessage::Text(msg_str.into())).await {
                            self.ui_provider.show_error(format!("Failed to send message: {}", e)).await;
                        }
                    }
                }
            }
            
            InternalCommand::ProposeSession { session_id, total, threshold, participants } => {
                let device_id = {
                    let state = self.app_state.lock().await;
                    state.device_id.clone()
                };
                session_commands::handle_propose_session(
                    session_id, total, threshold, participants,
                    self.app_state.clone(), self.internal_cmd_tx.clone(),
                    device_id
                ).await;
            }
            
            InternalCommand::AcceptSessionProposal(session_id) => {
                session_commands::handle_accept_session_proposal(
                    session_id, self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
            }
            
            InternalCommand::TriggerDkgRound1 => {
                let device_id = {
                    let state = self.app_state.lock().await;
                    state.device_id.clone()
                };
                dkg_commands::handle_trigger_dkg_round1(
                    self.app_state.clone(), device_id
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
                session_commands::handle_process_session_response(
                    from_device_id, response,
                    self.app_state.clone(), self.internal_cmd_tx.clone()
                ).await;
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
                dkg_commands::handle_trigger_dkg_round2(
                    self.app_state.clone()
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
                // Handle channel open - may trigger mesh ready
                let state = self.app_state.lock().await;
                let channels = state.data_channels.clone();
                drop(state);
                
                if !channels.contains_key(&device_id) {
                    self.ui_provider.add_log(format!("Channel opened with {}", device_id)).await;
                }
                
                // Check if mesh is ready
                let _ = self.internal_cmd_tx.send(InternalCommand::CheckAndTriggerDkg);
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
        }
        
        // Update UI after command processing
        self.update_ui_state().await?;
        
        Ok(())
    }
    
    /// Handle messages from server
    async fn handle_server_message(&self, msg: ServerMsg) -> Result<()> {
        use crate::network::webrtc::handle_webrtc_signal;
        
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
                self.ui_provider.add_log(format!("Relay from {}: {:?}", from, data)).await;
                
                // Handle different message types
                match serde_json::from_value::<WebSocketMessage>(data) {
                    Ok(WebSocketMessage::WebRTCSignal(signal)) => {
                        let device_id = {
                            let state = self.app_state.lock().await;
                            state.device_id.clone()
                        };
                        
                        handle_webrtc_signal(
                            from,
                            signal,
                            self.app_state.clone(),
                            device_id,
                            self.internal_cmd_tx.clone(),
                            self.device_connections.clone(),
                        ).await;
                    }
                    Ok(WebSocketMessage::SessionProposal(proposal)) => {
                        let invite_info = crate::protocal::signal::SessionInfo {
                            session_id: proposal.session_id.clone(),
                            proposer_id: from.clone(),
                            total: proposal.total,
                            threshold: proposal.threshold,
                            participants: proposal.participants.clone(),
                            accepted_devices: Vec::new(),
                            session_type: proposal.session_type.clone(),
                        };
                        
                        let mut state = self.app_state.lock().await;
                        state.invites.push(invite_info.clone());
                        drop(state);
                        
                        self.ui_provider.add_session_invite(invite_info).await;
                        self.ui_provider.add_log(format!(
                            "Received SessionProposal from {}: ID={}, Total={}, Threshold={}",
                            from, proposal.session_id, proposal.total, proposal.threshold
                        )).await;
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
                    Err(e) => {
                        self.ui_provider.add_log(format!("Failed to parse relay message: {}", e)).await;
                    }
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