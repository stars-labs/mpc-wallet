//! Real FROST DKG Implementation
//! 
//! This implementation uses the exact same FROST cryptographic logic as the dkg.rs example.
//! It properly implements all three phases of FROST DKG:
//! - Part 1: Generates and exchanges commitments
//! - Part 2: Generates and distributes secret shares 
//! - Part 3: Computes the real group public key from DKG output
//! 
//! The previous insecure implementation that derived group keys from session IDs
//! has been completely removed and replaced with proper FROST threshold cryptography.

use crate::protocal::signal::WebRTCMessage;
use crate::utils::appstate_compat::AppState;
use crate::utils::state::DkgState;
use frost_core::{Ciphersuite, Identifier};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use base64;

/// DKG execution mode for different coordination scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DkgMode {
    Online,    // Real-time WebRTC mesh coordination
    Offline,   // Air-gapped with file/QR code exchange
    Hybrid,    // Online coordination, offline key generation
}

impl Default for DkgMode {
    fn default() -> Self {
        DkgMode::Online
    }
}

// SimpleDkgPackage removed - using real FROST packages now

// Removed SimpleRound1Package, SimpleRound2Package, and SimpleKeyPackage - using real FROST types now

// Removed insecure derive_group_key function - now using real FROST DKG output

/// Start DKG Round 1 - Real FROST implementation
pub async fn handle_trigger_dkg_round1<C>(
    state: Arc<Mutex<AppState<C>>>, 
    self_device_id: String,
    _internal_cmd_tx: tokio::sync::mpsc::UnboundedSender<crate::utils::state::InternalCommand<C>>
) 
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    // Check if we have a session
    let session = match &guard.session {
        Some(s) => s.clone(),
        None => {
            guard.dkg_state = DkgState::Failed("No session available".to_string());
            return;
        }
    };
    
    // Start DKG Round 1
    guard.dkg_state = DkgState::Round1InProgress;
    
    // Determine our identifier (1-based index in participant list)
    let my_index = session.participants.iter()
        .position(|p| p == &self_device_id)
        .map(|i| i as u16 + 1)  // Convert to 1-based
        .unwrap_or(1);
    
    let my_identifier = Identifier::<C>::try_from(my_index).expect("Invalid identifier");
    
    // Generate real FROST DKG round 1
    // Use the frost_ed25519 rand_core for compatibility
    use frost_ed25519::rand_core::OsRng;
    let mut rng = OsRng;
    let (round1_secret_package, round1_public_package) = frost_core::keys::dkg::part1(
        my_identifier,
        session.total,
        session.threshold,
        &mut rng,
    ).expect("Failed to generate DKG round 1");
    
    // Store the secret package for later use
    guard.dkg_part1_secret_package = Some(round1_secret_package.serialize().expect("Failed to serialize secret package"));
    guard.dkg_part1_public_package = Some(round1_public_package.serialize().expect("Failed to serialize public package"));
    
    // Store our own round1 package
    guard.dkg_round1_packages.insert(my_identifier, round1_public_package.clone());
    
    // Serialize the public package for broadcasting
    let package_bytes = round1_public_package.serialize().expect("Failed to serialize round1 package");
    
    // Create WebRTC message for broadcasting
    let message = WebRTCMessage::SimpleMessage {
        text: format!("DKG_ROUND1:{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &package_bytes)),
    };
    
    // Broadcast to session participants
    let participants = session.participants.clone();
    drop(guard);
    
    for device_id in participants {
        if device_id != self_device_id {
            if let Err(_e) = crate::utils::device::send_webrtc_message(&device_id, &message, state.clone()).await {
                tracing::warn!("Failed to send DKG Round 1 package to {}: {:?}", device_id, _e);
            }
        }
    }
}

/// Process DKG Round 1 package - Real FROST implementation
pub async fn process_dkg_round1<C>(
    state: Arc<Mutex<AppState<C>>>,
    from_device_id: String,
    package_bytes: Vec<u8>,
) 
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    // Get session to determine sender's identifier
    let session = match &guard.session {
        Some(s) => s.clone(),
        None => return,
    };
    
    // Determine sender's identifier (1-based index)
    let sender_index = session.participants.iter()
        .position(|p| p == &from_device_id)
        .map(|i| i as u16 + 1)  // Convert to 1-based
        .unwrap_or(1);
    
    let sender_identifier = Identifier::<C>::try_from(sender_index).expect("Invalid identifier");
    
    // Deserialize the real FROST round1 package
    let round1_package = match frost_core::keys::dkg::round1::Package::<C>::deserialize(&package_bytes) {
        Ok(pkg) => pkg,
        Err(e) => {
            tracing::error!("Failed to deserialize DKG Round 1 package: {}", e);
            return;
        }
    };
    
    // Store the round1 package
    guard.dkg_round1_packages.insert(sender_identifier, round1_package);
    
    // Check if we have enough packages to proceed (need all participants including ourselves)
    let required_count = session.total as usize;
    let received_count = guard.dkg_round1_packages.len();
    
    tracing::info!("DKG Round 1: received {}/{} packages total", received_count, required_count);
    
    if received_count >= required_count {
        // Move to Round 2
        guard.dkg_state = DkgState::Round1Complete;
        tracing::info!("All DKG Round 1 packages received, triggering Round 2");
        
        // Trigger Round 2 immediately
        let self_device_id = guard.device_id.clone();
        drop(guard);
        
        handle_trigger_dkg_round2(state, self_device_id).await;
    }
}

/// Start DKG Round 2 - Real FROST part2 implementation
pub async fn handle_trigger_dkg_round2<C>(
    state: Arc<Mutex<AppState<C>>>,
    self_device_id: String,
) 
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    // Check state
    if !matches!(guard.dkg_state, DkgState::Round1Complete) {
        return;
    }
    
    guard.dkg_state = DkgState::Round2InProgress;
    
    let session = match &guard.session {
        Some(s) => s.clone(),
        None => {
            guard.dkg_state = DkgState::Failed("No session in Round 2".to_string());
            return;
        }
    };
    
    // Get our identifier
    let my_index = session.participants.iter()
        .position(|p| p == &self_device_id)
        .map(|i| i as u16 + 1)  // Convert to 1-based
        .unwrap_or(1);
    let my_identifier = Identifier::<C>::try_from(my_index).expect("Invalid identifier");
    
    // Get our secret package from round 1
    let secret_package = match &guard.dkg_part1_secret_package {
        Some(bytes) => frost_core::keys::dkg::round1::SecretPackage::<C>::deserialize(bytes)
            .expect("Failed to deserialize secret package"),
        None => {
            guard.dkg_state = DkgState::Failed("Missing round 1 secret package".to_string());
            return;
        }
    };
    
    // Collect all round 1 packages EXCLUDING our own (like in dkg.rs example)
    let round1_packages = guard.dkg_round1_packages.clone();
    let round1_packages_from_others: std::collections::BTreeMap<_, _> = round1_packages
        .iter()
        .filter(|(id, _)| **id != my_identifier)
        .map(|(id, pkg)| (*id, pkg.clone()))
        .collect();
    
    // Generate round 2 packages using FROST part2
    let (round2_secret_package, round2_public_packages) = match frost_core::keys::dkg::part2(
        secret_package,
        &round1_packages_from_others,
    ) {
        Ok(result) => result,
        Err(e) => {
            guard.dkg_state = DkgState::Failed(format!("DKG part2 failed: {:?}", e));
            return;
        }
    };
    
    // Store the round2 secret package for part3
    guard.dkg_part2_secret_package = Some(round2_secret_package.serialize().expect("Failed to serialize"));
    
    // Don't store our own packages - round2 packages are meant for others
    // We'll receive round2 packages meant for us via process_dkg_round2
    
    drop(guard);
    
    // Create identifier to device_id mapping
    let mut identifier_to_device_id = std::collections::HashMap::new();
    for (index, device_id) in session.participants.iter().enumerate() {
        let identifier = frost_core::Identifier::<C>::try_from((index + 1) as u16).expect("Invalid identifier");
        identifier_to_device_id.insert(identifier, device_id.clone());
    }
    
    // Broadcast round 2 packages to each participant
    for (receiver_id, package) in round2_public_packages {
        if let Some(receiver_device_id) = identifier_to_device_id.get(&receiver_id) {
            if receiver_device_id != &self_device_id {
                let package_bytes = package.serialize().expect("Failed to serialize round2 package");
                let message = WebRTCMessage::SimpleMessage {
                    text: format!("DKG_ROUND2:{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &package_bytes)),
                };
                
                if let Err(_e) = crate::utils::device::send_webrtc_message(receiver_device_id, &message, state.clone()).await {
                    tracing::warn!("Failed to send DKG Round 2 package to {}: {:?}", receiver_device_id, _e);
                }
            }
        } else {
            tracing::warn!("Could not find device_id for identifier {:?}", receiver_id);
        }
    }
}

/// Process DKG Round 2 package - Real FROST implementation with part3
pub async fn process_dkg_round2<C>(
    state: Arc<Mutex<AppState<C>>>,
    from_device_id: String,
    package_bytes: Vec<u8>,
) 
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    // Get session to determine sender's identifier  
    let session = match &guard.session {
        Some(s) => s.clone(),
        None => return,
    };
    
    // Get our identifier
    let my_index = session.participants.iter()
        .position(|p| p == &guard.device_id)
        .map(|i| i as u16 + 1)  // Convert to 1-based
        .unwrap_or(1);
    let my_identifier = Identifier::<C>::try_from(my_index).expect("Invalid identifier");
    
    // Determine sender's identifier
    let sender_index = session.participants.iter()
        .position(|p| p == &from_device_id)
        .map(|i| i as u16 + 1)  // Convert to 1-based
        .unwrap_or(1);
    let sender_identifier = Identifier::<C>::try_from(sender_index).expect("Invalid identifier");
    
    // Deserialize the real FROST round2 package
    let round2_package = match frost_core::keys::dkg::round2::Package::<C>::deserialize(&package_bytes) {
        Ok(pkg) => pkg,
        Err(e) => {
            tracing::error!("Failed to deserialize DKG Round 2 package: {}", e);
            return;
        }
    };
    
    // Store the round2 package
    guard.dkg_round2_packages.insert(sender_identifier, round2_package);
    
    // Check if we have received round2 packages from all other participants
    let session = match &guard.session {
        Some(s) => s.clone(),
        None => return,
    };
    
    let expected_senders = session.total as usize - 1; // All participants except ourselves
    let received_count = guard.dkg_round2_packages.len();
    
    tracing::info!("DKG Round 2: received {}/{} packages from other participants", received_count, expected_senders);
    
    if received_count >= expected_senders {
        // Now run FROST part3 to complete DKG
        
        // Get our round1 packages EXCLUDING our own (like in dkg.rs example)
        let round1_packages = guard.dkg_round1_packages.clone();
        let round1_packages_from_others: std::collections::BTreeMap<_, _> = round1_packages
            .iter()
            .filter(|(id, _)| **id != my_identifier)
            .map(|(id, pkg)| (*id, pkg.clone()))
            .collect();
        
        let round2_secret_package = match &guard.dkg_part2_secret_package {
            Some(bytes) => frost_core::keys::dkg::round2::SecretPackage::<C>::deserialize(bytes)
                .expect("Failed to deserialize round2 secret package"),
            None => {
                guard.dkg_state = DkgState::Failed("Missing round 2 secret package".to_string());
                return;
            }
        };
        
        // Get our round2 package (this contains only packages sent TO us)
        let round2_packages_for_us = guard.dkg_round2_packages.clone();
        
        // Run FROST part3 to get the key package and public key package
        let (key_package, pubkey_package) = match frost_core::keys::dkg::part3(
            &round2_secret_package,
            &round1_packages_from_others,
            &round2_packages_for_us,
        ) {
            Ok(result) => result,
            Err(e) => {
                guard.dkg_state = DkgState::Failed(format!("DKG part3 failed: {:?}", e));
                return;
            }
        };
        
        // Store the real key package and public key package
        guard.key_package = Some(key_package.clone());
        guard.public_key_package = Some(pubkey_package.clone());
        
        // Get the real verifying key from the public key package
        let verifying_key = pubkey_package.verifying_key();
        guard.group_public_key = Some(verifying_key.clone());
        
        // Complete DKG
        guard.dkg_state = DkgState::Complete;
        
        // Generate wallet ID
        let wallet_id = if let Some(session) = &guard.session {
            format!("wallet-{}", &session.session_id[..8])
        } else {
            "wallet-default".to_string()
        };
        guard.current_wallet_id = Some(wallet_id.clone());
        
        // Log the real group public key
        tracing::info!("🎉 DKG completed successfully!");
        tracing::info!("Group Verifying Key: {:?}", verifying_key);
        tracing::info!("Key Package Identifier: {:?}", key_package.identifier());
        tracing::info!("Min signers: {:?}", key_package.min_signers());
        
        // Now we can use the real verifying key to generate addresses
        // First serialize the verifying key properly
        let group_public_key_bytes = verifying_key.serialize()
            .expect("Failed to serialize verifying key");
        
        // Generate appropriate blockchain addresses based on curve type
        use crate::blockchain_config::{CurveType, get_compatible_chains, generate_address_for_chain};
        
        let curve_type = session.curve_type.clone();
        
        // Get ALL compatible chains for this curve and generate addresses
        let compatible_chains = get_compatible_chains(
            &CurveType::from_string(&curve_type).unwrap_or(CurveType::Secp256k1)
        );
        
        let mut generated_addresses = Vec::new();
        let mut blockchain_addresses = Vec::new();
        
        for (chain_id, _) in compatible_chains.iter() {
            match generate_address_for_chain(&group_public_key_bytes, &curve_type, chain_id) {
                Ok(address) => {
                    generated_addresses.push(format!("{}: {}", chain_id, address));
                    tracing::info!("Generated {} address: {}", chain_id, address);
                    
                    // Create BlockchainInfo for UI display
                    // Map chain_id to proper chain ID for EVM chains
                    let chain_id_num = match chain_id.as_ref() {
                        "ethereum" => Some(1u64),
                        "bsc" => Some(56u64),
                        "polygon" => Some(137u64),
                        "avalanche" => Some(43114u64),
                        "arbitrum" => Some(42161u64),
                        "optimism" => Some(10u64),
                        _ => None,
                    };
                    
                    // Determine address format based on chain
                    let addr_format = if chain_id == &"bitcoin" {
                        "P2WPKH".to_string()
                    } else if chain_id == &"solana" || chain_id == &"sui" || chain_id == &"aptos" {
                        "base58".to_string()
                    } else {
                        "EIP-55".to_string() // Ethereum and EVM chains
                    };
                    
                    let blockchain_info = crate::keystore::BlockchainInfo {
                        blockchain: chain_id.to_string(),
                        network: "mainnet".to_string(),
                        chain_id: chain_id_num,
                        address: address.clone(),
                        address_format: addr_format,
                        enabled: true,
                        rpc_endpoint: None,
                        metadata: None,
                    };
                    blockchain_addresses.push(blockchain_info);
                }
                Err(e) => {
                    tracing::warn!("Could not generate {} address: {}", chain_id, e);
                }
            }
        }
        
        // Store blockchain addresses for UI
        guard.blockchain_addresses = blockchain_addresses.clone();
        
        // Store the first compatible address for backward compatibility
        if let Some(first_address) = generated_addresses.first() {
            // Extract just the address part (after the ": ")
            if let Some(addr_part) = first_address.split(": ").nth(1) {
                guard.etherum_public_key = Some(addr_part.to_string());
            }
        }
        
        // Log successful DKG completion with real FROST key
        let display_address = guard.etherum_public_key.as_deref().unwrap_or("no address");
        tracing::info!("🎉 DKG completed successfully with REAL FROST!");
        tracing::info!("Wallet ID: {}, Primary Address: {}", wallet_id, display_address);
        tracing::info!("DKG State set to: {:?}", guard.dkg_state);
        tracing::info!("Generated {} blockchain addresses", guard.blockchain_addresses.len());
        for blockchain_info in &guard.blockchain_addresses {
            tracing::info!("  - {}: {}", blockchain_info.blockchain, blockchain_info.address);
        }
    }
}

/// Handle DKG finalization - simplified
pub async fn handle_dkg_finalization<C>(state: Arc<Mutex<AppState<C>>>) 
where
    C: Ciphersuite + Send + Sync + 'static,
{
    let mut guard = state.lock().await;
    
    if !matches!(guard.dkg_state, DkgState::Round2Complete) {
        return;
    }
    
    // Simple finalization
    guard.dkg_state = DkgState::Complete;
    
    tracing::info!("DKG finalization completed for device: {}", guard.device_id);
}

/// Finalize DKG - alias for compatibility
pub async fn finalize_dkg<C>(
    state: Arc<Mutex<AppState<C>>>,
    _device_id: String,  // Accept device_id parameter for compatibility
) 
where
    C: Ciphersuite + Send + Sync + 'static,
{
    handle_dkg_finalization(state).await;
}

/// Check if device is selected as signer - simplified helper
pub fn is_device_selected<C: Ciphersuite>(
    device_identifier: &Identifier<C>,
    selected_signers: &[Identifier<C>],
) -> bool {
    selected_signers.contains(device_identifier)
}

/// Create device ID to identifier map - simplified
pub fn create_device_id_map<C: Ciphersuite>(
    identifier_map: &std::collections::HashMap<String, Identifier<C>>
) -> std::collections::HashMap<Identifier<C>, String> {
    identifier_map.iter().map(|(k, v)| (*v, k.clone())).collect()
}

/// Map selected signers - stub
pub fn map_selected_signers<C: Ciphersuite>(
    _signers: Vec<String>
) -> Vec<Identifier<C>> {
    Vec::new()
}

/// Create signing package - stub
pub fn create_signing_package<C: Ciphersuite>(
    _message: &[u8],
    _signing_commitments: Vec<frost_core::round1::SigningCommitments<C>>,
) -> Result<frost_core::SigningPackage<C>, Box<dyn std::error::Error + Send + Sync>> {
    Err("Signing package creation is temporarily stubbed".into())
}

/// Generate signature share - stub
pub fn generate_signature_share<C: Ciphersuite>(
    _signing_package: &frost_core::SigningPackage<C>,
    _nonces: &frost_core::round1::SigningNonces<C>,
    _key_package: &frost_core::keys::KeyPackage<C>,
) -> Result<frost_core::round2::SignatureShare<C>, Box<dyn std::error::Error + Send + Sync>> {
    Err("Signature share generation is temporarily stubbed".into())
}

/// Aggregate signature - stub
pub fn aggregate_signature<C: Ciphersuite>(
    _signing_package: &frost_core::SigningPackage<C>,
    _signature_shares: &std::collections::BTreeMap<frost_core::Identifier<C>, frost_core::round2::SignatureShare<C>>,
    _group_public_key: &frost_core::VerifyingKey<C>,
) -> Result<frost_core::Signature<C>, Box<dyn std::error::Error + Send + Sync>> {
    Err("Signature aggregation is temporarily stubbed".into())
}

/// Generate signing commitment - stub
pub fn generate_signing_commitment<C: Ciphersuite>(
) -> Result<frost_core::round1::SigningCommitments<C>, Box<dyn std::error::Error + Send + Sync>> {
    Err("Signing commitment generation is temporarily stubbed".into())
}