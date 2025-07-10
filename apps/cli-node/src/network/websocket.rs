use futures_util::SinkExt;
// Import from lib.rs
use crate::utils::state::{AppState, InternalCommand}; // <-- Add SessionResponse here

use crate::protocal::signal::{SessionInfo, SessionResponse};
use std::sync::Arc;

use std::collections::HashMap;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::Message;

use webrtc::peer_connection::RTCPeerConnection;

use webrtc_signal_server::ServerMsg;
// Add display-related imports for better status handling
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
                        ServerMsg::Devices { devices } => {
                            let mut state_guard = state.lock().await;
                            state_guard.devices = devices.clone();
                        }
                        ServerMsg::Error { error } => {
                            let mut state_guard = state.lock().await;
                            state_guard.log.push(format!("Error: {}", error));
                        }
                        ServerMsg::Relay { from, data } => {
                            state.lock().await.log.push(format!(
                                "Relay from {}: {:?}",
                                from,
                                data.clone()
                            )); // Log data by cloning

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

                                Ok(crate::protocal::signal::WebSocketMessage::SessionProposal(
                                    proposal,
                                )) => {
                                    let mut state_guard = state.lock().await;
                                    state_guard.log.push(format!(
                                            "Received SessionProposal from {}: ID={}, Total={}, Threshold={}, Participants={:?}",
                                            from, proposal.session_id, proposal.total, proposal.threshold, proposal.participants
                                        ));

                                    let invite_info = SessionInfo {
                                        session_id: proposal.session_id.clone(),
                                        proposer_id: from.clone(), // Store the proposer's ID
                                        total: proposal.total,
                                        threshold: proposal.threshold,
                                        participants: proposal.participants.clone(),
                                        accepted_devices: Vec::new(),
                                        session_type: proposal.session_type.clone(),
                                    };
                                    state_guard.invites.push(invite_info);
                                }
                                Ok(crate::protocal::signal::WebSocketMessage::SessionResponse(
                                    response,
                                )) => {
                                    state.lock().await.log.push(format!(
                                        "Received SessionResponse from {}: {:?}",
                                        from,
                                        response.clone()
                                    ));
                                    // Convert to the internal SessionResponse type if needed
                                    let internal_response = SessionResponse {
                                        session_id: response.session_id.clone(),
                                        accepted: response.accepted,
                                        wallet_status: response.wallet_status.clone(),
                                    };
                                    if let Err(e) = internal_cmd_tx.send(
                                        InternalCommand::ProcessSessionResponse {
                                            from_device_id: from.clone(),
                                            response: internal_response,
                                        },
                                    ) {
                                        state.lock().await.log.push(format!(
                                            "Failed to send ProcessSessionResponse command: {}",
                                            e
                                        ));
                                    }
                                }
                                Err(e) => {
                                    state
                                        .lock()
                                        .await
                                        .log
                                        .push(format!("Error parsing WebSocketMessage: {}", e));
                                    state.lock().await.log.push(format!(
                                        "Error parsing WebSocketMessage: {}",
                                        data.clone()
                                    ));
                                }
                            }
                        }
                    }
                }

                Err(e) => {
                    state
                        .lock()
                        .await
                        .log
                        .push(format!("Error parsing server message: {}", e));
                }
            }
        }
        Message::Close(_) => {
            state
                .lock()
                .await
                .log
                .push("WebSocket connection closed by server.".to_string());
        }
        Message::Ping(ping_data) => {
            let _ = ws_sink.send(Message::Pong(ping_data)).await;
        }
        Message::Pong(_) => {}
        Message::Binary(_) => {
            state
                .lock()
                .await
                .log
                .push("Received unexpected binary message.".to_string());
        }
        Message::Frame(_) => {}
    }
}
