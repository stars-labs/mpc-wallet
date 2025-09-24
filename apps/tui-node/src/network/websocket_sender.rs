// Global WebSocket sender that actually processes InternalCommand messages
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;
use webrtc_signal_server::ClientMsg;
use crate::utils::state::InternalCommand;
use frost_core::Ciphersuite;
use tracing::{info, error};

// Global WebSocket sender instance
static mut GLOBAL_WS_SENDER: Option<Arc<Mutex<Option<mpsc::UnboundedSender<ClientMsg>>>>> = None;

/// Initialize the global WebSocket sender
pub fn init_global_ws_sender(tx: mpsc::UnboundedSender<ClientMsg>) {
    unsafe {
        GLOBAL_WS_SENDER = Some(Arc::new(Mutex::new(Some(tx))));
    }
    info!("✅ Global WebSocket sender initialized");
}

/// Get the global WebSocket sender
pub async fn get_global_ws_sender() -> Option<mpsc::UnboundedSender<ClientMsg>> {
    unsafe {
        if let Some(ref sender) = GLOBAL_WS_SENDER {
            sender.lock().await.clone()
        } else {
            None
        }
    }
}

/// Process InternalCommand and send through WebSocket
pub async fn process_internal_command<C>(
    cmd: InternalCommand<C>,
    ws_sink: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
) where
    C: Ciphersuite + Send + Sync + 'static,
{
    match cmd {
        InternalCommand::SendToServer(msg) => {
            // Send the message through WebSocket
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    info!("📤 Sending to WebSocket: {}", json);
                    if let Err(e) = ws_sink.send(Message::text(json)).await {
                        error!("❌ Failed to send message through WebSocket: {}", e);
                    } else {
                        info!("✅ Message sent through WebSocket");
                    }
                }
                Err(e) => {
                    error!("❌ Failed to serialize message: {}", e);
                }
            }
        }
        _ => {
            // Other commands not handled here
        }
    }
}

/// Run a WebSocket handler that processes InternalCommands
pub async fn run_websocket_handler<C>(
    mut internal_cmd_rx: mpsc::UnboundedReceiver<InternalCommand<C>>,
    mut ws_sink: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    info!("🚀 WebSocket handler started - listening for InternalCommands");

    while let Some(cmd) = internal_cmd_rx.recv().await {
        match cmd {
            InternalCommand::SendToServer(msg) => {
                // Send the message through WebSocket
                match serde_json::to_string(&msg) {
                    Ok(json) => {
                        info!("📤 WebSocket handler sending: {}", json);
                        if let Err(e) = ws_sink.send(Message::text(json)).await {
                            error!("❌ WebSocket handler failed to send: {}", e);
                        } else {
                            info!("✅ WebSocket handler sent message successfully");
                        }
                    }
                    Err(e) => {
                        error!("❌ WebSocket handler failed to serialize: {}", e);
                    }
                }
            }
            _ => {
                // Other commands not handled by WebSocket sender
            }
        }
    }

    info!("WebSocket handler stopped");
}