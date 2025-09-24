// Simple WebRTC initiation that doesn't require Ciphersuite bounds
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use webrtc::peer_connection::RTCPeerConnection;
use tracing::{info, error, warn};
use crate::protocal::signal::{WebRTCSignal, SDPInfo, WebSocketMessage};
use webrtc_signal_server::ClientMsg as SharedClientMsg;
use crate::utils::appstate_compat::AppState;
use serde_json;

/// Simple WebRTC connection initiation using existing WebSocket channel
pub async fn simple_initiate_webrtc_with_channel<C>(
    self_device_id: String,
    participants: Vec<String>,
    device_connections: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    app_state: Arc<Mutex<AppState<C>>>,
    ui_msg_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::elm::message::Message>>,
) where
    C: frost_core::Ciphersuite + 'static + Send + Sync,
    <<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    info!("🚀 Simple WebRTC initiation for {} participants", participants.len());

    // Get the WebSocket message channel from AppState (string-based for Send compatibility)
    let ws_msg_tx = {
        let state = app_state.lock().await;
        match &state.websocket_msg_tx {
            Some(tx) => {
                info!("✅ Got WebSocket message channel from AppState");
                tx.clone()
            }
            None => {
                error!("❌ No WebSocket message channel found in AppState - WebRTC offers cannot be sent!");
                return;
            }
        }
    };

    // Create debug log
    let debug_msg = format!(
        "[{}] 🚀 simple_initiate_webrtc called: self={}, participants={:?}",
        chrono::Local::now().format("%H:%M:%S%.3f"),
        self_device_id, participants
    );
    let _ = std::fs::write(format!("/tmp/{}-webrtc-simple.log", self_device_id), &debug_msg);

    // Filter out self
    let other_participants: Vec<String> = participants
        .into_iter()
        .filter(|p| p != &self_device_id)
        .collect();

    if other_participants.is_empty() {
        info!("No other participants to connect to");
        return;
    }

    // For each participant, ensure a peer connection exists
    for participant in other_participants.iter() {
        let needs_creation = {
            let conns = device_connections.lock().await;
            !conns.contains_key(participant)
        };

        if needs_creation {
            info!("📱 Creating peer connection for {}", participant);

            // Create a simple peer connection using webrtc crate directly
            let config = webrtc::peer_connection::configuration::RTCConfiguration {
                ice_servers: vec![],
                ..Default::default()
            };

            match webrtc::api::APIBuilder::new()
                .build()
                .new_peer_connection(config)
                .await
            {
                Ok(pc) => {
                    let mut conns = device_connections.lock().await;
                    conns.insert(participant.clone(), Arc::new(pc));
                    info!("✅ Created peer connection for {}", participant);
                }
                Err(e) => {
                    error!("❌ Failed to create peer connection for {}: {}", participant, e);
                }
            }
        } else {
            info!("✓ Peer connection already exists for {}", participant);
        }
    }

    // Now create offers for participants where we have lower ID (perfect negotiation)
    let devices_to_offer: Vec<String> = other_participants.clone()
        .into_iter()
        .filter(|p| self_device_id < *p)
        .collect();

    info!("📤 Will send offers to {} devices: {:?}", devices_to_offer.len(), devices_to_offer);
    
    // IMPORTANT: Log what connections we expect to receive offers for
    let devices_expecting_offers: Vec<String> = other_participants.clone()
        .into_iter()
        .filter(|p| self_device_id > *p)
        .collect();
    
    if !devices_expecting_offers.is_empty() {
        info!("📥 Expecting to receive offers from {} devices: {:?}", 
               devices_expecting_offers.len(), devices_expecting_offers);
    }

    for device_id in devices_to_offer {
        let conns = device_connections.lock().await;
        if let Some(pc) = conns.get(&device_id) {
            info!("🎯 Creating offer for {}", device_id);

            // Create data channel first
            // Set up connection state handler
            let device_id_state = device_id.clone();
            let ui_msg_tx_state = ui_msg_tx.clone();
            pc.on_peer_connection_state_change(Box::new(move |state: webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState| {
                let device_id_state = device_id_state.clone();
                let ui_msg_tx_state = ui_msg_tx_state.clone();
                Box::pin(async move {
                    let is_connected = matches!(state, webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected);
                    
                    // Send UI update
                    if let Some(tx) = ui_msg_tx_state {
                        let _ = tx.send(crate::elm::message::Message::UpdateParticipantWebRTCStatus {
                            device_id: device_id_state.clone(),
                            webrtc_connected: is_connected,
                            data_channel_open: false, // Will be updated when data channel opens
                        });
                    }
                    
                    match state {
                        webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected => {
                            info!("✅ WebRTC connection ESTABLISHED with {}", device_id_state);
                        }
                        webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Failed => {
                            error!("❌ WebRTC connection FAILED with {}", device_id_state);
                        }
                        webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Disconnected => {
                            warn!("⚠️ WebRTC connection DISCONNECTED from {}", device_id_state);
                        }
                        webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Closed => {
                            info!("🔒 WebRTC connection CLOSED with {}", device_id_state);
                        }
                        _ => {
                            info!("WebRTC connection state with {}: {:?}", device_id_state, state);
                        }
                    }
                })
            }));

            // Set up ICE candidate handler before creating offer
            let device_id_ice = device_id.clone();
            let ws_msg_tx_ice = ws_msg_tx.clone();
            let _pc_weak = Arc::downgrade(pc);

            pc.on_ice_candidate(Box::new(move |candidate: Option<webrtc::ice_transport::ice_candidate::RTCIceCandidate>| {
                let device_id_ice = device_id_ice.clone();
                let ws_msg_tx_ice = ws_msg_tx_ice.clone();
                let _pc_weak = _pc_weak.clone();

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
                                let _ = ws_msg_tx_ice.send(json);
                            }
                        }
                    }
                })
            }));

            match pc.create_data_channel("data", None).await {
                Ok(dc) => {
                    info!("✅ Created data channel for {}", device_id);

                    // Set up data channel handlers
                    let device_id_dc = device_id.clone();
                    let self_device_id_dc = self_device_id.clone();
                    let dc_arc = Arc::new(dc.clone());
                    let dc_for_open = dc_arc.clone();
                    let app_state_for_mesh = app_state.clone();
                    
                    let ui_msg_tx_open = ui_msg_tx.clone();
                    dc.on_open(Box::new(move || {
                        let device_id_open = device_id_dc.clone();
                        let self_id = self_device_id_dc.clone();
                        let dc_open = dc_for_open.clone();
                        let app_state_mesh = app_state_for_mesh.clone();
                        let ui_msg_tx_open = ui_msg_tx_open.clone();
                        
                        Box::pin(async move {
                            info!("📂 Data channel OPENED with {}", device_id_open);
                            
                            // Send UI update for data channel open
                            if let Some(tx) = ui_msg_tx_open {
                                let _ = tx.send(crate::elm::message::Message::UpdateParticipantWebRTCStatus {
                                    device_id: device_id_open.clone(),
                                    webrtc_connected: true,
                                    data_channel_open: true,
                                });
                            }
                            
                            // Send channel_open message to peer
                            let channel_open_msg = serde_json::json!({
                                "type": "channel_open",
                                "payload": {
                                    "device_id": self_id
                                }
                            });
                            
                            if let Ok(msg_str) = serde_json::to_string(&channel_open_msg) {
                                let _ = dc_open.send_text(msg_str).await;
                                info!("📤 Sent channel_open message to {}", device_id_open);
                            }
                            
                            // Check if all channels are open and send mesh_ready if so
                            // Note: Cannot use tokio::spawn due to Send constraints
                            // Small delay to allow other channels to open  
                            
                            let state = app_state_mesh.lock().await;
                            let session = state.session.clone();
                            let participants = session.as_ref().map(|s| s.participants.clone()).unwrap_or_default();
                            let device_conns = state.device_connections.clone();
                            let own_mesh_ready_sent = state.own_mesh_ready_sent;
                            drop(state);
                            
                            // Check if all expected connections are established
                            let conns = device_conns.lock().await;
                            let expected_count = participants.len().saturating_sub(1); // Exclude self
                            let connected_count = conns.len();
                            
                            if connected_count >= expected_count && expected_count > 0 && !own_mesh_ready_sent {
                                info!("✅ All {} peer connections established, sending mesh_ready", connected_count);
                                
                                // Send mesh_ready to all peers
                                let mesh_ready_msg = serde_json::json!({
                                    "type": "mesh_ready",
                                    "payload": {
                                        "session_id": session.as_ref().map(|s| s.session_id.clone()).unwrap_or_default(),
                                        "device_id": self_id
                                    }
                                });
                                
                                if let Ok(msg_str) = serde_json::to_string(&mesh_ready_msg) {
                                    let _ = dc_open.send_text(msg_str).await;
                                    info!("📤 Sent mesh_ready signal via data channel");
                                    
                                    // Mark as sent
                                    let mut state = app_state_mesh.lock().await;
                                    state.own_mesh_ready_sent = true;
                                }
                            }
                        })
                    }));

                    let device_id_msg = device_id.clone();
                    let app_state_for_msg = app_state.clone();
                    dc.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                        let device_id_recv = device_id_msg.clone();
                        let app_state_msg = app_state_for_msg.clone();
                        Box::pin(async move {
                            info!("📥 Received message from {} via data channel: {} bytes",
                                device_id_recv, msg.data.len());
                            
                            // Try to parse as JSON message
                            if let Ok(text) = String::from_utf8(msg.data.to_vec()) {
                                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(msg_type) = json_msg.get("type").and_then(|v| v.as_str()) {
                                        match msg_type {
                                            "channel_open" => {
                                                info!("📂 Received channel_open from {}", device_id_recv);
                                            },
                                            "mesh_ready" => {
                                                info!("✅ Received mesh_ready from {}", device_id_recv);
                                                // Mark this peer as mesh ready
                                                let mut state = app_state_msg.lock().await;
                                                state.pending_mesh_ready_signals.insert(device_id_recv.clone());
                                                
                                                // Check if all peers are ready
                                                let session = state.session.clone();
                                                if let Some(session) = session {
                                                    let expected_peers = session.participants.len() - 1;
                                                    let ready_peers = state.pending_mesh_ready_signals.len();
                                                    
                                                    if ready_peers >= expected_peers && !state.own_mesh_ready_sent {
                                                        info!("🎉 All {} peers are mesh ready!", ready_peers);
                                                        state.mesh_status = crate::utils::state::MeshStatus::Ready;
                                                        state.own_mesh_ready_sent = true;
                                                    }
                                                }
                                            },
                                            _ => {
                                                // Forward to DKG protocol handler
                                                info!("📨 Forwarding {} message from {} to protocol handler", msg_type, device_id_recv);
                                            }
                                        }
                                    }
                                }
                            }
                        })
                    }));

                    // TODO: Store the data channel for sending messages
                    // Note: Cannot access AppState here due to Ciphersuite Send constraint

                    // Now create offer
                    match pc.create_offer(None).await {
                        Ok(offer) => {
                            info!("✅ Created offer for {}", device_id);

                            // Set local description
                            if let Err(e) = pc.set_local_description(offer.clone()).await {
                                error!("Failed to set local description: {}", e);
                            } else {
                                info!("✅ Set local description for {}", device_id);

                                // Send offer via existing WebSocket channel
                                let signal = WebRTCSignal::Offer(SDPInfo { sdp: offer.sdp });
                                let websocket_message = WebSocketMessage::WebRTCSignal(signal);

                                match serde_json::to_value(websocket_message) {
                                    Ok(json_val) => {
                                        let relay_msg = SharedClientMsg::Relay {
                                            to: device_id.clone(),
                                            data: json_val,
                                        };

                                        // Serialize the message immediately to avoid Send issues
                                        match serde_json::to_string(&relay_msg) {
                                            Ok(json) => {
                                                info!("📤 Sending WebRTC offer to {} via WebSocket", device_id);
                                                if let Err(e) = ws_msg_tx.send(json) {
                                                    error!("❌ Failed to send offer to {}: {}", device_id, e);
                                                } else {
                                                    info!("✅ WebRTC offer sent to {} via WebSocket", device_id);
                                                }
                                            }
                                            Err(e) => {
                                                error!("❌ Failed to serialize relay message for {}: {}", device_id, e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("❌ Failed to serialize offer for {}: {}", device_id, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("❌ Failed to create offer for {}: {}", device_id, e);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to create data channel for {}: {}", device_id, e);
                }
            }
        }
    }

    info!("✅ Simple WebRTC initiation complete");
}