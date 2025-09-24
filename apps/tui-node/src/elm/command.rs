//! Command - Side effects to be executed
//!
//! Commands represent operations that have side effects and need to be executed
//! outside of the pure update function. They handle async operations, I/O, and
//! interactions with external systems.

use crate::elm::message::{Message, SigningRequest};
use crate::elm::model::WalletConfig;
use crate::protocal::signal::SessionInfo;
use tokio::sync::mpsc::{self, UnboundedSender};
use std::sync::Arc;
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use std::path::PathBuf;
use tracing::{info, error, warn};

/// Commands represent side effects to be executed
#[derive(Debug, Clone)]
pub enum Command {
    // Data loading commands
    LoadWallets,
    LoadSessions,
    LoadWalletDetails { wallet_id: String },
    LoadSigningRequests,
    
    // Network operations
    ConnectWebSocket { url: String },
    ReconnectWebSocket,
    DisconnectWebSocket,
    SendNetworkMessage { to: String, data: Vec<u8> },
    BroadcastMessage { data: Vec<u8> },
    InitiateWebRTCConnections { participants: Vec<String> },
    VerifyWebRTCMesh,
    EnsureFullMesh,
    
    // Keystore operations
    InitializeKeystore { path: String, device_id: String },
    SaveWallet { wallet_data: Vec<u8> },
    DeleteWallet { wallet_id: String },
    ExportWallet { wallet_id: String, path: PathBuf },
    ImportWallet { path: PathBuf },
    
    // DKG operations
    StartDKG { config: WalletConfig },
    JoinDKG { session_id: String },
    CancelDKG,
    
    // Signing operations
    StartSigning { request: SigningRequest },
    ApproveSignature { request_id: String },
    RejectSignature { request_id: String },
    
    // UI operations
    SendMessage(Message),
    ScheduleMessage { delay_ms: u64, message: Box<Message> },
    RefreshUI,
    
    // Settings operations
    SaveSettings { websocket_url: String, device_id: String },
    LoadSettings,
    
    // System operations
    Quit,
    None,
}

impl Command {
    /// Execute the command and send resulting messages back to the update loop
    pub async fn execute<C: frost_core::Ciphersuite + Send + Sync + 'static>(
        self,
        tx: UnboundedSender<Message>,
        app_state: &std::sync::Arc<tokio::sync::Mutex<crate::utils::appstate_compat::AppState<C>>>,
    ) -> anyhow::Result<()>
    where
        <<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
        <<<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
    {
        match self {
            Command::LoadWallets => {
                info!("Loading wallets from keystore");
                
                let state = app_state.lock().await;
                if let Some(ref keystore) = state.keystore {
                    let wallets = keystore.list_wallets();
                    // Convert Vec<&WalletMetadata> to Vec<WalletMetadata> by cloning
                    let wallets: Vec<crate::keystore::WalletMetadata> = wallets.into_iter()
                        .cloned()
                        .collect();
                    let _ = tx.send(Message::WalletsLoaded { wallets });
                } else {
                    let _ = tx.send(Message::Error { 
                        message: "Keystore not initialized".to_string() 
                    });
                }
            }
            
            Command::LoadSessions => {
                info!("Loading available sessions - connecting to signal server for discovery");
                
                // Get the configured signal server URL and real device ID from app state
                let (signal_server_url, _device_id) = {
                    let state = app_state.lock().await;
                    (state.signal_server_url.clone(), state.device_id.clone())
                };
                
                let tx_clone = tx.clone();
                
                tokio::spawn(async move {
                    info!("Connecting to signal server for session discovery: {}", signal_server_url);
                    
                    // Connect to WebSocket for session discovery
                    match connect_async(&signal_server_url).await {
                        Ok((ws_stream, _response)) => {
                            info!("Connected to signal server for session discovery");
                            let (mut ws_tx, mut ws_rx) = ws_stream.split();
                            
                            // DO NOT register for discovery - just request sessions
                            // Registration should only happen when actually joining a session
                            info!("Requesting active sessions WITHOUT registering device");
                            
                            // Request active sessions directly without registering
                            let request_msg = serde_json::json!({
                                "type": "request_active_sessions"
                            });
                            
                            if ws_tx.send(tokio_tungstenite::tungstenite::Message::text(request_msg.to_string())).await.is_ok() {
                                info!("Requested active sessions from signal server");
                                
                                // Collect session responses with timeout
                                let mut sessions = Vec::new();
                                let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(2));
                                tokio::pin!(timeout);
                                
                                loop {
                                    tokio::select! {
                                        Some(msg) = ws_rx.next() => {
                                            if let Ok(tokio_tungstenite::tungstenite::Message::Text(txt)) = msg {
                                                info!("Session discovery received message: {}", txt);
                                                    
                                                    // Try to parse as ServerMsg first
                                                    if let Ok(server_msg) = serde_json::from_str::<webrtc_signal_server::ServerMsg>(&txt) {
                                                        match server_msg {
                                                            webrtc_signal_server::ServerMsg::SessionAvailable { session_info } => {
                                                                // Parse session_info JSON into SessionInfo
                                                                if let Some(sid) = session_info.get("session_id").and_then(|v| v.as_str()) {
                                                                    if let Some(total) = session_info.get("total").and_then(|v| v.as_u64()) {
                                                                        if let Some(threshold) = session_info.get("threshold").and_then(|v| v.as_u64()) {
                                                                            let participants = session_info.get("participants")
                                                                                .and_then(|v| v.as_array())
                                                                                .map(|arr| arr.iter()
                                                                                    .filter_map(|v| v.as_str().map(String::from))
                                                                                    .collect())
                                                                                .unwrap_or_default();
                                                                            
                                                                            info!("Discovered session: {} ({}/{})", sid, threshold, total);
                                                                            
                                                                            sessions.push(SessionInfo {
                                                                                session_id: sid.to_string(),
                                                                                proposer_id: session_info.get("proposer_id")
                                                                                    .and_then(|v| v.as_str())
                                                                                    .unwrap_or("unknown")
                                                                                    .to_string(),
                                                                                total: total as u16,
                                                                                threshold: threshold as u16,
                                                                                participants,
                                                                                session_type: crate::protocal::signal::SessionType::DKG,
                                                                                curve_type: session_info.get("curve_type")
                                                                                    .and_then(|v| v.as_str())
                                                                                    .unwrap_or("Secp256k1")
                                                                                    .to_string(),
                                                                                coordination_type: session_info.get("coordination_type")
                                                                                    .and_then(|v| v.as_str())
                                                                                    .unwrap_or("Network")
                                                                                    .to_string(),
                                                                            });
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }
                                            _ = &mut timeout => {
                                                info!("Session discovery timeout - found {} sessions", sessions.len());
                                                break;
                                            }
                                        }
                                    }
                                    
                                    // Send discovered sessions
                                    let _ = tx_clone.send(Message::SessionsLoaded { sessions });
                                    
                            } else {
                                warn!("Failed to request active sessions");
                                let _ = tx_clone.send(Message::SessionsLoaded { sessions: vec![] });
                            }
                        }
                        Err(e) => {
                            error!("Failed to connect for session discovery: {}", e);
                            let _ = tx_clone.send(Message::SessionsLoaded { sessions: vec![] });
                        }
                    }
                });
            }
            
            Command::LoadWalletDetails { wallet_id } => {
                info!("Loading details for wallet: {}", wallet_id);
                
                let state = app_state.lock().await;
                if let Some(ref keystore) = state.keystore {
                    if let Some(_wallet) = keystore.get_wallet(&wallet_id) {
                        // Wallet details loaded, update UI
                        let _ = tx.send(Message::Success { 
                            message: format!("Wallet {} loaded", wallet_id) 
                        });
                    } else {
                        let _ = tx.send(Message::Error { 
                            message: format!("Wallet {} not found", wallet_id) 
                        });
                    }
                }
            }
            
            Command::InitializeKeystore { path, device_id } => {
                info!("Initializing keystore at: {}", path);
                
                use crate::keystore::Keystore;
                match Keystore::new(&path, &device_id) {
                    Ok(keystore) => {
                        let mut state = app_state.lock().await;
                        state.keystore = Some(std::sync::Arc::new(keystore));
                        let _ = tx.send(Message::KeystoreInitialized { path });
                    }
                    Err(e) => {
                        error!("Failed to initialize keystore: {}", e);
                        let _ = tx.send(Message::KeystoreError { 
                            error: e.to_string() 
                        });
                    }
                }
            }
            
            Command::StartDKG { config } => {
                info!("Starting REAL DKG with config: {:?}", config);

                // CRITICAL FIX: Check if DKG is already in progress to prevent duplicates
                {
                    let mut state = app_state.lock().await;
                    if state.dkg_in_progress {
                        info!("⚠️ DKG already in progress, skipping duplicate StartDKG");
                        let _ = tx.send(Message::Info {
                            message: "DKG already in progress, please wait...".to_string()
                        });
                        return Ok(()); // Exit early to prevent duplicate session
                    }
                    state.dkg_in_progress = true; // Mark DKG as in progress
                }

                // Check if we're in online mode
                if config.mode == crate::elm::model::WalletMode::Online {
                    // For online mode, use the real DKG session manager
                    info!("Online mode - need {} participants with threshold {}", 
                          config.total_participants, config.threshold);
                    
                    // Send initial progress
                    let _ = tx.send(Message::UpdateDKGProgress { 
                        round: crate::elm::message::DKGRound::Initialization,
                        progress: 0.1,
                    });
                    
                    // Start the real DKG with session manager
                    let tx_clone = tx.clone();
                    let config_clone = config.clone();
                    
                    // Note: We can't use tokio::spawn here due to Send/Sync constraints
                    // with FROST cryptographic types. For now, show informative messages.

                    // CRITICAL FIX: Check if we already have an active session ID
                    // This prevents creating new sessions on WebSocket reconnection
                    let session_id = {
                        let state = app_state.lock().await;
                        if let Some(ref session) = state.session {
                            // Reuse existing session ID to prevent session chaos
                            info!("🔄 Reusing existing session ID: {}", session.session_id);
                            session.session_id.clone()
                        } else {
                            // Only generate new session ID if we don't have one
                            let new_id = format!("dkg_{}", uuid::Uuid::new_v4());
                            info!("🆕 Creating new session ID: {}", new_id);
                            new_id
                        }
                    };

                    let _ = tx_clone.send(Message::UpdateDKGSessionId {
                        real_session_id: session_id.clone()
                    });
                    
                    let _ = tx_clone.send(Message::Info { 
                        message: format!("📝 Created DKG session: {}", session_id)
                    });
                    
                    // Show instructions
                    let _ = tx_clone.send(Message::Info { 
                        message: "📋 To complete REAL DKG in online mode:".to_string()
                    });
                    let _ = tx_clone.send(Message::Info { 
                        message: format!("1. Share session ID '{}' with other participants", session_id)
                    });
                    let _ = tx_clone.send(Message::Info { 
                        message: "2. Each participant must run this TUI with 'Join Session'".to_string()
                    });
                    let _ = tx_clone.send(Message::Info { 
                        message: format!("3. Need {} total participants connected", config_clone.total_participants)
                    });
                    
                    // Get the configured signal server URL and device ID from app state
                    let (signal_server_url, device_id) = {
                        let state = app_state.lock().await;
                        (state.signal_server_url.clone(), state.device_id.clone())
                    };
                    
                    let _ = tx_clone.send(Message::Info { 
                        message: format!("🔌 Connecting to signal server: {}", signal_server_url)
                    });
                    
                    // Update the app state to indicate we're connecting
                    {
                        let mut state = app_state.lock().await;
                        state.websocket_connecting = true;
                        state.websocket_connected = false;
                    }
                    
                    // Actually connect to WebSocket using tokio-tungstenite
                    use tokio_tungstenite::connect_async;
                    
                    match connect_async(&signal_server_url).await {
                        Ok((ws_stream, _response)) => {
                            // Send WebSocketConnected message to update UI state
                            let _ = tx_clone.send(Message::WebSocketConnected);
                            
                            // Also update app state
                            {
                                let mut state = app_state.lock().await;
                                state.websocket_connected = true;
                                state.websocket_connecting = false;
                            }
                            
                            let _ = tx_clone.send(Message::Info { 
                                message: "✅ WebSocket connected successfully!".to_string()
                            });
                            
                            // Split the WebSocket stream
                            use futures_util::{SinkExt, StreamExt};
                            let (mut ws_sink, mut ws_stream) = ws_stream.split();

                            // Create a channel for WebSocket message processing IMMEDIATELY
                            // This must be done before announcing session so WebRTC can use it
                            let (ws_msg_tx, mut ws_msg_rx) = mpsc::unbounded_channel::<String>();

                            // Store the string-based channel sender in app state for WebRTC to use
                            {
                                let mut state = app_state.lock().await;
                                state.websocket_msg_tx = Some(ws_msg_tx.clone());
                                info!("✅ Stored WebSocket message channel in AppState (EARLY)");
                            }

                            // Start WebRTC status polling task to update UI
                            let tx_status_poll = tx_clone.clone();
                            let app_state_poll = app_state.clone();
                            tokio::spawn(async move {
                                info!("🔄 Starting WebRTC status polling task");
                                let mut last_statuses: std::collections::HashMap<String, (bool, bool)> = std::collections::HashMap::new();

                                loop {
                                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                                    // Check device statuses in AppState
                                    let state = app_state_poll.lock().await;
                                    for (device_id, status) in &state.device_statuses {
                                        let webrtc_connected = matches!(status, webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected);
                                        let data_channel_open = state.data_channels.contains_key(device_id);

                                        // Check if status changed
                                        let new_status = (webrtc_connected, data_channel_open);
                                        let should_update = if let Some(&old_status) = last_statuses.get(device_id) {
                                            old_status != new_status
                                        } else {
                                            true // First time seeing this device
                                        };

                                        if should_update {
                                            // Status changed or new device, send UI update
                                            let _ = tx_status_poll.send(Message::UpdateParticipantWebRTCStatus {
                                                device_id: device_id.clone(),
                                                webrtc_connected: new_status.0,
                                                data_channel_open: new_status.1,
                                            });
                                            info!("📊 WebRTC status update for {}: WebRTC={}, Channel={}",
                                                  device_id, new_status.0, new_status.1);
                                            last_statuses.insert(device_id.clone(), new_status);
                                        }
                                    }
                                }
                            });

                            // Register device with the signal server (send directly before moving ws_sink)
                            let register_msg = webrtc_signal_server::ClientMsg::Register { 
                                device_id: device_id.clone() 
                            };
                            let register_json = serde_json::to_string(&register_msg).unwrap();
                            
                            if let Err(e) = ws_sink.send(tokio_tungstenite::tungstenite::Message::text(register_json)).await {
                                let _ = tx_clone.send(Message::Error { 
                                    message: format!("Failed to register with signal server: {}", e)
                                });
                            } else {
                                let _ = tx_clone.send(Message::Info { 
                                    message: format!("✅ Registered as device: {}", device_id)
                                });
                            }
                            
                            // Announce a DKG session
                            let session_info = serde_json::json!({
                                "session_id": session_id.clone(),
                                "total": config_clone.total_participants,
                                "threshold": config_clone.threshold,
                                "session_type": "dkg",
                                "proposer_id": device_id.clone(),
                                "participants": [device_id.clone()],
                                "curve_type": match config_clone.curve {
                                    crate::elm::model::CurveType::Secp256k1 => "Secp256k1",
                                    crate::elm::model::CurveType::Ed25519 => "Ed25519",
                                },
                                "coordination_type": "Network"
                            });
                            
                            let announce_session_msg = webrtc_signal_server::ClientMsg::AnnounceSession {
                                session_info: session_info.clone(),
                            };
                            let session_json = serde_json::to_string(&announce_session_msg).unwrap();

                            info!("Announcing session to signal server: {}", session_json);

                            // Send directly through ws_sink for session announcement
                            // (ws_msg_tx channel not created yet at this point)
                            if let Err(e) = ws_sink.send(tokio_tungstenite::tungstenite::Message::text(session_json)).await {
                                let _ = tx_clone.send(Message::Error {
                                    message: format!("Failed to create session: {}", e)
                                });
                            } else {
                                let _ = tx_clone.send(Message::Info { 
                                    message: format!("📝 Session created: {}", session_id)
                                });
                            }
                            
                            // Mark as connected and store session info
                            {
                                let mut state = app_state.lock().await;
                                state.websocket_connecting = false;
                                state.websocket_connected = true;
                                
                                // Store the session info
                                state.session = Some(crate::protocal::signal::SessionInfo {
                                    session_id: session_id.clone(),
                                    proposer_id: device_id.clone(),
                                    participants: vec![device_id.clone()],
                                    threshold: config_clone.threshold,
                                    total: config_clone.total_participants,
                                    session_type: crate::protocal::signal::SessionType::DKG,
                                    curve_type: match config_clone.curve {
                                        crate::elm::model::CurveType::Secp256k1 => "Secp256k1".to_string(),
                                        crate::elm::model::CurveType::Ed25519 => "Ed25519".to_string(),
                                    },
                                    coordination_type: "Network".to_string(),
                                });
                            }
                            
                            let _ = tx_clone.send(Message::Info { 
                                message: "⏳ Waiting for other participants to join...".to_string()
                            });
                            
                            let _ = tx_clone.send(Message::UpdateDKGProgress {
                                round: crate::elm::message::DKGRound::WaitingForParticipants,
                                progress: 0.2,
                            });

                            // Now spawn the WebSocket sender task (after direct sends are done)
                            tokio::spawn(async move {
                                info!("🚀 WebSocket sender task started");

                                // Process string messages and send through WebSocket
                                while let Some(msg) = ws_msg_rx.recv().await {
                                    info!("📤 Sending through WebSocket: {}", msg);
                                    if let Err(e) = ws_sink.send(tokio_tungstenite::tungstenite::Message::text(msg)).await {
                                        error!("❌ Failed to send through WebSocket: {}", e);
                                    } else {
                                        info!("✅ Sent through WebSocket successfully");
                                    }
                                }
                                info!("WebSocket sender task stopped");
                            });

                            // Spawn a task to handle incoming WebSocket messages
                            let tx_msg = tx_clone.clone();
                            let session_id_clone = session_id.clone();
                            let total_participants = config_clone.total_participants;
                            let device_id_clone = device_id.clone();

                            // Get session info before spawning
                            let our_session_id = {
                                let state = app_state.lock().await;
                                state.session.as_ref().map(|s| s.session_id.clone())
                            };

                            // Clone app_state for the spawned task
                            let app_state_clone = app_state.clone();

                            tokio::spawn(async move {
                                let mut participants_seen = std::collections::HashSet::new();
                                participants_seen.insert(device_id_clone.clone());
                                
                                while let Some(msg) = ws_stream.next().await {
                                    match msg {
                                        Ok(tokio_tungstenite::tungstenite::Message::Text(txt)) => {
                                            // Try to parse server messages
                                            if let Ok(server_msg) = serde_json::from_str::<webrtc_signal_server::ServerMsg>(&txt) {
                                                match server_msg {
                                                    webrtc_signal_server::ServerMsg::SessionAvailable { session_info } => {
                                                        // Another participant announced a session - check if it's us joining theirs
                                                        if let Some(sid) = session_info.get("session_id").and_then(|v| v.as_str()) {
                                                            if sid != session_id_clone {
                                                                // Different session
                                                                let _ = tx_msg.send(Message::Info { 
                                                                    message: format!("📢 Another session available: {}", sid)
                                                                });
                                                            }
                                                        }
                                                    }
                                                    webrtc_signal_server::ServerMsg::Devices { devices } => {
                                                        // Check if any devices in our session
                                                        let _ = tx_msg.send(Message::Info { 
                                                            message: format!("📡 Connected devices: {:?}", devices)
                                                        });
                                                        
                                                        // Count unique participants
                                                        for device in &devices {
                                                            participants_seen.insert(device.clone());
                                                        }
                                                        
                                                        // Send UpdateParticipants message to update the model
                                                        let participants_list: Vec<String> = participants_seen.iter().cloned().collect();
                                                        let _ = tx_msg.send(Message::UpdateParticipants { 
                                                            participants: participants_list 
                                                        });
                                                        
                                                        let participants_count = participants_seen.len();
                                                        if participants_count > 1 {
                                                            let _ = tx_msg.send(Message::Info { 
                                                                message: format!("👥 Current participants: {}/{}", 
                                                                    participants_count, total_participants)
                                                            });
                                                        }
                                                        
                                                        if participants_count >= total_participants as usize {
                                                            let _ = tx_msg.send(Message::Info { 
                                                                message: "🎉 All participants connected! Starting DKG...".to_string()
                                                            });
                                                            
                                                            // Actually initiate WebRTC connections NOW
                                                            let _ = tx_msg.send(Message::Info { 
                                                                message: "🔗 Establishing peer-to-peer connections...".to_string()
                                                            });
                                                            
                                                            // Get participants list without self
                                                            let self_device = device_id_clone.clone();
                                                            let other_participants: Vec<String> = participants_seen.iter()
                                                                .filter(|p| **p != self_device)
                                                                .cloned()
                                                                .collect();
                                                            
                                                            if !other_participants.is_empty() {
                                                                let _ = tx_msg.send(Message::Info { 
                                                                    message: format!("🔗 Initiating WebRTC with {} participants", other_participants.len())
                                                                });
                                                                
                                                                // Directly call WebRTC initiation
                                                                // Note: We don't have app_state here, so we need to send a message
                                                                let _ = tx_msg.send(Message::InitiateWebRTCWithParticipants {
                                                                    participants: other_participants,
                                                                });
                                                            }
                                                            
                                                            // Update DKG progress
                                                            let _ = tx_msg.send(Message::UpdateDKGProgress {
                                                                round: crate::elm::message::DKGRound::Round1,
                                                                progress: 0.3,
                                                            });
                                                        }
                                                    }
                                                    webrtc_signal_server::ServerMsg::Relay { from, data } => {
                                                        let _ = tx_msg.send(Message::Info {
                                                            message: format!("📨 Received relay from {}", from)
                                                        });

                                                        // Check if it's a WebRTC signal from another device
                                                        if from != "server" {
                                                            // Handle WebRTC signals (offer/answer/ICE)
                                                            if let Some(msg_type) = data.get("websocket_msg_type").and_then(|v| v.as_str()) {
                                                                if msg_type == "WebRTCSignal" {
                                                                    info!("🎯 Received WebRTC signal from {}", from);

                                                                    // Check if it's an offer that needs an answer
                                                                    if let Some(offer_data) = data.get("Offer") {
                                                                        if let Some(sdp) = offer_data.get("sdp").and_then(|v| v.as_str()) {
                                                                            info!("📥 Received WebRTC offer from {}, need to send answer", from);
                                                                            let _ = tx_msg.send(Message::Info {
                                                                                message: format!("📥 Received WebRTC offer from {}, preparing answer...", from)
                                                                            });

                                                                            // Create and send WebRTC answer
                                                                            let from_device = from.clone();
                                                                            let sdp_string = sdp.to_string();
                                                                            let app_state_for_answer = app_state_clone.clone();
                                                                            let ws_tx_clone = ws_msg_tx.clone();
                                                                            let _self_device_id = device_id.clone();
                                                                            let tx_msg_spawn = tx_msg.clone();

                                                                            tokio::spawn(async move {
                                                                                info!("🎯 Processing WebRTC offer from {}", from_device);

                                                                                // Get or create peer connection for this device
                                                                                let pc = {
                                                                                    let state = app_state_for_answer.lock().await;
                                                                                    let device_connections_clone = state.device_connections.clone();
                                                                                    drop(state); // Release the lock early
                                                                                    let mut conns = device_connections_clone.lock().await;
                                                                                    if let Some(existing_pc) = conns.get(&from_device) {
                                                                                        existing_pc.clone()
                                                                                    } else {
                                                                                        // Create new peer connection for this device
                                                                                        info!("📱 Creating peer connection for {} (to handle offer)", from_device);
                                                                                        let config = webrtc::peer_connection::configuration::RTCConfiguration {
                                                                                            ice_servers: vec![],
                                                                                            ..Default::default()
                                                                                        };

                                                                                        match webrtc::api::APIBuilder::new()
                                                                                            .build()
                                                                                            .new_peer_connection(config)
                                                                                            .await
                                                                                        {
                                                                                            Ok(new_pc) => {
                                                                                                let arc_pc = Arc::new(new_pc);

                                                                                                // Set up handler for incoming data channels
                                                                                                let from_device_dc = from_device.clone();
                                                                                                let tx_msg_dc = tx_msg_spawn.clone();
                                                                                                arc_pc.on_data_channel(Box::new(move |dc: Arc<webrtc::data_channel::RTCDataChannel>| {
                                                                                                    let device_id_dc = from_device_dc.clone();
                                                                                                    let tx_msg_dc = tx_msg_dc.clone();
                                                                                                    Box::pin(async move {
                                                                                                        info!("📂 Incoming data channel from {}: {}", device_id_dc, dc.label());

                                                                                                        // Set up message handlers for the incoming data channel
                                                                                                        let device_id_open = device_id_dc.clone();
                                                                                                        let tx_msg_open = tx_msg_dc.clone();
                                                                                                        dc.on_open(Box::new(move || {
                                                                                                            let device_open = device_id_open.clone();
                                                                                                            let tx_msg_open = tx_msg_open.clone();
                                                                                                            Box::pin(async move {
                                                                                                                info!("📂 Data channel OPENED from {}", device_open);
                                                                                                                
                                                                                                                // Send UI update for data channel open
                                                                                                                let _ = tx_msg_open.send(Message::UpdateParticipantWebRTCStatus {
                                                                                                                    device_id: device_open.clone(),
                                                                                                                    webrtc_connected: true,
                                                                                                                    data_channel_open: true,
                                                                                                                });
                                                                                                            })
                                                                                                        }));

                                                                                                        let device_id_msg = device_id_dc.clone();
                                                                                                        dc.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                                                                                                            let device_msg = device_id_msg.clone();
                                                                                                            Box::pin(async move {
                                                                                                                info!("📥 Received message from {} via data channel: {} bytes",
                                                                                                                    device_msg, msg.data.len());
                                                                                                                // TODO: Forward to DKG protocol handler
                                                                                                            })
                                                                                                        }));

                                                                                                        // TODO: Store dc for sending messages back
                                                                                                    })
                                                                                                }));

                                                                                                // Set up connection state handler
                                                                                                let device_id_state = from_device.clone();
                                                                                                let tx_msg_state = tx_msg_spawn.clone();
                                                                                                arc_pc.on_peer_connection_state_change(Box::new(move |state: webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState| {
                                                                                                    let device_id_state = device_id_state.clone();
                                                                                                    let tx_msg_state = tx_msg_state.clone();
                                                                                                    Box::pin(async move {
                                                                                                        let is_connected = matches!(state, webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected);
                                                                                                        
                                                                                                        // Send UI update
                                                                                                        let _ = tx_msg_state.send(Message::UpdateParticipantWebRTCStatus {
                                                                                                            device_id: device_id_state.clone(),
                                                                                                            webrtc_connected: is_connected,
                                                                                                            data_channel_open: false, // Will be updated when data channel opens
                                                                                                        });
                                                                                                        
                                                                                                        match state {
                                                                                                            webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected => {
                                                                                                                info!("✅ WebRTC connection ESTABLISHED with {} (from answer)", device_id_state);
                                                                                                            }
                                                                                                            webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Failed => {
                                                                                                                error!("❌ WebRTC connection FAILED with {} (from answer)", device_id_state);
                                                                                                            }
                                                                                                            _ => {
                                                                                                                info!("WebRTC connection state with {} (from answer): {:?}", device_id_state, state);
                                                                                                            }
                                                                                                        }
                                                                                                    })
                                                                                                }));

                                                                                                // Set up ICE candidate handler
                                                                                                let device_id_ice = from_device.clone();
                                                                                                let ws_tx_ice = ws_tx_clone.clone();

                                                                                                arc_pc.on_ice_candidate(Box::new(move |candidate: Option<webrtc::ice_transport::ice_candidate::RTCIceCandidate>| {
                                                                                                    let device_id_ice = device_id_ice.clone();
                                                                                                    let ws_tx_ice = ws_tx_ice.clone();

                                                                                                    Box::pin(async move {
                                                                                                        if let Some(candidate) = candidate {
                                                                                                            info!("🧊 Generated ICE candidate for {}", device_id_ice);

                                                                                                            // Send ICE candidate to peer
                                                                                                            let candidate_json = candidate.to_json().unwrap();
                                                                                                            let ice_signal = crate::protocal::signal::WebRTCSignal::Candidate(
                                                                                                                crate::protocal::signal::CandidateInfo {
                                                                                                                    candidate: candidate_json.candidate,
                                                                                                                    sdp_mid: candidate_json.sdp_mid,
                                                                                                                    sdp_mline_index: candidate_json.sdp_mline_index,
                                                                                                                }
                                                                                                            );

                                                                                                            let websocket_message = crate::protocal::signal::WebSocketMessage::WebRTCSignal(ice_signal);

                                                                                                            if let Ok(json_val) = serde_json::to_value(websocket_message) {
                                                                                                                let relay_msg = webrtc_signal_server::ClientMsg::Relay {
                                                                                                                    to: device_id_ice.clone(),
                                                                                                                    data: json_val,
                                                                                                                };

                                                                                                                if let Ok(json) = serde_json::to_string(&relay_msg) {
                                                                                                                    info!("📤 Sending ICE candidate to {} via WebSocket", device_id_ice);
                                                                                                                    let _ = ws_tx_ice.send(json);
                                                                                                                }
                                                                                                            }
                                                                                                        }
                                                                                                    })
                                                                                                }));

                                                                                                conns.insert(from_device.clone(), arc_pc.clone());
                                                                                                arc_pc
                                                                                            }
                                                                                            Err(e) => {
                                                                                                error!("❌ Failed to create peer connection for {}: {}", from_device, e);
                                                                                                return;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                };

                                                                                // Set remote description (the offer)
                                                                                let offer = webrtc::peer_connection::sdp::session_description::RTCSessionDescription::offer(sdp_string).unwrap();

                                                                                if let Err(e) = pc.set_remote_description(offer).await {
                                                                                    error!("❌ Failed to set remote description for {}: {}", from_device, e);
                                                                                    return;
                                                                                }
                                                                                info!("✅ Set remote description (offer) from {}", from_device);

                                                                                // Create answer
                                                                                match pc.create_answer(None).await {
                                                                                    Ok(answer) => {
                                                                                        info!("✅ Created answer for {}", from_device);

                                                                                        // Set local description (the answer)
                                                                                        if let Err(e) = pc.set_local_description(answer.clone()).await {
                                                                                            error!("❌ Failed to set local description for {}: {}", from_device, e);
                                                                                            return;
                                                                                        }
                                                                                        info!("✅ Set local description (answer) for {}", from_device);

                                                                                        // Send answer back via WebSocket
                                                                                        let signal = crate::protocal::signal::WebRTCSignal::Answer(
                                                                                            crate::protocal::signal::SDPInfo { sdp: answer.sdp }
                                                                                        );
                                                                                        let websocket_message = crate::protocal::signal::WebSocketMessage::WebRTCSignal(signal);

                                                                                        match serde_json::to_value(websocket_message) {
                                                                                            Ok(json_val) => {
                                                                                                let relay_msg = webrtc_signal_server::ClientMsg::Relay {
                                                                                                    to: from_device.clone(),
                                                                                                    data: json_val,
                                                                                                };

                                                                                                match serde_json::to_string(&relay_msg) {
                                                                                                    Ok(json) => {
                                                                                                        info!("📤 Sending WebRTC answer to {} via WebSocket", from_device);
                                                                                                        if let Err(e) = ws_tx_clone.send(json) {
                                                                                                            error!("❌ Failed to send answer to {}: {}", from_device, e);
                                                                                                        } else {
                                                                                                            info!("✅ WebRTC answer sent to {}", from_device);
                                                                                                        }
                                                                                                    }
                                                                                                    Err(e) => {
                                                                                                        error!("❌ Failed to serialize answer for {}: {}", from_device, e);
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                            Err(e) => {
                                                                                                error!("❌ Failed to serialize WebRTC answer for {}: {}", from_device, e);
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    Err(e) => {
                                                                                        error!("❌ Failed to create answer for {}: {}", from_device, e);
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    } else if let Some(answer_data) = data.get("Answer") {
                                                                        if let Some(sdp) = answer_data.get("sdp").and_then(|v| v.as_str()) {
                                                                            info!("📥 Received WebRTC answer from {}", from);
                                                                            let _ = tx_msg.send(Message::Info {
                                                                                message: format!("📥 Received WebRTC answer from {}, setting remote description...", from)
                                                                            });

                                                                            // Set remote description with the answer
                                                                            let from_device = from.clone();
                                                                            let sdp_string = sdp.to_string();
                                                                            let app_state_for_remote = app_state_clone.clone();

                                                                            tokio::spawn(async move {
                                                                                info!("🎯 Processing WebRTC answer from {}", from_device);

                                                                                // Get peer connection for this device
                                                                                let state = app_state_for_remote.lock().await;
                                                                                let device_connections_clone = state.device_connections.clone();
                                                                                drop(state); // Release the lock early
                                                                                let conns = device_connections_clone.lock().await;
                                                                                if let Some(pc) = conns.get(&from_device) {
                                                                                    // Set remote description (the answer)
                                                                                    let answer = webrtc::peer_connection::sdp::session_description::RTCSessionDescription::answer(sdp_string).unwrap();

                                                                                    if let Err(e) = pc.set_remote_description(answer).await {
                                                                                        error!("❌ Failed to set remote description (answer) for {}: {}", from_device, e);
                                                                                    } else {
                                                                                        info!("✅ Set remote description (answer) from {}, WebRTC connection should be establishing!", from_device);
                                                                                    }
                                                                                } else {
                                                                                    error!("❌ No peer connection found for {} when receiving answer", from_device);
                                                                                }
                                                                            });
                                                                        }
                                                                    } else if let Some(ice_data) = data.get("Candidate") {
                                                                        info!("📥 Received ICE candidate from {}", from);

                                                                        // Parse ICE candidate data
                                                                        if let (Some(candidate), Some(sdp_mid), Some(sdp_mline_index)) = (
                                                                            ice_data.get("candidate").and_then(|v| v.as_str()),
                                                                            ice_data.get("sdpMid").and_then(|v| v.as_str()),
                                                                            ice_data.get("sdpMLineIndex").and_then(|v| v.as_u64())
                                                                        ) {
                                                                            let from_device = from.clone();
                                                                            let candidate_str = candidate.to_string();
                                                                            let sdp_mid_str = sdp_mid.to_string();
                                                                            let sdp_mline_index_u16 = sdp_mline_index as u16;
                                                                            let app_state_for_ice = app_state_clone.clone();

                                                                            tokio::spawn(async move {
                                                                                info!("🎯 Adding ICE candidate from {}", from_device);

                                                                                // Get peer connection for this device
                                                                                let state = app_state_for_ice.lock().await;
                                                                                let device_connections_clone = state.device_connections.clone();
                                                                                drop(state); // Release the lock early
                                                                                let conns = device_connections_clone.lock().await;
                                                                                if let Some(pc) = conns.get(&from_device) {
                                                                                    // Create ICE candidate init
                                                                                    let ice_candidate_init = webrtc::ice_transport::ice_candidate::RTCIceCandidateInit {
                                                                                        candidate: candidate_str,
                                                                                        sdp_mid: Some(sdp_mid_str),
                                                                                        sdp_mline_index: Some(sdp_mline_index_u16),
                                                                                        username_fragment: None,
                                                                                    };

                                                                                    if let Err(e) = pc.add_ice_candidate(ice_candidate_init).await {
                                                                                        error!("❌ Failed to add ICE candidate from {}: {}", from_device, e);
                                                                                    } else {
                                                                                        info!("✅ Added ICE candidate from {}", from_device);
                                                                                    }
                                                                                } else {
                                                                                    error!("❌ No peer connection found for {} when adding ICE candidate", from_device);
                                                                                }
                                                                            });
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        // Check if it's a participant update from the server
                                                        else if from == "server" {
                                                            if let Some(msg_type) = data.get("type").and_then(|v| v.as_str()) {
                                                                if msg_type == "participant_update" {
                                                                    // Handle participant update to trigger WebRTC
                                                                    if let (Some(session_id), Some(session_info)) = (
                                                                        data.get("session_id").and_then(|v| v.as_str()),
                                                                        data.get("session_info")
                                                                    ) {
                                                                        // Check if this is our session
                                                                        let is_our_session = our_session_id.as_ref()
                                                                            .map(|s| s == session_id)
                                                                            .unwrap_or(false);
                                                                        
                                                                        if is_our_session {
                                                                            // Extract participants from session_info
                                                                            let new_participants = if let Some(participants_arr) = session_info
                                                                                .get("participants")
                                                                                .and_then(|v| v.as_array()) {
                                                                                participants_arr.iter()
                                                                                    .filter_map(|v| v.as_str())
                                                                                    .filter(|&p| p != device_id) // Filter out self
                                                                                    .map(String::from)
                                                                                    .collect::<Vec<_>>()
                                                                            } else {
                                                                                Vec::new()
                                                                            };
                                                                            
                                                                            if !new_participants.is_empty() {
                                                                                info!("📡 Received participant update, triggering WebRTC with {} participants", new_participants.len());

                                                                                // Update the session participants in app state
                                                                                let all_participants = {
                                                                                    let mut state = app_state_clone.lock().await;

                                                                                    // Update session info with all participants (including self)
                                                                                    let mut all_parts = new_participants.clone();
                                                                                    all_parts.push(device_id.clone());

                                                                                    if let Some(ref mut session) = state.session {
                                                                                        session.participants = all_parts.clone();
                                                                                        info!("✅ Updated session participants: {:?}", all_parts);
                                                                                    }

                                                                                    all_parts
                                                                                };

                                                                                // Send message to trigger WebRTC through update loop
                                                                                info!("🚀 Triggering WebRTC initiation from participant update");

                                                                                let _ = tx_msg.send(Message::InitiateWebRTCWithParticipants {
                                                                                    participants: all_participants
                                                                                });

                                                                                let _ = tx_msg.send(Message::Info {
                                                                                    message: format!("📡 Triggered WebRTC with participants: {:?}", new_participants)
                                                                                });
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                                            let _ = tx_msg.send(Message::Info { 
                                                message: "WebSocket connection closed".to_string()
                                            });
                                            break;
                                        }
                                        Err(e) => {
                                            let _ = tx_msg.send(Message::Error { 
                                                message: format!("WebSocket error: {}", e)
                                            });
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            });
                            
                            // Show current participant count
                            let _ = tx_clone.send(Message::Info { 
                                message: format!("👥 Current participants: 1/{}", config_clone.total_participants)
                            });
                            
                            // Update DKG progress to show we're waiting for participants  
                            let _ = tx_clone.send(Message::UpdateDKGProgress {
                                round: crate::elm::message::DKGRound::WaitingForParticipants,
                                progress: 0.2,
                            });
                            
                            // Keep the DKG progress screen open and wait for participants
                            // Don't automatically fail - let the user cancel if they want
                            let _ = tx_clone.send(Message::Info { 
                                message: format!("⏳ Waiting for {} more participants...", config_clone.total_participants - 1)
                            });
                            
                            let _ = tx_clone.send(Message::Info { 
                                message: format!("📋 Share this session ID with other participants: {}", session_id)
                            });
                            
                            // The WebSocket handler will continue running and listening for participants
                            // User can press Esc to cancel if needed
                        },
                        Err(e) => {
                            // Send WebSocketDisconnected to update UI state
                            let _ = tx_clone.send(Message::WebSocketDisconnected);
                            
                            // Update app state
                            {
                                let mut state = app_state.lock().await;
                                state.websocket_connected = false;
                                state.websocket_connecting = false;
                            }
                            
                            let _ = tx_clone.send(Message::Info { 
                                message: format!("❌ Cannot reach signal server: {}", e)
                            });
                            let _ = tx_clone.send(Message::Info { 
                                message: "💡 Make sure the signal server is running:".to_string()
                            });
                            let _ = tx_clone.send(Message::Info { 
                                message: "   cargo run --bin webrtc-signal-server".to_string()
                            });
                            
                            let _ = tx_clone.send(Message::DKGFailed { 
                                error: format!("Signal server not reachable at {}", signal_server_url)
                            });
                        }
                    }
                } else {
                    // Offline mode - use SD card exchange
                    info!("Offline mode selected - air-gapped DKG");
                    
                    let _ = tx.send(Message::Info { 
                        message: "🔒 Offline DKG Mode".to_string()
                    });
                    let _ = tx.send(Message::Info { 
                        message: "📋 Steps for offline DKG:".to_string()
                    });
                    let _ = tx.send(Message::Info { 
                        message: "1. Each participant generates their Round 1 commitment".to_string()
                    });
                    let _ = tx.send(Message::Info { 
                        message: "2. Export commitments to SD card".to_string()
                    });
                    let _ = tx.send(Message::Info { 
                        message: "3. Exchange SD cards physically".to_string()
                    });
                    let _ = tx.send(Message::Info { 
                        message: "4. Import other participants' commitments".to_string()
                    });
                    let _ = tx.send(Message::Info { 
                        message: "5. Generate and exchange Round 2 shares".to_string()
                    });
                    
                    // TODO: Implement offline DKG with SD card exchange
                    let _ = tx.send(Message::DKGFailed { 
                        error: "Offline DKG implementation in progress. For now, please use online mode with multiple nodes.".to_string()
                    });
                }
            }
            
            Command::JoinDKG { session_id } => {
                info!("Joining DKG session: {}", session_id);
                
                // Send initial message
                let _ = tx.send(Message::Info { 
                    message: format!("🔗 Joining DKG session: {}", session_id)
                });
                
                // Get the configured signal server URL and device ID from app state
                let (signal_server_url, device_id) = {
                    let state = app_state.lock().await;
                    (state.signal_server_url.clone(), state.device_id.clone())
                };
                
                let _ = tx.send(Message::Info { 
                    message: format!("🔌 Connecting to signal server: {}", signal_server_url)
                });
                
                // Actually connect to WebSocket using tokio-tungstenite
                use tokio_tungstenite::connect_async;
                
                let tx_clone = tx.clone();
                match connect_async(&signal_server_url).await {
                    Ok((ws_stream, _response)) => {
                        // Send WebSocketConnected message to update UI state
                        let _ = tx_clone.send(Message::WebSocketConnected);
                        
                        // Also update app state
                        {
                            let mut state = app_state.lock().await;
                            state.websocket_connected = true;
                            state.websocket_connecting = false;
                        }
                        
                        let _ = tx_clone.send(Message::Info { 
                            message: "✅ WebSocket connected successfully!".to_string()
                        });
                        
                        // Split the WebSocket stream
                        use futures_util::{SinkExt, StreamExt};
                        let (mut ws_sink, mut ws_stream) = ws_stream.split();

                        // Create a channel for WebSocket message processing IMMEDIATELY
                        // This must be done before joining session so WebRTC can use it
                        let (ws_msg_tx, mut ws_msg_rx) = mpsc::unbounded_channel::<String>();

                        // Store the string-based channel sender in app state for WebRTC to use
                        {
                            let mut state = app_state.lock().await;
                            state.websocket_msg_tx = Some(ws_msg_tx.clone());
                            info!("✅ Stored WebSocket message channel in AppState (JoinDKG)");
                        }

                        // Register device with the signal server
                        let register_msg = webrtc_signal_server::ClientMsg::Register { 
                            device_id: device_id.clone() 
                        };
                        let register_json = serde_json::to_string(&register_msg).unwrap();
                        
                        if let Err(e) = ws_sink.send(tokio_tungstenite::tungstenite::Message::text(register_json)).await {
                            let _ = tx_clone.send(Message::Error { 
                                message: format!("Failed to register with signal server: {}", e)
                            });
                        } else {
                            let _ = tx_clone.send(Message::Info { 
                                message: format!("✅ Registered as device: {}", device_id)
                            });
                        }
                        
                        // Send a session status update to join the existing session
                        // This updates the session with our participant info
                        let session_update = serde_json::json!({
                            "session_id": session_id.clone(),
                            "participant_joined": device_id.clone(),
                        });
                        
                        let status_update_msg = webrtc_signal_server::ClientMsg::SessionStatusUpdate {
                            session_info: session_update,
                        };
                        let status_json = serde_json::to_string(&status_update_msg).unwrap();
                        
                        if let Err(e) = ws_sink.send(tokio_tungstenite::tungstenite::Message::text(status_json)).await {
                            let _ = tx_clone.send(Message::Error { 
                                message: format!("Failed to join session: {}", e)
                            });
                        } else {
                            let _ = tx_clone.send(Message::Info { 
                                message: format!("✅ Joined session: {}", session_id)
                            });
                            
                            // Navigate to DKG Progress after joining
                            let _ = tx_clone.send(Message::UpdateDKGProgress {
                                round: crate::elm::message::DKGRound::WaitingForParticipants,
                                progress: 0.2,
                            });
                            
                            // Store a basic session in app state that will be updated
                            // when we receive SessionAvailable messages from the server
                            // Use the curve type from any existing available sessions or default to Ed25519
                            {
                                let mut state = app_state.lock().await;
                                
                                // Try to find the curve type from available sessions
                                let curve_type = state.available_sessions.iter()
                                    .find(|s| s.session_code == session_id)
                                    .map(|s| s.curve_type.clone())
                                    .unwrap_or_else(|| "Ed25519".to_string());  // Default to Ed25519 as that's what the session was created with
                                
                                info!("📊 Joining session with curve type: {}", curve_type);
                                
                                state.session = Some(crate::protocal::signal::SessionInfo {
                                    session_id: session_id.clone(),
                                    proposer_id: "unknown".to_string(),
                                    participants: vec![device_id.clone()],
                                    threshold: 2,  // Will be updated from SessionAvailable
                                    total: 3,      // Will be updated from SessionAvailable
                                    session_type: crate::protocal::signal::SessionType::DKG,
                                    curve_type,
                                    coordination_type: "Network".to_string(),
                                });
                            }
                        }
                        
                        // Spawn the WebSocket sender task (simple version without WebRTC)
                        tokio::spawn(async move {
                            info!("🚀 WebSocket sender task started (JoinDKG)");

                            // Process string messages and send through WebSocket
                            while let Some(msg) = ws_msg_rx.recv().await {
                                info!("📤 Sending through WebSocket: {}", msg);
                                if let Err(e) = ws_sink.send(tokio_tungstenite::tungstenite::Message::text(msg)).await {
                                    error!("❌ Failed to send through WebSocket: {}", e);
                                } else {
                                    info!("✅ Sent through WebSocket successfully");
                                }
                            }
                            info!("WebSocket sender task stopped");
                        });

                        // Spawn a task to handle incoming WebSocket messages
                        let tx_msg = tx_clone.clone();
                        let session_id_clone = session_id.clone();
                        let _device_id_clone = device_id.clone();
                        let session_total = 3u16;  // Default to 3, will be updated from SessionAvailable
                        
                        // Get session info before spawning
                        let our_session_id = {
                            let state = app_state.lock().await;
                            state.session.as_ref().map(|s| s.session_id.clone())
                        };

                        // Clone app_state for the spawned task
                        let app_state_clone = app_state.clone();

                        tokio::spawn(async move {
                            let mut participants_seen = std::collections::HashSet::new();
                            // Don't add ourselves yet - wait for server to confirm
                            
                            while let Some(msg) = ws_stream.next().await {
                                match msg {
                                    Ok(tokio_tungstenite::tungstenite::Message::Text(txt)) => {
                                        // Try to parse server messages
                                        if let Ok(server_msg) = serde_json::from_str::<webrtc_signal_server::ServerMsg>(&txt) {
                                            match server_msg {
                                                webrtc_signal_server::ServerMsg::SessionAvailable { session_info } => {
                                                    // Check if this is our session being announced/updated
                                                    if let Some(sid) = session_info.get("session_id").and_then(|v| v.as_str()) {
                                                        if sid == session_id_clone {
                                                            // Our session - update full session info
                                                            let curve_type = session_info.get("curve_type")
                                                                .and_then(|v| v.as_str())
                                                                .unwrap_or("Ed25519")
                                                                .to_string();
                                                            
                                                            let _ = tx_msg.send(Message::Info { 
                                                                message: format!("📋 Session update - curve type: {}", curve_type)
                                                            });
                                                            
                                                            // Update the session in app state with correct curve type
                                                            {
                                                                let mut state = app_state_clone.lock().await;
                                                                if let Some(ref mut session) = state.session {
                                                                    session.curve_type = curve_type.clone();
                                                                    
                                                                    // Also update other session fields
                                                                    if let Some(total) = session_info.get("total").and_then(|v| v.as_u64()) {
                                                                        session.total = total as u16;
                                                                    }
                                                                    if let Some(threshold) = session_info.get("threshold").and_then(|v| v.as_u64()) {
                                                                        session.threshold = threshold as u16;
                                                                    }
                                                                }
                                                            }
                                                            
                                                            // Update participants list
                                                            if let Some(participants) = session_info.get("participants").and_then(|v| v.as_array()) {
                                                                let _ = tx_msg.send(Message::Info { 
                                                                    message: format!("📋 Session update - participants: {}", participants.len())
                                                                });
                                                                
                                                                participants_seen.clear();
                                                                for p in participants {
                                                                    if let Some(pid) = p.as_str() {
                                                                        participants_seen.insert(pid.to_string());
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                webrtc_signal_server::ServerMsg::Devices { devices } => {
                                                    let _ = tx_msg.send(Message::Info { 
                                                        message: format!("📡 Connected devices: {:?}", devices)
                                                    });
                                                    
                                                    // Track previous count to detect new participants
                                                    let prev_count = participants_seen.len();
                                                    
                                                    // Count unique participants in our session
                                                    for device in &devices {
                                                        participants_seen.insert(device.clone());
                                                    }
                                                    
                                                    // Send UpdateParticipants message to update the model
                                                    let participants_list: Vec<String> = participants_seen.iter().cloned().collect();
                                                    let _ = tx_msg.send(Message::UpdateParticipants { 
                                                        participants: participants_list.clone() 
                                                    });
                                                    
                                                    let participants_count = participants_seen.len();
                                                    
                                                    let _ = tx_msg.send(Message::Info { 
                                                        message: format!("👥 Current participants: {}/{}", 
                                                            participants_count, session_total)
                                                    });
                                                    
                                                    // Re-initiate WebRTC if we have new participants
                                                    if participants_count > prev_count && participants_count > 1 {
                                                        let _ = tx_msg.send(Message::Info { 
                                                            message: format!("🔄 New participant detected, re-initiating WebRTC with all {} participants", participants_count)
                                                        });
                                                        
                                                        // Get participants list WITHOUT self for WebRTC initiation
                                                        let self_device = _device_id_clone.clone();
                                                        let other_participants: Vec<String> = participants_seen.iter()
                                                            .filter(|p| **p != self_device)
                                                            .cloned()
                                                            .collect();
                                                        
                                                        // Re-initiate WebRTC with OTHER participants only
                                                        let _ = tx_msg.send(Message::InitiateWebRTCWithParticipants {
                                                            participants: other_participants,
                                                        });
                                                    }
                                                    
                                                    if participants_count >= session_total as usize {
                                                        let _ = tx_msg.send(Message::Info { 
                                                            message: "🎉 All participants connected! Starting DKG...".to_string()
                                                        });
                                                        
                                                        // Final WebRTC initiation to ensure all connections
                                                        let _ = tx_msg.send(Message::Info { 
                                                            message: "🔗 Ensuring all peer-to-peer connections are established...".to_string()
                                                        });
                                                        
                                                        // Send with ALL participants to ensure full mesh
                                                        let _ = tx_msg.send(Message::InitiateWebRTCWithParticipants {
                                                            participants: participants_list,
                                                        });
                                                        
                                                        // Schedule mesh verification after a delay
                                                        let tx_verify = tx_msg.clone();
                                                        tokio::spawn(async move {
                                                            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                                                            let _ = tx_verify.send(Message::VerifyMeshConnectivity);
                                                        });
                                                        
                                                        // Update DKG progress
                                                        let _ = tx_msg.send(Message::UpdateDKGProgress {
                                                            round: crate::elm::message::DKGRound::Round1,
                                                            progress: 0.3,
                                                        });
                                                    }
                                                }
                                                webrtc_signal_server::ServerMsg::Relay { from, data } => {
                                                    let _ = tx_msg.send(Message::Info {
                                                        message: format!("📨 Received relay from {}", from)
                                                    });

                                                    // Check if it's a WebRTC signal from another device
                                                    if from != "server" {
                                                        // Handle WebRTC signals (offer/answer/ICE)
                                                        if let Some(msg_type) = data.get("websocket_msg_type").and_then(|v| v.as_str()) {
                                                            if msg_type == "WebRTCSignal" {
                                                                info!("🎯 Received WebRTC signal from {}", from);

                                                                // Check if it's an offer that needs an answer
                                                                if let Some(offer_data) = data.get("Offer") {
                                                                    if let Some(sdp) = offer_data.get("sdp").and_then(|v| v.as_str()) {
                                                                        info!("📥 Received WebRTC offer from {}, need to send answer", from);
                                                                        let _ = tx_msg.send(Message::Info {
                                                                            message: format!("📥 Received WebRTC offer from {}, preparing answer...", from)
                                                                        });

                                                                        // Create and send WebRTC answer
                                                                        let from_device = from.clone();
                                                                        let sdp_string = sdp.to_string();
                                                                        let app_state_for_answer = app_state_clone.clone();
                                                                        let ws_tx_clone = ws_msg_tx.clone();
                                                                        let _self_device_id = device_id.clone();
                                                                        let tx_msg_spawn = tx_msg.clone();

                                                                        tokio::spawn(async move {
                                                                            info!("🎯 Processing WebRTC offer from {}", from_device);

                                                                            // Get or create peer connection for this device
                                                                            let pc = {
                                                                                let state = app_state_for_answer.lock().await;
                                                                                let device_connections_clone = state.device_connections.clone();
                                                                                drop(state); // Release the lock early
                                                                                let mut conns = device_connections_clone.lock().await;
                                                                                if let Some(existing_pc) = conns.get(&from_device) {
                                                                                    existing_pc.clone()
                                                                                } else {
                                                                                    // Create new peer connection for this device
                                                                                    info!("📱 Creating peer connection for {} (to handle offer)", from_device);
                                                                                    let config = webrtc::peer_connection::configuration::RTCConfiguration {
                                                                                        ice_servers: vec![],
                                                                                        ..Default::default()
                                                                                    };

                                                                                    match webrtc::api::APIBuilder::new()
                                                                                        .build()
                                                                                        .new_peer_connection(config)
                                                                                        .await
                                                                                    {
                                                                                        Ok(new_pc) => {
                                                                                            let arc_pc = Arc::new(new_pc);

                                                                                            // Set up handler for incoming data channels
                                                                                            let from_device_dc = from_device.clone();
                                                                                            let tx_msg_dc = tx_msg_spawn.clone();
                                                                                            arc_pc.on_data_channel(Box::new(move |dc: Arc<webrtc::data_channel::RTCDataChannel>| {
                                                                                                let device_id_dc = from_device_dc.clone();
                                                                                                let tx_msg_dc = tx_msg_dc.clone();
                                                                                                Box::pin(async move {
                                                                                                    info!("📂 Incoming data channel from {}: {}", device_id_dc, dc.label());

                                                                                                    // Set up message handlers for the incoming data channel
                                                                                                    let device_id_open = device_id_dc.clone();
                                                                                                    let tx_msg_open = tx_msg_dc.clone();
                                                                                                    dc.on_open(Box::new(move || {
                                                                                                        let device_open = device_id_open.clone();
                                                                                                        let tx_msg_open = tx_msg_open.clone();
                                                                                                        Box::pin(async move {
                                                                                                            info!("📂 Data channel OPENED from {}", device_open);
                                                                                                            
                                                                                                            // Send UI update for data channel open
                                                                                                            let _ = tx_msg_open.send(Message::UpdateParticipantWebRTCStatus {
                                                                                                                device_id: device_open.clone(),
                                                                                                                webrtc_connected: true,
                                                                                                                data_channel_open: true,
                                                                                                            });
                                                                                                        })
                                                                                                    }));

                                                                                                    let device_id_msg = device_id_dc.clone();
                                                                                                    dc.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                                                                                                        let device_msg = device_id_msg.clone();
                                                                                                        Box::pin(async move {
                                                                                                            info!("📥 Received message from {} via data channel: {} bytes",
                                                                                                                device_msg, msg.data.len());
                                                                                                            // TODO: Forward to DKG protocol handler
                                                                                                        })
                                                                                                    }));

                                                                                                    // TODO: Store dc for sending messages back
                                                                                                })
                                                                                            }));

                                                                                            // Set up connection state handler
                                                                                            let device_id_state = from_device.clone();
                                                                                            let tx_msg_state = tx_msg_spawn.clone();
                                                                                            arc_pc.on_peer_connection_state_change(Box::new(move |state: webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState| {
                                                                                                let device_id_state = device_id_state.clone();
                                                                                                let tx_msg_state = tx_msg_state.clone();
                                                                                                Box::pin(async move {
                                                                                                    let is_connected = matches!(state, webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected);
                                                                                                    
                                                                                                    // Send UI update
                                                                                                    let _ = tx_msg_state.send(Message::UpdateParticipantWebRTCStatus {
                                                                                                        device_id: device_id_state.clone(),
                                                                                                        webrtc_connected: is_connected,
                                                                                                        data_channel_open: false, // Will be updated when data channel opens
                                                                                                    });
                                                                                                    
                                                                                                    match state {
                                                                                                        webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected => {
                                                                                                            info!("✅ WebRTC connection ESTABLISHED with {} (from answer)", device_id_state);
                                                                                                        }
                                                                                                        webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Failed => {
                                                                                                            error!("❌ WebRTC connection FAILED with {} (from answer)", device_id_state);
                                                                                                        }
                                                                                                        _ => {
                                                                                                            info!("WebRTC connection state with {} (from answer): {:?}", device_id_state, state);
                                                                                                        }
                                                                                                    }
                                                                                                })
                                                                                            }));

                                                                                            // Set up ICE candidate handler
                                                                                            let device_id_ice = from_device.clone();
                                                                                            let ws_tx_ice = ws_tx_clone.clone();

                                                                                            arc_pc.on_ice_candidate(Box::new(move |candidate: Option<webrtc::ice_transport::ice_candidate::RTCIceCandidate>| {
                                                                                                let device_id_ice = device_id_ice.clone();
                                                                                                let ws_tx_ice = ws_tx_ice.clone();

                                                                                                Box::pin(async move {
                                                                                                    if let Some(candidate) = candidate {
                                                                                                        info!("🧊 Generated ICE candidate for {}", device_id_ice);

                                                                                                        // Send ICE candidate to peer
                                                                                                        let candidate_json = candidate.to_json().unwrap();
                                                                                                        let ice_signal = crate::protocal::signal::WebRTCSignal::Candidate(
                                                                                                            crate::protocal::signal::CandidateInfo {
                                                                                                                candidate: candidate_json.candidate,
                                                                                                                sdp_mid: candidate_json.sdp_mid,
                                                                                                                sdp_mline_index: candidate_json.sdp_mline_index,
                                                                                                            }
                                                                                                        );

                                                                                                        let websocket_message = crate::protocal::signal::WebSocketMessage::WebRTCSignal(ice_signal);

                                                                                                        if let Ok(json_val) = serde_json::to_value(websocket_message) {
                                                                                                            let relay_msg = webrtc_signal_server::ClientMsg::Relay {
                                                                                                                to: device_id_ice.clone(),
                                                                                                                data: json_val,
                                                                                                            };

                                                                                                            if let Ok(json) = serde_json::to_string(&relay_msg) {
                                                                                                                info!("📤 Sending ICE candidate to {} via WebSocket", device_id_ice);
                                                                                                                let _ = ws_tx_ice.send(json);
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                })
                                                                                            }));

                                                                                            conns.insert(from_device.clone(), arc_pc.clone());
                                                                                            arc_pc
                                                                                        }
                                                                                        Err(e) => {
                                                                                            error!("❌ Failed to create peer connection for {}: {}", from_device, e);
                                                                                            return;
                                                                                        }
                                                                                    }
                                                                                }
                                                                            };

                                                                            // Set remote description (the offer)
                                                                            let offer = webrtc::peer_connection::sdp::session_description::RTCSessionDescription::offer(sdp_string).unwrap();

                                                                            if let Err(e) = pc.set_remote_description(offer).await {
                                                                                error!("❌ Failed to set remote description for {}: {}", from_device, e);
                                                                                return;
                                                                            }
                                                                            info!("✅ Set remote description (offer) from {}", from_device);

                                                                            // Process any queued ICE candidates for this peer
                                                                            {
                                                                                let state = app_state_for_answer.lock().await;
                                                                                let ice_queue_clone = state.ice_candidate_queue.clone();
                                                                                drop(state);
                                                                                
                                                                                let mut queue = ice_queue_clone.lock().await;
                                                                                if let Some(candidates) = queue.remove(&from_device) {
                                                                                    info!("📦 Processing {} queued ICE candidates for {}", candidates.len(), from_device);
                                                                                    for candidate in candidates {
                                                                                        if let Err(e) = pc.add_ice_candidate(candidate).await {
                                                                                            error!("❌ Failed to add queued ICE candidate from {}: {}", from_device, e);
                                                                                        } else {
                                                                                            info!("✅ Added queued ICE candidate from {}", from_device);
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }

                                                                            // Create answer
                                                                            match pc.create_answer(None).await {
                                                                                Ok(answer) => {
                                                                                    info!("✅ Created answer for {}", from_device);

                                                                                    // Set local description (the answer)
                                                                                    if let Err(e) = pc.set_local_description(answer.clone()).await {
                                                                                        error!("❌ Failed to set local description for {}: {}", from_device, e);
                                                                                        return;
                                                                                    }
                                                                                    info!("✅ Set local description (answer) for {}", from_device);

                                                                                    // Send answer back via WebSocket
                                                                                    let signal = crate::protocal::signal::WebRTCSignal::Answer(
                                                                                        crate::protocal::signal::SDPInfo { sdp: answer.sdp }
                                                                                    );
                                                                                    let websocket_message = crate::protocal::signal::WebSocketMessage::WebRTCSignal(signal);

                                                                                    match serde_json::to_value(websocket_message) {
                                                                                        Ok(json_val) => {
                                                                                            let relay_msg = webrtc_signal_server::ClientMsg::Relay {
                                                                                                to: from_device.clone(),
                                                                                                data: json_val,
                                                                                            };

                                                                                            match serde_json::to_string(&relay_msg) {
                                                                                                Ok(json) => {
                                                                                                    info!("📤 Sending WebRTC answer to {} via WebSocket", from_device);
                                                                                                    if let Err(e) = ws_tx_clone.send(json) {
                                                                                                        error!("❌ Failed to send answer to {}: {}", from_device, e);
                                                                                                    } else {
                                                                                                        info!("✅ WebRTC answer sent to {}", from_device);
                                                                                                    }
                                                                                                }
                                                                                                Err(e) => {
                                                                                                    error!("❌ Failed to serialize answer for {}: {}", from_device, e);
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                        Err(e) => {
                                                                                            error!("❌ Failed to serialize WebRTC answer for {}: {}", from_device, e);
                                                                                        }
                                                                                    }
                                                                                }
                                                                                Err(e) => {
                                                                                    error!("❌ Failed to create answer for {}: {}", from_device, e);
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                } else if let Some(answer_data) = data.get("Answer") {
                                                                    if let Some(sdp) = answer_data.get("sdp").and_then(|v| v.as_str()) {
                                                                        info!("📥 Received WebRTC answer from {}", from);
                                                                        let _ = tx_msg.send(Message::Info {
                                                                            message: format!("📥 Received WebRTC answer from {}, setting remote description...", from)
                                                                        });

                                                                        // Set remote description with the answer
                                                                        let from_device = from.clone();
                                                                        let sdp_string = sdp.to_string();
                                                                        let app_state_for_remote = app_state_clone.clone();

                                                                        tokio::spawn(async move {
                                                                            info!("🎯 Processing WebRTC answer from {}", from_device);

                                                                            // Get peer connection for this device
                                                                            let state = app_state_for_remote.lock().await;
                                                                            let device_connections_clone = state.device_connections.clone();
                                                                            drop(state); // Release the lock early
                                                                            let conns = device_connections_clone.lock().await;
                                                                            if let Some(pc) = conns.get(&from_device) {
                                                                                // Set remote description (the answer)
                                                                                let answer = webrtc::peer_connection::sdp::session_description::RTCSessionDescription::answer(sdp_string).unwrap();

                                                                                if let Err(e) = pc.set_remote_description(answer).await {
                                                                                    error!("❌ Failed to set remote description (answer) for {}: {}", from_device, e);
                                                                                } else {
                                                                                    info!("✅ Set remote description (answer) from {}, WebRTC connection should be establishing!", from_device);
                                                                                    
                                                                                    // Process any queued ICE candidates for this peer
                                                                                    let state = app_state_for_remote.lock().await;
                                                                                    let ice_queue_clone = state.ice_candidate_queue.clone();
                                                                                    drop(state);
                                                                                    
                                                                                    let mut queue = ice_queue_clone.lock().await;
                                                                                    if let Some(candidates) = queue.remove(&from_device) {
                                                                                        info!("📦 Processing {} queued ICE candidates for {}", candidates.len(), from_device);
                                                                                        for candidate in candidates {
                                                                                            if let Err(e) = pc.add_ice_candidate(candidate).await {
                                                                                                error!("❌ Failed to add queued ICE candidate from {}: {}", from_device, e);
                                                                                            } else {
                                                                                                info!("✅ Added queued ICE candidate from {}", from_device);
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                error!("❌ No peer connection found for {} when receiving answer", from_device);
                                                                            }
                                                                        });
                                                                    }
                                                                } else if let Some(ice_data) = data.get("Candidate") {
                                                                    info!("📥 Received ICE candidate from {}", from);

                                                                    // Parse ICE candidate data
                                                                    if let (Some(candidate), Some(sdp_mid), Some(sdp_mline_index)) = (
                                                                        ice_data.get("candidate").and_then(|v| v.as_str()),
                                                                        ice_data.get("sdpMid").and_then(|v| v.as_str()),
                                                                        ice_data.get("sdpMLineIndex").and_then(|v| v.as_u64())
                                                                    ) {
                                                                        let from_device = from.clone();
                                                                        let candidate_str = candidate.to_string();
                                                                        let sdp_mid_str = sdp_mid.to_string();
                                                                        let sdp_mline_index_u16 = sdp_mline_index as u16;
                                                                        let app_state_for_ice = app_state_clone.clone();

                                                                        tokio::spawn(async move {
                                                                            info!("🎯 Adding ICE candidate from {}", from_device);

                                                                            // Get peer connection for this device
                                                                            let state = app_state_for_ice.lock().await;
                                                                            let device_connections_clone = state.device_connections.clone();
                                                                            let ice_queue_clone = state.ice_candidate_queue.clone();
                                                                            drop(state); // Release the lock early
                                                                            let conns = device_connections_clone.lock().await;
                                                                            if let Some(pc) = conns.get(&from_device) {
                                                                                // Create ICE candidate init
                                                                                let ice_candidate_init = webrtc::ice_transport::ice_candidate::RTCIceCandidateInit {
                                                                                    candidate: candidate_str,
                                                                                    sdp_mid: Some(sdp_mid_str),
                                                                                    sdp_mline_index: Some(sdp_mline_index_u16),
                                                                                    username_fragment: None,
                                                                                };

                                                                                // Check if remote description is set
                                                                                if pc.remote_description().await.is_none() {
                                                                                    // Queue the ICE candidate
                                                                                    let mut queue = ice_queue_clone.lock().await;
                                                                                    queue.entry(from_device.clone())
                                                                                        .or_insert_with(Vec::new)
                                                                                        .push(ice_candidate_init);
                                                                                    info!("📦 Queued ICE candidate from {} (remote description not ready)", from_device);
                                                                                } else {
                                                                                    // Add the ICE candidate immediately
                                                                                    if let Err(e) = pc.add_ice_candidate(ice_candidate_init).await {
                                                                                        error!("❌ Failed to add ICE candidate from {}: {}", from_device, e);
                                                                                    } else {
                                                                                        info!("✅ Added ICE candidate from {}", from_device);
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                error!("❌ No peer connection found for {} when adding ICE candidate", from_device);
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    // Check if it's a participant update from the server
                                                    else if from == "server" {
                                                        if let Some(msg_type) = data.get("type").and_then(|v| v.as_str()) {
                                                            if msg_type == "participant_update" {
                                                                // Handle participant update to trigger WebRTC
                                                                if let (Some(session_id), Some(session_info)) = (
                                                                    data.get("session_id").and_then(|v| v.as_str()),
                                                                    data.get("session_info")
                                                                ) {
                                                                    // Check if this is our session
                                                                    let is_our_session = our_session_id.as_ref()
                                                                        .map(|s| s == session_id)
                                                                        .unwrap_or(false);
                                                                    
                                                                    if is_our_session {
                                                                        // Extract participants from session_info
                                                                        let new_participants = if let Some(participants_arr) = session_info
                                                                            .get("participants")
                                                                            .and_then(|v| v.as_array()) {
                                                                            participants_arr.iter()
                                                                                .filter_map(|v| v.as_str())
                                                                                .filter(|&p| p != device_id) // Filter out self
                                                                                .map(String::from)
                                                                                .collect::<Vec<_>>()
                                                                        } else {
                                                                            Vec::new()
                                                                        };
                                                                        
                                                                        if !new_participants.is_empty() {
                                                                            info!("📡 Received participant update, triggering WebRTC with {} participants", new_participants.len());

                                                                            // Update the session participants in app state
                                                                            let all_participants = {
                                                                                let mut state = app_state_clone.lock().await;

                                                                                // Update session info with all participants (including self)
                                                                                let mut all_parts = new_participants.clone();
                                                                                all_parts.push(device_id.clone());

                                                                                if let Some(ref mut session) = state.session {
                                                                                    session.participants = all_parts.clone();
                                                                                    info!("✅ Updated session participants: {:?}", all_parts);
                                                                                }

                                                                                all_parts
                                                                            };

                                                                            // Send message to trigger WebRTC through update loop
                                                                            info!("🚀 Triggering WebRTC initiation from participant update (JoinDKG)");

                                                                            let _ = tx_msg.send(Message::InitiateWebRTCWithParticipants {
                                                                                participants: all_participants
                                                                            });

                                                                            let _ = tx_msg.send(Message::Info {
                                                                                message: format!("📡 Triggered WebRTC with participants: {:?}", new_participants)
                                                                            });
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                                        let _ = tx_msg.send(Message::Info { 
                                            message: "WebSocket connection closed".to_string()
                                        });
                                        break;
                                    }
                                    Err(e) => {
                                        let _ = tx_msg.send(Message::Error { 
                                            message: format!("WebSocket error: {}", e)
                                        });
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        });
                        
                        // Show initial status
                        let _ = tx_clone.send(Message::Info { 
                            message: format!("⏳ Waiting for other participants to join...")
                        });
                        
                        let _ = tx_clone.send(Message::Info { 
                            message: format!("📋 Session ID: {}", session_id)
                        });
                        
                        // The WebSocket handler will continue running and manage the DKG
                        // Don't send error - let it proceed
                    },
                    Err(e) => {
                        // Send WebSocketDisconnected to update UI state
                        let _ = tx_clone.send(Message::WebSocketDisconnected);
                        
                        // Update app state
                        {
                            let mut state = app_state.lock().await;
                            state.websocket_connected = false;
                            state.websocket_connecting = false;
                        }
                        
                        let _ = tx_clone.send(Message::Info { 
                            message: format!("❌ Cannot reach signal server: {}", e)
                        });
                        let _ = tx_clone.send(Message::DKGFailed { 
                            error: format!("Failed to connect to signal server: {}", e)
                        });
                    }
                }
            }
            
            Command::InitiateWebRTCConnections { participants } => {
                info!("Initiating WebRTC connections with {} participants", participants.len());
                
                // Store participants in app state for WebRTC handler to process
                let (self_device_id, device_connections_arc, _signal_server_url) = {
                    let mut state = app_state.lock().await;
                    // Update session participants
                    if let Some(ref mut session) = state.session {
                        // Merge new participants with existing ones
                        for p in &participants {
                            if !session.participants.contains(p) {
                                session.participants.push(p.clone());
                            }
                        }
                        info!("Updated session participants: {:?}", session.participants);
                    }
                    (state.device_id.clone(), state.device_connections.clone(), state.signal_server_url.clone())
                };
                
                // Send message to trigger WebRTC through the UI
                let _ = tx.send(Message::Info { 
                    message: format!("🚀 WebRTC mesh creation triggered for {} participants", participants.len())
                });
                
                let _ = tx.send(Message::Info { 
                    message: format!("⏳ Starting WebRTC connection process...")
                });
                
                // CRITICAL FIX: Actually initiate WebRTC connections NOW
                info!("🚀 Actually initiating WebRTC for participants: {:?}", participants);

                // Store participant count before moving the vector
                let expected_peer_connections = participants.len() - 1; // Exclude self

                // Call the WebRTC initiation directly with UI message sender
                crate::network::webrtc_simple::simple_initiate_webrtc_with_channel(
                    self_device_id,
                    participants,
                    device_connections_arc,
                    app_state.clone(),
                    Some(tx.clone()),  // Pass the UI message sender
                ).await;

                // Also update DKG progress to show we're connecting
                let _ = tx.send(Message::UpdateDKGProgress {
                    round: crate::elm::message::DKGRound::Round1,
                    progress: 0.35,
                });

                // KISS Fix: Start a simple periodic mesh status checker
                // This polls the connection state every 500ms until mesh is ready
                let tx_mesh = tx.clone();
                let app_state_mesh = app_state.clone();

                tokio::spawn(async move {
                    let mut attempts = 0;
                    const MAX_ATTEMPTS: u32 = 60; // 30 seconds max

                    loop {
                        attempts += 1;
                        if attempts > MAX_ATTEMPTS {
                            let _ = tx_mesh.send(Message::Error {
                                message: "Timeout waiting for WebRTC mesh to be ready".to_string()
                            });
                            break;
                        }

                        // Wait 500ms between checks
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                        // Check if all connections are established and in Connected state
                        let mesh_ready = {
                            let state = app_state_mesh.lock().await;

                            // Check device_connections to see if we have all peer connections
                            let device_connections = state.device_connections.clone();

                            let connections = device_connections.lock().await;
                            let total_connections = connections.len();

                            // Count how many are actually in Connected state
                            let mut connected_count = 0;
                            for (_device_id, pc) in connections.iter() {
                                let connection_state = pc.connection_state();
                                if connection_state == webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected {
                                    connected_count += 1;
                                }
                            }

                            info!("🔍 Mesh check: {}/{} peer connections in Connected state (total connections: {})",
                                  connected_count, expected_peer_connections, total_connections);

                            // Mesh is ready when we have connected to all other participants
                            connected_count >= expected_peer_connections
                        };

                        if mesh_ready {
                            info!("✅ WebRTC mesh is ready! Connected to all {} other participants", expected_peer_connections);

                            // Update UI that mesh is complete
                            let _ = tx_mesh.send(Message::Info {
                                message: "✅ WebRTC mesh established successfully!".to_string()
                            });

                            // Trigger DKG Round 1
                            let _ = tx_mesh.send(Message::Info {
                                message: "🚀 Starting DKG Round 1...".to_string()
                            });

                            // Update progress to show DKG is actually starting
                            let _ = tx_mesh.send(Message::UpdateDKGProgress {
                                round: crate::elm::message::DKGRound::Round1,
                                progress: 0.5,
                            });

                            // Actually start DKG protocol here
                            // Get session info to create wallet config
                            let wallet_config = {
                                let state = app_state_mesh.lock().await;
                                if let Some(ref session) = state.session {
                                    let curve = match session.curve_type.as_str() {
                                        "secp256k1" => crate::elm::model::CurveType::Secp256k1,
                                        "ed25519" => crate::elm::model::CurveType::Ed25519,
                                        _ => crate::elm::model::CurveType::Secp256k1, // default
                                    };

                                    Some(crate::elm::model::WalletConfig {
                                        name: format!("MPC Wallet {}", &session.session_id[..8]),
                                        total_participants: session.total,
                                        threshold: session.threshold,
                                        curve,
                                        mode: crate::elm::model::WalletMode::Online,
                                    })
                                } else {
                                    None
                                }
                            };

                            if let Some(config) = wallet_config {
                                // Trigger actual DKG using InitiateDKG message
                                let _ = tx_mesh.send(crate::elm::message::Message::InitiateDKG {
                                    params: crate::elm::message::DKGParams {
                                        wallet_config: config,
                                        session_id: None,
                                        coordinator: true, // Assume we're coordinator since we're triggering
                                    }
                                });

                                let _ = tx_mesh.send(crate::elm::message::Message::Info {
                                    message: "🚀 Mesh ready! Starting real DKG protocol...".to_string()
                                });
                            } else {
                                // Fallback if no session info available
                                let _ = tx_mesh.send(crate::elm::message::Message::Info {
                                    message: "⚠️ Mesh ready but no session info available for DKG".to_string()
                                });
                            }

                            // Mark that we're ready
                            {
                                let mut state = app_state_mesh.lock().await;
                                state.own_mesh_ready_sent = true;
                            }

                            break;
                        }
                    }
                });
            }
            
            Command::VerifyWebRTCMesh => {
                info!("🔍 Verifying WebRTC mesh connectivity");
                
                let (self_device_id, expected_connections) = {
                    let state = app_state.lock().await;
                    let expected = if let Some(ref session) = state.session {
                        session.participants.len() - 1  // Exclude self
                    } else {
                        0
                    };
                    (state.device_id.clone(), expected)
                };
                
                // Check current connection status
                let connections_status = {
                    let state = app_state.lock().await;
                    let device_connections = state.device_connections.clone();
                    let connections = device_connections.lock().await;
                    
                    let mut status_report = Vec::new();
                    let mut connected_count = 0;
                    let mut failed_count = 0;
                    
                    for (peer_id, pc) in connections.iter() {
                        let conn_state = pc.connection_state();
                        let is_connected = conn_state == webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected;
                        
                        if is_connected {
                            connected_count += 1;
                            status_report.push(format!("✅ {} -> {}: Connected", self_device_id, peer_id));
                        } else {
                            failed_count += 1;
                            status_report.push(format!("❌ {} -> {}: {:?}", self_device_id, peer_id, conn_state));
                        }
                    }
                    
                    (connected_count, failed_count, status_report, connections.len())
                };
                
                let (connected_count, failed_count, status_report, _total_connections) = connections_status;
                
                // Send status report
                let _ = tx.send(Message::Info {
                    message: format!("📊 Mesh Status: {}/{} connected ({} failed)", 
                                   connected_count, expected_connections, failed_count)
                });
                
                for status_line in status_report {
                    info!("{}", status_line);
                }
                
                // If not all connections are established, trigger re-initiation
                if connected_count < expected_connections {
                    warn!("⚠️ Incomplete mesh: only {}/{} connections established", connected_count, expected_connections);
                    
                    // Get participants and re-initiate for missing connections
                    let participants = {
                        let state = app_state.lock().await;
                        if let Some(ref session) = state.session {
                            session.participants.clone()
                        } else {
                            vec![]
                        }
                    };
                    
                    if !participants.is_empty() {
                        let _ = tx.send(Message::Info {
                            message: "🔄 Re-initiating WebRTC for missing connections...".to_string()
                        });
                        
                        let _ = tx.send(Message::InitiateWebRTCWithParticipants {
                            participants: participants.into_iter()
                                .filter(|p| p != &self_device_id)
                                .collect()
                        });
                    }
                } else {
                    let _ = tx.send(Message::Success {
                        message: format!("✅ Full mesh established: {} connections", connected_count)
                    });
                }
            }
            
            Command::EnsureFullMesh => {
                info!("🔗 Ensuring full mesh connectivity");
                
                let (self_device_id, participants) = {
                    let state = app_state.lock().await;
                    let participants = if let Some(ref session) = state.session {
                        session.participants.clone()
                    } else {
                        vec![]
                    };
                    (state.device_id.clone(), participants)
                };
                
                if participants.is_empty() {
                    let _ = tx.send(Message::Warning {
                        message: "No active session to verify mesh for".to_string()
                    });
                    return Ok(());
                }
                
                // Check each expected connection
                let mut missing_connections = Vec::new();
                {
                    let state = app_state.lock().await;
                    let device_connections = state.device_connections.clone();
                    let connections = device_connections.lock().await;
                    
                    for participant in &participants {
                        if participant == &self_device_id {
                            continue;
                        }
                        
                        match connections.get(participant) {
                            Some(pc) => {
                                let conn_state = pc.connection_state();
                                if conn_state != webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected {
                                    info!("⚠️ Connection to {} is in state: {:?}", participant, conn_state);
                                    missing_connections.push(participant.clone());
                                }
                            }
                            None => {
                                info!("❌ No connection exists to {}", participant);
                                missing_connections.push(participant.clone());
                            }
                        }
                    }
                }
                
                if !missing_connections.is_empty() {
                    let _ = tx.send(Message::Warning {
                        message: format!("Missing connections to: {:?}", missing_connections)
                    });
                    
                    // Re-initiate WebRTC for all participants to fix missing connections
                    let _ = tx.send(Message::Info {
                        message: "🔄 Re-establishing WebRTC connections...".to_string()
                    });
                    
                    let _ = tx.send(Message::InitiateWebRTCWithParticipants {
                        participants: participants.into_iter()
                            .filter(|p| p != &self_device_id)
                            .collect()
                    });
                    
                    // Schedule a verification check after a delay
                    let tx_check = tx.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                        let _ = tx_check.send(Message::CheckWebRTCConnections);
                    });
                } else {
                    let _ = tx.send(Message::Success {
                        message: "✅ Full mesh connectivity confirmed".to_string()
                    });
                }
            }
            
            Command::DeleteWallet { wallet_id } => {
                info!("Deleting wallet: {}", wallet_id);
                
                // TODO: Implement wallet deletion in keystore
                // For now, just send an error message
                let _ = tx.send(Message::Error { 
                    message: "Wallet deletion not yet implemented".to_string() 
                });
            }
            
            Command::ConnectWebSocket { url } => {
                info!("Connecting to WebSocket: {}", url);
                // WebSocket connection will be handled by AppRunner
                // Just send a message to indicate connection attempt
                let _ = tx.send(Message::Info { 
                    message: format!("Connecting to {}", url) 
                });
            }
            
            Command::ReconnectWebSocket => {
                info!("Attempting to reconnect WebSocket");
                // Trigger reconnection logic
                let _ = tx.send(Message::Info { 
                    message: "Reconnecting...".to_string() 
                });
            }
            
            Command::SendMessage(msg) => {
                // Forward the message
                let _ = tx.send(msg);
            }
            
            Command::ScheduleMessage { delay_ms, message } => {
                // Schedule a message to be sent after a delay
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    let _ = tx.send(*message);
                });
            }
            
            Command::RefreshUI => {
                // UI refresh handled by the view layer
                info!("UI refresh requested");
            }
            
            Command::Quit => {
                info!("Application quit requested");
                // Send quit message to trigger app shutdown
                let _ = tx.send(Message::Quit);
            }
            
            Command::None => {
                // No operation
            }
            
            _ => {
                info!("Command not yet implemented: {:?}", self);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_creation() {
        let cmd = Command::LoadWallets;
        assert!(matches!(cmd, Command::LoadWallets));
        
        let cmd = Command::StartDKG { 
            config: WalletConfig {
                name: "Test".to_string(),
                total_participants: 3,
                threshold: 2,
                curve: crate::elm::model::CurveType::Secp256k1,
                mode: crate::elm::model::WalletMode::Online,
            }
        };
        assert!(matches!(cmd, Command::StartDKG { .. }));
    }
}