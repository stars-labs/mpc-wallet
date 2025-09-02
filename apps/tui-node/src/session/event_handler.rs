use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use anyhow::{Result, Context};

use super::{SessionEvent, SessionStateMachine, ConnectionPool};
use crate::protocal::signal::{
    WebSocketMessage, SessionProposal, SessionResponse, SessionUpdate, 
    SessionUpdateType, WebRTCSignal, SDPInfo
};

/// Handles session events and their side effects
pub struct SessionEventHandler {
    device_id: String,
    state_machine: Arc<RwLock<SessionStateMachine>>,
    connection_pool: Arc<ConnectionPool>,
    /// Channel to send WebSocket messages
    ws_tx: Option<tokio::sync::mpsc::UnboundedSender<WebSocketMessage>>,
    /// Channel to send internal events
    event_tx: Option<tokio::sync::mpsc::UnboundedSender<SessionEvent>>,
}

impl SessionEventHandler {
    pub fn new(
        device_id: String,
        state_machine: Arc<RwLock<SessionStateMachine>>,
        connection_pool: Arc<ConnectionPool>,
    ) -> Self {
        Self {
            device_id,
            state_machine,
            connection_pool,
            ws_tx: None,
            event_tx: None,
        }
    }
    
    /// Set the WebSocket message channel
    pub fn set_ws_channel(&mut self, tx: tokio::sync::mpsc::UnboundedSender<WebSocketMessage>) {
        self.ws_tx = Some(tx);
    }
    
    /// Set the internal event channel
    pub fn set_event_channel(&mut self, tx: tokio::sync::mpsc::UnboundedSender<SessionEvent>) {
        self.event_tx = Some(tx);
    }
    
    /// Handle a session event
    pub async fn handle(&mut self, event: SessionEvent) -> Result<()> {
        tracing::debug!("Handling event: {:?}", event);
        
        // First, transition the state machine
        let mut state_machine = self.state_machine.write().await;
        state_machine.transition(event.clone())
            .context("State transition failed")?;
        drop(state_machine); // Release lock early
        
        // Then handle side effects based on the event
        match event {
            SessionEvent::CreateSession { session_id, threshold, total } => {
                self.handle_create_session(session_id, threshold, total).await?;
            }
            
            SessionEvent::JoinSession { session_id } => {
                self.handle_join_session(session_id).await?;
            }
            
            SessionEvent::ProposalReceived { from, proposal } => {
                self.handle_proposal_received(from, proposal).await?;
            }
            
            SessionEvent::AcceptProposal => {
                self.handle_accept_proposal().await?;
            }
            
            SessionEvent::ResponseReceived { from, response } => {
                self.handle_response_received(from, response).await?;
            }
            
            SessionEvent::SessionUpdate { from, update } => {
                self.handle_session_update(from, update).await?;
            }
            
            SessionEvent::MeshReady => {
                self.handle_mesh_ready().await?;
            }
            
            SessionEvent::LeaveSession => {
                self.handle_leave_session().await?;
            }
            
            SessionEvent::RetryJoin { session_id, attempt } => {
                self.handle_retry_join(session_id, attempt).await?;
            }
            
            _ => {
                // Other events may not need side effects
                tracing::debug!("Event {:?} requires no side effects", event);
            }
        }
        
        Ok(())
    }
    
    async fn handle_create_session(
        &mut self,
        session_id: String,
        threshold: u16,
        total: u16,
    ) -> Result<()> {
        tracing::info!("Created session: {} ({}/{})", session_id, threshold, total);
        
        // Broadcast session availability via SessionUpdate
        if let Some(ws_tx) = &self.ws_tx {
            let update = SessionUpdate {
                session_id: session_id.clone(),
                accepted_devices: vec![self.device_id.clone()],
                update_type: SessionUpdateType::FullSync,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            
            let msg = WebSocketMessage::SessionUpdate(update);
            ws_tx.send(msg).context("Failed to send announce message")?;
        }
        
        Ok(())
    }
    
    async fn handle_join_session(&mut self, session_id: String) -> Result<()> {
        tracing::info!("Sending join request for session: {}", session_id);
        
        // Send join request as SessionUpdate
        let update = SessionUpdate {
            session_id: session_id.clone(),
            accepted_devices: vec![self.device_id.clone()],
            update_type: SessionUpdateType::ParticipantJoined,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        self.send_with_retry(
            WebSocketMessage::SessionUpdate(update),
            3, // max retries
        ).await?;
        
        // Set timeout for response
        let session_id_clone = session_id.clone();
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            if let Some(tx) = event_tx {
                let _ = tx.send(SessionEvent::JoinTimeout {
                    session_id: session_id_clone,
                });
            }
        });
        
        Ok(())
    }
    
    async fn handle_proposal_received(
        &mut self,
        from: String,
        proposal: SessionProposal,
    ) -> Result<()> {
        tracing::info!(
            "Received proposal from {}: {} ({}/{})", 
            from, proposal.session_id, proposal.threshold, proposal.total
        );
        
        // Auto-accept if we initiated the join
        let state_machine = self.state_machine.read().await;
        if let Some(session_id) = state_machine.get_session_id() {
            if session_id == proposal.session_id {
                drop(state_machine);
                // Auto-accept
                tracing::info!("Auto-accepting proposal for session we joined: {}", session_id);
                if let Some(tx) = &self.event_tx {
                    let _ = tx.send(SessionEvent::AcceptProposal);
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_accept_proposal(&mut self) -> Result<()> {
        let state_machine = self.state_machine.read().await;
        
        if let Some(session_id) = state_machine.get_session_id() {
            tracing::info!("Accepting proposal for session: {}", session_id);
            
            // Send acceptance response
            if let Some(ws_tx) = &self.ws_tx {
                let response = SessionResponse {
                    session_id: session_id.clone(),
                    from_device_id: self.device_id.clone(),
                    accepted: true,
                    wallet_status: None,
                    reason: None,
                };
                
                let msg = WebSocketMessage::SessionResponse(response);
                ws_tx.send(msg).context("Failed to send response")?;
            }
            
            // Also send a session update to notify others
            if let Some(ws_tx) = &self.ws_tx {
                let update = SessionUpdate {
                    session_id: session_id.clone(),
                    accepted_devices: vec![self.device_id.clone()],
                    update_type: SessionUpdateType::ParticipantJoined,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };
                
                let msg = WebSocketMessage::SessionUpdate(update);
                ws_tx.send(msg).context("Failed to send update")?;
            }
            
            // Initiate WebRTC connections
            if let Some(participants) = state_machine.get_participants() {
                drop(state_machine);
                self.initiate_webrtc_connections(participants).await?;
            }
        }
        
        Ok(())
    }
    
    async fn handle_response_received(
        &mut self,
        from: String,
        response: SessionResponse,
    ) -> Result<()> {
        if response.accepted {
            tracing::info!("{} accepted session: {}", from, response.session_id);
            
            // Check if we have all acceptances
            let state_machine = self.state_machine.read().await;
            if let Some(accepted) = state_machine.get_accepted_devices() {
                if let Some(participants) = state_machine.get_participants() {
                    if accepted.len() == participants.len() {
                        tracing::info!("All participants accepted, establishing mesh");
                        drop(state_machine);
                        self.establish_mesh().await?;
                    }
                }
            }
        } else {
            tracing::warn!("{} rejected session: {}", from, response.session_id);
        }
        
        Ok(())
    }
    
    async fn handle_session_update(
        &mut self,
        from: String,
        update: SessionUpdate,
    ) -> Result<()> {
        tracing::debug!(
            "Session update from {}: {} devices accepted", 
            from, update.accepted_devices.len()
        );
        
        // Check if mesh can be established
        let state_machine = self.state_machine.read().await;
        if let Some(accepted) = state_machine.get_accepted_devices() {
            if let Some(_participants) = state_machine.get_participants() {
                // Check if we have enough acceptances for threshold
                let threshold = 2; // TODO: Get from session info
                if accepted.len() >= threshold as usize && !state_machine.is_mesh_ready() {
                    tracing::info!("Threshold reached, establishing mesh");
                    drop(state_machine);
                    self.establish_mesh().await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_mesh_ready(&mut self) -> Result<()> {
        tracing::info!("Mesh is ready, can start DKG");
        
        // Trigger DKG if appropriate
        // DKG will be triggered separately based on mesh ready state
        
        Ok(())
    }
    
    async fn handle_leave_session(&mut self) -> Result<()> {
        tracing::info!("Leaving session");
        
        // Clean up connections
        self.connection_pool.cleanup().await;
        
        // Notify others
        let state_machine = self.state_machine.read().await;
        if let Some(session_id) = state_machine.get_session_id() {
            if let Some(ws_tx) = &self.ws_tx {
                let update = SessionUpdate {
                    session_id: session_id.clone(),
                    accepted_devices: vec![],
                    update_type: SessionUpdateType::ParticipantLeft,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };
                
                let msg = WebSocketMessage::SessionUpdate(update);
                ws_tx.send(msg).context("Failed to send leave message")?;
            }
        }
        
        Ok(())
    }
    
    async fn handle_retry_join(&mut self, session_id: String, attempt: u32) -> Result<()> {
        tracing::info!("Retrying join for session: {} (attempt {})", session_id, attempt);
        
        // Exponential backoff
        let delay = Duration::from_secs(2u64.pow(attempt - 1));
        tokio::time::sleep(delay).await;
        
        // Send join request again
        self.handle_join_session(session_id).await?;
        
        Ok(())
    }
    
    async fn initiate_webrtc_connections(&mut self, participants: Vec<String>) -> Result<()> {
        tracing::info!("Initiating WebRTC connections with {} participants", participants.len());
        
        // Filter out self
        let peers: Vec<String> = participants
            .into_iter()
            .filter(|p| p != &self.device_id)
            .collect();
        
        if peers.is_empty() {
            tracing::debug!("No peers to connect to");
            // Mark mesh as ready if we're alone
            if let Some(tx) = &self.event_tx {
                let _ = tx.send(SessionEvent::MeshReady);
            }
            return Ok(());
        }
        
        // Determine who creates offers based on lexicographic ordering
        for peer in &peers {
            if self.device_id < *peer {
                // We create the offer
                self.create_webrtc_offer(peer.clone()).await?;
            } else {
                // We wait for their offer
                tracing::debug!("Waiting for offer from {}", peer);
            }
        }
        
        Ok(())
    }
    
    async fn create_webrtc_offer(&mut self, peer: String) -> Result<()> {
        tracing::debug!("Creating WebRTC offer for {}", peer);
        
        // Use connection pool to get or create connection
        let connection = self.connection_pool.get_or_create(&peer).await?;
        
        // Create offer through the connection
        let offer = connection.create_offer(None).await?;
        
        // Send offer via WebSocket
        if let Some(ws_tx) = &self.ws_tx {
            let sdp_info = SDPInfo {
                sdp: offer.sdp,
            };
            let signal = WebRTCSignal::Offer(sdp_info);
            
            let msg = WebSocketMessage::WebRTCSignal(signal);
            ws_tx.send(msg).context("Failed to send WebRTC offer")?;
        }
        
        Ok(())
    }
    
    async fn establish_mesh(&mut self) -> Result<()> {
        let state_machine = self.state_machine.read().await;
        
        if let Some(participants) = state_machine.get_participants() {
            tracing::info!("Establishing mesh with {} participants", participants.len());
            
            // Filter out self
            let peers: Vec<String> = participants
                .into_iter()
                .filter(|p| p != &self.device_id)
                .collect();
            
            drop(state_machine);
            
            // Use connection pool for parallel establishment
            self.connection_pool.establish_mesh(peers).await?;
            
            // Mark mesh as ready
            if let Some(tx) = &self.event_tx {
                let _ = tx.send(SessionEvent::MeshReady);
            }
        }
        
        Ok(())
    }
    
    async fn send_with_retry(
        &mut self,
        message: WebSocketMessage,
        max_retries: u32,
    ) -> Result<()> {
        let mut attempt = 0;
        
        loop {
            if let Some(ws_tx) = &self.ws_tx {
                match ws_tx.send(message.clone()) {
                    Ok(_) => return Ok(()),
                    Err(e) if attempt < max_retries => {
                        attempt += 1;
                        tracing::warn!("Send failed (attempt {}/{}): {}", attempt, max_retries, e);
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt - 1))).await;
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to send after {} retries: {}", max_retries, e));
                    }
                }
            } else {
                return Err(anyhow::anyhow!("WebSocket channel not set"));
            }
        }
    }
}