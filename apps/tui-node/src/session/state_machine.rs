use std::time::{Duration, Instant};
// use serde::{Deserialize, Serialize}; // Unused for now
use crate::protocal::signal::{SessionInfo, SessionProposal, SessionResponse, SessionUpdate, SessionType};

/// The state of a session in the state machine
#[derive(Debug, Clone)]
pub enum SessionState {
    /// No active session
    Idle,
    
    /// Discovering available sessions
    Discovering {
        started_at: Instant,
    },
    
    /// Join request sent, waiting for proposal
    JoinRequested {
        session_id: String,
        attempt: u32,
        timeout_at: Instant,
    },
    
    /// Proposal received, waiting for user decision
    ProposalReceived {
        proposal: SessionProposal,
        from: String,
        expires_at: Instant,
    },
    
    /// Session is active
    Active {
        session: SessionInfo,
        participants: Vec<String>,
        mesh_ready: bool,
    },
    
    /// Session failed
    Failed {
        reason: String,
        can_retry: bool,
        failed_at: Instant,
    },
}

/// Events that can trigger state transitions
#[derive(Debug, Clone)]
pub enum SessionEvent {
    // User actions
    CreateSession {
        session_id: String,
        threshold: u16,
        total: u16,
    },
    DiscoverSessions,
    JoinSession {
        session_id: String,
    },
    LeaveSession,
    AcceptProposal,
    RejectProposal,
    
    // Network events
    ProposalReceived {
        from: String,
        proposal: SessionProposal,
    },
    ResponseReceived {
        from: String,
        response: SessionResponse,
    },
    SessionUpdate {
        from: String,
        update: SessionUpdate,
    },
    ConnectionEstablished {
        peer: String,
    },
    ConnectionLost {
        peer: String,
    },
    MeshReady,
    
    // Timeout events
    JoinTimeout {
        session_id: String,
    },
    ProposalExpired,
    DiscoveryTimeout,
    
    // Recovery events
    RetryJoin {
        session_id: String,
        attempt: u32,
    },
    JoinFailed {
        session_id: String,
        reason: String,
    },
    Reset,
    
    // Error events
    NetworkError {
        error: String,
    },
    ValidationError {
        reason: String,
    },
}

/// Errors that can occur during state transitions
#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    #[error("Invalid transition from {from:?} with event {event:?}")]
    InvalidTransition {
        from: String,
        event: String,
    },
    
    #[error("Session already active")]
    SessionAlreadyActive,
    
    #[error("No active session")]
    NoActiveSession,
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// The session state machine
pub struct SessionStateMachine {
    state: SessionState,
    device_id: String,
}

impl SessionStateMachine {
    pub fn new() -> Self {
        Self {
            state: SessionState::Idle,
            device_id: String::new(),
        }
    }
    
    pub fn with_device_id(device_id: String) -> Self {
        Self {
            state: SessionState::Idle,
            device_id,
        }
    }
    
    pub fn get_state(&self) -> SessionState {
        self.state.clone()
    }
    
    /// Process an event and transition to new state
    pub fn transition(&mut self, event: SessionEvent) -> Result<(), TransitionError> {
        let new_state = match (&self.state, &event) {
            // From Idle
            (SessionState::Idle, SessionEvent::CreateSession { session_id, threshold, total }) => {
                SessionState::Active {
                    session: SessionInfo {
                        session_id: session_id.clone(),
                        proposer_id: self.device_id.clone(),
                        total: *total,
                        threshold: *threshold,
                        participants: vec![self.device_id.clone()],
                        session_type: SessionType::DKG,
                        curve_type: "secp256k1".to_string(),
                        coordination_type: "network".to_string(),
                    },
                    participants: vec![self.device_id.clone()],
                    mesh_ready: false,
                }
            }
            
            (SessionState::Idle, SessionEvent::DiscoverSessions) => {
                SessionState::Discovering {
                    started_at: Instant::now(),
                }
            }
            
            (SessionState::Idle, SessionEvent::JoinSession { session_id }) => {
                SessionState::JoinRequested {
                    session_id: session_id.clone(),
                    attempt: 1,
                    timeout_at: Instant::now() + Duration::from_secs(10),
                }
            }
            
            // From Discovering
            (SessionState::Discovering { .. }, SessionEvent::JoinSession { session_id }) => {
                SessionState::JoinRequested {
                    session_id: session_id.clone(),
                    attempt: 1,
                    timeout_at: Instant::now() + Duration::from_secs(10),
                }
            }
            
            (SessionState::Discovering { .. }, SessionEvent::DiscoveryTimeout) => {
                SessionState::Idle
            }
            
            // From JoinRequested
            (SessionState::JoinRequested { session_id, .. }, SessionEvent::ProposalReceived { from, proposal }) 
                if &proposal.session_id == session_id => {
                SessionState::ProposalReceived {
                    proposal: proposal.clone(),
                    from: from.clone(),
                    expires_at: Instant::now() + Duration::from_secs(30),
                }
            }
            
            (SessionState::JoinRequested { session_id, .. }, SessionEvent::JoinTimeout { .. }) => {
                SessionState::Failed {
                    reason: format!("Join request for {} timed out", session_id),
                    can_retry: true,
                    failed_at: Instant::now(),
                }
            }
            
            (SessionState::JoinRequested { .. }, SessionEvent::RetryJoin { session_id, attempt }) => {
                SessionState::JoinRequested {
                    session_id: session_id.clone(),
                    attempt: *attempt,
                    timeout_at: Instant::now() + Duration::from_secs(10 * (*attempt as u64)),
                }
            }
            
            (SessionState::JoinRequested { .. }, SessionEvent::JoinFailed { reason, .. }) => {
                SessionState::Failed {
                    reason: reason.clone(),
                    can_retry: false,
                    failed_at: Instant::now(),
                }
            }
            
            // From ProposalReceived
            (SessionState::ProposalReceived { proposal, .. }, SessionEvent::AcceptProposal) => {
                SessionState::Active {
                    session: SessionInfo {
                        session_id: proposal.session_id.clone(),
                        proposer_id: proposal.proposer_device_id.clone(),
                        total: proposal.total,
                        threshold: proposal.threshold,
                        participants: proposal.participants.clone(),
                        session_type: proposal.session_type.clone(),
                        curve_type: proposal.curve_type.clone(),
                        coordination_type: proposal.coordination_type.clone(),
                    },
                    participants: proposal.participants.clone(),
                    mesh_ready: false,
                }
            }
            
            (SessionState::ProposalReceived { .. }, SessionEvent::RejectProposal) => {
                SessionState::Idle
            }
            
            (SessionState::ProposalReceived { .. }, SessionEvent::ProposalExpired) => {
                SessionState::Failed {
                    reason: "Proposal expired before response".to_string(),
                    can_retry: true,
                    failed_at: Instant::now(),
                }
            }
            
            // From Active
            (SessionState::Active { session, participants, .. }, 
             SessionEvent::ResponseReceived { from, response }) if response.accepted => {
                let mut new_accepted = participants.clone();
                if !new_accepted.contains(from) {
                    new_accepted.push(from.clone());
                }
                SessionState::Active {
                    session: session.clone(),
                    participants: new_accepted,
                    mesh_ready: false,
                }
            }
            
            (SessionState::Active { session, participants, .. }, 
             SessionEvent::SessionUpdate { update, .. }) => {
                // Merge accepted devices from update
                let mut new_accepted = participants.clone();
                for device in &update.participants {
                    if !new_accepted.contains(device) {
                        new_accepted.push(device.clone());
                    }
                }
                SessionState::Active {
                    session: session.clone(),
                    participants: new_accepted,
                    mesh_ready: false,
                }
            }
            
            (SessionState::Active { session, participants, .. }, 
             SessionEvent::MeshReady) => {
                SessionState::Active {
                    session: session.clone(),
                    participants: participants.clone(),
                    mesh_ready: true,
                }
            }
            
            (SessionState::Active { .. }, SessionEvent::LeaveSession) => {
                SessionState::Idle
            }
            
            (SessionState::Active { .. }, SessionEvent::NetworkError { error }) => {
                SessionState::Failed {
                    reason: error.clone(),
                    can_retry: true,
                    failed_at: Instant::now(),
                }
            }
            
            // From Failed
            (SessionState::Failed { can_retry: true, .. }, SessionEvent::Reset) => {
                SessionState::Idle
            }
            
            // Default: invalid transition
            _ => {
                return Err(TransitionError::InvalidTransition {
                    from: format!("{:?}", self.state),
                    event: format!("{:?}", event),
                });
            }
        };
        
        tracing::debug!("State transition: {:?} -> {:?}", 
            self.state_name(), 
            Self::state_name_for(&new_state)
        );
        
        self.state = new_state;
        Ok(())
    }
    
    /// Check if the state machine is in an active session
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active { .. })
    }
    
    /// Check if mesh is ready
    pub fn is_mesh_ready(&self) -> bool {
        matches!(self.state, SessionState::Active { mesh_ready: true, .. })
    }
    
    /// Get the current session ID if active
    pub fn get_session_id(&self) -> Option<String> {
        match &self.state {
            SessionState::Active { session, .. } => Some(session.session_id.clone()),
            SessionState::JoinRequested { session_id, .. } => Some(session_id.clone()),
            SessionState::ProposalReceived { proposal, .. } => Some(proposal.session_id.clone()),
            _ => None,
        }
    }
    
    /// Get participants if in active session
    pub fn get_participants(&self) -> Option<Vec<String>> {
        match &self.state {
            SessionState::Active { participants, .. } => Some(participants.clone()),
            SessionState::ProposalReceived { proposal, .. } => Some(proposal.participants.clone()),
            _ => None,
        }
    }
    
    fn state_name(&self) -> &str {
        Self::state_name_for(&self.state)
    }
    
    fn state_name_for(state: &SessionState) -> &str {
        match state {
            SessionState::Idle => "Idle",
            SessionState::Discovering { .. } => "Discovering",
            SessionState::JoinRequested { .. } => "JoinRequested",
            SessionState::ProposalReceived { .. } => "ProposalReceived",
            SessionState::Active { .. } => "Active",
            SessionState::Failed { .. } => "Failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_idle_to_join_requested() {
        let mut sm = SessionStateMachine::new();
        
        let result = sm.transition(SessionEvent::JoinSession {
            session_id: "test-session".to_string(),
        });
        
        assert!(result.is_ok());
        assert!(matches!(sm.get_state(), SessionState::JoinRequested { .. }));
    }
    
    #[test]
    fn test_join_to_proposal_received() {
        let mut sm = SessionStateMachine::new();
        
        sm.transition(SessionEvent::JoinSession {
            session_id: "test-session".to_string(),
        }).unwrap();
        
        let proposal = SessionProposal {
            session_id: "test-session".to_string(),
            proposer_device_id: "proposer".to_string(),
            participants: vec!["proposer".to_string(), "joiner".to_string()],
            threshold: 2,
            total: 3,
            session_type: crate::protocal::signal::SessionType::DKG,
            curve_type: "secp256k1".to_string(),
            coordination_type: "network".to_string(),
        };
        
        let result = sm.transition(SessionEvent::ProposalReceived {
            from: "proposer".to_string(),
            proposal,
        });
        
        assert!(result.is_ok());
        assert!(matches!(sm.get_state(), SessionState::ProposalReceived { .. }));
    }
    
    #[test]
    fn test_proposal_to_active() {
        let mut sm = SessionStateMachine::with_device_id("test-device".to_string());
        
        sm.state = SessionState::ProposalReceived {
            proposal: SessionProposal {
                session_id: "test-session".to_string(),
                proposer_device_id: "proposer".to_string(),
                participants: vec!["proposer".to_string(), "test-device".to_string()],
                threshold: 2,
                total: 3,
                session_type: crate::protocal::signal::SessionType::DKG,
                curve_type: "secp256k1".to_string(),
                coordination_type: "network".to_string(),
            },
            from: "proposer".to_string(),
            expires_at: Instant::now() + Duration::from_secs(30),
        };
        
        let result = sm.transition(SessionEvent::AcceptProposal);
        
        assert!(result.is_ok());
        assert!(sm.is_active());
        assert_eq!(sm.get_session_id(), Some("test-session".to_string()));
    }
    
    #[test]
    fn test_active_to_mesh_ready() {
        let mut sm = SessionStateMachine::with_device_id("test-device".to_string());
        
        sm.state = SessionState::Active {
            session: SessionInfo {
                session_id: "test-session".to_string(),
                proposer_id: "creator".to_string(),
                total: 3,
                threshold: 2,
                participants: vec!["creator".to_string(), "test-device".to_string()],
                participants: vec!["creator".to_string(), "test-device".to_string()],
                session_type: crate::protocal::signal::SessionType::DKG,
                curve_type: "secp256k1".to_string(),
                coordination_type: "network".to_string(),
            },
            participants: vec!["creator".to_string(), "test-device".to_string()],
            participants: vec!["creator".to_string(), "test-device".to_string()],
            mesh_ready: false,
        };
        
        let result = sm.transition(SessionEvent::MeshReady);
        
        assert!(result.is_ok());
        assert!(sm.is_mesh_ready());
    }
    
    #[test]
    fn test_invalid_transition() {
        let mut sm = SessionStateMachine::new();
        
        // Can't accept proposal when idle
        let result = sm.transition(SessionEvent::AcceptProposal);
        
        assert!(result.is_err());
        assert!(matches!(sm.get_state(), SessionState::Idle));
    }
}