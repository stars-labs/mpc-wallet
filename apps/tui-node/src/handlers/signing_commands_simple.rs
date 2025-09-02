//! Simplified Signing Commands Implementation following KISS principle

use crate::utils::appstate_compat::AppState;
use crate::utils::state::{SigningState, InternalCommand};
use crate::protocal::signal::WebRTCMessage;
use frost_core::{Ciphersuite, Identifier};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use serde::{Serialize, Deserialize};

/// Simplified signing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleSigningRequest {
    pub signing_id: String,
    pub transaction_data: String,
    pub blockchain: String,
    pub chain_id: Option<u64>,
}

/// Initialize signing process - simplified
pub async fn handle_initiate_signing<C>(
    transaction_data: String,
    blockchain: String,
    chain_id: Option<u64>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) 
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    // Check if we have a session
    let session = match &guard.session {
        Some(s) => s.clone(),
        None => {
            guard.signing_state = SigningState::Failed {
                signing_id: "unknown".to_string(),
                reason: "No session available".to_string(),
            };
            return;
        }
    };
    
    // Generate signing ID
    let signing_id = format!("sign-{}-{}", guard.device_id, chrono::Utc::now().timestamp());
    
    // Set to awaiting acceptance
    guard.signing_state = SigningState::AwaitingAcceptance {
        signing_id: signing_id.clone(),
        transaction_data: transaction_data.clone(),
        initiator: guard.device_id.clone(),
        required_signers: session.threshold as usize,
        accepted_signers: std::collections::HashSet::new(),
        blockchain: blockchain.clone(),
        chain_id,
    };
    
    // Create signing request
    let request = SimpleSigningRequest {
        signing_id: signing_id.clone(),
        transaction_data,
        blockchain,
        chain_id,
    };
    
    let participants = session.participants.clone();
    let self_device_id = guard.device_id.clone();
    drop(guard);
    
    // Send to all participants
    let message = WebRTCMessage::SigningRequest {
        signing_id,
        transaction_data: request.transaction_data,
        required_signers: session.threshold as usize,
        blockchain: request.blockchain,
        chain_id,
    };
    
    for device_id in participants {
        if device_id != self_device_id {
            if let Err(_e) = crate::utils::device::send_webrtc_message(&device_id, &message, state.clone()).await {
                tracing::warn!("Failed to send signing request to {}: {:?}", device_id, _e);
            }
        }
    }
}

/// Process signing request - simplified
pub async fn handle_process_signing_request<C>(
    from_device_id: String,
    signing_id: String,
    transaction_data: String,
    _timestamp: String,
    blockchain: String,
    chain_id: Option<u64>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    // Set signing state
    guard.signing_state = SigningState::AwaitingAcceptance {
        signing_id: signing_id.clone(),
        transaction_data: transaction_data.clone(),
        initiator: from_device_id.clone(),
        required_signers: 2, // Simple 2/3 threshold
        accepted_signers: std::collections::HashSet::new(),
        blockchain,
        chain_id,
    };
    
    tracing::info!("Received signing request {} from {}", signing_id, from_device_id);
}

/// Accept signing request - simplified
pub async fn handle_accept_signing_request<C>(
    signing_id: String,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    // Check if we're in the right state
    match &guard.signing_state {
        SigningState::AwaitingAcceptance { signing_id: current_id, .. } if current_id == &signing_id => {
            // Move to signing phase (simplified)
            guard.signing_state = SigningState::Complete {
                signing_id: signing_id.clone(),
                signature: b"simple_signature".to_vec(),
            };
            
            tracing::info!("Accepted and completed signing request {}", signing_id);
        }
        _ => {
            tracing::warn!("Invalid state for accepting signing request {}", signing_id);
        }
    }
}

/// Handle signing acceptance - simplified
pub async fn handle_process_signing_acceptance<C>(
    from_device_id: String,
    signing_id: String,
    _timestamp: String,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    // Check the current state and extract required info
    let should_complete = match &mut guard.signing_state {
        SigningState::AwaitingAcceptance { 
            signing_id: current_id, 
            accepted_signers, 
            required_signers,
            ..
        } if current_id == &signing_id => {
            accepted_signers.insert(from_device_id.clone());
            accepted_signers.len() >= *required_signers
        }
        _ => {
            tracing::warn!("Invalid state for processing signing acceptance from {}", from_device_id);
            return;
        }
    };
    
    // Complete signing if we have enough signers
    if should_complete {
        guard.signing_state = SigningState::Complete {
            signing_id: signing_id.clone(),
            signature: b"aggregated_signature".to_vec(),
        };
        
        tracing::info!("Signing completed for {}", signing_id);
    }
}

/// Generate signing commitment - simplified stub
pub fn generate_signing_commitment<C>(_key_package: &[u8]) -> Result<SigningCommitmentResult<C>, String> 
where
    C: Ciphersuite,
{
    Err("Signing commitment generation is stubbed".to_string())
}

/// Signing commitment result - simplified
#[derive(Debug, Clone)]
pub struct SigningCommitmentResult<C: Ciphersuite> {
    pub nonces: Vec<u8>, // Simplified as bytes
    pub commitments: Vec<u8>, // Simplified as bytes
    pub _phantom: std::marker::PhantomData<C>,
}

/// Process signing commitment - simplified stub
pub async fn handle_process_signing_commitment<C>(
    _from_device_id: String,
    _signing_id: String,
    _commitment: frost_core::round1::SigningCommitments<C>, // Match InternalCommand
    _state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    tracing::info!("Signing commitment processing is stubbed");
}

/// Process signature share - simplified stub
pub async fn handle_process_signature_share<C>(
    _from_device_id: String,
    _signing_id: String,
    _share: frost_core::round2::SignatureShare<C>, // Match InternalCommand
    _state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    tracing::info!("Signature share processing is stubbed");
}

/// Process aggregated signature - simplified
pub async fn handle_process_aggregated_signature<C>(
    _from_device_id: String,
    _signing_id: String,
    signature: Vec<u8>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    guard.signing_state = SigningState::Complete {
        signing_id: "aggregated".to_string(),
        signature,
    };
    
    tracing::info!("Processed aggregated signature");
}

/// Process signer selection - simplified stub
pub async fn handle_process_signer_selection<C>(
    _from_device_id: String,
    _signing_id: String,
    _selected_signers: Vec<Identifier<C>>,
    _state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    tracing::info!("Signer selection processing is stubbed");
}

/// Initiate FROST Round 1 - simplified stub
pub async fn handle_initiate_frost_round1<C>(
    _signing_id: String,
    _transaction_data: String,
    _selected_signers: Vec<Identifier<C>>,
    _state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    tracing::info!("FROST Round 1 initiation is stubbed");
}

/// Handle accept signing - wrapper for compatibility
pub async fn handle_accept_signing<C>(
    signing_id: String,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
)
where
    C: Ciphersuite + Send + Sync + 'static,
{
    handle_accept_signing_request(signing_id, state, internal_cmd_tx).await;
}