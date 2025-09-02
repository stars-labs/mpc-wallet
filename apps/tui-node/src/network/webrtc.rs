use lazy_static::lazy_static;
use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::policy::{
    bundle_policy::RTCBundlePolicy, ice_transport_policy::RTCIceTransportPolicy,
    rtcp_mux_policy::RTCRtcpMuxPolicy,
};
// Import from lib.rs
use crate::utils::appstate_compat::AppState;
use crate::utils::state::InternalCommand;

use std::sync::Arc;

use std::{collections::HashMap,};
use tokio::sync::{Mutex, mpsc};

use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::RTCPeerConnection;

use webrtc_signal_server::{ClientMsg as SharedClientMsg};
// Add display-related imports for better status handling
use frost_core::{
    Ciphersuite
};
use crate::protocal::signal::{
     WebRTCSignal, SDPInfo, CandidateInfo,
};
use crate::utils::device::{create_and_setup_device_connection, apply_pending_candidates};
use crate::protocal::signal::WebSocketMessage;
use crate::utils::negotiation::initiate_offers_for_session;
// --- WebRTC API Setup ---
lazy_static! {
    pub static ref WEBRTC_CONFIG: RTCConfiguration = RTCConfiguration {
        ice_servers: vec![
            // Primary STUN servers - using multiple to increase reliability
            RTCIceServer {
                urls: vec![
                    "stun:stun.l.google.com:19302".to_owned(),
                    "stun:stun1.l.google.com:19302".to_owned(),
                    "stun:stun2.l.google.com:19302".to_owned(),
                    "stun:stun3.l.google.com:19302".to_owned(),
                    "stun:stun4.l.google.com:19302".to_owned(),
                ],
                ..Default::default()
            },
            // 添加更多可靠的TURN服务器 - 改善NAT穿透
            RTCIceServer {
                urls: vec!["turn:numb.viagenie.ca".to_owned()],
                username: "muazkh".to_owned(),
                credential: "webrtc@live.com".to_owned(),
            },
            // 备用公共TURN服务器
            RTCIceServer {
                urls: vec!["turn:openrelay.metered.ca:80".to_owned()],
                username: "openrelayproject".to_owned(),
                credential: "openrelayproject".to_owned(),
            },
        ],
        ice_transport_policy: RTCIceTransportPolicy::All,
        bundle_policy: RTCBundlePolicy::MaxBundle,
        rtcp_mux_policy: RTCRtcpMuxPolicy::Require,
        ice_candidate_pool_size: 10, // 增加候选池大小以提高连接成功率

        ..Default::default()
    };
    pub static ref WEBRTC_API: webrtc::api::API = {
        let mut m = MediaEngine::default();
        // NOTE: Registering codecs is required for audio/video, but not for data channels.
        // m.register_default_codecs().unwrap();
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m).unwrap();
        APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build()
    };
}
/// Handler for WebRTC signaling messages
pub async fn handle_webrtc_signal<C>(
    from_device_id: String,
    signal: WebRTCSignal,
    state: Arc<Mutex<AppState<C>>>,
    self_device_id: String,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    device_connections_arc: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
) where C: Ciphersuite + Send + Sync + 'static, 
<<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync, 
<<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,     
{
    // Clone necessary variables for the spawned task
    let from_clone = from_device_id.clone();
    let state_log_clone = state.clone();
    let self_device_id_clone = self_device_id.clone();
    let internal_cmd_tx_clone = internal_cmd_tx.clone();
    let pc_arc_net_clone = device_connections_arc.clone(); // Use passed parameter
    let signal_clone = signal.clone();
    tokio::spawn(async move {
        // Get or create device connection
        let pc_to_use_result = {
            let device_conns_guard = pc_arc_net_clone.lock().await; // Guard for modification
            match device_conns_guard.get(&from_clone).cloned() {
                Some(pc) => Ok(pc),
                None => {
                    drop(device_conns_guard);
                    // Log removed: WebRTC signal from device received, but connection object missing
                    create_and_setup_device_connection(
                        from_clone.clone(),
                        self_device_id_clone.clone(),
                        pc_arc_net_clone.clone(),
                        internal_cmd_tx_clone.clone(),
                        state_log_clone.clone(),
                        &WEBRTC_API,
                        &WEBRTC_CONFIG,
                    )
                    .await
                }
            }
        };
        match pc_to_use_result {
            Ok(pc_clone) => match signal_clone {
                WebRTCSignal::Offer(offer_info) => {
                    handle_webrtc_offer(
                        &from_clone,
                        offer_info, // Pass offer_info here
                        pc_clone,
                        state_log_clone.clone(),
                        internal_cmd_tx_clone.clone(),
                    ) 
                    .await;
                }
                WebRTCSignal::Answer(answer_info) => {
                    handle_webrtc_answer(
                        &from_clone,
                        answer_info,
                        pc_clone,
                        state_log_clone.clone(),
                    )
                    .await;
                }
                WebRTCSignal::Candidate(candidate_info) => {
                    handle_webrtc_candidate(
                        &from_clone,
                        candidate_info,
                        pc_clone,
                        state_log_clone.clone(),
                    )
                    .await;
                }
            },
            Err(_e) => {
                // Log removed: Failed to create/retrieve connection object for device to handle signal
            }
        }
    });
}

pub async fn handle_webrtc_offer<C>(
    from_device_id: &str,
    offer_info: SDPInfo, // Add offer_info parameter
    pc: Arc<RTCPeerConnection>,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where C: Ciphersuite {    
    // Log removed: WEBRTC OFFER RECEIVED from device
    
    let pc_to_use = pc.clone(); // Start with the assumption we use the passed pc

    match webrtc::peer_connection::sdp::session_description::RTCSessionDescription::offer(
        offer_info.sdp, // Use offer_info.sdp here
    ) {
        Ok(offer_sdp) => {
            if let Err(_e) = pc_to_use.set_remote_description(offer_sdp).await {
                // Log removed: Offer from device: Error setting remote description
                return;
            }
            // Log removed: Offer from device: Remote description set successfully

            // Apply any pending candidates that might have arrived before the offer was set
            apply_pending_candidates(from_device_id, pc_to_use.clone(), state.clone()).await;

            // Create Answer
            match pc_to_use.create_answer(None).await {
                Ok(answer) => {
                    // Clone answer for set_local_description and for sending
                    let answer_to_send = answer.clone();
                    if let Err(_e) = pc_to_use.set_local_description(answer).await {
                        // Log removed: Offer from device: Error setting local description
                        return;
                    }
                    // Log removed: Offer from device: Local description created and set

                    let answer_info_to_send = SDPInfo {
                        sdp: answer_to_send.sdp,                    
                    };
                    // Wrap the WebRTCSignal in WebSocketMessage
                    let webrtc_signal = WebRTCSignal::Answer(answer_info_to_send);
                    let websocket_message = WebSocketMessage::WebRTCSignal(webrtc_signal);

                    match serde_json::to_value(websocket_message) { // Serialize the WebSocketMessage
                        Ok(json_val) => {
                            let relay_msg = SharedClientMsg::Relay {
                                to: from_device_id.to_string(),
                                data: json_val,
                            };
                            if let Err(_e) = internal_cmd_tx.send(InternalCommand::SendToServer(relay_msg)) {
                                // Log removed: Offer from device: Failed to send answer to server
                            } else {
                                // Log removed: Offer from device: Answer sent
                            }
                        }
                        Err(_e) => {
                            // Log removed: Offer from device: Error serializing answer
                        }
                    }
                }
                Err(_e) => {
                    // Log removed: Offer from device: Error creating answer
                }
            }
        }
        Err(_e) => {
            // Log removed: Offer from device: Error parsing offer SDP
        }
    }
}


pub async fn handle_webrtc_answer<C>(
    from_device_id: &str,
    answer_info: SDPInfo,
    pc: Arc<RTCPeerConnection>,
    state: Arc<Mutex<AppState<C>>>,
) where C: Ciphersuite {
    // Log removed: WEBRTC ANSWER RECEIVED from device
    match webrtc::peer_connection::sdp::session_description::RTCSessionDescription::answer(
        answer_info.sdp,
    ) {
        Ok(answer) => {
            if let Err(_e) = pc.set_remote_description(answer).await {
                // Log removed: Error setting remote description (answer) from device
            } else {
                // Log removed: Set remote description (answer) from device
                apply_pending_candidates(from_device_id, pc.clone(), state.clone()).await; 
            }   
        }   
        Err(_e) => {
            // Log removed: Error parsing answer from device
        }
    }
}

/// Handler for WebRTC ICE candidate signals
pub async fn handle_webrtc_candidate<C>(
    from_device_id: &str,
    candidate_info: CandidateInfo,
    pc: Arc<RTCPeerConnection>,
    state: Arc<Mutex<AppState<C>>>,
) where C: Ciphersuite {
    // Log removed: Processing candidate from device
    let candidate_init = RTCIceCandidateInit {
        candidate: candidate_info.candidate,
        sdp_mid: candidate_info.sdp_mid,
        sdp_mline_index: candidate_info.sdp_mline_index,
        username_fragment: None,
    };  
    // Check if remote description is set before adding ICE candidate
    let current_state = pc.signaling_state();
    let remote_description_set = match current_state {
        webrtc::peer_connection::signaling_state::RTCSignalingState::HaveRemoteOffer
        | webrtc::peer_connection::signaling_state::RTCSignalingState::HaveLocalPranswer
        | webrtc::peer_connection::signaling_state::RTCSignalingState::HaveRemotePranswer
        | webrtc::peer_connection::signaling_state::RTCSignalingState::Stable => true,
        _ => false,
    };
    if remote_description_set {
        // Log removed: Remote description is set for device. Adding ICE candidate now 
        if let Err(_e) = pc.add_ice_candidate(candidate_init.clone()).await {
            // Log removed: Error adding ICE candidate from device
        } else {
            // Log removed: Added ICE candidate from device
        }   
    } else {
        let mut state_guard = state.lock().await;
        // Log removed: Storing ICE candidate from device for later
        let candidates = state_guard
            .pending_ice_candidates
            .entry(from_device_id.to_string())
            .or_insert_with(Vec::new);
        candidates.push(candidate_init);
        // Log removed: Queued ICE candidate count
    }  
}  

pub async fn initiate_webrtc_connections<C>( //bRTC connections with all session participants
    participants: Vec<String>,
    self_device_id: String,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    // Debug logging to file
    let debug_msg = format!(
        "[{}] 🚀 initiate_webrtc_connections called: self={}, participants={:?}",
        chrono::Local::now().format("%H:%M:%S%.3f"),
        self_device_id, participants
    );
    let _ = std::fs::write(format!("/tmp/{}-webrtc.log", self_device_id), &debug_msg);
    
    // Log removed: initiate_webrtc_connections called with participants
    
    // Log who's calling this function for debugging
    let _session_role = {
        let state_guard = state.lock().await;
        if let Some(ref session) = state_guard.session {
            if session.proposer_id == self_device_id {
                "CREATOR"
            } else {
                "JOINER"
            }
        } else {
            "UNKNOWN"
        }
    };
    
    // Log removed: Role is initiating WebRTC connections
    
    let device_connections_arc = state.lock().await.device_connections.clone();

    // Step 1: Ensure RTCDeviceConnection objects exist for all other participants.
    // This is necessary for both sending and receiving offers.
    for device_id_str in participants.iter().filter(|p| **p != self_device_id) {
        let (needs_creation, is_connected);
        { // Scope for the lock guard
            let device_conns_guard = device_connections_arc.lock().await;
            if let Some(pc) = device_conns_guard.get(device_id_str) {
                // Check if connection already exists and is in a good state
                let conn_state = pc.connection_state();
                is_connected = matches!(
                    conn_state,
                    webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected |
                    webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connecting
                );
                needs_creation = false;
            } else {
                needs_creation = true;
                is_connected = false;
            }
        }

        // Skip if already connected or connecting
        if is_connected {
            // Log removed: Skipping device: connection already exists and is active
            continue;
        }

        if needs_creation {
            match create_and_setup_device_connection(
                device_id_str.clone(),
                self_device_id.clone(),
                device_connections_arc.clone(),
                internal_cmd_tx.clone(),
                state.clone(),
                &WEBRTC_API,
                &WEBRTC_CONFIG,
            )
            .await
            {
                Ok(_) => {
                    // Log removed: Created device connection for device
                }
                Err(_e) => {
                    // Log removed: Error creating device connection for device
                }
            }
        }
    }

    // Step 2: Filter participants to whom self should make an offer (politeness rule).
    // The device with the LOWER ID creates the offer
    let other_participants: Vec<String> = participants
        .iter()
        .filter(|p_id| **p_id != self_device_id)
        .cloned()
        .collect();
    
    let devices_to_offer_to: Vec<String> = other_participants
        .iter()
        .filter(|p_id| self_device_id < **p_id) // Offer if our ID is smaller
        .cloned()
        .collect();

    // Log removed: Politeness rule check comparing device IDs
    
    // Log detailed comparison for debugging
    for other_id in &other_participants {
        let _comparison = if self_device_id < *other_id {
            format!("{} < {} = TRUE (we create offer)", self_device_id, other_id)
        } else {
            format!("{} < {} = FALSE (we wait for offer)", self_device_id, other_id)
        };
        // Log removed: Comparison result for politeness rule
    }

    if !devices_to_offer_to.is_empty() {
        // Log removed: Will CREATE OFFERS to devices
        initiate_offers_for_session(
            devices_to_offer_to, // Pass only the filtered list of devices to offer to
            self_device_id.clone(),
            device_connections_arc.clone(),
            internal_cmd_tx.clone(),
            state.clone(),
        )
        .await;
    } else {
        // Log removed: Will WAIT FOR OFFERS from devices
    }
}

// Tests commented out - webrtc_test.rs file not found
// #[cfg(test)]
// #[path = "webrtc_test.rs"]
// mod tests;

