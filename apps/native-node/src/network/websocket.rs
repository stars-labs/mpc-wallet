use anyhow::Result;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info, warn};

use crate::commands::{ClientMessage, InternalCommand, ServerMessage};
use crate::state::SharedAppState;

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect_websocket(url: &str, device_id: &str) -> Result<WsStream> {
    info!("Connecting to WebSocket server: {}", url);
    
    let (mut ws_stream, _) = connect_async(url).await?;
    
    // Send registration message
    let register_msg = ClientMessage::Register {
        device_id: device_id.to_string(),
    };
    
    let message = serde_json::to_string(&register_msg)?;
    ws_stream.send(Message::Text(message)).await?;
    
    info!("Successfully connected and registered with device ID: {}", device_id);
    Ok(ws_stream)
}

pub async fn handle_websocket_message(
    message: Message,
    state: SharedAppState,
    command_tx: &tokio::sync::mpsc::Sender<InternalCommand>,
) -> Result<()> {
    match message {
        Message::Text(text) => {
            match serde_json::from_str::<ServerMessage>(&text) {
                Ok(server_msg) => {
                    process_server_message(server_msg, state, command_tx).await?;
                }
                Err(e) => {
                    error!("Failed to parse server message: {}", e);
                }
            }
        }
        Message::Close(_) => {
            warn!("WebSocket connection closed by server");
            let mut state = state.lock().await;
            state.websocket_connected = false;
            state.add_log("WebSocket connection closed".to_string());
        }
        _ => {}
    }
    
    Ok(())
}

async fn process_server_message(
    message: ServerMessage,
    state: SharedAppState,
    command_tx: &tokio::sync::mpsc::Sender<InternalCommand>,
) -> Result<()> {
    match message {
        ServerMessage::DeviceList { devices } => {
            let mut state = state.lock().await;
            state.add_log(format!("Connected devices: {}", devices.join(", ")));
            
            // Update device status
            for device_id in devices {
                if device_id != state.device_id {
                    state.update_device_status(device_id, true);
                }
            }
        }
        
        ServerMessage::Relay { from_device, to_device: _, message } => {
            // Parse the relayed message
            if let Ok(signal) = serde_json::from_value::<crate::commands::WebRTCSignal>(message.clone()) {
                // Handle WebRTC signal
                command_tx.send(InternalCommand::SendWebRTCSignal {
                    target_device: from_device,
                    signal,
                }).await?;
            } else if let Ok(direct_msg) = serde_json::from_value::<crate::commands::DirectMessage>(message) {
                // Handle direct message
                handle_direct_message(direct_msg, from_device, state, command_tx).await?;
            }
        }
        
        ServerMessage::Error { message } => {
            let mut state = state.lock().await;
            state.add_log(format!("Server error: {}", message));
            error!("Server error: {}", message);
        }
    }
    
    Ok(())
}

async fn handle_direct_message(
    message: crate::commands::DirectMessage,
    from_device: String,
    state: SharedAppState,
    command_tx: &tokio::sync::mpsc::Sender<InternalCommand>,
) -> Result<()> {
    use crate::commands::DirectMessage;
    
    match message {
        DirectMessage::SessionProposal { session_id, total_participants, threshold, curve } => {
            let invite = crate::state::SessionInvite {
                session_id,
                from_device,
                total_participants,
                threshold,
                curve,
                timestamp: chrono::Utc::now().timestamp(),
            };
            
            command_tx.send(InternalCommand::ShowSessionProposal { invite }).await?;
        }
        
        DirectMessage::SessionResponse { session_id, accepted } => {
            let mut state = state.lock().await;
            if accepted {
                state.add_log(format!("{} accepted session {}", from_device, session_id));
            } else {
                state.add_log(format!("{} rejected session {}", from_device, session_id));
            }
        }
        
        DirectMessage::MeshReady { session_id } => {
            let mut state = state.lock().await;
            state.add_log(format!("Mesh ready signal from {} for session {}", from_device, session_id));
            
            // Check if we should trigger DKG
            drop(state);
            command_tx.send(InternalCommand::CheckAndTriggerDkg).await?;
        }
        
        DirectMessage::DkgRound1 { data } => {
            command_tx.send(InternalCommand::ProcessDkgRound1 { from_device, data }).await?;
        }
        
        DirectMessage::DkgRound2 { data } => {
            command_tx.send(InternalCommand::ProcessDkgRound2 { from_device, data }).await?;
        }
        
        DirectMessage::SigningRequest { request_id, transaction_data, blockchain } => {
            let request = crate::commands::SigningRequestMessage {
                request_id,
                from_device,
                transaction_data,
                blockchain,
            };
            command_tx.send(InternalCommand::ProcessSigningRequest { request }).await?;
        }
        
        DirectMessage::SigningCommitment { request_id, commitment } => {
            command_tx.send(InternalCommand::ProcessSigningCommitment { 
                from_device, 
                request_id, 
                commitment 
            }).await?;
        }
        
        DirectMessage::SigningShare { request_id, share } => {
            command_tx.send(InternalCommand::ProcessSigningShare { 
                from_device, 
                request_id, 
                share 
            }).await?;
        }
    }
    
    Ok(())
}

pub async fn send_websocket_message(
    ws_stream: &mut WsStream,
    message: ClientMessage,
) -> Result<()> {
    let message_str = serde_json::to_string(&message)?;
    ws_stream.send(Message::Text(message_str)).await?;
    Ok(())
}