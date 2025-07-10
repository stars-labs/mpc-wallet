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
use crate::utils::state::{InternalCommand, AppState}; // <-- Add SessionResponse here

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
                    state_log_clone.lock().await.log.push(format!(
                        "WebRTC signal from {} received, but connection object missing. Attempting creation...",
                        from_clone 
                    ));
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
            Err(e) => {
                state_log_clone.lock().await.log.push(format!(
                    "Failed to create/retrieve connection object for {} to handle signal: {}",
                    from_clone, e
                ));
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
    let pc_to_use = pc.clone(); // Start with the assumption we use the passed pc

    match webrtc::peer_connection::sdp::session_description::RTCSessionDescription::offer(
        offer_info.sdp, // Use offer_info.sdp here
    ) {
        Ok(offer_sdp) => {
            if let Err(e) = pc_to_use.set_remote_description(offer_sdp).await {
                state.lock().await.log.push(format!(
                    "Offer from {}: Error setting remote description (offer): {}",
                    from_device_id, e
                ));
                return;
            }
            state.lock().await.log.push(format!(
                "Offer from {}: Remote description (offer) set successfully.",
                from_device_id
            ));

            // Apply any pending candidates that might have arrived before the offer was set
            apply_pending_candidates(from_device_id, pc_to_use.clone(), state.clone()).await;

            // Create Answer
            match pc_to_use.create_answer(None).await {
                Ok(answer) => {
                    // Clone answer for set_local_description and for sending
                    let answer_to_send = answer.clone();
                    if let Err(e) = pc_to_use.set_local_description(answer).await {
                        state.lock().await.log.push(format!(
                            "Offer from {}: Error setting local description (answer): {}",
                            from_device_id, e
                        ));
                        return;
                    }
                    state.lock().await.log.push(format!(
                        "Offer from {}: Local description (answer) created and set.",
                        from_device_id
                    ));

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
                            if let Err(e) = internal_cmd_tx.send(InternalCommand::SendToServer(relay_msg)) {
                                state.lock().await.log.push(format!(
                                    "Offer from {}: Failed to send answer to server: {}",
                                    from_device_id, e
                                ));
                            } else {
                                state.lock().await.log.push(format!(
                                    "Offer from {}: Answer sent to {}",
                                    from_device_id, from_device_id
                                ));
                            }
                        }
                        Err(e) => {
                            state.lock().await.log.push(format!(
                                "Offer from {}: Error serializing answer: {}",
                                from_device_id, e
                            ));
                        }
                    }
                }
                Err(e) => {
                    state.lock().await.log.push(format!(
                        "Offer from {}: Error creating answer: {}",
                        from_device_id, e
                    ));
                }
            }
        }
        Err(e) => {
            state.lock().await.log.push(format!(
                "Offer from {}: Error parsing offer SDP: {}",
                from_device_id, e
            ));
        }
    }
}


pub async fn handle_webrtc_answer<C>(
    from_device_id: &str,
    answer_info: SDPInfo,
    pc: Arc<RTCPeerConnection>,
    state: Arc<Mutex<AppState<C>>>,
) where C: Ciphersuite {
    state
        .lock()
        .await
        .log
        .push(format!("Processing answer from {}...", from_device_id));
    match webrtc::peer_connection::sdp::session_description::RTCSessionDescription::answer(
        answer_info.sdp,
    ) {
        Ok(answer) => {
            if let Err(e) = pc.set_remote_description(answer).await {
                state.lock().await.log.push(format!(
                    "Error setting remote description (answer) from {}: {}",
                    from_device_id, e
                ));
            } else {
                state.lock().await.log.push(format!(
                    "Set remote description (answer) from {}",
                    from_device_id
                ));
                apply_pending_candidates(from_device_id, pc.clone(), state.clone()).await; 
            }   
        }   
        Err(e) => {
            state
                .lock()
                .await
                .log
                .push(format!("Error parsing answer from {}: {}", from_device_id, e));
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
    state
        .lock()
        .await
        .log //it
        .push(format!("Processing candidate from {}...", from_device_id));
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
        state.lock().await.log.push(format!(
            "Remote description is set for {}. Adding ICE candidate now.",
            from_device_id
        )); 
        if let Err(e) = pc.add_ice_candidate(candidate_init.clone()).await {
            state.lock().await.log.push(format!(
                "Error adding ICE candidate from {}: {}",
                from_device_id, e
            ));
        } else {
            state
                .lock()
                .await
                .log
                .push(format!("Added ICE candidate from {}", from_device_id));
        }   
    } else {
        let mut state_guard = state.lock().await;
        state_guard.log.push(format!(
            "Storing ICE candidate from {} for later (remote description not set yet)",
            from_device_id 
        ));
        let candidates = state_guard
            .pending_ice_candidates
            .entry(from_device_id.to_string())
            .or_insert_with(Vec::new);
        candidates.push(candidate_init);
        let queued_msg = format!(
            "Queued ICE candidate from {}. Total queued: {}",
            from_device_id,
            candidates.len()
        );
        state_guard.log.push(queued_msg);
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
    let device_connections_arc = state.lock().await.device_connections.clone();

    // Step 1: Ensure RTCDeviceConnection objects exist for all other participants.
    // This is necessary for both sending and receiving offers.
    for device_id_str in participants.iter().filter(|p| **p != self_device_id) {
        let needs_creation;
        { // Scope for the lock guard
            let device_conns_guard = device_connections_arc.lock().await;
            needs_creation = !device_conns_guard.contains_key(device_id_str);
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
                    state
                        .lock()
                        .await
                        .log
                        .push(format!("Created device connection for {}", device_id_str));
                }
                Err(e) => {
                    state.lock().await.log.push(format!(
                        "Error creating device connection for {}: {}",
                        device_id_str, e
                    ));
                }
            }
        }
    }

    // Step 2: Filter participants to whom self should make an offer (politeness rule).
    let devices_to_offer_to: Vec<String> = participants
        .iter()
        .filter(|p_id| **p_id != self_device_id && *self_device_id < ***p_id) // Offer if self_device_id is smaller
        .cloned()
        .collect();

    if !devices_to_offer_to.is_empty() {
        state.lock().await.log.push(format!(
            "Initiating offers to devices based on ID ordering: {:?}",
            devices_to_offer_to
        ));
        initiate_offers_for_session(
            devices_to_offer_to, // Pass only the filtered list of devices to offer to
            self_device_id.clone(),
            device_connections_arc.clone(),
            internal_cmd_tx.clone(),
            state.clone(),
        )
        .await;
    } else {
        state.lock().await.log.push(
            "No devices to initiate offers to based on ID ordering. Waiting for incoming offers.".to_string()
        );
    }
}

#[cfg(test)]
#[path = "webrtc_test.rs"]
mod tests;

