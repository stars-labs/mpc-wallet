//! Handler functions for keystore-related commands.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;

use crate::{
    keystore::Keystore,
    utils::state::AppState};

/// Show wallet file location for direct sharing with Chrome extension
pub async fn handle_locate_wallet<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    wallet_id: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let mut app_state = state.lock().await;
    
    if let Some(keystore) = &app_state.keystore {
        if let Some(wallet) = keystore.get_wallet(&wallet_id) {
            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let wallet_path = home_dir
                .join(".frost_keystore")
                .join(&keystore.device_id())
                .join(&wallet.curve_type)
                .join(format!("{}.json", wallet_id));
            
            // Clone wallet fields to avoid borrowing issues
            let wallet_device_id = wallet.device_id.clone();
            let wallet_participant_index = wallet.participant_index;
            let wallet_threshold = wallet.threshold;
            let wallet_total_participants = wallet.total_participants;
            
            app_state.log.push("═══════════════════════════════════════════════════".to_string());
            app_state.log.push("WALLET FILE LOCATION".to_string());
            app_state.log.push("═══════════════════════════════════════════════════".to_string());
            app_state.log.push(format!("Wallet: {}", wallet_id));
            app_state.log.push(format!("Your device: {} (participant #{})", wallet_device_id, wallet_participant_index));
            app_state.log.push(format!("Threshold: {}/{}", wallet_threshold, wallet_total_participants));
            app_state.log.push(format!("File: {}", wallet_path.display()));
            app_state.log.push("".to_string());
            app_state.log.push("🚀 CHROME EXTENSION IMPORT:".to_string());
            app_state.log.push("1. Copy this JSON file".to_string());
            app_state.log.push("2. In Chrome extension, click 'Import Wallet'".to_string());
            app_state.log.push("3. Select this file or paste its contents".to_string());
            app_state.log.push("4. Use the same password (your device ID)".to_string());
            app_state.log.push("".to_string());
            app_state.log.push("💡 TIP: You can share this file with teammates for".to_string());
            app_state.log.push("    collaborative threshold signing!".to_string());
            app_state.log.push("═══════════════════════════════════════════════════".to_string());
        } else {
            app_state.log.push(format!("❌ Wallet '{}' not found", wallet_id));
        }
    } else {
        app_state.log.push("❌ Keystore not initialized".to_string());
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
            app_state.log.push(format!("Keystore initialized at {} for device '{}'", path, device_name));
            app_state.keystore = Some(Arc::new(keystore));
        }
        Err(e) => {
            app_state.log.push(format!("Failed to initialize keystore: {}", e));
        }
    }
}

/// Handles the list_wallets command
pub async fn handle_list_wallets<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    state: Arc<Mutex<AppState<C>>>
) {
    let mut app_state = state.lock().await;
    
    if let Some(keystore) = &app_state.keystore {
        // First, collect the wallet info
        let wallets = keystore.list_wallets();
        
        if wallets.is_empty() {
            app_state.log.push("No wallets found in keystore.".to_string());
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
            app_state.log.push("═══════════════════════════════════════════════════".to_string());
            app_state.log.push("Your wallets:".to_string());
            app_state.log.push("═══════════════════════════════════════════════════".to_string());
            
            for (_id, name, threshold, total, curve, blockchains, created_at, device_id) in wallet_infos {
                app_state.log.push(format!(
                    "• {} ({}/{}, {}) - {} devices",
                    name, threshold, total, curve, total
                ));
                app_state.log.push(format!(
                    "  Your device: {}",
                    device_id
                ));
                app_state.log.push(format!(
                    "  Created: {}",
                    chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|_| created_at)
                ));
                
                // Show enabled blockchain addresses
                for blockchain_info in blockchains.iter().filter(|b| b.enabled) {
                    app_state.log.push(format!(
                        "  {}: {}",
                        blockchain_info.blockchain, blockchain_info.address
                    ));
                }
            }
            app_state.log.push("═══════════════════════════════════════════════════".to_string());
            app_state.log.push("".to_string());
            app_state.log.push("Use wallet name with /propose to start signing".to_string());
        }
    } else {
        app_state.log.push("Keystore is not initialized. Use /init_keystore first.".to_string());
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
    let mut app_state = state.lock().await;
    
    // Check if keystore is initialized
    if app_state.keystore.is_none() {
        app_state.log.push("Keystore is not initialized. Use /init_keystore first.".to_string());
        return;
    }
    
    // Check if DKG is completed
    if !matches!(app_state.dkg_state, crate::utils::state::DkgState::Complete) {
        app_state.log.push("DKG process is not complete. Cannot create wallet yet.".to_string());
        return;
    }
    
    // Get required data from DKG results
    if app_state.key_package.is_none() || app_state.group_public_key.is_none() || app_state.session.is_none() {
        app_state.log.push("Missing DKG results. Cannot create wallet.".to_string());
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
            app_state.log.push(format!("✅ Wallet created successfully with ID: {}", wallet_id));
            app_state.current_wallet_id = Some(wallet_id.clone());
            
            // Show wallet file location immediately after creation
            if let Some(keystore) = &app_state.keystore {
                let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                let wallet_path = home_dir
                    .join(".frost_keystore")
                    .join(&keystore.device_id())
                    .join(curve_type)
                    .join(format!("{}.json", wallet_id));
                
                app_state.log.push("".to_string());
                app_state.log.push("═══════════════════════════════════════════════════".to_string());
                app_state.log.push("WALLET CREATED - FILE LOCATION".to_string());
                app_state.log.push("═══════════════════════════════════════════════════".to_string());
                app_state.log.push(format!("📂 File: {}", wallet_path.display()));
                app_state.log.push(format!("🔑 Your device: {}", device_id));
                app_state.log.push("".to_string());
                app_state.log.push("🚀 CHROME EXTENSION IMPORT:".to_string());
                app_state.log.push("1. Copy this JSON file to share with teammates".to_string());
                app_state.log.push("2. In Chrome extension, click 'Import Wallet'".to_string());
                app_state.log.push("3. Select this file or paste its contents".to_string());
                app_state.log.push("4. Use the same password (your device ID)".to_string());
                app_state.log.push("".to_string());
                app_state.log.push("💡 TIP: Use /locate_wallet {} to show this info again".to_string().replace("{}", &wallet_id));
                app_state.log.push("═══════════════════════════════════════════════════".to_string());
            }
        },
        Err(e) => {
            app_state.log.push(format!("❌ Failed to create wallet: {}", e));
        }
    }
}