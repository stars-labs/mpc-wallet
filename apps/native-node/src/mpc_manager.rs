use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn, error, debug};

pub struct MpcManager {
    device_id: String,
    websocket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    current_session: Option<SessionState>,
    keystore_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: String,
    pub total_participants: u16,
    pub threshold: u16,
    pub participants: Vec<String>,
    pub is_creator: bool,
    pub state: SessionStateType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionStateType {
    Created,
    Joined,
    DkgInProgress,
    DkgComplete,
    SigningInProgress,
    SigningComplete,
    Failed(String),
}

impl MpcManager {
    pub async fn new(device_id: &str) -> Result<Self> {
        let keystore_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mpc-wallet")
            .join("keystores");
            
        tokio::fs::create_dir_all(&keystore_dir).await?;
        
        let keystore_path = keystore_dir.join(format!("{}.json", device_id));
        
        Ok(Self {
            device_id: device_id.to_string(),
            websocket: None,
            current_session: None,
            keystore_path,
        })
    }
    
    pub async fn connect_websocket(&mut self, url: &str) -> Result<()> {
        info!("Connecting to WebSocket server: {}", url);
        
        let (ws_stream, _) = connect_async(url).await?;
        self.websocket = Some(ws_stream);
        
        // Send device registration message
        self.send_message(serde_json::json!({
            "type": "register",
            "device_id": self.device_id,
            "timestamp": chrono::Utc::now().timestamp()
        })).await?;
        
        info!("Successfully connected and registered device: {}", self.device_id);
        Ok(())
    }
    
    pub async fn create_session(&mut self, session_id: String, total: u16, threshold: u16) -> Result<()> {
        info!("Creating session '{}' with {}/{} participants", session_id, threshold, total);
        
        if threshold > total {
            return Err(anyhow::anyhow!("Threshold cannot be greater than total participants"));
        }
        
        let session_state = SessionState {
            session_id: session_id.clone(),
            total_participants: total,
            threshold,
            participants: vec![self.device_id.clone()],
            is_creator: true,
            state: SessionStateType::Created,
        };
        
        let device_id = self.device_id.clone();
        self.current_session = Some(session_state);
        
        self.send_message(serde_json::json!({
            "type": "create_session",
            "session_id": session_id,
            "device_id": device_id,
            "total_participants": total,
            "threshold": threshold,
            "timestamp": chrono::Utc::now().timestamp()
        })).await?;
        
        Ok(())
    }
    
    pub async fn join_session(&mut self, session_id: String) -> Result<()> {
        info!("Joining session: {}", session_id);
        
        let device_id = self.device_id.clone();
        self.send_message(serde_json::json!({
            "type": "join_session",
            "session_id": session_id,
            "device_id": device_id,
            "timestamp": chrono::Utc::now().timestamp()
        })).await?;
        
        Ok(())
    }
    
    pub async fn start_dkg(&mut self) -> Result<()> {
        let session = self.current_session.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;
            
        if session.participants.len() < session.total_participants as usize {
            return Err(anyhow::anyhow!("Not enough participants joined"));
        }
        
        info!("Starting DKG for session: {}", session.session_id);
        session.state = SessionStateType::DkgInProgress;
        
        let device_id = self.device_id.clone();
        let session_id = session.session_id.clone();
        let participants = session.participants.clone();
        let threshold = session.threshold;
        
        self.send_message(serde_json::json!({
            "type": "start_dkg",
            "session_id": session_id,
            "device_id": device_id,
            "participants": participants,
            "threshold": threshold,
            "timestamp": chrono::Utc::now().timestamp()
        })).await?;
        
        Ok(())
    }
    
    pub async fn export_keystore(&self) -> Result<String> {
        let session = self.current_session.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;
            
        if !matches!(session.state, SessionStateType::DkgComplete) {
            return Err(anyhow::anyhow!("DKG not complete, cannot export keystore"));
        }
        
        // Create export data structure
        let export_data = serde_json::json!({
            "device_id": self.device_id,
            "session_id": session.session_id,
            "threshold": session.threshold,
            "total_participants": session.total_participants,
            "participants": session.participants,
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "format_version": "1.0"
        });
        
        let export_path = self.keystore_path.with_extension("exported.json");
        tokio::fs::write(&export_path, serde_json::to_string_pretty(&export_data)?).await?;
        
        info!("Keystore exported to: {}", export_path.display());
        Ok(export_path.to_string_lossy().to_string())
    }
    
    pub async fn initiate_signing(&mut self, tx_data: String, blockchain: String) -> Result<()> {
        let session = self.current_session.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;
            
        if !matches!(session.state, SessionStateType::DkgComplete) {
            return Err(anyhow::anyhow!("DKG not complete, cannot initiate signing"));
        }
        
        info!("Initiating signing for {} transaction", blockchain);
        session.state = SessionStateType::SigningInProgress;
        
        let device_id = self.device_id.clone();
        let session_id = session.session_id.clone();
        let participants = session.participants.clone();
        
        self.send_message(serde_json::json!({
            "type": "initiate_signing",
            "session_id": session_id,
            "device_id": device_id,
            "transaction_data": tx_data,
            "blockchain": blockchain,
            "participants": participants,
            "timestamp": chrono::Utc::now().timestamp()
        })).await?;
        
        Ok(())
    }
    
    async fn send_message(&mut self, message: Value) -> Result<()> {
        if let Some(websocket) = &mut self.websocket {
            let message_str = serde_json::to_string(&message)?;
            debug!("Sending message: {}", message_str);
            websocket.send(Message::Text(message_str)).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("WebSocket not connected"))
        }
    }
    
    pub async fn handle_incoming_messages(&mut self) -> Result<()> {
        let mut messages_to_process = Vec::new();
        
        if let Some(websocket) = &mut self.websocket {
            while let Some(message) = websocket.next().await {
                match message? {
                    Message::Text(text) => {
                        messages_to_process.push(text);
                    }
                    Message::Close(_) => {
                        warn!("WebSocket connection closed");
                        break;
                    }
                    _ => {}
                }
            }
        }
        
        // Process messages after releasing the websocket borrow
        for text in messages_to_process {
            if let Err(e) = self.process_message(&text).await {
                error!("Error processing message: {}", e);
            }
        }
        
        Ok(())
    }
    
    async fn process_message(&mut self, message: &str) -> Result<()> {
        let data: Value = serde_json::from_str(message)?;
        let message_type = data["type"].as_str().unwrap_or("");
        
        debug!("Processing message type: {}", message_type);
        
        match message_type {
            "session_joined" => {
                if let Some(session) = &mut self.current_session {
                    if let Some(participant) = data["device_id"].as_str() {
                        if !session.participants.contains(&participant.to_string()) {
                            session.participants.push(participant.to_string());
                            info!("Participant joined session: {}", participant);
                        }
                    }
                }
            }
            "dkg_round1_complete" => {
                info!("DKG Round 1 completed");
            }
            "dkg_round2_complete" => {
                info!("DKG Round 2 completed");
                if let Some(session) = &mut self.current_session {
                    session.state = SessionStateType::DkgComplete;
                }
            }
            "dkg_failed" => {
                warn!("DKG failed: {}", data["error"].as_str().unwrap_or("Unknown error"));
                if let Some(session) = &mut self.current_session {
                    session.state = SessionStateType::Failed(
                        data["error"].as_str().unwrap_or("DKG failed").to_string()
                    );
                }
            }
            "signing_complete" => {
                info!("Signing completed successfully");
                if let Some(session) = &mut self.current_session {
                    session.state = SessionStateType::SigningComplete;
                }
            }
            "signing_failed" => {
                warn!("Signing failed: {}", data["error"].as_str().unwrap_or("Unknown error"));
                if let Some(session) = &mut self.current_session {
                    session.state = SessionStateType::Failed(
                        data["error"].as_str().unwrap_or("Signing failed").to_string()
                    );
                }
            }
            _ => {
                debug!("Unhandled message type: {}", message_type);
            }
        }
        
        Ok(())
    }
    
    pub fn get_current_session(&self) -> Option<&SessionState> {
        self.current_session.as_ref()
    }
}