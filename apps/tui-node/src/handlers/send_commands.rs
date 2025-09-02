use crate::protocal::signal::WebRTCMessage;
use crate::utils::device::send_webrtc_message;
use crate::utils::appstate_compat::AppState;
use frost_core::Ciphersuite;
use futures_util::SinkExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

/// Handles sending a message to the WebRTC signaling server
pub async fn handle_send_to_server<C>(
    shared_msg: webrtc_signal_server::ClientMsg,
    _state: Arc<Mutex<AppState<C>>>,
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
    if let Ok(msg_str) = serde_json::to_string(&shared_msg) {
        match ws_sink.send(Message::Text(msg_str.into())).await {
            Ok(_) => {
                // Successfully sent to server
            }
            Err(_e) => {
                // Failed to send to server
            }
        }
    } else {
        // Failed to serialize message
    }
}

/// Handles sending a direct message to a device
pub async fn handle_send_direct<C>(to: String, message: String, state: Arc<Mutex<AppState<C>>>)
where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar:
        Send + Sync,
{
    let state_clone = state.clone();
    tokio::spawn(async move {
        let webrtc_msg = WebRTCMessage::SimpleMessage { text: message };
        if let Err(_e) = send_webrtc_message(&to, &webrtc_msg, state_clone.clone()).await {
            // Error sending direct message
        } else {
            // Sent direct message
        }
    });
}
