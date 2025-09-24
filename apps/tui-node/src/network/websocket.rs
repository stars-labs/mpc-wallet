use futures_util::SinkExt;
use crate::utils::appstate_compat::AppState;
use crate::utils::state::InternalCommand;
use crate::protocal::signal::{SessionInfo, SessionResponse};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::Message;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc_signal_server::{ServerMsg, ClientMsg};
use crate::network::webrtc::handle_webrtc_signal;
use frost_core::Ciphersuite;
/// Handler for WebSocket messages received from the server
pub async fn handle_websocket_message<C>(
    msg: Message,
    state: Arc<Mutex<AppState<C>>>,
    self_device_id: String,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    device_connections_arc: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    ws_sink: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar:
        Send + Sync,
{
    match msg {
        Message::Text(txt) => {
            match serde_json::from_str::<ServerMsg>(&txt) {
                Ok(server_msg) => {
                    match server_msg {
                        ServerMsg::SessionsForDevice { .. } => {
                            // Handled in app_runner - just skip here
                            tracing::info!("SessionsForDevice message (handled in app_runner)");
                        }
                        ServerMsg::SessionRemoved { .. } => {
                            // Handled in app_runner - just skip here
                            tracing::info!("SessionRemoved message (handled in app_runner)");
                        }
                        ServerMsg::Devices { devices } => {
                            let mut state_guard = state.lock().await;
                            let old_devices = state_guard.devices.clone();
                            state_guard.devices = devices.clone();
                            
                            // Collect devices to reconnect (do this before dropping the lock)
                            let mut devices_to_reconnect = Vec::new();
                            
                            // Check for devices that went offline (were in old_devices but not in new)
                            if let Some(ref mut session) = state_guard.session {
                                let participants = session.participants.clone();
                                
                                // First, check for disconnected devices
                                for old_device in &old_devices {
                                    if participants.contains(old_device) && !devices.contains(old_device) {
                                        // Device went offline - cleaning up connection
                                        
                                        // Clean up the connection
                                        state_guard.device_statuses.insert(
                                            old_device.clone(),
                                            webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Disconnected
                                        );
                                        
                                        // Remove data channel
                                        state_guard.data_channels.remove(old_device);
                                        
                                        // Mark mesh as incomplete
                                        if state_guard.mesh_status == crate::utils::state::MeshStatus::Ready {
                                            state_guard.mesh_status = crate::utils::state::MeshStatus::Incomplete;
                                        }
                                    }
                                }
                                
                                // Then check for reconnected devices (devices that were offline and are now back)
                                for device in &devices {
                                    // Check if this device is part of our session and was previously offline
                                    if participants.contains(device) && !old_devices.contains(device) {
                                        // Device came back online - will attempt reconnection
                                        
                                        // Mark device for reconnection
                                        state_guard.device_statuses.insert(
                                            device.clone(), 
                                            webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Disconnected
                                        );
                                        
                                        // Add to reconnection list
                                        devices_to_reconnect.push(device.clone());

                                    }
                                }
                            }
                            
                            // Get self device ID before dropping lock
                            let self_device_id = state_guard.device_id.clone();
                            drop(state_guard);
                            
                            // Now handle reconnections outside the lock
                            for device_to_reconnect in devices_to_reconnect {
                                tracing::info!("🔄 Triggering WebRTC reconnection for {}", device_to_reconnect);
                                
                                // Clear the old connection if it exists and ensure proper cleanup
                                {
                                    let mut conns = device_connections_arc.lock().await;
                                    if let Some(old_conn) = conns.remove(&device_to_reconnect) {
                                        // Close the connection gracefully
                                        let _ = old_conn.close().await;
                                        tracing::info!("🔄 Closed old connection for {}", device_to_reconnect);
                                        
                                        // Wait a bit to ensure cleanup completes
                                        drop(conns);
                                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                    }
                                }
                                
                                // Also clear any stale state from AppState
                                {
                                    let mut state_guard = state.lock().await;
                                    // Reset device status to allow fresh connection
                                    state_guard.device_statuses.insert(
                                        device_to_reconnect.clone(),
                                        webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::New
                                    );
                                    // Clear any pending WebRTC flags that might block reconnection
                                    if state_guard.webrtc_initiation_in_progress {
                                        // Clearing WebRTC initiation flag for reconnection
                                        state_guard.webrtc_initiation_in_progress = false;
                                        state_guard.webrtc_initiation_started_at = None;
                                    }
                                }
                                
                                // Re-initiate WebRTC connection with just this device
                                crate::network::webrtc::initiate_webrtc_connections(
                                    vec![device_to_reconnect.clone()],
                                    self_device_id.clone(),
                                    state.clone(),
                                    internal_cmd_tx.clone(),
                                ).await;
                            }
                        }
                        ServerMsg::Error { error } => {
                            tracing::error!("Server error: {}", error);
                        }
                        ServerMsg::SessionAvailable { session_info } => {
                            // Parse and add to available sessions
                            match serde_json::from_value::<crate::protocal::signal::SessionAnnouncement>(session_info) {
                                Ok(announcement) => {
                                    let mut state_guard = state.lock().await;
                                    
                                    // Update or add the session
                                    if let Some(existing) = state_guard.available_sessions.iter_mut()
                                        .find(|s| s.session_code == announcement.session_code) {
                                        *existing = announcement;
                                    } else {
                                        state_guard.available_sessions.push(announcement);
                                    }
                                    
                                }
                                Err(_e) => {
                                }
                            }
                        }
                        ServerMsg::SessionListRequest { from } => {
                            // ANY participant with an active session should respond with session info
                            // This ensures sessions persist even if the original creator leaves
                            let state_guard = state.lock().await;
                            if let Some(session) = &state_guard.session {
                                // Check if we're an accepted participant (not just the creator)
                                if session.participants.contains(&state_guard.device_id) {
                                    // We are a participant, send our session announcement
                                    tracing::info!("📢 Responding to SessionListRequest from {} as participant", from);
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
                                        // Use original creator, but any participant can announce
                                        creator_device: session.proposer_id.clone(),
                                        participants_joined: session.participants.len() as u16,
                                        description: None,
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs(),
                                    };
                                    
                                    // Send back to the requester
                                    if let Ok(announcement_json) = serde_json::to_value(&announcement) {
                                        let _ = internal_cmd_tx.send(InternalCommand::SendToServer(
                                            ClientMsg::Relay {
                                                to: from,
                                                data: announcement_json,
                                            }
                                        ));
                                    }
                                }
                            }
                        }
                        ServerMsg::Relay { from, data } => {
                            // Relay message received
                            tracing::info!("📨 Received relay from {}: {:?}", from, data);
                            
                            // Check if it's a participant update from the server
                            if from == "server" {
                                tracing::info!("📨 Processing server relay message");
                                if let Some(msg_type) = data.get("type").and_then(|v| v.as_str()) {
                                    if msg_type == "participant_update" {
                                        // Handle participant update to trigger WebRTC
                                        if let (Some(session_id), Some(session_info)) = (
                                            data.get("session_id").and_then(|v| v.as_str()),
                                            data.get("session_info")
                                        ) {
                                            // Check if this is our session
                                            let (is_our_session, self_device_id, new_participants) = {
                                                let state_guard = state.lock().await;
                                                let is_ours = state_guard.session.as_ref()
                                                    .map(|s| s.session_id == session_id)
                                                    .unwrap_or(false);
                                                
                                                let device_id = state_guard.device_id.clone();
                                                
                                                // Extract participants from session_info
                                                let participants = if let Some(participants_arr) = session_info
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
                                                
                                                (is_ours, device_id, participants)
                                            };
                                            
                                            if is_our_session && !new_participants.is_empty() {
                                                tracing::info!("📡 Received participant update, initiating WebRTC with {} participants", new_participants.len());
                                                
                                                // Initiate WebRTC connections with the new participants
                                                crate::network::webrtc::initiate_webrtc_connections(
                                                    new_participants,
                                                    self_device_id,
                                                    state.clone(),
                                                    internal_cmd_tx.clone(),
                                                ).await;
                                            }
                                        }
                                        return; // Don't process further
                                    }
                                }
                            }
                            
                            // Check if it's a SessionAnnouncement (response to RequestActiveSessions)
                            if let Ok(announcement) = serde_json::from_value::<crate::protocal::signal::SessionAnnouncement>(data.clone()) {
                                let mut state_guard = state.lock().await;
                                
                                // Update or add the session
                                if let Some(existing) = state_guard.available_sessions.iter_mut()
                                    .find(|s| s.session_code == announcement.session_code) {
                                    *existing = announcement.clone();
                                } else {
                                    state_guard.available_sessions.push(announcement);
                                }
                                
                                return; // Don't process further
                            }
                            
                            // Check for backward compatibility raw join request
                            // This should not normally happen as we now use WebSocketMessage
                            if let Some(msg_type) = data.get("type").and_then(|v| v.as_str()) {
                                if msg_type == "session_join_request" {
                                    tracing::warn!("⚠️ Received legacy raw join request from {}. Please update client.", from);
                                    if let (Some(session_id), Some(device_id)) = (
                                        data.get("session_id").and_then(|v| v.as_str()),
                                        data.get("device_id").and_then(|v| v.as_str()),
                                    ) {
                                        let is_rejoin = data.get("is_rejoin").and_then(|v| v.as_bool()).unwrap_or(false);
                                        
                                        let mut state_guard = state.lock().await;
                                        // Join/rejoin request received
                                        
                                        // Check if we're an accepted participant of this session
                                        // Any accepted participant can handle join requests (not just the creator)
                                        let can_handle_join = state_guard.session.as_ref()
                                            .map(|s| {
                                                s.session_id == session_id && 
                                                s.participants.contains(&state_guard.device_id)
                                            })
                                            .unwrap_or(false);
                                        
                                        // Prefer the original creator if they're still online
                                        let is_original_creator = state_guard.session.as_ref()
                                            .map(|s| s.proposer_id == state_guard.device_id)
                                            .unwrap_or(false);
                                        
                                        if is_original_creator {
                                            tracing::debug!("Session proposer confirmed");
                                        }
                                        
                                        if can_handle_join {
                                            // Get the original proposer before logging
                                            let original_proposer = state_guard.session.as_ref()
                                                .map(|s| s.proposer_id.clone())
                                                .unwrap_or_default();
                                            
                                            // We can handle the join request
                                            let is_original_creator = original_proposer == state_guard.device_id;
                                            if is_original_creator {
                                                tracing::debug!("Original session proposer handling join request");
                                            }
                                            
                                            // Get device_id before mutable borrow
                                            let creator_device_id = state_guard.device_id.clone();
                                            
                                            // Update session and prepare proposal
                                            let (proposal, participant_log, _all_participants) = if let Some(ref mut session) = state_guard.session {
                                                // For rejoin, ensure device is in participants list
                                                let log_msg = if is_rejoin {
                                                    // For rejoin, ensure they're in participants list
                                                    if !session.participants.contains(&device_id.to_string()) {
                                                        session.participants.push(device_id.to_string());
                                                    }
                                                    // Remove from participants if present (they'll re-accept)
                                                    session.participants.retain(|d| d != device_id);
                                                    Some(format!(
                                                        "📊 {} rejoining. Participants ({}/{}): {:?}",
                                                        device_id,
                                                        session.participants.len(),
                                                        session.total,
                                                        session.participants
                                                    ))
                                                } else if !session.participants.contains(&device_id.to_string()) {
                                                    // Add to invited participants if not already there
                                                    session.participants.push(device_id.to_string());
                                                    Some(format!(
                                                        "📊 Invited participants ({}/{}): {:?}",
                                                        session.participants.len(),
                                                        session.total,
                                                        session.participants
                                                    ))
                                                } else {
                                                    None
                                                };
                                                
                                                // Create session proposal with ALL invited participants
                                                // The joiner will accept and send SessionResponse
                                                let proposal = crate::protocal::signal::SessionProposal {
                                                    session_id: session.session_id.clone(),
                                                    total: session.total,
                                                    threshold: session.threshold,
                                                    participants: session.participants.clone(), // Send ALL invited participants
                                                    session_type: session.session_type.clone(),
                                                    curve_type: session.curve_type.clone(),
                                                    coordination_type: session.coordination_type.clone(),
                                                    proposer_device_id: creator_device_id.clone(),
                                                };
                                                (Some(proposal), log_msg, session.participants.clone())
                                            } else {
                                                (None, None, Vec::new())
                                            };
                                            
                                            // Log participant update if needed
                                            if let Some(_log_msg) = participant_log {
                                            }
                                            
                                            if let Some(proposal) = proposal {
                                                // Send proposal to the new joiner
                                                let websocket_msg = crate::protocal::signal::WebSocketMessage::SessionProposal(proposal.clone());
                                                let relay_msg = webrtc_signal_server::ClientMsg::Relay {
                                                    to: from.clone(),
                                                    data: serde_json::to_value(websocket_msg).unwrap_or(serde_json::Value::Null),
                                                };
                                                
                                                if let Ok(msg_text) = serde_json::to_string(&relay_msg) {
                                                    match ws_sink.send(Message::Text(msg_text.into())).await {
                                                        Ok(_) => {
                                                            // Sent SessionProposal to new joiner
                                                        },
                                                        Err(_e) => {
                                                            // Failed to send SessionProposal
                                                            // Continue processing other participants
                                                        }
                                                    }
                                                }
                                                
                                                // Don't send to other participants yet - wait for SessionResponse
                                                // The joiner will send SessionResponse which will trigger proper broadcast
                                                // Only send if there are already accepted participants
                                                let accepted = state_guard.session.as_ref()
                                                    .map(|s| s.participants.clone())
                                                    .unwrap_or_default();
                                                for existing_device in &accepted {
                                                    if existing_device != &device_id && existing_device != &creator_device_id {
                                                        let updated_msg = crate::protocal::signal::WebSocketMessage::SessionProposal(proposal.clone());
                                                        let relay_to_existing = webrtc_signal_server::ClientMsg::Relay {
                                                            to: existing_device.clone(),
                                                            data: serde_json::to_value(updated_msg).unwrap_or(serde_json::Value::Null),
                                                        };
                                                        
                                                        if let Ok(msg_text) = serde_json::to_string(&relay_to_existing) {
                                                            let _ = ws_sink.send(Message::Text(msg_text.into())).await;
                                                            
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Capture accepted devices BEFORE dropping the lock
                                        let self_device_id = state_guard.device_id.clone();
                                        let participants_for_webrtc = if let Some(ref session) = state_guard.session {
                                            // CRITICAL: Filter out self from accepted devices
                                            session.participants
                                                .iter()
                                                .filter(|dev| **dev != self_device_id)
                                                .cloned()
                                                .collect()
                                        } else {
                                            Vec::new()
                                        };
                                        
                                        
                                        // Drop the lock BEFORE calling WebRTC functions
                                        drop(state_guard);
                                        
                                        // Directly initiate WebRTC connections with the captured list
                                        if !participants_for_webrtc.is_empty() {
                                            crate::network::webrtc::initiate_webrtc_connections(
                                                participants_for_webrtc,
                                                self_device_id,
                                                state.clone(),
                                                internal_cmd_tx.clone(),
                                            ).await;
                                        } else {
                                        }
                                        
                                        return; // Don't process as WebSocketMessage
                                    }
                                }
                            }

                            // Log the raw message type for debugging
                            if let Some(_msg_type) = data.get("websocket_msg_type").and_then(|v| v.as_str()) {
                            // Log removed
                            }
                            
                            match serde_json::from_value::<crate::protocal::signal::WebSocketMessage>(
                                data.clone(),
                            ) {
                                Ok(crate::protocal::signal::WebSocketMessage::WebRTCSignal(
                                    signal,
                                )) => {
                                    handle_webrtc_signal(
                                        from,
                                        signal,
                                        state.clone(),
                                        self_device_id.clone(),
                                        internal_cmd_tx.clone(),
                                        device_connections_arc.clone(),
                                    )
                                    .await;
                                }

                                Ok(crate::protocal::signal::WebSocketMessage::SessionJoinRequest(
                                    request,
                                )) => {
                                    // Handle join request - forward to internal command
                                    let _ = internal_cmd_tx.send(crate::utils::state::InternalCommand::ProcessJoinRequest {
                                        from_device: from.clone(),
                                        session_id: request.session_id,
                                        device_id: request.device_id,
                                        is_rejoin: request.is_rejoin,
                                    });
                                }

                                Ok(crate::protocal::signal::WebSocketMessage::SessionProposal(
                                    proposal,
                                )) => {
                                    let mut state_guard = state.lock().await;
                            // Log removed

                                    // Check if we're already in this session (receiving updated participant list)
                                    let already_in_session = state_guard.session.as_ref()
                                        .map(|s| s.session_id == proposal.session_id)
                                        .unwrap_or(false);
                                    
                                    if already_in_session {
                                        // We're already in the session - this is an update with new participants
                            // Log removed
                                        // Update our session with the new participant list
                                        if let Some(ref mut session) = state_guard.session {
                                            let old_participants = session.participants.clone();
                                            let session_total = session.total; // Extract before validation
                                            
                                            // Validate that new list is a superset of old list
                                            // (participants should only be added, never removed during session)
                                            let is_valid_update = old_participants.iter()
                                                .all(|old_p| proposal.participants.contains(old_p));
                                            if !is_valid_update {
                            // Log removed
                                                return; // Reject invalid update
                                            }
                                            
                                            // Validate that the new list doesn't exceed the session total
                                            if proposal.participants.len() > session_total as usize {
                            // Log removed
                                                return; // Reject invalid update
                                            }
                                            
                                            session.participants = proposal.participants.clone();
                                            
                                            // Find new participants we need to connect to
                                            let new_participants: Vec<String> = proposal.participants.iter()
                                                .filter(|p| !old_participants.contains(p) && **p != state_guard.device_id)
                                                .cloned()
                                                .collect();
                                            
                                            if !new_participants.is_empty() {
                                                // Only connect to NEW participants (optimization)
                                                let self_device_id = state_guard.device_id.clone();
                                                
                                                // Drop lock and initiate WebRTC with NEW participants only
                                                drop(state_guard);
                                                
                                                // Initiate WebRTC connections with NEW participants only
                                                // This avoids duplicate connection attempts to existing participants
                                                crate::network::webrtc::initiate_webrtc_connections(
                                                    new_participants,
                                                    self_device_id,
                                                    state.clone(),
                                                    internal_cmd_tx.clone(),
                                                ).await;
                                            }
                                        }
                                    } else {
                                        // Check if we already have this invite (from UI selection)
                                        let existing_invite_idx = state_guard.invites.iter()
                                            .position(|i| i.session_id == proposal.session_id);
                                        
                                        if let Some(idx) = existing_invite_idx {
                                            // Update the existing invite with full proposal data
                                            state_guard.invites[idx].participants = proposal.participants.clone();
                                            state_guard.invites[idx].proposer_id = proposal.proposer_device_id.clone(); // Use proposer_device_id from proposal
                                            state_guard.invites[idx].session_type = proposal.session_type.clone();
                            // Log removed
                                        } else {
                                            // New session invitation
                                            let invite_info = SessionInfo {
                                                session_id: proposal.session_id.clone(),
                                                proposer_id: proposal.proposer_device_id.clone(), // Use the proposer_device_id from proposal
                                                total: proposal.total,
                                                threshold: proposal.threshold,
                                                participants: proposal.participants.clone(),
                                                session_type: proposal.session_type.clone(),
                                                curve_type: proposal.curve_type.clone(),
                                                coordination_type: proposal.coordination_type.clone(),
                                            };
                                            state_guard.invites.push(invite_info);
                            // Log removed
                                        }
                                        
                                        // If we're actively joining a session, automatically accept the proposal
                                        // This happens when we selected a session from the available list
                                        let should_auto_accept = state_guard.joining_session_id.as_ref()
                                            .map(|id| id == &proposal.session_id)
                                            .unwrap_or(false);
                                        
                                        if should_auto_accept {
                            // Log removed
                                            // Clear the joining flag since we're accepting now
                                            state_guard.joining_session_id = None;
                                            drop(state_guard); // Release lock before sending command
                                            
                                            // Send accept command which will trigger WebRTC connection
                                            let _ = internal_cmd_tx.send(InternalCommand::AcceptSessionProposal(
                                                proposal.session_id.clone()
                                            ));
                                        } else {
                                            drop(state_guard);
                                        }
                                    }
                                }
                                Ok(crate::protocal::signal::WebSocketMessage::SessionResponse(
                                    response,
                                )) => {
                                    // Convert to the internal SessionResponse type if needed
                                    let internal_response = SessionResponse {
                                        session_id: response.session_id.clone(),
                                        from_device_id: response.from_device_id.clone(),
                                        accepted: response.accepted,
                                        wallet_status: response.wallet_status.clone(),
                                        reason: response.reason.clone(),
                                    };
                                    if let Err(_e) = internal_cmd_tx.send(
                                        InternalCommand::ProcessSessionResponse {
                                            from_device_id: from.clone(),
                                            response: internal_response,
                                        },
                                    ) {
                            // Log removed
                                    }
                                }
                                Ok(crate::protocal::signal::WebSocketMessage::SessionUpdate(
                                    update,
                                )) => {
                                    let mut state_guard = state.lock().await;
                                    
                                    // Update our session state with the new participant list
                                    let self_device_id = state_guard.device_id.clone();
                                    tracing::debug!("Processing participants update for device: {}", self_device_id);
                                    let has_session = state_guard.session.is_some();
                                    
                                    if has_session {
                                        let session_matches = state_guard.session.as_ref()
                                            .map(|s| s.session_id == update.session_id)
                                            .unwrap_or(false);
                                        
                                        if session_matches {
                                            let should_initiate_webrtc;
                                            let participants_for_webrtc;
                                            
                                            if let Some(ref mut session) = state_guard.session {
                                                let old_count = session.participants.len();
                                                session.participants = update.participants.clone();
                                                let new_count = session.participants.len();
                                                
                                                if new_count != old_count {
                                                    tracing::info!("Session participants updated: {}/{}", new_count, session.total);
                                                }
                                                
                            // Log removed
                                                
                                                // If this is a new participant joining, we need to initiate WebRTC
                                                should_initiate_webrtc = matches!(&update.update_type, 
                                                    crate::protocal::signal::SessionUpdateType::ParticipantJoined | 
                                                    crate::protocal::signal::SessionUpdateType::ParticipantRejoined);
                                                
                                                // Get all participants except self for WebRTC connection
                                                participants_for_webrtc = update.participants.iter()
                                                    .filter(|&p| p != &self_device_id)
                                                    .cloned()
                                                    .collect::<Vec<_>>();
                                            } else {
                                                should_initiate_webrtc = false;
                                                participants_for_webrtc = Vec::new();
                                            }
                                            
                                            // Drop the state lock before initiating WebRTC
                                            drop(state_guard);
                                            
                                            // Now initiate WebRTC connections if needed
                                            if should_initiate_webrtc && !participants_for_webrtc.is_empty() {
                                                tracing::info!("🚀 Triggering WebRTC connections after SessionUpdate with {} participants", 
                                                    participants_for_webrtc.len());
                                                
                                                // Initiate WebRTC connections with all participants
                                                crate::network::webrtc::initiate_webrtc_connections(
                                                    participants_for_webrtc,
                                                    self_device_id.clone(),
                                                    state.clone(),
                                                    internal_cmd_tx.clone(),
                                                ).await;
                                            }
                                        } else {
                                            // Log current session for debugging
                                            if let Some(session) = &state_guard.session {
                                                tracing::debug!("Processing participant update for session: {}", session.session_id);
                                            }
                            // Log removed
                                        }
                                    } else {
                            // Log removed
                                    }
                                }
                                
                                // Add missing patterns to fix compilation
                                Ok(crate::protocal::signal::WebSocketMessage::SessionOffer(_offer)) => {
                                    tracing::info!("Session offer handling is stubbed");
                                }
                                
                                Ok(crate::protocal::signal::WebSocketMessage::SessionAccepted { .. }) => {
                                    tracing::info!("Session accepted handling is stubbed");  
                                }
                                
                                Err(_e) => {
                                    tracing::warn!("Error parsing WebSocketMessage: {:?}", _e);
                                    // Error parsing WebSocketMessage
                                }
                            }
                        }
                    }
                }

                Err(_e) => {
                    // Error parsing server message
                }
            }
        }
        Message::Close(_) => {
            // WebSocket connection closed by server
        }
        Message::Ping(ping_data) => {
            let _ = ws_sink.send(Message::Pong(ping_data)).await;
        }
        Message::Pong(_) => {}
        Message::Binary(_) => {
            // Received unexpected binary message
        }
        Message::Frame(_) => {}
    }
}
