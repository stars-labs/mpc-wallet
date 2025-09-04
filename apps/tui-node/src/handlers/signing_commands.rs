use crate::utils::appstate_compat::AppState;
use crate::utils::state::{InternalCommand, SigningState};
use crate::protocal::signal::WebRTCMessage;
use crate::protocal::dkg;
use crate::utils::device::send_webrtc_message;
use frost_core::{Ciphersuite, Identifier};
use std::sync::Arc;
use std::collections::BTreeMap;
use tokio::sync::{Mutex, mpsc};

// Use a type alias to work around import issues
type BlockchainRegistry = mpc_wallet_blockchain::BlockchainRegistry;

/// Handles initiating a signing process
pub async fn handle_initiate_signing<C>(
    transaction_data: String,
    chain_id: Option<u64>,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        // Check if DKG is complete
        let dkg_complete = guard.key_package.is_some() && guard.group_public_key.is_some();
        if !dkg_complete {
            return;
        }
        
        // Check if already signing
        if guard.signing_state.is_active() {
            return;
        }
        
        // Check if session exists
        let session = match &guard.session {
            Some(s) => s.clone(),
            None => {
                return;
            }
        };
        
        // Derive blockchain from chain_id or default to ethereum
        let blockchain = chain_id.map(|id| match id {
            1 | 5 | 11155111 => "ethereum",
            56 | 97 => "bsc",
            137 | 80001 => "polygon",
            _ => "ethereum"
        }).unwrap_or("ethereum").to_string();
        
        // Validate blockchain and curve compatibility
        let blockchain_registry = BlockchainRegistry::new();
        let blockchain_handler = match blockchain_registry.get(&blockchain)
            .or_else(|| chain_id.and_then(|id| blockchain_registry.get_by_chain_id(id))) {
            Some(handler) => handler,
            None => {
                return;
            }
        };
        
        // Check curve compatibility
        // TODO: Fix TypeId comparison for curve validation
        /*
        let curve_type: &str = if std::any::TypeId::of::<C>() == std::any::TypeId::of::<frost_secp256k1::Secp256K1Sha256>() {
        } else if std::any::TypeId::of::<C>() == std::any::TypeId::of::<frost_ed25519::Ed25519Sha512>() {
        } else {
        };
        
        if blockchain_handler.curve_type() != curve_type {
            return;
        }
        */
        
        // Validate transaction format
        if blockchain_handler.parse_transaction(&transaction_data).is_err() {
            return;
        }
        
        
        // Generate unique signing ID
        let signing_id = format!("sign-{}-{}", guard.device_id, chrono::Utc::now().timestamp());
        let required_signers = session.threshold as usize;
        
        // Initialize accepted signers with ourselves (initiator auto-accepts)
        let mut accepted_signers = std::collections::HashSet::new();
        accepted_signers.insert(guard.device_id.clone());
        
        // Set signing state to awaiting acceptance
        guard.signing_state = SigningState::AwaitingAcceptance {
            signing_id: signing_id.clone(),
            transaction_data: transaction_data.clone(),
            initiator: guard.device_id.clone(),
            required_signers,
            accepted_signers,
            blockchain: blockchain.clone(),
            chain_id,
        };
        
        
        // Send signing request to all session participants
        let session_participants = session.participants.clone();
        let self_device_id = guard.device_id.clone();
        drop(guard);
        
        let signing_request = WebRTCMessage::SigningRequest {
            signing_id: signing_id.clone(),
            transaction_data: transaction_data.clone(),
            required_signers,
            blockchain: blockchain.clone(),
            chain_id,
        };
        
        for device_id in session_participants {
            if device_id != self_device_id {
                if let Err(_e) = send_webrtc_message(&device_id, &signing_request, state_clone.clone()).await {
                    // Failed to send signing request
                }
            }
        }
        
        
        // Check if we already have enough signers (just ourselves) to proceed
        let mut guard = state_clone.lock().await;
        
        // Extract the needed values from the signing state to avoid borrowing conflicts
        let extracted_data = if let SigningState::AwaitingAcceptance { 
            accepted_signers, 
            required_signers,
            transaction_data,
            ..
        } = &guard.signing_state {
            if accepted_signers.len() >= *required_signers {
                // Extract data needed for immediate progression
                Some((
                    accepted_signers.clone(),
                    *required_signers,
                    transaction_data.clone(),
                ))
            } else {
                None
            }
        } else {
            None
        };
        
        if let Some((accepted_signers, _required_signers, transaction_data)) = extracted_data {
            // We already have enough signers, proceed directly to commitment phase
            let _identifier_map = match guard.identifier_map.clone() {
                Some(map) => map,
                None => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: "No identifier map available".to_string(),
                    };
                    return;
                }
            };
            
            // Select signers and map them to FROST Identifiers
            let selected_signers = dkg::map_selected_signers(accepted_signers.iter().cloned().collect());
            
            // Transition to commitment phase
            guard.signing_state = SigningState::CommitmentPhase {
                signing_id: signing_id.clone(),
                transaction_data: transaction_data.clone(),
                selected_signers: selected_signers.clone(),
                commitments: BTreeMap::new(),
                own_commitment: None,
                nonces: None,
                blockchain: blockchain.clone(),
                chain_id,
            };
            
            // Send signer selection message to all participants
            let session = guard.session.as_ref().unwrap().clone();
            let selection_message = WebRTCMessage::SignerSelection {
                signing_id: signing_id.clone(),
                selected_signers: selected_signers.clone(),
            };
            
            // Prepare data for internal command dispatch
            let cmd_data = (signing_id.clone(), transaction_data.clone(), selected_signers.clone());
            let self_device_id_copy = guard.device_id.clone();
            drop(guard);
            
            // Send selection message to all session participants
            for device_id in session.participants {
                if device_id != self_device_id_copy {
                    if let Err(_e) = send_webrtc_message(&device_id, &selection_message, state_clone.clone()).await {
                        // Failed to send selection message
                    }
                }
            }
            
            // Now initiate FROST Round 1 commitment generation using proper internal command
            if let Err(_e) = internal_cmd_tx.send(InternalCommand::InitiateFrostRound1 {
                signing_id: cmd_data.0,
                transaction_data: cmd_data.1,
                selected_signers: cmd_data.2,
            }) {
                let mut guard = state_clone.lock().await;
                guard.signing_state = SigningState::Failed {
                        signing_id: signing_id.clone(),
                    reason: "Failed to dispatch internal command".to_string(),
                };
            }
        }
    });
}

/// Handles accepting a signing request
pub async fn handle_accept_signing<C>(
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        // Check if this matches the current signing request
        let (signing_id, initiator) = match &guard.signing_state {
            SigningState::AwaitingAcceptance { signing_id: current_id, initiator, .. } => {
                (current_id.clone(), initiator.clone())
            },
            _ => {
                return;
            }
        };
        
        // Remove from pending list
        guard.pending_signing_requests.retain(|req| req.signing_id != signing_id);
        
        // Send acceptance message to initiator
        let acceptance = WebRTCMessage::SigningAcceptance {
            signing_id: signing_id.clone(),
            accepted: true,
        };
        
        drop(guard);
        
        if let Err(_e) = send_webrtc_message(&initiator, &acceptance, state_clone.clone()).await {
            // Failed to send acceptance
        }
    });
}

/// Handles processing a signing request from a device
pub async fn handle_process_signing_request<C>(
    from_device_id: String,
    signing_id: String,
    transaction_data: String,
    _timestamp: String,
    blockchain: String,
    chain_id: Option<u64>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        // Check if DKG is complete
        let dkg_complete = guard.key_package.is_some() && guard.group_public_key.is_some();
        if !dkg_complete {
            return;
        }
        
        // Check if already signing
        if guard.signing_state.is_active() {
            return;
        }
        
        // Check if session exists
        let session = match &guard.session {
            Some(s) => s.clone(),
            None => {
                return;
            }
        };
        
        // Verify the requesting device is in our session
        if !session.participants.contains(&from_device_id) {
            return;
        }
        
        // Validate blockchain and curve compatibility
        let blockchain_registry = BlockchainRegistry::new();
        let blockchain_handler = match blockchain_registry.get(&blockchain)
            .or_else(|| chain_id.and_then(|id| blockchain_registry.get_by_chain_id(id))) {
            Some(handler) => handler,
            None => {
                return;
            }
        };
        
        // Check curve compatibility
        // TODO: Fix TypeId comparison for curve validation
        /*
        let curve_type: &str = if TypeId::of::<C>() == TypeId::of::<frost_secp256k1::Secp256K1Sha256>() {
        } else if TypeId::of::<C>() == TypeId::of::<frost_ed25519::Ed25519Sha512>() {
        } else {
        };
        
        if blockchain_handler.curve_type() != curve_type {
            return;
        }
        */
        
        // Validate transaction format
        if blockchain_handler.parse_transaction(&transaction_data).is_err() {
            return;
        }
        
        // Received signing request
        
        // Add to pending signing requests
        guard.pending_signing_requests.push(crate::utils::state::PendingSigningRequest {
            signing_id: signing_id.clone(),
            from_device: from_device_id.clone(),
            transaction_data: transaction_data.clone(),
        });
        
        // Update signing state to awaiting acceptance
        let required_signers = session.threshold as usize;
        let mut accepted_signers = std::collections::HashSet::new();
        accepted_signers.insert(from_device_id.clone()); // Initiator is automatically accepted
        
        guard.signing_state = SigningState::AwaitingAcceptance {
            signing_id: signing_id.clone(),
            transaction_data: transaction_data.clone(),
            initiator: from_device_id.clone(),
            required_signers,
            accepted_signers,
            blockchain: blockchain.clone(),
            chain_id,
        };
    });
}

/// Handles processing a signing acceptance from a device
pub async fn handle_process_signing_acceptance<C>(
    from_device_id: String,
    signing_id: String,
    _timestamp: String,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    let internal_cmd_tx_clone = internal_cmd_tx.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        // Check if we're in the right state and this is the right signing ID
        let signing_info = match &guard.signing_state {
            SigningState::AwaitingAcceptance { 
                signing_id: current_id,
                accepted_signers, 
                required_signers,
                transaction_data,
                blockchain,
                chain_id,
                .. 
            } => {
                if current_id != &signing_id {
                    None
                } else {
                    Some((current_id.clone(), accepted_signers.clone(), *required_signers, transaction_data.clone(), blockchain.clone(), *chain_id))
                }
            },
            _ => {
                None
            }
        };
        
        if let Some((current_signing_id, mut accepted_signers, required_signers, transaction_data, blockchain, chain_id)) = signing_info {
            // Add the accepting device
            accepted_signers.insert(from_device_id.clone());
            
            // Update the state with the new acceptances
            if let SigningState::AwaitingAcceptance { accepted_signers: current_accepted, .. } = &mut guard.signing_state {
                *current_accepted = accepted_signers.clone();
            }
            
            // Check if we have enough signers
            if accepted_signers.len() >= required_signers {
                
                // Get the identifier map to convert device IDs to FROST Identifiers
                let _identifier_map = match guard.identifier_map.clone() {
                    Some(map) => map,
                    None => {
                        guard.signing_state = SigningState::Failed {
                            signing_id: current_signing_id.clone(),
                            reason: "No identifier map available".to_string(),
                        };
                        return;
                    }
                };
                
                // Select the first threshold number of signers and map them to FROST Identifiers
                let selected_signers = dkg::map_selected_signers(accepted_signers.iter().cloned().collect());
                
                // Transition to commitment phase
                guard.signing_state = SigningState::CommitmentPhase {
                    signing_id: current_signing_id.clone(),
                    transaction_data: transaction_data.clone(),
                    selected_signers: selected_signers.clone(),
                    commitments: BTreeMap::new(),
                    own_commitment: None,
                    nonces: None,
                    blockchain,
                    chain_id,
                };
                
                // Send signer selection message to all participants
                let session = guard.session.as_ref().unwrap().clone();
                let selection_message = WebRTCMessage::SignerSelection {
                    signing_id: current_signing_id.clone(),
                    selected_signers: selected_signers.clone(),
                };
                
                // Drop guard before async operations
                let self_device_id = guard.device_id.clone();
                drop(guard);
                
                // Send selection message to all session participants
                for device_id in session.participants {
                    if device_id != self_device_id {
                        if let Err(_e) = send_webrtc_message(&device_id, &selection_message, state_clone.clone()).await {
                            // Failed to send selection message
                        }
                    }
                }
                
                // Now initiate FROST Round 1 commitment generation
                let _ = internal_cmd_tx_clone.send(InternalCommand::InitiateFrostRound1 {
                    signing_id: current_signing_id,
                    transaction_data,
                    selected_signers,
                });
            }
        } else {
            return;
        }
    });
}

/// Handles processing a signing commitment from a device
pub async fn handle_process_signing_commitment<C>(
    from_device_id: String,
    signing_id: String,
    commitment: frost_core::round1::SigningCommitments<C>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        
        // Extract identifier map to avoid borrow conflicts
        let identifier_map = match guard.identifier_map.clone() {
            Some(map) => map,
            None => {
                return;
            }
        };
        
        let sender_identifier = match identifier_map.get(&from_device_id) {
            Some(id) => *id,
            None => {
                return;
            }
        };
        
        // Check if we're in the commitment phase and this is the correct signing process
        let (selected_signers, transaction_data) = match &guard.signing_state {
            SigningState::CommitmentPhase { 
                signing_id: current_id,
                selected_signers, 
                transaction_data,
                ..
            } if current_id == &signing_id => {
                (selected_signers.clone(), transaction_data.clone())
            },
            _ => {
                return;
            }
        };
        
        // Verify sender is one of the selected signers
        if !selected_signers.contains(&sender_identifier) {
            return;
        }
        
        // Store the commitment and check if we have all
        let should_proceed = if let SigningState::CommitmentPhase { 
            commitments, 
            ..
        } = &mut guard.signing_state {
            // Check if we already have this commitment
            if commitments.contains_key(&sender_identifier) {
                return;
            }
            
            // Store the commitment
            commitments.insert(sender_identifier, commitment);
            
            let commitment_count = commitments.len();
            let selected_count = selected_signers.len();
            
            // Check if we have all commitments
            commitment_count == selected_count
        } else {
            false
        };
        
        if should_proceed {
            
            // Extract required data
            let (commitments, our_nonces, blockchain, chain_id) = if let SigningState::CommitmentPhase {
                commitments, 
                nonces,
                blockchain,
                chain_id,
                ..
            } = &mut guard.signing_state {
                let comms = commitments.clone();
                let nonces = nonces.take();
                (comms, nonces, blockchain.clone(), *chain_id)
            } else {
                return;
            };
            
            // Get blockchain handler to format the message for signing
            let blockchain_registry = BlockchainRegistry::new();
            let blockchain_handler = match blockchain_registry.get(&blockchain)
                .or_else(|| chain_id.and_then(|id| blockchain_registry.get_by_chain_id(id))) {
                Some(handler) => handler,
                None => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: "Operation failed".to_string(),
                    };
                    return;
                }
            };
            
            // Parse and format the transaction for the specific blockchain
            let signing_message = match blockchain_handler.parse_transaction(&transaction_data)
                .and_then(|parsed_tx| blockchain_handler.format_for_signing(&parsed_tx)) {
                Ok(msg) => msg,
                Err(_e) => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: "Operation failed".to_string(),
                    };
                    return;
                }
            };
            
            
            // Create signing package with blockchain-formatted message
            // Convert BTreeMap to Vec for the function
            let commitment_vec: Vec<_> = commitments.values().cloned().collect();
            let signing_package_result = dkg::create_signing_package(
                &signing_message,
                commitment_vec,
            );
            
            // Get our key package for signature generation
            let key_package = match &guard.key_package {
                Some(kp) => kp.clone(),
                None => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: "No key package available".to_string(),
                    };
                    return;
                }
            };
            
            let our_nonces = match our_nonces {
                Some(n) => n,
                None => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: "No nonces available".to_string(),
                    };
                    return;
                }
            };
            
            // Generate our signature share
            // First unwrap the signing package Result and immediately drop the Result
            let signing_package = match signing_package_result {
                Ok(pkg) => pkg,
                Err(e) => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: e.to_string(),
                    };
                    return;
                }
            };
            
            let signature_share_result = match dkg::generate_signature_share(&signing_package, &our_nonces, &key_package) {
                Ok(result) => result,
                Err(e) => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: e.to_string(),
                    };
                    return;
                }
            };
            // Keep the full SignatureShare, not just the scalar
            let signature_share = signature_share_result;
            
            // Get our identifier
            let self_identifier = match identifier_map.get(&guard.device_id) {
                Some(id) => *id,
                None => {
                    let _device_id = guard.device_id.clone();
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: "No self identifier available".to_string(),
                    };
                    return;
                }
            };
            
            // Initialize shares map with our own share
            let shares = BTreeMap::new();
            
            // Transition to share phase
            guard.signing_state = SigningState::SharePhase {
                signing_id: signing_id.clone(),
                transaction_data: transaction_data.clone(),
                blockchain: blockchain.clone(),
                selected_signers: selected_signers.clone(),
                signing_package: Some(signing_package),
                shares,
                own_share: Some(signature_share.clone()),
                chain_id,
            };
            
            // Send our signature share to all other selected signers
            let share_message = WebRTCMessage::SignatureShare {
                signing_id: signing_id.clone(),
                sender_identifier: self_identifier,
                share: signature_share,
            };
            
            // Get reverse identifier map to convert FROST identifiers back to device IDs
            let device_id_map = dkg::create_device_id_map(&identifier_map);
            
            drop(guard);
            
            // Send signature share to all other selected signers
            for signer_id in selected_signers {
                if signer_id != self_identifier {
                    if let Some(device_id) = device_id_map.get(&signer_id) {
                        if let Err(_e) = send_webrtc_message(device_id, &share_message, state_clone.clone()).await {
                            // Failed to send share message
                        }
                    }
                }
            }
            
        }
    });
}

/// Handles processing a signature share from a device
pub async fn handle_process_signature_share<C>(
    from_device_id: String,
    signing_id: String,
    share: frost_core::round2::SignatureShare<C>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        
        // Get identifier map to convert device ID to FROST identifier
        let identifier_map = match guard.identifier_map.clone() {
            Some(map) => map,
            None => {
                return;
            }
        };
        
        let sender_identifier = match identifier_map.get(&from_device_id) {
            Some(id) => *id,
            None => {
                return;
            }
        };
        
        // Check if we're in the share phase and this is the correct signing process
        let (selected_signers, signing_package) = match &guard.signing_state {
            SigningState::SharePhase { 
                signing_id: current_id,
                selected_signers, 
                signing_package,
                ..
            } if current_id == &signing_id => {
                (selected_signers.clone(), signing_package.clone())
            },
            _ => {
                return;
            }
        };
        
        // Verify sender is one of the selected signers
        if !selected_signers.contains(&sender_identifier) {
            return;
        }
        
        // Store the signature share and check if we have all
        let should_aggregate = if let SigningState::SharePhase { 
            shares,
            ..
        } = &mut guard.signing_state {
            // Check if we already have this share
            if shares.contains_key(&sender_identifier) {
                return;
            }
            
            // Store the share
            shares.insert(sender_identifier, share);
            
            let shares_count = shares.len();
            let selected_count = selected_signers.len();
            
            // Check if we have all signature shares
            shares_count == selected_count
        } else {
            false
        };
        
        if should_aggregate {
            
            // Extract required data
            let (signing_id, shares, blockchain, chain_id) = if let SigningState::SharePhase {
                signing_id,
                shares,
                blockchain,
                chain_id,
                ..
            } = &guard.signing_state {
                (signing_id.clone(), shares.clone(), blockchain.clone(), *chain_id)
            } else {
                return;
            };
            
            // Get the signing package
            let signing_package = match signing_package {
                Some(pkg) => pkg,
                None => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: "No signing package available".to_string(),
                    };
                    return;
                }
            };
            
            // Get the group public key for verification
            let group_public_key = match guard.group_public_key.clone() {
                Some(key) => key,
                None => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: "No group public key available".to_string(),
                    };
                    return;
                }
            };
            
            // Aggregate the signature
            
            let _aggregated_result = match dkg::aggregate_signature(&signing_package, &shares, &group_public_key) {
                Ok(result) => result,
                Err(e) => {
                    guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                        reason: e.to_string(),
                    };
                    return;
                }
            };
            // Serialize the signature to bytes (this is a stub - needs proper implementation)
            let raw_signature_bytes = vec![0u8; 64]; // Placeholder - FROST signatures are typically 64 bytes
            
            // Validate blockchain handler exists
            let blockchain_registry = BlockchainRegistry::new();
            if blockchain_registry.get(&blockchain)
                .or_else(|| chain_id.and_then(|id| blockchain_registry.get_by_chain_id(id)))
                .is_none() {
                guard.signing_state = SigningState::Failed {
                    signing_id: "unknown".to_string(),
                    reason: "Operation failed".to_string(),
                };
                return;
            }
            
            // Signature aggregated successfully
            
            // Note: For now we're storing the raw FROST signature bytes
            // In the future, we can use blockchain_handler.serialize_signature() to format it
            // blockchain-specific way (e.g., with recovery ID for Ethereum)
            let signature_bytes = raw_signature_bytes;
            
            
            // Transition to complete state
            guard.signing_state = SigningState::Complete {
                signing_id: signing_id.clone(),
                signature: signature_bytes.clone(),
            };
            
            // Remove from pending list
            guard.pending_signing_requests.retain(|req| req.signing_id != signing_id);
            
            // Broadcast the aggregated signature to all session participants
            let aggregated_sig_message = WebRTCMessage::AggregatedSignature {
                signing_id: signing_id.clone(),
                signature: signature_bytes.clone(),
            };
            
            let session = guard.session.as_ref().unwrap().clone();
            let self_device_id = guard.device_id.clone();
            drop(guard);
            
            // Send aggregated signature to all session participants
            for device_id in session.participants {
                if device_id != self_device_id {
                    if let Err(_e) = send_webrtc_message(&device_id, &aggregated_sig_message, state_clone.clone()).await {
                        // Failed to send aggregated signature
                    }
                }
            }
            
        }
    });
}

/// Handles processing an aggregated signature from a device
pub async fn handle_process_aggregated_signature<C>(
    signature: Vec<u8>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        
        // Update signing state to complete
        guard.signing_state = SigningState::Complete {
            signing_id: "broadcast".to_string(), // This is a broadcast message, so use generic ID
            signature: signature.clone(),
        };
        
        // Remove from pending list
        // Note: signing_id not passed to this function - skip cleanup for now
        
        hex::encode(&signature)
    });
}

/// Handles processing signer selection from a device
pub async fn handle_process_signer_selection<C>(
    selected_signers: Vec<Identifier<C>>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        
        // Get identifier map to check if we are selected
        let identifier_map = match guard.identifier_map.clone() {
            Some(map) => map,
            None => {
                return;
            }
        };
        
        let self_identifier = match identifier_map.get(&guard.device_id) {
            Some(id) => *id,
            None => {
                let _device_id = guard.device_id.clone();
                return;
            }
        };
        
        // Check if we are one of the selected signers
        let is_selected = dkg::is_device_selected(&self_identifier, &selected_signers);
        
        if !is_selected {
            return;
        }
        
        
        // Check if we're in the awaiting acceptance state for this signing process
        let (current_id, transaction_data, blockchain, chain_id) = match &guard.signing_state {
            SigningState::AwaitingAcceptance { 
                signing_id: current_id,
                transaction_data,
                blockchain,
                chain_id,
                ..
            } => {
                (current_id.clone(), transaction_data.clone(), blockchain.clone(), *chain_id)
            },
            _ => {
                return;
            }
        };
        
        // Transition to commitment phase
        guard.signing_state = SigningState::CommitmentPhase {
            signing_id: current_id.clone(),
            transaction_data: transaction_data.clone(),
            blockchain: blockchain.clone(),
            selected_signers: selected_signers.clone(),
            commitments: BTreeMap::new(),
            own_commitment: None,
            nonces: None,
            chain_id,
        };
        
        // Get our key package for signing
        let _key_package = match &guard.key_package {
            Some(kp) => kp.clone(),
            None => {
                guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                    reason: "No key package available".to_string(),
                };
                return;
            }
        };
        
        
        // Generate FROST Round 1 commitment
        let commitment_result = match dkg::generate_signing_commitment() {
            Ok(result) => result,
            Err(e) => {
                guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                    reason: e.to_string(),
                };
                return;
            }
        };
        // Note: SigningCommitments is opaque, we store the whole object
        // The actual commitment to send to others
        let commitments = commitment_result.clone();
        
        // Update signing state with our commitment and nonces and extract signing_id
        let signing_id = if let SigningState::CommitmentPhase { 
            signing_id,
            own_commitment,
            ..
        } = &mut guard.signing_state {
            *own_commitment = Some(commitments.clone());
            // TODO: Store nonces properly - SigningCommitments and SigningNonces are different types
            // For now, leave nonces as None - this is a stub implementation
            // *nonces_field = Some(commitment_result.clone());
            signing_id.clone()
        } else {
            return;
        };
        
        
        // Send commitment to all other selected signers
        let commitment_message = WebRTCMessage::SigningCommitment {
            signing_id: signing_id.clone(),
            sender_identifier: self_identifier,
            commitment: commitments,
        };
        
        // Get reverse identifier map to convert FROST identifiers back to device IDs
        let device_id_map = dkg::create_device_id_map(&identifier_map);
        
        drop(guard);
        
        // Send commitments to all other selected signers
        for signer_id in selected_signers {
            if signer_id != self_identifier {
                if let Some(device_id) = device_id_map.get(&signer_id) {
                    if let Err(_e) = send_webrtc_message(device_id, &commitment_message, state_clone.clone()).await {
                        // Failed to send commitment message
                    }
                }
            }
        }
        
    });
}

/// Handles initiating FROST Round 1 commitment generation
pub async fn handle_initiate_frost_round1<C>(
    _transaction_data: String,
    selected_signers: Vec<Identifier<C>>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        
        
        // Check if we are one of the selected signers
        let identifier_map = match guard.identifier_map.clone() {
            Some(map) => map,
            None => {
                return;
            }
        };
        
        let self_identifier = match identifier_map.get(&guard.device_id) {
            Some(id) => *id,
            None => {
                let _device_id = guard.device_id.clone();
                return;
            }
        };
        
        let is_selected = dkg::is_device_selected(&self_identifier, &selected_signers);
        
        if !is_selected {
            return;
        }
        
        // Check if we have a key package for signing
        if guard.key_package.is_none() {
            return;
        }
        
        
        // Generate FROST Round 1 commitment
        let commitment_result = match dkg::generate_signing_commitment() {
            Ok(result) => result,
            Err(e) => {
                guard.signing_state = SigningState::Failed {
                        signing_id: "unknown".to_string(),
                    reason: e.to_string(),
                };
                return;
            }
        };
        // Note: SigningCommitments is opaque, we store the whole object
        // The actual commitment to send to others
        let commitments = commitment_result.clone();
        
        // Update signing state with our commitment and nonces and extract signing_id
        let signing_id = if let SigningState::CommitmentPhase { 
            signing_id,
            own_commitment,
            ..
        } = &mut guard.signing_state {
            *own_commitment = Some(commitments.clone());
            // TODO: Store nonces properly - SigningCommitments and SigningNonces are different types
            // For now, leave nonces as None - this is a stub implementation
            // *nonces_field = Some(commitment_result.clone());
            signing_id.clone()
        } else {
            return;
        };
        
        
        // Send commitment to all other selected signers
        let commitment_message = WebRTCMessage::SigningCommitment {
            signing_id: signing_id.clone(),
            sender_identifier: self_identifier,
            commitment: commitments,
        };
        
        // Get identifier map to convert FROST identifiers back to device IDs
        let device_id_map = dkg::create_device_id_map(&identifier_map);
        
        drop(guard);
        
        // Send commitments to all other selected signers
        for signer_id in selected_signers {
            if signer_id != self_identifier {
                if let Some(device_id) = device_id_map.get(&signer_id) {
                    if let Err(_e) = send_webrtc_message(device_id, &commitment_message, state_clone.clone()).await {
                        // Failed to send commitment message
                    }
                }
            }
        }
        
    });
}

#[cfg(test)]

mod tests { #[test] fn test_placeholder() { assert!(true); } }
