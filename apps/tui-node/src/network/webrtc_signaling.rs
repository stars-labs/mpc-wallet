// WebRTC signaling that actually sends offers/answers through WebSocket
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use tracing::{info, error};
use crate::protocal::signal::{WebRTCSignal, SDPInfo, WebSocketMessage};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use futures_util::SinkExt;

/// Global WebSocket sender that can be used to send signals
pub static mut GLOBAL_WS_TX: Option<Arc<Mutex<Option<Arc<Mutex<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, WsMessage>>>>>>> = None;

/// Set the global WebSocket sender
pub unsafe fn set_global_ws_tx(tx: Arc<Mutex<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, WsMessage>>>) {
    GLOBAL_WS_TX = Some(Arc::new(Mutex::new(Some(tx))));
}

/// Complete WebRTC signaling implementation
pub async fn complete_webrtc_signaling(
    self_device_id: String,
    participants: Vec<String>,
    device_connections: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
) {
    info!("🚀 Complete WebRTC signaling for {} participants", participants.len());
    
    // Create debug log
    let debug_msg = format!(
        "[{}] 🚀 complete_webrtc_signaling: self={}, participants={:?}",
        chrono::Local::now().format("%H:%M:%S%.3f"),
        self_device_id, participants
    );
    let _ = std::fs::write(format!("/tmp/{}-webrtc-complete.log", self_device_id), &debug_msg);
    
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
    let devices_to_offer: Vec<String> = other_participants
        .into_iter()
        .filter(|p| self_device_id < *p)
        .collect();
    
    info!("📤 Will send offers to {} devices: {:?}", devices_to_offer.len(), devices_to_offer);
    
    for device_id in devices_to_offer {
        let conns = device_connections.lock().await;
        if let Some(pc) = conns.get(&device_id) {
            info!("🎯 Creating offer for {}", device_id);
            
            // Create data channel first
            match pc.create_data_channel("data", None).await {
                Ok(_dc) => {
                    info!("✅ Created data channel for {}", device_id);
                    
                    // Now create offer
                    match pc.create_offer(None).await {
                        Ok(offer) => {
                            info!("✅ Created offer for {}", device_id);
                            
                            // Set local description
                            if let Err(e) = pc.set_local_description(offer.clone()).await {
                                error!("Failed to set local description: {}", e);
                            } else {
                                info!("✅ Set local description for {}", device_id);
                                
                                // CRITICAL: Actually send the offer through WebSocket
                                if let Err(e) = send_offer_via_websocket(
                                    &self_device_id,
                                    &device_id,
                                    offer
                                ).await {
                                    error!("❌ Failed to send offer to {}: {}", device_id, e);
                                } else {
                                    info!("✅ Sent offer to {} via WebSocket", device_id);
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
    
    info!("✅ Complete WebRTC signaling finished");
}

/// Send an offer through the WebSocket
async fn send_offer_via_websocket(
    from_device_id: &str,
    to_device_id: &str,
    offer: RTCSessionDescription
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the global WebSocket sender
    let ws_tx_option = unsafe {
        GLOBAL_WS_TX.as_ref().map(|arc| arc.clone())
    };
    
    if let Some(ws_tx_arc) = ws_tx_option {
        let ws_tx_guard = ws_tx_arc.lock().await;
        if let Some(ws_tx) = ws_tx_guard.as_ref() {
            let mut ws_tx = ws_tx.lock().await;
            
            // Create WebRTC signal
            let signal = WebRTCSignal::Offer(SDPInfo {
                sdp: offer.sdp,
            });
            
            // Wrap in WebSocket message
            let ws_msg = WebSocketMessage::WebRTCSignal(signal);
            
            // Create relay message for the target device
            let relay_msg = serde_json::json!({
                "type": "relay",
                "to": to_device_id,
                "data": ws_msg
            });
            
            // Send through WebSocket
            let msg_text = serde_json::to_string(&relay_msg)?;
            ws_tx.send(WsMessage::Text(msg_text)).await?;
            
            info!("📤 Sent WebRTC offer from {} to {}", from_device_id, to_device_id);
            Ok(())
        } else {
            Err("WebSocket sender not available".into())
        }
    } else {
        Err("Global WebSocket sender not initialized".into())
    }
}