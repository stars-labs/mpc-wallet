//! Handler functions for keystore-related commands.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;

use crate::{
    keystore::Keystore,
    utils::appstate_compat::AppState};

/// Show wallet file location for direct sharing with Chrome extension
pub async fn handle_locate_wallet<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    wallet_id: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let app_state = state.lock().await;
    
    if let Some(keystore) = &app_state.keystore {
        if let Some(wallet) = keystore.get_wallet(&wallet_id) {
            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let _wallet_path = home_dir
                .join(".frost_keystore")
                .join(&keystore.device_id())
                .join(&wallet.curve_type)
                .join(format!("{}.json", wallet_id));
            
            // Clone wallet fields to avoid borrowing issues
            let _wallet_device_id = wallet.device_id.clone();
            let _wallet_participant_index = wallet.participant_index;
            let _wallet_threshold = wallet.threshold;
            let _wallet_total_participants = wallet.total_participants;
            
        } else {
        }
    } else {
    }
}

/// Handles the init_keystore command
pub async fn handle_init_keystore<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    path: String,
    device_name: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let mut app_state = state.lock().await;
    
    match Keystore::new(&path, &device_name) {
        Ok(keystore) => {
            app_state.keystore = Some(Arc::new(keystore));
        }
        Err(_e) => {
        }
    }
}

/// Handles the list_wallets command
pub async fn handle_list_wallets<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    state: Arc<Mutex<AppState<C>>>
) {
    let app_state = state.lock().await;
    
    if let Some(keystore) = &app_state.keystore {
        // First, collect the wallet info
        let wallets = keystore.list_wallets();
        
        if wallets.is_empty() {
        } else {
            // Clone wallet information to avoid borrow issues
            let wallet_infos = wallets
                .iter()
                .map(|w| {
                    // Get blockchain info - prioritize new format, fall back to legacy
                    let blockchains = if !w.blockchains.is_empty() {
                        w.blockchains.clone()
                    } else if let (Some(blockchain), Some(address)) = (&w.blockchain, &w.public_address) {
                        // Convert legacy format
                        vec![crate::keystore::BlockchainInfo {
                            blockchain: blockchain.clone(),
                            network: "mainnet".to_string(),
                            chain_id: if blockchain == "ethereum" { Some(1) } else { None },
                            address: address.clone(),
                            address_format: if blockchain == "ethereum" { "EIP-55".to_string() } else { "base58".to_string() },
                            enabled: true,
                            rpc_endpoint: None,
                            metadata: None,
                        }]
                    } else {
                        Vec::new()
                    };
                    
                    (
                        w.session_id.clone(),
                        w.session_id.clone(), // session_id serves as the name
                        w.threshold,
                        w.total_participants,
                        w.curve_type.clone(),
                        blockchains,
                        w.created_at.clone(),
                        w.device_id.clone()
                    )
                })
                .collect::<Vec<_>>();
            
            // Now that we're done with the keystore borrow, update the UI
            
            for (_id, _name, _threshold, _total, _curve, blockchains, _created_at, _device_id) in wallet_infos {
                // Wallet information available
                
                // Show enabled blockchain addresses
                for _blockchain_info in blockchains.iter().filter(|b| b.enabled) {
                    // Blockchain: blockchain_info.blockchain, Address: blockchain_info.address
                }
            }
        }
    } else {
    }
}

/// Handles the create_wallet command, creating a new wallet from DKG results
pub async fn handle_create_wallet<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    name: String,
    description: Option<String>,
    password: String,
    tags: Vec<String>,
    state: Arc<Mutex<AppState<C>>>,
) {
    let app_state = state.lock().await;
    
    // Check if keystore is initialized
    if app_state.keystore.is_none() {
        return;
    }
    
    // Check if DKG is completed
    if !matches!(app_state.dkg_state, crate::utils::state::DkgState::Complete) {
        return;
    }
    
    // Get required data from DKG results
    if app_state.key_package.is_none() || app_state.group_public_key.is_none() || app_state.session.is_none() {
        return;
    }
    
    // Determine curve type based on TypeId
    use std::any::TypeId;
    
    let curve_type_id = TypeId::of::<C>();
    let curve_type = if curve_type_id == TypeId::of::<frost_secp256k1::Secp256K1Sha256>() {
        "secp256k1"
    } else if curve_type_id == TypeId::of::<frost_ed25519::Ed25519Sha512>() {
        "ed25519"
    } else {
        "unknown"
    };
    
    // Get blockchain addresses from app state
    let blockchains = app_state.blockchain_addresses.clone();
    
    // Clone necessary data before dropping the lock
    let session_id = app_state.session.as_ref().unwrap().session_id.clone();
    let threshold = app_state.session.as_ref().unwrap().threshold;
    let total_participants = app_state.session.as_ref().unwrap().total;
    let device_id = app_state.device_id.clone();
    
    // Serialize the key package data
    let key_package_json = serde_json::to_string(app_state.key_package.as_ref().unwrap()).unwrap_or_default();
    let group_public_key_json = serde_json::to_string(app_state.group_public_key.as_ref().unwrap()).unwrap_or_default();
    
    // Serialize the KeyPackage and other necessary data
    let key_share_data = json!({
        "key_package": key_package_json,
        "group_public_key": group_public_key_json,
        "session_id": session_id,
        "device_id": device_id
    }).to_string();
    
    // Create wallet in keystore
    // We need to get a mutable reference to the inner keystore
    let keystore_clone = app_state.keystore.as_ref().unwrap().clone();
    
    // We need to drop the app_state lock before we try to get a mutable reference to keystore
    drop(app_state);
    
    // Since keystore is behind Arc, we need to get a mutable reference to it
    // This is unsafe but needed because Rust doesn't support Arc::get_mut with shared references
    // In a real-world application, we might want to use a better synchronization mechanism
    let keystore_ptr = Arc::into_raw(keystore_clone) as *mut Keystore;
    let result = unsafe {
        let keystore_mut = &mut *keystore_ptr;
        
        keystore_mut.create_wallet_multi_chain(
            &name,
            curve_type,
            blockchains,
            threshold,
            total_participants,
            &group_public_key_json, // Already serialized
            key_share_data.as_bytes(),
            &password,
            tags,
            description,
            1, // Default participant_index for manual wallet creation
        )
    };
    
    // Re-wrap the pointer in an Arc so it will be properly deallocated
    let _keystore = unsafe { Arc::from_raw(keystore_ptr) };
    
    // Now regain the lock and update the app state
    let mut app_state = state.lock().await;
    
    match result {
        Ok(wallet_id) => {
            app_state.current_wallet_id = Some(wallet_id.clone());
            
            // Show wallet file location immediately after creation
            if let Some(keystore) = &app_state.keystore {
                let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                let _wallet_path = home_dir
                    .join(".frost_keystore")
                    .join(&keystore.device_id())
                    .join(curve_type)
                    .join(format!("{}.json", wallet_id));
                
            }
        },
        Err(_e) => {
        }
    }
}