use crate::protocal::signal::{SDPInfo, WebRTCSignal, WebSocketMessage};
use crate::utils::state::AppState;
use frost_core::Ciphersuite;

use crate::utils::state::InternalCommand;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, mpsc};

use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc_signal_server::ClientMsg as SharedClientMsg;

pub async fn initiate_offers_for_session<C>(
    participants: Vec<String>,
    self_device_id: String,
    device_connections: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    state: Arc<Mutex<AppState<C>>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar:
        Send + Sync,
{
    state
        .lock()
        .await
        .log
        .push("Initiating WebRTC offers check...".to_string()); // Renamed log

    // Lock connections once
    let device_conns = device_connections.lock().await;
    state.lock().await.log.push(format!(
        "Found {} device connection objects.",
        device_conns.len()
    ));

    for device_id in participants {
        if device_id == self_device_id {
            continue;
        }

        let should_initiate = self_device_id < device_id;

        state.lock().await.log.push(format!(
            "Checking device {}: Should initiate? {}",
            device_id, should_initiate
        ));

        if should_initiate {
            if let Some(pc_arc) = device_conns.get(&device_id) {
                state
                    .lock()
                    .await
                    .log
                    .push(format!("Found PC object for device {}", device_id));
                let current_state = pc_arc.connection_state();
                let signaling_state = pc_arc.signaling_state();

                let negotiation_needed = match current_state {
                    RTCPeerConnectionState::New
                    | RTCPeerConnectionState::Closed
                    | RTCPeerConnectionState::Disconnected
                    | RTCPeerConnectionState::Failed => true,
                    _ => match signaling_state {
                        webrtc::peer_connection::signaling_state::RTCSignalingState::Stable => {
                            false
                        }
                        _ => false,
                    },
                };

                state.lock().await.log.push(format!(
                    "Device {}: Negotiation needed? {} (State: {:?}/{:?})",
                    device_id, negotiation_needed, current_state, signaling_state
                ));

                if !negotiation_needed {
                    continue;
                }

                let is_already_making_offer = state
                    .lock()
                    .await
                    .making_offer
                    .get(&device_id)
                    .copied()
                    .unwrap_or(false);

                state.lock().await.log.push(format!(
                    "Device {}: Already making offer? {}",
                    device_id, is_already_making_offer
                ));

                if is_already_making_offer {
                    continue;
                }

                state.lock().await.log.push(format!(
                    "Proceeding to spawn offer task for device {}",
                    device_id
                ));
                let pc_arc_clone = pc_arc.clone();
                let device_id_clone = device_id.clone();
                let state_clone = state.clone();
                let cmd_tx_clone = cmd_tx.clone();

                tokio::spawn(async move {
                    state_clone
                        .lock()
                        .await
                        .making_offer
                        .insert(device_id_clone.clone(), true);
                    state_clone
                        .lock()
                        .await
                        .log
                        .push(format!("Set making_offer=true for {}", device_id_clone));

                    let offer_result = async {
                        state_clone.lock().await.log.push(format!(
                            "Offer Task [{}]: Creating offer (data channel already created in device setup)...", device_id_clone
                        ));

                        match pc_arc_clone.create_offer(None).await {
                            Ok(offer) => {
                                state_clone.lock().await.log.push(format!(
                                    "Offer Task [{}]: Created offer. Setting local description...", device_id_clone
                                ));

                                if let Err(e) = pc_arc_clone.set_local_description(offer.clone()).await {
                                    state_clone.lock().await.log.push(format!(
                                        "Offer Task [{}]: Error setting local description (offer): {}",
                                        device_id_clone, e
                                    ));
                                    return Err(());
                                }
                                state_clone.lock().await.log.push(format!(
                                    "Offer Task [{}]: Set local description (offer). Serializing and sending...",
                                    device_id_clone
                                ));

                                let signal = WebRTCSignal::Offer(SDPInfo { sdp: offer.sdp });
                                let websocket_message = WebSocketMessage::WebRTCSignal(signal);

                                match serde_json::to_value(websocket_message) {
                                    Ok(json_val) => {
                                        let relay_cmd = InternalCommand::SendToServer(SharedClientMsg::Relay {
                                            to: device_id_clone.clone(),
                                            data: json_val,
                                        });
                                        let _ = cmd_tx_clone.send(relay_cmd);
                                        state_clone
                                            .lock()
                                            .await
                                            .log
                                            .push(format!("Offer Task [{}]: Sent offer.", device_id_clone));
                                    }
                                    Err(e) => {
                                        state_clone.lock().await.log.push(format!(
                                            "Offer Task [{}]: Error serializing offer: {}", device_id_clone, e
                                        ));
                                        return Err(());
                                    }
                                }
                            }
                            Err(e) => {
                                state_clone
                                    .lock()
                                    .await
                                    .log
                                    .push(format!("Offer Task [{}]: Error creating offer: {}", device_id_clone, e));
                                return Err(());
                            }
                        }
                        Ok(())
                    }.await;

                    let outcome = if offer_result.is_ok() {
                        "succeeded"
                    } else {
                        "failed"
                    };
                    state_clone.lock().await.log.push(format!(
                        "Offer Task [{}] {} negotiation.",
                        device_id_clone, outcome
                    ));
                    state_clone
                        .lock()
                        .await
                        .making_offer
                        .insert(device_id_clone.clone(), false);
                    state_clone
                        .lock()
                        .await
                        .log
                        .push(format!("Set making_offer=false for {}", device_id_clone));
                });
            } else {
                state.lock().await.log.push(format!(
                    "Should initiate offer to {}, but connection object not found!",
                    device_id
                ));
            }
        }
    }

    state
        .lock()
        .await
        .log
        .push("Finished WebRTC offers check.".to_string());
}
