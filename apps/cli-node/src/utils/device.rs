// use crate::network::websocket; // Unused
// use crate::protocal::signal::*; // Unused
use crate::protocal::signal::{WebRTCMessage, WebRTCSignal};
use crate::utils::state::{AppState, DkgState, InternalCommand};
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};

use webrtc::data_channel::RTCDataChannel;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::data_channel_state::RTCDataChannelState;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;

use frost_core::Ciphersuite;

use webrtc_signal_server::ClientMsg as SharedClientMsg;
use crate::protocal::signal::{CandidateInfo, WebSocketMessage}; // Updated path
use crate::utils::state::MeshStatus;


pub const DATA_CHANNEL_LABEL: &str = "frost-dkg"; 

pub async fn send_webrtc_message<C>(
    target_device_id: &str,
    message: &WebRTCMessage<C>,
    state_log: Arc<Mutex<AppState<C>>>,
) -> Result<(), String> where C: Ciphersuite {
    let data_channel = {
        let guard = state_log.lock().await;
        guard.data_channels.get(target_device_id).cloned()
    };

    if let Some(dc) = data_channel {
 
        if dc.ready_state() == RTCDataChannelState::Open {
   
            let msg_json = serde_json::to_string(&message)
                .map_err(|e| format!("Failed to serialize envelope: {}", e))?;

            if let Err(e) = dc.send_text(msg_json).await {
                state_log.lock().await.log.push(format!(
                    "Error sending message to {}: {}",
                    target_device_id, e
                ));
                return Err(format!("Failed to send message: {}", e));
            }
            //         // udpate conncetion status
            // let mut state_guard = state_log.lock().await;
            // state_guard.device_statuses.insert(
            //             target_device_id.to_string(),
            //             webrtc::device_connection::device_connection_state::RTCDeviceConnectionState::Connected,
            // );

            state_log.lock().await.log.push(format!(
                "Successfully sent WebRTC message to {}",
                target_device_id
            ));
            Ok(())
        } else {
            let err_msg = format!(
                "Data channel for {} is not open (state: {:?})",
                target_device_id,
                dc.ready_state()
            );
            state_log.lock().await.log.push(err_msg.clone());
            Err(err_msg)
        }
    } else {
        let err_msg = format!("Data channel not found for device {}", target_device_id);
        state_log.lock().await.log.push(err_msg.clone());
        Err(err_msg)
    }
}

pub async fn create_and_setup_device_connection<C>(
    device_id: String,
    self_device_id: String, // Pass self_device_id
    device_connections_arc: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>, // Use InternalCommand
    state_log: Arc<Mutex<AppState<C>>>,
    api: &'static webrtc::api::API,
    config: &'static RTCConfiguration,
) -> Result<Arc<RTCPeerConnection>, String> where C: Ciphersuite + Send + Sync + 'static, 
<<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync, 
<<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,     
{
    {
        let device_conns = device_connections_arc.lock().await;
        if let Some(existing_pc) = device_conns.get(&device_id) {
            state_log.lock().await.log.push(format!(
                "WebRTC connection object for {} already exists. Skipping creation.",
                device_id
            ));
            return Ok(existing_pc.clone());
        }
    }

    state_log
        .lock()
        .await
        .log
        .push(format!("Creating WebRTC connection object for {}", device_id));

    // Use passed-in api and config
    match api.new_peer_connection(config.clone()).await {
        Ok(pc) => {
            let pc_arc = Arc::new(pc);

            if self_device_id < device_id {
                match pc_arc.create_data_channel(DATA_CHANNEL_LABEL, None).await {
                    Ok(dc) => {
                        state_log.lock().await.log.push(format!(
                            "Initiator: Created data channel '{}' for {}",
                            DATA_CHANNEL_LABEL, device_id
                        ));
                        setup_data_channel_callbacks(
                            dc,
                            device_id.clone(),
                            state_log.clone(),
                            cmd_tx.clone(),
                        ).await;
                    }
                    Err(e) => {
                        state_log.lock().await.log.push(format!(
                            "Initiator: Failed to create data channel for {}: {}",
                            device_id, e
                        ));
                    }
                }
            }

            let device_id_on_ice = device_id.clone();
            let cmd_tx_on_ice = cmd_tx.clone(); // Clones the sender for internal ClientMsg
            let state_log_on_ice = state_log.clone();
            pc_arc.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
                let device_id = device_id_on_ice.clone();
                let cmd_tx = cmd_tx_on_ice.clone();
                let state_log = state_log_on_ice.clone();
                Box::pin(async move {
                    if let Some(c) = candidate {
                        // ... existing ICE candidate sending logic ...
                        match c.to_json() {
                            Ok(init) => {
                                let signal = WebRTCSignal::Candidate(CandidateInfo {
                                    candidate: init.candidate,
                                    sdp_mid: init.sdp_mid,
                                    sdp_mline_index: init.sdp_mline_index,
                                });
                                let websocket_msg = WebSocketMessage::WebRTCSignal(signal);
                                match serde_json::to_value(websocket_msg) {
                                    Ok(json_val) => {
                                        // Wrap the Relay message inside SendToServer command
                                        let relay_cmd =
                                            InternalCommand::SendToServer(SharedClientMsg::Relay {
                                                to: device_id.clone(),
                                                data: json_val,
                                            });
                                        let _ = cmd_tx.send(relay_cmd); // Send the internal command
                                        state_log
                                            .lock()
                                            .await
                                            .log
                                            .push(format!("Sent ICE candidate to {}", device_id));
                                    }
                                    // FIX: Use error variable 'e'
                                    Err(e) => {
                                        state_log.lock().await.log.push(format!(
                                            "Error serializing ICE candidate for {}: {}",
                                            device_id, e
                                        ));
                                    }
                                }
                            }
                            // FIX: Use error variable 'e'
                            Err(e) => {
                                state_log.lock().await.log.push(format!(
                                    "Error converting ICE candidate to JSON for {}: {}",
                                    device_id, e
                                ));
                            }
                        }
                    }
                })
            }));

            // Setup state change handler with DKG trigger logic
            let state_log_on_state = state_log.clone();
            let device_id_on_state = device_id.clone();
            // Fix the setup_device_connection_callbacks function
            // Clone before moving into closure
            let pc_arc_for_state = pc_arc.clone();
            pc_arc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                // Fix: Use pc_arc directly instead of undefined pc_arc_for_state
                let pc_arc = pc_arc_for_state.clone();
                
                // Log both connectionState and iceConnectionState together
                let ice_state = pc_arc.ice_connection_state();
                println!(
                    "Device {}: connectionState={:?}, iceConnectionState={:?}",
                    device_id, s, ice_state
                );
                if let Ok(mut app_state_guard) = state_log.try_lock() {
                    app_state_guard.device_statuses.insert(device_id.clone(), s);
                    app_state_guard.log.push(format!(
                        "WebRTC state with {}: {:?}, ICE: {:?}",
                        device_id, s, ice_state
                    ));
                }

                // Handle state changes with improved logic
                match s {
                    RTCPeerConnectionState::Connected => {
                        if let Ok(mut guard) = state_log.try_lock() {
                            guard.log.push(format!("!!! WebRTC CONNECTED with {} !!!", device_id));
                            guard.reconnection_tracker.record_success(&device_id);
                        }
                    }
                    RTCPeerConnectionState::Disconnected => {
                        // Handle disconnection with more aggressive reconnection
                        if let Ok(mut guard) = state_log.try_lock() {
                            guard.log.push(format!("!!! WebRTC DISCONNECTED with {} !!!", device_id));
                                                        
                            // Reset DKG state if a device disconnects during DKG
                            if guard.dkg_state != DkgState::Idle && guard.dkg_state != DkgState::Complete {
                                guard.log.push(format!("Resetting DKG state due to disconnection with {}", device_id));
                                guard.dkg_state = DkgState::Failed(format!("Device {} disconnected", device_id));
                                // Clear intermediate DKG data if needed
                                guard.dkg_part1_public_package = None;
                                guard.dkg_part1_secret_package = None;
                                guard.received_dkg_packages.clear();
                            }
                            
                            // Always attempt immediate reconnection on Disconnected state
                            if let Some(current_session) = guard.session.clone() {
                                let session_id_to_rejoin = current_session.session_id;
                                guard.log.push(format!(
                                    "Attempting immediate reconnection to session '{}' due to DISCONNECTED state with {} (no JoinSession message sent, logic removed)",
                                    session_id_to_rejoin, device_id
                                ));
                                // Drop the guard before sending the command
                                drop(guard);
                                // No JoinSession message sent
                            }
                        }
                    }
                    RTCPeerConnectionState::Failed => {
                        if let Ok(mut guard) = state_log.try_lock() {
                            guard.log.push(format!("!!! WebRTC FAILED with {} !!!", device_id));
                            
                            // Reset DKG state if a device disconnects during DKG
                            if guard.dkg_state != DkgState::Idle && guard.dkg_state != DkgState::Complete {
                                guard.log.push(format!("Resetting DKG state due to connection failure with {}", device_id));
                                guard.dkg_state = DkgState::Failed(format!("Device {} connection failed", device_id));
                                guard.dkg_part1_public_package = None;
                                guard.dkg_part1_secret_package = None;
                                guard.received_dkg_packages.clear();
                            }
                            
                            // Attempt to rejoin with backoff strategy
                            if guard.reconnection_tracker.should_attempt(&device_id) {
                                if let Some(current_session) = guard.session.clone() {
                                    let session_id_to_rejoin = current_session.session_id;
                                    guard.log.push(format!(
                                        "Attempting reconnection to session '{}' due to FAILED state with {} (no JoinSession message sent, logic removed)",
                                        session_id_to_rejoin, device_id
                                    ));
                                    // Drop the guard before sending the command
                                    drop(guard);
                                    // No JoinSession message sent
                                }
                            }
                        }
                    }
                    RTCPeerConnectionState::Connecting | RTCPeerConnectionState::New => {
                        // We don't need special handling for these states,
                        // they're already logged above when updating device_statuses
                    }
                    RTCPeerConnectionState::Closed => {
                        if let Ok(mut guard) = state_log.try_lock() {
                            guard.log.push(format!("WebRTC connection CLOSED with {}", device_id));
                        }
                    }
                    // Handle the Unspecified state to fix the compilation error
                    RTCPeerConnectionState::Unspecified => {
                        if let Ok(mut guard) = state_log.try_lock() {
                            guard.log.push(format!("WebRTC in UNSPECIFIED state with {}", device_id));
                            // No specific action needed for unspecified state
                        }
                    }
                }
                Box::pin(async {})
            }));

            // --- Setup ICE connection monitoring callback ---
            let state_log_ice = state_log_on_state.clone();
            let device_id_ice = device_id_on_state.clone();
            let pc_arc_for_ice = pc_arc.clone();
            pc_arc.on_ice_connection_state_change(Box::new(move |ice_state| {
                let state_log = state_log_ice.clone();
                let device_id = device_id_ice.clone();
                let pc_arc = pc_arc_for_ice.clone();

                // Log both connectionState and iceConnectionState together
                let conn_state = pc_arc.connection_state();
                println!(
                    "Device {}: connectionState={:?}, iceConnectionState={:?}",
                    device_id, conn_state, ice_state
                );
                if let Ok(mut guard) = state_log.try_lock() {
                    guard.log.push(format!(
                        "ICE connection state with {}: {:?}, DeviceConnection: {:?}",
                        device_id, ice_state, conn_state
                    ));
                }
                // No async work, just return a ready future
                Box::pin(async {})
            }));

            // --- Only set up callbacks for the main data channel (responder side) ---
            let state_log_on_data = state_log_on_state.clone();
            let device_id_on_data = device_id_on_state.clone();
            let cmd_tx_on_data = cmd_tx.clone();
            pc_arc.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
                let state_log = state_log_on_data.clone();
                let device_id = device_id_on_data.clone();
                let cmd_tx_clone = cmd_tx_on_data.clone();

                Box::pin(async move {
                    if dc.label() == DATA_CHANNEL_LABEL {
                        state_log.lock().await.log.push(format!(
                            "Responder: Data channel '{}' opened by {}",
                            dc.label(),
                            device_id
                        ));
                        setup_data_channel_callbacks(dc, device_id, state_log, cmd_tx_clone).await;
                    }
                })
            }));

            // --- Store the connection object ---
            {
                let mut device_conns = device_connections_arc.lock().await;
                device_conns.insert(device_id_on_state.clone(), pc_arc.clone());
                state_log_on_state
                    .lock()
                    .await
                    .log
                    .push(format!("Stored WebRTC connection object for {}", device_id_on_state));
            } // Drop lock

            Ok(pc_arc)
        }
        Err(e) => {
            // FIX: Add actual log message
            let err_msg = format!(
                "Error creating device connection object for {}: {}",
                device_id, e
            );
            state_log.lock().await.log.push(err_msg.clone());
            Err(err_msg)
        }
    }
}

pub async fn setup_data_channel_callbacks<C>(
    dc: Arc<RTCDataChannel>,
    device_id: String,
    state: Arc<Mutex<AppState<C>>>,
    // Update the sender type here
    cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>, // Use InternalCommand
) where C: Ciphersuite + Send + Sync + 'static, 
<<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync, 
<<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,     
 {
    let dc_arc = dc.clone(); // Clone the Arc for the data channel

    // Only store and set up callbacks for the main data channel
    if dc_arc.label() == DATA_CHANNEL_LABEL {
        let mut guard = state.lock().await;
        guard.data_channels.insert(device_id.clone(), dc_arc.clone());
        guard
            .log
            .push(format!("Data channel for {} stored in app state", device_id));
    }

    let state_log_open = state.clone();
    let device_id_open = device_id.clone();
    let dc_clone = dc_arc.clone();
    let cmd_tx_open = cmd_tx.clone();
    dc_arc.on_open(Box::new(move || {
        let state_log_open = state_log_open.clone();
        let device_id_open = device_id_open.clone();
        let dc_clone = dc_clone.clone();
        let cmd_tx_open = cmd_tx_open.clone();
        Box::pin(async move {
            state_log_open.lock().await.log.push(format!(
                "Data channel '{}' open confirmed with {}",
                dc_clone.label(),
                device_id_open
            ));
            
            // Send ReportChannelOpen command to trigger mesh ready signaling
            let _ = cmd_tx_open.send(InternalCommand::ReportChannelOpen {
                device_id: device_id_open.clone(),
            });
        })
    }));

    let state_log_msg = state.clone();
    let device_id_msg = device_id.clone();
    let cmd_tx_msg = cmd_tx.clone(); // Clone internal cmd_tx for on_message
    let dc_arc_msg = dc_arc.clone(); // Clone for use inside async block
    dc_arc.on_message(Box::new(move |msg: DataChannelMessage| {
        let device_id = device_id_msg.clone();
        let state_log = state_log_msg.clone();
        let cmd_tx = cmd_tx_msg.clone();
        let dc_arc = dc_arc_msg.clone(); // Use a clone inside the async block

        Box::pin(async move {
            // Only process messages if this is the main frost-dkg channel
            if dc_arc.label() != DATA_CHANNEL_LABEL {
                return;
            }

            if let Ok(text) = String::from_utf8(msg.data.to_vec()) {
                // DEBUG: Log the raw message content to see exactly what we're receiving
                state_log.lock().await.log.push(format!(
                    "Raw message from {}: {}",
                    device_id, text
                ));
                
                // Parse envelope
                match serde_json::from_str::<WebRTCMessage<C>>(&text) {
                    Ok(envelope) => {
                        match envelope {
                            WebRTCMessage::DkgRound1Package { package } => {
                                    let _ = cmd_tx.send(InternalCommand::ProcessDkgRound1 {
                                        from_device_id: device_id.clone(),
                                        package,
                                    });
                            }
                            WebRTCMessage::DkgRound2Package { package } => {
                                // FIX: Add type annotation for from_value
                                    state_log.lock().await.log.push(format!(
                                        "Received DKG Round 2 package from {}",
                                        device_id
                                    ));
                                    let _ = cmd_tx.send(InternalCommand::ProcessDkgRound2 {
                                        from_device_id: device_id.clone(),
                                        package,
                                    });
                            }
                            WebRTCMessage::SimpleMessage { text } => {
                                    state_log.lock().await.log.push(format!(
                                        "Receiver: Message from {}: {}",
                                        device_id, text
                                    ));
                            },
                            WebRTCMessage::ChannelOpen { device_id: _ } => {
                                // Just log the channel open notification, don't trigger ReportChannelOpen
                                // to avoid infinite feedback loops
                                state_log.lock().await.log.push(format!(
                                    "Received channel open notification from {}",
                                    device_id
                                ));
                            },
                            WebRTCMessage::MeshReady { session_id, device_id } => {
                                state_log.lock().await.log.push(format!(
                                    "Mesh ready notification from {}: session_id: {}, device_id: {}",
                                    device_id, session_id, device_id
                                ));
                                let _ = cmd_tx.send(InternalCommand::ProcessMeshReady {
                                    device_id: device_id.clone(),
                                });
                            },
                            // Signing message handlers
                            WebRTCMessage::SigningRequest { signing_id, transaction_data, required_signers, blockchain, chain_id } => {
                                state_log.lock().await.log.push(format!(
                                    "Received signing request from {}: id={}, blockchain={}, required_signers={}",
                                    device_id, signing_id, blockchain, required_signers
                                ));
                                let _ = cmd_tx.send(InternalCommand::ProcessSigningRequest {
                                    from_device_id: device_id.clone(),
                                    signing_id,
                                    transaction_data,
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                    blockchain,
                                    chain_id,
                                });
                            },
                            WebRTCMessage::SigningAcceptance { signing_id, accepted } => {
                                state_log.lock().await.log.push(format!(
                                    "Received signing acceptance from {}: id={}, accepted={}",
                                    device_id, signing_id, accepted
                                ));
                                let _ = cmd_tx.send(InternalCommand::ProcessSigningAcceptance {
                                    from_device_id: device_id.clone(),
                                    signing_id,
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                });
                            },
                            WebRTCMessage::SignerSelection { signing_id, selected_signers } => {
                                state_log.lock().await.log.push(format!(
                                    "Received signer selection from {}: id={}, signers={:?}",
                                    device_id, signing_id, selected_signers
                                ));
                                let _ = cmd_tx.send(InternalCommand::ProcessSignerSelection {
                                    from_device_id: device_id.clone(),
                                    signing_id,
                                    selected_signers,
                                });
                            },
                            WebRTCMessage::SigningCommitment { signing_id, sender_identifier, commitment } => {
                                state_log.lock().await.log.push(format!(
                                    "Received signing commitment from {}: id={}, sender_id={:?}",
                                    device_id, signing_id, sender_identifier
                                ));
                                let _ = cmd_tx.send(InternalCommand::ProcessSigningCommitment {
                                    from_device_id: device_id.clone(),
                                    signing_id,
                                    commitment,
                                });
                            },
                            WebRTCMessage::SignatureShare { signing_id, sender_identifier, share } => {
                                state_log.lock().await.log.push(format!(
                                    "Received signature share from {}: id={}, sender_id={:?}",
                                    device_id, signing_id, sender_identifier
                                ));
                                let _ = cmd_tx.send(InternalCommand::ProcessSignatureShare {
                                    from_device_id: device_id.clone(),
                                    signing_id,
                                    share,
                                });
                            },
                            WebRTCMessage::AggregatedSignature { signing_id, signature } => {
                                state_log.lock().await.log.push(format!(
                                    "Received aggregated signature from {}: id={}",
                                    device_id, signing_id
                                ));
                                let _ = cmd_tx.send(InternalCommand::ProcessAggregatedSignature {
                                    from_device_id: device_id.clone(),
                                    signing_id,
                                    signature,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        state_log
                            .lock()
                            .await
                            .log
                            .push(format!("Failed to parse envelope from {}: {}", device_id, e));
                    }
                }
            } else {
                state_log
                    .lock()
                    .await
                    .log
                    .push(format!("Received non-UTF8 data from {}", device_id));
            }
        })
    }));

    let state_log_close = state.clone();
    let device_id_close = device_id.clone();
    dc.on_close(Box::new(move || {
        let state_log_close = state_log_close.clone();
        let device_id_close = device_id_close.clone();
        Box::pin(async move {
            state_log_close.lock().await.log.push(format!(
                "Data channel '{}' closed with {}",
                DATA_CHANNEL_LABEL, device_id_close
            ));
        })
    }));

    let state_log_error = state.clone();
    let device_id_error = device_id.clone();
    dc.on_error(Box::new(move |e| {
        let state_log_error = state_log_error.clone();
        let device_id_error = device_id_error.clone();
        Box::pin(async move {
            state_log_error.lock().await.log.push(format!(
                "Data channel '{}' error with {}: {}",
                DATA_CHANNEL_LABEL, device_id_error, e
            ));
        })
    }));
}

// Apply any pending ICE candidates for a device
pub async fn apply_pending_candidates<C>(
    device_id: &str,
    pc: Arc<RTCPeerConnection>,
    state_log: Arc<Mutex<AppState<C>>>,
) where C: Ciphersuite {
    // Take the pending candidates for this device
    let candidates = {
        let mut state_guard = state_log.lock().await;
        let pending = state_guard.pending_ice_candidates.remove(device_id);
        if let Some(candidates) = &pending {
            if !candidates.is_empty() {
                state_guard.log.push(format!(
                    "Applying {} stored ICE candidate(s) for {}",
                    candidates.len(),
                    device_id
                ));
            }
        }
        pending
    };

    // If there are pending candidates, apply them
    if let Some(candidates) = candidates {
        // Apply each candidate
        for candidate in candidates {
            match pc.add_ice_candidate(candidate.clone()).await {
                Ok(_) => {
                    let mut state_guard = state_log.lock().await;
                    state_guard
                        .log
                        .push(format!("Applied stored ICE candidate for {}", device_id));
                    // apply candidate to the device connection
                    
                }
                Err(e) => {
                    let mut state_guard = state_log.lock().await;
                    state_guard.log.push(format!(
                            "Error applying stored ICE candidate for {}: {}",
                            device_id, e
                        ));                
                }
            }
        }
    }
}

pub async fn check_and_send_mesh_ready<C>( //all data channels are open and send mesh_ready if needed
   state: Arc<Mutex<AppState<C>>>,
    cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where C: Ciphersuite + Send + Sync + 'static, 
<<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync, 
<<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,     
{
    let mut all_channels_open_debug = false;
    let mut all_responses_received_debug = false;
    let mut already_sent_own_ready_debug = false;
    let mut session_exists_debug = false;
    let mut participants_to_check_debug: Vec<String> = Vec::new();
    let mut data_channels_keys_debug: Vec<String> = Vec::new();
    let self_device_id_debug: String;
    let current_mesh_status_debug: MeshStatus;

    {
        let state_guard = state.lock().await;
        self_device_id_debug = state_guard.device_id.clone();
        current_mesh_status_debug = state_guard.mesh_status.clone(); // Clone for logging
        if let Some(session) = &state_guard.session {
            session_exists_debug = true;
            let device_id_clone = state_guard.device_id.clone(); // Clone to avoid borrow issues
            participants_to_check_debug = session
                .participants
                .iter()
                .filter(|p| **p != device_id_clone)
                .cloned()
                .collect();
            
            data_channels_keys_debug = state_guard.data_channels.keys().cloned().collect();

            all_channels_open_debug = participants_to_check_debug
                .iter()
                .all(|p| state_guard.data_channels.contains_key(p));

            // Check if all session responses received (all participants accepted)
            all_responses_received_debug = session.accepted_devices.len() == session.participants.len();
            
            // Clone values before the match to avoid borrowing conflicts
            let _current_session_size = session.participants.len();
            
            // Check if we've already sent our own mesh ready signal using explicit tracking
            // This replaces the flawed logic that incorrectly inferred from mesh status
            already_sent_own_ready_debug = state_guard.own_mesh_ready_sent;
        }
    } // state_guard is dropped

    // Log outside the lock to minimize lock holding time
    let mut log_guard = state.lock().await;
    log_guard.log.push(format!(
        "[MeshCheck-{}] Status: {:?}, SessionExists: {}, ParticipantsToCheck: {:?}, OpenDCKeys: {:?}, AllOpenCalc: {}, AllResponsesReceivedCalc: {}, AlreadySentCalc: {}",
        self_device_id_debug,
        current_mesh_status_debug, // Log current status
        session_exists_debug,
        participants_to_check_debug,
        data_channels_keys_debug,
        all_channels_open_debug,
        all_responses_received_debug,
        already_sent_own_ready_debug
    ));
    drop(log_guard);


    if session_exists_debug && all_channels_open_debug && all_responses_received_debug && !already_sent_own_ready_debug {
        // Re-acquire lock for the specific log message and subsequent command sending
        state 
            .lock()
            .await
            .log
            .push(format!("[MeshCheck-{}] All local data channels open AND all session responses received! Signaling to process own mesh readiness...", self_device_id_debug));
        
        if let Err(e) = cmd_tx.send(InternalCommand::SendOwnMeshReadySignal) {
            // Clone necessary items for the async logging task
            let state_clone_for_err = state.clone(); 
            let self_device_id_err_clone = self_device_id_debug.clone();
            tokio::spawn(async move { 
                state_clone_for_err
                    .lock()
                    .await
                    .log
                    .push(format!("[MeshCheck-{}] Failed to send SendOwnMeshReadySignal command: {}", self_device_id_err_clone, e));
            });   
        }
    } else {
        // Log reason for not sending, re-acquiring lock briefly
        let mut final_log_guard = state.lock().await;
        if !session_exists_debug {
            final_log_guard.log.push(format!("[MeshCheck-{}] No active session, cannot send SendOwnMeshReadySignal.", self_device_id_debug));
        } else if !all_channels_open_debug {
            final_log_guard.log.push(format!("[MeshCheck-{}] Not all channels open yet (expected {:?}, have {:?}), cannot send SendOwnMeshReadySignal.", self_device_id_debug, participants_to_check_debug, data_channels_keys_debug));
        } else if !all_responses_received_debug {
            final_log_guard.log.push(format!("[MeshCheck-{}] Not all session responses received yet, cannot send SendOwnMeshReadySignal.", self_device_id_debug));
        } else if already_sent_own_ready_debug {
            final_log_guard.log.push(format!("[MeshCheck-{}] Already sent own ready signal (Status: {:?}), not sending again.", self_device_id_debug, current_mesh_status_debug));
        }
    }
}