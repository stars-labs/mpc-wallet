use crate::utils::appstate_compat::AppState;
use crate::utils::state::InternalCommand;
use crate::protocal::signal::SessionResponse;
use frost_core::Ciphersuite;
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc_signal_server::ClientMsg;

/// Handle device disconnection - clean up session state
pub async fn handle_device_disconnected<C: Ciphersuite>(
    device_id: String,
    app_state: Arc<Mutex<AppState<C>>>,
) {
    let mut state = app_state.lock().await;
    
    // Remove from participants and participants
    if let Some(ref mut session) = state.session {
        // Remove from accepted devices
        session.participants.retain(|d| d != &device_id);
        
        // Remove from participants but keep in invite list for potential rejoin
        // Don't remove from participants immediately - mark as disconnected instead
        
        // Device disconnected, can rejoin
        
        // Remove data channels and connection state
        state.data_channels.remove(&device_id);
        state.device_statuses.remove(&device_id);
        
        // Clear any mesh ready status since mesh is now incomplete
        state.mesh_status = crate::utils::state::MeshStatus::Incomplete;
    }
}

/// Enhanced session acceptance that handles rejoining scenarios
pub async fn handle_accept_session_with_rejoin<C: Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
) where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut state = app_state.lock().await;
    let device_id = state.device_id.clone();
    
    
    // Check if this is a rejoin scenario
    let is_rejoin = if let Some(ref session) = state.session {
        // If we already have this session but we're not in participants, it's a rejoin
        session.session_id == session_id && !session.participants.contains(&device_id)
    } else {
        false
    };
    
    if is_rejoin {
        
        // CRITICAL: Force close all old WebRTC connections before rejoin
        // Get the connections Arc to close them
        let connections_arc = state.device_connections.clone();
        {
            let mut conns = connections_arc.lock().await;
            // Close each connection properly
            let peer_ids: Vec<String> = conns.keys().cloned().collect();
            for peer_id in peer_ids {
                if let Some(conn) = conns.get(&peer_id) {
                    let _ = conn.close().await;
                }
            }
            conns.clear();
        }
        
        // Clear all WebRTC-related state
        state.data_channels.clear();
        state.device_statuses.clear();
        state.pending_ice_candidates.clear();
        state.making_offer.clear();
        state.mesh_status = crate::utils::state::MeshStatus::Incomplete;
        
        // Reset DKG state for fresh start
        state.dkg_state = crate::utils::state::DkgState::Idle;
        state.received_dkg_packages.clear();
        state.received_dkg_round2_packages.clear();
    }
    
    // Process the session acceptance (create from invite if needed)
    if state.session.is_none() {
        // Find the invite for this session
        let invite_clone = state.invites.iter()
            .find(|i| i.session_id == session_id)
            .cloned();
        
        if let Some(invite) = invite_clone {
            state.session = Some(invite);
        } else {
            drop(state);
            return;
        }
    }
    
    // Update session state - ALWAYS add to participants for rejoin
    let (proposer_id, should_send_update, log_messages) = if let Some(ref mut session) = state.session {
        if session.session_id == session_id {
            let mut logs = Vec::new();
            
            // Force add to participants (handles rejoin case)
            if !session.participants.contains(&device_id) {
                session.participants.push(device_id.clone());
                logs.push(format!("✅ Added {} to participants (rejoin)", device_id));
            }
            
            // Also ensure we're in participants list
            if !session.participants.contains(&device_id) {
                session.participants.push(device_id.clone());
                logs.push(format!("✅ Added {} to participants (rejoin)", device_id));
            }
            
            (session.proposer_id.clone(), true, logs)
        } else {
            (String::new(), false, Vec::new())
        }
    } else {
        (String::new(), false, Vec::new())
    };
    
    // Add log messages after releasing the borrow
    for _log_msg in log_messages {
    }
    
    // Clear the joining_session_id flag now that we've successfully joined
    state.joining_session_id = None;
    
    drop(state);
    
    if should_send_update {
        // Send acceptance response
        let response = SessionResponse {
            session_id: session_id.clone(),
            from_device_id: device_id.clone(),
            accepted: true,
            wallet_status: None,
            reason: if is_rejoin { 
                Some("Rejoining session".to_string()) 
            } else { 
                None 
            },
        };
        
        // Send to proposer - MUST wrap in WebSocketMessage
        let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionResponse(response);
        if let Ok(response_json) = serde_json::to_value(&websocket_msg) {
            let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
                ClientMsg::Relay {
                    to: proposer_id.clone(),
                    data: response_json,
                }
            ));
        }
        
        // Also broadcast a SessionUpdate to notify all participants of the rejoin
        if is_rejoin {
            broadcast_rejoin_update(
                session_id,
                device_id,
                app_state.clone(),
                internal_cmd_tx.clone()
            ).await;
        }
    }
}

/// Broadcast a rejoin update to all session participants
async fn broadcast_rejoin_update<C: Ciphersuite>(
    session_id: String,
    rejoining_device: String,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
) {
    let state = app_state.lock().await;
    
    if let Some(ref session) = state.session {
        let update = crate::protocal::signal::SessionUpdate {
            session_id: session_id.clone(),
            update_type: crate::protocal::signal::SessionUpdateType::ParticipantRejoined,
            participants: session.participants.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        let _participants_count = session.participants.len();
        
        // Send update to all other participants - MUST wrap in WebSocketMessage
        let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionUpdate(update);
        for participant in &session.participants {
            if participant != &rejoining_device {
                if let Ok(update_json) = serde_json::to_value(&websocket_msg) {
                    let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
                        ClientMsg::Relay {
                            to: participant.clone(),
                            data: update_json,
                        }
                    ));
                }
            }
        }
        
        // Device rejoining session
    }
}

/// Enhanced session response handler that properly handles rejoins
pub async fn handle_process_session_response_with_rejoin<C: Ciphersuite + Send + Sync + 'static>(
    from_device_id: String,
    response: SessionResponse,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<InternalCommand<C>>,
) where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut state = app_state.lock().await;
    
    // Check if this is a rejoin
    let is_rejoin = response.reason.as_ref()
        .map(|r| r.contains("rejoin"))
        .unwrap_or(false);
    
    if is_rejoin {
        // Device is rejoining
        
        // Clear any stale connection state for this device
        {
            let mut conns = state.device_connections.lock().await;
            conns.remove(&from_device_id);
        }
        state.data_channels.remove(&from_device_id);
        state.device_statuses.remove(&from_device_id);
    } else {
        // New device joining
    }
    
    if !response.accepted {
        drop(state);
        return;
    }
    
    // Update session state - force update for rejoins
    let log_message = if let Some(ref mut session) = state.session {
        if session.session_id == response.session_id {
            // For rejoins, ensure the device is properly added back
            if is_rejoin || !session.participants.contains(&from_device_id) {
                // Remove first if rejoin to ensure clean state
                if is_rejoin {
                    session.participants.retain(|d| d != &from_device_id);
                    session.participants.retain(|d| d != &from_device_id);
                }
                
                // Add back to both lists
                session.participants.push(from_device_id.clone());
                session.participants.push(from_device_id.clone());
                
                let log_msg = format!(
                    "✅ {} successfully {} session {} ({}/{})",
                    from_device_id,
                    if is_rejoin { "rejoined" } else { "joined" },
                    response.session_id,
                    session.participants.len(),
                    session.total
                );
                Some(log_msg)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Add log message if we have one
    if let Some(_log_msg) = log_message {
    }
    
    // Prepare session update
    let session_update_data = if let Some(ref session) = state.session {
        if session.session_id == response.session_id {
            Some((
                crate::protocal::signal::SessionUpdate {
                    session_id: session.session_id.clone(),
                    update_type: if is_rejoin {
                        crate::protocal::signal::SessionUpdateType::ParticipantRejoined
                    } else {
                        crate::protocal::signal::SessionUpdateType::ParticipantJoined
                    },
                    participants: session.participants.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                },
                session.participants.clone(),
            ))
        } else {
            None
        }
    } else {
        None
    };
    
    // Get device_id before dropping state for WebRTC (not currently used but may be needed)
    let _self_device_id = state.device_id.clone();
    drop(state);
    
    // Send update to all participants if we have one - MUST wrap in WebSocketMessage
    if let Some((session_update, participants)) = session_update_data {
        // KISS: No need to update server - it just stores announcements
        // Clients handle join/rejoin logic themselves
        
        let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionUpdate(session_update.clone());
        
        // CRITICAL FIX: Send update to ALL participants INCLUDING the newly joined device
        // This ensures the newly joined device learns about all other participants
        for participant in &participants {
            // Send to everyone (including from_device_id) so they all have complete state
            if let Ok(update_json) = serde_json::to_value(&websocket_msg) {
                let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
                    ClientMsg::Relay {
                        to: participant.clone(),
                        data: update_json,
                    }
                ));
            }
        }
        
        // IMPORTANT: Also send directly to the joining device if not in participants yet
        // This handles the case where from_device_id just joined and needs the full state
        if !participants.contains(&from_device_id) {
            if let Ok(update_json) = serde_json::to_value(&websocket_msg) {
                tracing::info!("📤 Sending SessionUpdate to newly joined device {}", from_device_id);
                let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
                    ClientMsg::Relay {
                        to: from_device_id.clone(),
                        data: update_json,
                    }
                ));
            }
        }
    }
    
    // Trigger WebRTC connections for both join and rejoin
    // CRITICAL: Must always trigger WebRTC connections when session updates
    // Get fresh participants list from state for WebRTC
    let state = app_state.lock().await;
    let participants = state.session.as_ref().map(|s| s.participants.clone()).unwrap_or_default();
    let device_id = state.device_id.clone();
    let device_connections_arc = state.device_connections.clone();
    drop(state);

    // Use the simplified WebRTC initiation with AppState
    crate::network::webrtc_simple::simple_initiate_webrtc_with_channel(
        device_id,
        participants,
        device_connections_arc,
        app_state.clone(),
        None,  // No UI message sender available here
    ).await;
}