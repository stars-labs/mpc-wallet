// Real session handling implementation that sends SessionResponse

use std::sync::Arc;
use tokio::sync::Mutex;
use frost_core::Ciphersuite;
use crate::utils::appstate_compat::AppState;
use crate::utils::state::InternalCommand;
use crate::protocal::signal::{SessionResponse, SessionType, SessionInfo, SessionAnnouncement};
use webrtc_signal_server::ClientMsg;

/// Configuration for wallet creation session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletSessionConfig {
    pub wallet_name: String,
    pub description: Option<String>,
    pub total: u16,
    pub threshold: u16,
    pub curve_type: String, // "secp256k1" or "ed25519"
    pub mode: WalletCreationMode,
    pub timeout_hours: u8, // Session timeout
    pub auto_discovery: bool, // Enable participant auto-discovery
    pub blockchain_config: Vec<BlockchainConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WalletCreationMode {
    Online,   // Real-time WebRTC coordination
    Offline,  // Air-gapped with file/QR code exchange
    Hybrid,   // Online coordination, offline key generation
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockchainConfig {
    pub blockchain: String,   // "ethereum", "bitcoin", "solana", etc.
    pub network: String,      // "mainnet", "testnet", etc.
    pub enabled: bool,
    pub chain_id: Option<u64>,
}

/// Generate a deterministic session ID based on wallet name
/// This ensures the same wallet name ALWAYS generates the same group address
fn generate_session_id(wallet_name: &str) -> String {
    // CRITICAL: This must be deterministic for the same wallet name
    // to ensure consistent group address generation across all nodes
    // Do NOT add timestamp or random elements here
    
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(b"FROST_SESSION_V1:");
    hasher.update(wallet_name.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(&hash[..16]); // Use 16 bytes for better uniqueness
    
    // Return just the hash for maximum determinism
    // This ensures identical session IDs for the same wallet name
    hash_hex
}

/// Enhanced session proposal handler supporting wallet creation flow
pub async fn handle_propose_wallet_session<C: Ciphersuite + Send + Sync + 'static>(
    session_config: WalletSessionConfig,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
    device_id: String,
) -> Result<(), String>
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    tracing::info!("Creating wallet session {} with {}/{} threshold, mode: {:?}", 
        session_config.wallet_name, session_config.threshold, session_config.total, session_config.mode);
    
    // Validate session configuration
    if session_config.threshold > session_config.total {
        return Err("Threshold cannot exceed total participants".to_string());
    }
    
    if session_config.threshold == 0 || session_config.total == 0 {
        return Err("Threshold and total must be greater than 0".to_string());
    }
    
    // Create session in state with enhanced metadata
    let mut state = app_state.lock().await;
    let session_id = generate_session_id(&session_config.wallet_name);
    
    state.session = Some(SessionInfo {
        session_id: session_id.clone(),
        proposer_id: device_id.clone(),
        total: session_config.total,
        threshold: session_config.threshold,
        participants: vec![device_id.clone()],
        accepted_devices: vec![device_id.clone()],
        session_type: SessionType::DKG,
        curve_type: session_config.curve_type.clone(),
        coordination_type: match session_config.mode {
            WalletCreationMode::Online => "network".to_string(),
            WalletCreationMode::Offline => "airgapped".to_string(),
            WalletCreationMode::Hybrid => "hybrid".to_string(),
        },
    });
    
    // Store wallet creation metadata
    state.wallet_creation_config = Some(session_config.clone());
    state.session_start_time = Some(std::time::Instant::now());
    
    // Session created with configuration
    
    drop(state);
    
    // Always announce session for discovery (even for offline/hybrid modes)
    // This allows other nodes to see and join the session
    let session_announcement = create_session_announcement(&session_id, &session_config, &device_id);
    
    tracing::info!("📢 Announcing session '{}' to network", session_id);
    
    let serialized_announcement = serde_json::to_value(&session_announcement)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;
    
    // Send both AnnounceSession and a Relay broadcast for compatibility
    let send_result = internal_cmd_tx.send(InternalCommand::SendToServer(
        ClientMsg::AnnounceSession { 
            session_info: serialized_announcement.clone()
        }
    ));
    
    // Also send as a broadcast relay to all devices
    let session_proposal = crate::protocal::signal::SessionProposal {
        session_id: session_id.clone(),
        total: session_config.total,
        threshold: session_config.threshold,
        participants: vec![device_id.clone()],
        session_type: crate::protocal::signal::SessionType::DKG,
        proposer_device_id: device_id.clone(),
        curve_type: session_config.curve_type.clone(),
        coordination_type: match session_config.mode {
            WalletCreationMode::Online => "network".to_string(),
            WalletCreationMode::Offline => "airgapped".to_string(),
            WalletCreationMode::Hybrid => "hybrid".to_string(),
        },
    };
    
    // Wrap in WebSocketMessage for proper deserialization
    let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionProposal(session_proposal);
    
    let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
        ClientMsg::Relay {
            to: "*".to_string(), // Broadcast to all
            data: serde_json::to_value(&websocket_msg).unwrap_or(serde_json::Value::Null),
        }
    ));
    
    match send_result {
        Ok(_) => tracing::info!("✅ Session announcement queued for broadcast"),
        Err(_e) => {
            tracing::error!("❌ Failed to queue session announcement: {}", _e);
            return Err(format!("Failed to announce session: {}", _e));
        }
    }
    
    // Also trigger session discovery to ensure other nodes are aware
    let _ = internal_cmd_tx.send(InternalCommand::DiscoverSessions);
    
    // Handle different coordination modes
    match session_config.mode {
        WalletCreationMode::Online => {
            handle_online_session_creation(session_id, session_config, internal_cmd_tx, device_id).await
        },
        WalletCreationMode::Offline => {
            handle_offline_session_creation(session_id, session_config, app_state, device_id).await
        },
        WalletCreationMode::Hybrid => {
            handle_hybrid_session_creation(session_id, session_config, internal_cmd_tx, device_id).await
        },
    }
}

/// Legacy compatibility wrapper
pub async fn handle_propose_session<C: Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    total: u16,
    threshold: u16,
    _participants: Vec<String>,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
    device_id: String,
) where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    tracing::info!("Creating session {} with {}/{} threshold", session_id, threshold, total);
    
    // Create session in state
    let mut state = app_state.lock().await;
    state.session = Some(SessionInfo {
        session_id: session_id.clone(),
        proposer_id: device_id.clone(),
        total,
        threshold,
        participants: vec![device_id.clone()],
        accepted_devices: vec![device_id.clone()],
        session_type: SessionType::DKG,
        curve_type: "secp256k1".to_string(),
        coordination_type: "network".to_string(),
    });
    drop(state);
    
    // Broadcast session availability
    let session_info = serde_json::json!({
        "creator_device": device_id,
        "curve_type": "secp256k1",
        "description": null,
        "participants_joined": 1,
        "session_code": session_id,
        "threshold": threshold,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "total": total,
        "wallet_type": "Business Wallet",
    });
    
    // Send announcement through internal command channel
    let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
        ClientMsg::AnnounceSession { session_info }
    ));
}

/// Handle accepting a session proposal - send SessionResponse back
pub async fn handle_accept_session_proposal<C: Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
) where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    // Use the enhanced rejoin-aware handler
    crate::handlers::session_rejoin::handle_accept_session_with_rejoin(
        session_id,
        app_state,
        internal_cmd_tx
    ).await;
    return;
    
    // Original code below (kept for reference but bypassed)
    #[allow(unreachable_code)]
    let mut state = app_state.lock().await;
    let device_id = state.device_id.clone();
    
    // If we don't have a session yet (we're a joiner), create it from the invite
    if state.session.is_none() {
        // Find the invite for this session
        let invite_clone = state.invites.iter()
            .find(|i| i.session_id == session_id)
            .cloned();
        
        if let Some(invite) = invite_clone {
            // Session invitation accepted
            // The invite should already have the correct proposer_id from the SessionProposal
            state.session = Some(invite);
        } else {
            drop(state);
            return;
        }
    }
    
    // Update session state to mark ourselves as accepted
    let (should_send_response, proposer_id) = if let Some(ref mut session) = state.session {
        if session.session_id == session_id {
            // Add ourselves to both accepted_devices AND participants if not already there
            let added_to_accepted = if !session.accepted_devices.contains(&device_id) {
                session.accepted_devices.push(device_id.clone());
                true
            } else {
                false
            };
            
            // CRITICAL: Also add ourselves to participants for WebRTC mesh formation
            let added_to_participants = if !session.participants.contains(&device_id) {
                session.participants.push(device_id.clone());
                true
            } else {
                false
            };
            
            let proposer_id = session.proposer_id.clone();
            let _participants_info = session.participants.clone();
            let _accepted_devices_info = session.accepted_devices.clone();
            
            // Release the mutable borrow - no explicit drop needed for owned values
            
            // Log after we've finished borrowing the session
            if added_to_participants {
            }
            
            (true, Some((proposer_id, added_to_accepted)))
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };
    
    if let Some((proposer_id, added)) = proposer_id {
        if added {
        }
        drop(state);
        
        if should_send_response {
            
            // Send SessionResponse to the proposer
            let response = SessionResponse {
                session_id: session_id.clone(),
                from_device_id: device_id.clone(),
                accepted: true,
                wallet_status: None,
                reason: None,
            };
            
            // Properly wrap in WebSocketMessage and serialize
            let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionResponse(response);
            let msg = serde_json::to_value(&websocket_msg)
                .map_err(|e| format!("Failed to serialize SessionResponse: {}", e))
                .unwrap_or(serde_json::Value::Null);
            
            // Send SessionResponse through internal command channel
            let result = internal_cmd_tx.send(InternalCommand::SendToServer(
                ClientMsg::Relay {
                    to: proposer_id.clone(),
                    data: msg.clone(),
                }
            ));
            
            // Get state again to log
            let mut state = app_state.lock().await;
            match result {
                Ok(_) => {
                },
                Err(_e) => {
                },
            }
            drop(state);
            
            // Now initiate WebRTC connections with other participants
            let _ = internal_cmd_tx.send(InternalCommand::InitiateWebRTCConnections);
        }
    } else {
        drop(state);
    }
}

/// Handle processing a session response from another participant
pub async fn handle_process_session_response<C: Ciphersuite + Send + Sync + 'static>(
    from_device_id: String,
    response: SessionResponse,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
) where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    // Use the enhanced rejoin-aware handler
    crate::handlers::session_rejoin::handle_process_session_response_with_rejoin(
        from_device_id,
        response,
        app_state,
        internal_cmd_tx
    ).await;
    return;
    
    // Original code below (kept for reference but bypassed)
    #[allow(unreachable_code)]
    let mut state = app_state.lock().await;
    
    if !response.accepted {
        drop(state);
        return;
    }
    
    
    // Extract device_id before mutable borrow
    let device_id = state.device_id.clone();
    
    if let Some(ref mut session) = state.session {
        if session.session_id == response.session_id {
            // Add the accepting device to accepted_devices if not already there
            let was_added_to_accepted = if !session.accepted_devices.contains(&from_device_id) {
                session.accepted_devices.push(from_device_id.clone());
                true
            } else {
                false
            };
            
            // CRITICAL: Also add the responding device to participants for WebRTC mesh formation
            let was_added_to_participants = if !session.participants.contains(&from_device_id) {
                session.participants.push(from_device_id.clone());
                true
            } else {
                false
            };
            
            // Extract values we need before dropping the mutable reference
            let session_id = session.session_id.clone();
            let accepted_devices = session.accepted_devices.clone();
            let participants = session.participants.clone();
            let total = session.total;
            let accepted_count = accepted_devices.len();
            
            // Log the update (extract values first to avoid borrowing conflicts)
            // Release the mutable borrow before logging - no explicit drop needed for owned values
            if was_added_to_accepted {
            }
            if was_added_to_participants {
            }
            if was_added_to_accepted || was_added_to_participants {
                // Update accepted devices list
            }
            
            // Broadcast the updated accepted_devices list to all participants
            // Create a proper SessionUpdate struct that matches the protocol
            let update = crate::protocal::signal::SessionUpdate {
                session_id: session_id.clone(),
                accepted_devices: accepted_devices.clone(),
                update_type: crate::protocal::signal::SessionUpdateType::ParticipantJoined,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            
            // Wrap it in WebSocketMessage
            let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionUpdate(update);
            
            // Send update to all participants except ourselves (the session creator)
            // IMPORTANT: We MUST send to the newly joined participant (from_device_id) so they know about others!
            // Update sent to participants
            
            for participant in &participants {
                // Only exclude ourselves (the creator), NOT the newly joined participant
                if participant != &device_id {
                    
                    let send_result = internal_cmd_tx.send(InternalCommand::SendToServer(
                        webrtc_signal_server::ClientMsg::Relay {
                            to: participant.clone(),
                            data: serde_json::to_value(&websocket_msg).unwrap_or(serde_json::Value::Null),
                        }
                    ));
                    
                    match send_result {
                        Ok(_) => {},
                        Err(_e) => tracing::error!("Failed to send session proposal: {}", _e),
                    }
                }
            }
            
            // Check if we have enough participants for the session (based on total, not participants list)
            if accepted_count as u16 >= total {
                // Only initiate WebRTC if we haven't already started
                if state.mesh_status == crate::utils::state::MeshStatus::Incomplete {
                    state.mesh_status = crate::utils::state::MeshStatus::WebRTCInitiated;
                    drop(state);
                    
                    // Initiate WebRTC connections with all participants
                    let _ = internal_cmd_tx.send(InternalCommand::InitiateWebRTCConnections);
                } else {
                    let _status = state.mesh_status.clone();
                    drop(state);
                }
            } else {
                // All participants have joined
            }
        }
    }
}

/// Handle online session creation with WebRTC mesh networking
async fn handle_online_session_creation<C: Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    config: WalletSessionConfig,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
    _device_id: String,
) -> Result<(), String> {
    // Session already announced in the main handler
    
    // Start participant discovery if enabled
    if config.auto_discovery {
        let _ = internal_cmd_tx.send(InternalCommand::StartParticipantDiscovery {
            session_id: session_id.clone(),
            required_participants: config.total,
        });
    }
    
    Ok(())
}

/// Handle offline session creation with file-based coordination
async fn handle_offline_session_creation<C: Ciphersuite + Send + Sync + 'static>(
    _session_id: String,
    _config: WalletSessionConfig,
    app_state: Arc<Mutex<AppState<C>>>,
    _device_id: String,
) -> Result<(), String> {
    let _state = app_state.lock().await;
    
    // TODO: Implement proper offline session creation
    // For now, just log that offline mode is selected
    
    Ok(())
}

/// Handle hybrid session creation with online coordination, offline key generation
async fn handle_hybrid_session_creation<C: Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    config: WalletSessionConfig,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
    device_id: String,
) -> Result<(), String> {
    // First, establish online coordination
    handle_online_session_creation(session_id.clone(), config.clone(), internal_cmd_tx.clone(), device_id).await?;
    
    // Set hybrid mode flag for DKG execution
    let _ = internal_cmd_tx.send(InternalCommand::SetDkgMode(crate::protocal::dkg::DkgMode::Hybrid));
    
    Ok(())
}

/// Create session announcement for broadcasting
fn create_session_announcement(session_id: &str, config: &WalletSessionConfig, device_id: &str) -> SessionAnnouncement {
    SessionAnnouncement {
        session_code: session_id.to_string(),
        wallet_type: config.wallet_name.clone(),
        threshold: config.threshold,
        total: config.total,
        curve_type: config.curve_type.clone(),
        creator_device: device_id.to_string(),
        participants_joined: 1,
        description: config.description.clone(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

/// Session discovery handler for finding available DKG sessions
pub async fn handle_session_discovery<C: Ciphersuite + Send + Sync + 'static>(
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
) -> Result<Vec<SessionAnnouncement>, String>
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state = app_state.lock().await;
    
    // Request session list from signaling server
    tracing::info!("📤 Requesting active sessions from server");
    let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
        ClientMsg::RequestActiveSessions
    ));
    
    // Return currently known sessions
    let sessions = state.available_sessions.clone();
    
    Ok(sessions)
}

/// Start participant discovery for a wallet creation session
pub async fn handle_start_participant_discovery<C: Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    required_participants: u16,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
) {
    let mut state = app_state.lock().await;
    
    // Update progress
    if let Some(ref mut progress) = state.wallet_creation_progress {
        progress.stage = WalletCreationStage::ParticipantDiscovery;
        progress.current_step = 2;
        progress.message = format!("Discovering participants ({} required)...", required_participants);
    }
    
    // Starting participant discovery for session
    
    drop(state);
    
    // Send discovery message to signaling server
    let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
        webrtc_signal_server::ClientMsg::Relay {
            to: "*".to_string(), // Broadcast to all
            data: serde_json::json!({
                "type": "session_discovery",
                "session_id": session_id,
                "required_participants": required_participants,
            }),
        }
    ));
    
    // Also trigger session discovery
    let _ = internal_cmd_tx.send(InternalCommand::DiscoverSessions);
}

impl std::fmt::Display for WalletCreationStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletCreationStage::Configuration => write!(f, "Configuration"),
            WalletCreationStage::ParticipantDiscovery => write!(f, "Participant Discovery"),
            WalletCreationStage::MeshFormation => write!(f, "Mesh Formation"),
            WalletCreationStage::DkgRound1 => write!(f, "DKG Round 1"),
            WalletCreationStage::DkgRound2 => write!(f, "DKG Round 2"),
            WalletCreationStage::Finalization => write!(f, "Finalization"),
            WalletCreationStage::Complete => write!(f, "Complete"),
            WalletCreationStage::Failed => write!(f, "Failed"),
        }
    }
}

/// Progress update handler for UI state synchronization
pub async fn handle_progress_update<C: Ciphersuite + Send + Sync + 'static>(
    app_state: Arc<Mutex<AppState<C>>>,
    progress: WalletCreationProgress,
) {
    let mut state = app_state.lock().await;
    state.wallet_creation_progress = Some(progress.clone());
    
    // Log progress update
    let _status_msg = match progress.stage {
        WalletCreationStage::Configuration => "⚙️ Configuring wallet parameters",
        WalletCreationStage::ParticipantDiscovery => "👥 Discovering participants",
        WalletCreationStage::MeshFormation => "🌐 Establishing secure connections",
        WalletCreationStage::DkgRound1 => "🔑 Generating cryptographic commitments",
        WalletCreationStage::DkgRound2 => "🔐 Distributing key shares",
        WalletCreationStage::Finalization => "✨ Finalizing wallet creation",
        WalletCreationStage::Complete => "✅ Wallet created successfully",
        WalletCreationStage::Failed => "❌ Wallet creation failed",
    };
    
    // Progress updated
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletCreationProgress {
    pub stage: WalletCreationStage,
    pub current_step: u8,
    pub total_steps: u8,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WalletCreationStage {
    Configuration,
    ParticipantDiscovery,
    MeshFormation,
    DkgRound1,
    DkgRound2,
    Finalization,
    Complete,
    Failed,
}