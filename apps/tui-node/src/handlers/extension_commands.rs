//! Handler functions for Chrome extension compatibility commands.

use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs::File;
use std::io::Write;

use crate::{
    keystore::{
        Keystore, WalletData, ExtensionKeystoreBackup, ExtensionBackupWallet,
        ExtensionWalletMetadata, ExtensionKeyShareData,
        encrypt_for_extension, decrypt_from_extension
    },
    utils::{
        appstate_compat::AppState,
        // device::DeviceInfo as UtilDeviceInfo,
    }
};

/// Export a wallet in Chrome extension backup format using browser-compatible encryption
pub async fn handle_export_extension_backup<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    wallet_id: String,
    password: String,
    output_path: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let app_state = state.lock().await;
    
    if let Some(keystore) = &app_state.keystore {
        match export_wallet_to_extension_format(keystore, &wallet_id, &password) {
            Ok(backup) => {
                // Write backup to file
                match write_backup_to_file(&backup, &output_path) {
                    Ok(_) => {
                        // Successfully exported wallet to browser-compatible extension format
                    }
                    Err(_e) => {
                    }
                }
            }
            Err(_e) => {
            }
        }
    } else {
    }
}

/// Import a wallet from Chrome extension backup format
pub async fn handle_import_extension_backup<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    backup_path: String,
    password: String,
    new_password: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let app_state = state.lock().await;
    
    if app_state.keystore.is_none() {
        return;
    }
    
    // Read backup from file
    match read_backup_from_file(&backup_path) {
        Ok(backup) => {
            // Clone keystore Arc to avoid borrow issues
            let keystore_clone = app_state.keystore.as_ref().unwrap().clone();
            
            // Drop the app_state lock before getting mutable keystore reference
            drop(app_state);
            
            // Import wallet
            let result = import_wallet_from_extension_format(
                keystore_clone,
                &backup,
                &password,
                &new_password
            );
            
            // Reacquire lock and report result
            let mut _app_state = state.lock().await;
            match result {
                Ok(wallet_ids) => {
                    // Successfully imported wallets from extension backup
                    for _wallet_id in wallet_ids {
                        // Wallet imported: wallet_id
                    }
                }
                Err(_e) => {
                }
            }
        }
        Err(_e) => {
        }
    }
}

/// Convert current DKG session to extension format
pub async fn handle_convert_dkg_to_extension<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    password: String,
    output_path: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let app_state = state.lock().await;
    
    // Check if DKG is complete
    if !matches!(app_state.dkg_state, crate::utils::state::DkgState::Complete) {
        return;
    }
    
    // Get required data from DKG results
    if app_state.key_package.is_none() || app_state.group_public_key.is_none() || app_state.session.is_none() {
        return;
    }
    
    // Determine curve type and blockchain
    use std::any::TypeId;
    let curve_type_id = TypeId::of::<C>();
    let (curve_type, blockchain) = if curve_type_id == TypeId::of::<frost_secp256k1::Secp256K1Sha256>() {
        ("secp256k1", "ethereum")
    } else if curve_type_id == TypeId::of::<frost_ed25519::Ed25519Sha512>() {
        ("ed25519", "solana")
    } else {
        ("unknown", "unknown")
    };
    
    // Get public address
    let address = if blockchain == "ethereum" {
        app_state.etherum_public_key.clone().unwrap_or_else(|| "N/A".to_string())
    } else {
        app_state.solana_public_key.clone().unwrap_or_else(|| "N/A".to_string())
    };
    
    // Create extension key share data
    let session = app_state.session.as_ref().unwrap();
    let extension_data = ExtensionKeyShareData {
        key_package: base64_helper::encode(
            serde_json::to_vec(app_state.key_package.as_ref().unwrap())
                .unwrap_or_default()
        ),
        public_key_package: base64_helper::encode(
            serde_json::to_vec(app_state.group_public_key.as_ref().unwrap())
                .unwrap_or_default()
        ),
        group_public_key: hex::encode(
            serde_json::to_vec(app_state.group_public_key.as_ref().unwrap())
                .unwrap_or_default()
        ),
        session_id: session.session_id.clone(),
        device_id: app_state.device_id.clone(),
        participant_index: 1, // TODO: Get actual participant index
        threshold: session.threshold,
        total_participants: session.total,
        participants: session.participants.clone(),
        curve: curve_type.to_string(),
        ethereum_address: if blockchain == "ethereum" { Some(address.clone()) } else { None },
        solana_address: if blockchain == "solana" { Some(address.clone()) } else { None },
        created_at: chrono::Utc::now().timestamp_millis(),
        last_used: None,
        backup_date: Some(chrono::Utc::now().timestamp_millis()),
    };
    
    // Encrypt the data
    match encrypt_for_extension(&extension_data, &password, &session.session_id) {
        Ok(encrypted) => {
            // Create backup structure
            let backup = ExtensionKeystoreBackup {
                version: "1.0.0".to_string(),
                device_id: app_state.device_id.clone(),
                exported_at: chrono::Utc::now().timestamp_millis(),
                wallets: vec![ExtensionBackupWallet {
                    metadata: ExtensionWalletMetadata {
                        id: session.session_id.clone(),
                        name: format!("MPC Wallet {}", &session.session_id[..8]),
                        blockchain: blockchain.to_string(),
                        address: address.clone(),
                        session_id: session.session_id.clone(),
                        is_active: true,
                        has_backup: true,
                    },
                    encrypted_share: encrypted,
                }],
            };
            
            // Write to file
            match write_backup_to_file(&backup, &output_path) {
                Ok(_) => {
                    // Successfully exported DKG session to extension format
                }
                Err(_e) => {
                }
            }
        }
        Err(_e) => {
        }
    }
}

// Helper functions

fn export_wallet_to_extension_format(
    keystore: &Keystore,
    wallet_id: &str,
    password: &str,
) -> Result<ExtensionKeystoreBackup, crate::keystore::KeystoreError> {
    use chrono::Utc;
    
    // Load wallet
    let wallet_info = keystore.get_wallet(wallet_id)
        .ok_or_else(|| crate::keystore::KeystoreError::WalletNotFound(wallet_id.to_string()))?;
    
    // Load wallet data
    let wallet_data_bytes = keystore.load_wallet_file(wallet_id, password)?;
    let wallet_data_json: serde_json::Value = serde_json::from_slice(&wallet_data_bytes)
        .map_err(|e| crate::keystore::KeystoreError::DecryptionError(e.to_string()))?;
    
    // Convert to WalletData structure
    let wallet_data = if wallet_info.curve_type == "secp256k1" {
        WalletData {
            secp256k1_key_package: Some(
                serde_json::from_str(wallet_data_json["key_package"].as_str().unwrap_or(""))
                    .map_err(|e| crate::keystore::KeystoreError::DecryptionError(e.to_string()))?
            ),
            secp256k1_public_key: Some(
                serde_json::from_str(wallet_data_json["group_public_key"].as_str().unwrap_or(""))
                    .map_err(|e| crate::keystore::KeystoreError::DecryptionError(e.to_string()))?
            ),
            ed25519_key_package: None,
            ed25519_public_key: None,
            session_id: wallet_data_json["session_id"].as_str().unwrap_or("").to_string(),
            device_id: wallet_data_json["device_id"].as_str().unwrap_or("").to_string(),
        }
    } else {
        WalletData {
            secp256k1_key_package: None,
            secp256k1_public_key: None,
            ed25519_key_package: Some(
                serde_json::from_str(wallet_data_json["key_package"].as_str().unwrap_or(""))
                    .map_err(|e| crate::keystore::KeystoreError::DecryptionError(e.to_string()))?
            ),
            ed25519_public_key: Some(
                serde_json::from_str(wallet_data_json["group_public_key"].as_str().unwrap_or(""))
                    .map_err(|e| crate::keystore::KeystoreError::DecryptionError(e.to_string()))?
            ),
            session_id: wallet_data_json["session_id"].as_str().unwrap_or("").to_string(),
            device_id: wallet_data_json["device_id"].as_str().unwrap_or("").to_string(),
        }
    };
    
    // Get device info
    let device_info = keystore.get_this_device()
        .ok_or_else(|| crate::keystore::KeystoreError::DeviceNotFound(keystore.device_id().to_string()))?;
    
    // Device info is already available from wallet_info
    
    // Convert to extension format
    let extension_data = ExtensionKeyShareData::from_cli_wallet_metadata(
        &wallet_data,
        wallet_info,
        &device_info,
    )?;
    
    // Encrypt the data
    let encrypted_share = encrypt_for_extension(&extension_data, password, wallet_id)?;
    
    // Get primary blockchain info from WalletMetadata
    let (blockchain, address) = if !wallet_info.blockchains.is_empty() {
        let primary = wallet_info.blockchains.iter()
            .find(|b| b.enabled)
            .or_else(|| wallet_info.blockchains.first())
            .ok_or_else(|| crate::keystore::KeystoreError::General("No blockchain found".into()))?;
        (primary.blockchain.clone(), primary.address.clone())
    } else {
        // Fall back to legacy fields
        (
            wallet_info.blockchain.clone().unwrap_or_else(|| "unknown".to_string()),
            wallet_info.public_address.clone().unwrap_or_else(|| "".to_string())
        )
    };
    
    // Create metadata
    let metadata = ExtensionWalletMetadata {
        id: wallet_id.to_string(),
        name: wallet_info.session_id.clone(), // session_id serves as name
        blockchain,
        address,
        session_id: wallet_data.session_id.clone(),
        is_active: true,
        has_backup: true,
    };
    
    Ok(ExtensionKeystoreBackup {
        version: "1.0.0".to_string(),
        device_id: keystore.device_id().to_string(),
        exported_at: Utc::now().timestamp_millis(),
        wallets: vec![ExtensionBackupWallet {
            metadata,
            encrypted_share,
        }],
    })
}

fn import_wallet_from_extension_format(
    keystore: Arc<Keystore>,
    backup: &ExtensionKeystoreBackup,
    password: &str,
    new_password: &str,
) -> Result<Vec<String>, crate::keystore::KeystoreError> {
    let mut imported_wallet_ids = Vec::new();
    
    for wallet in &backup.wallets {
        // Decrypt key share data
        let key_share_data = decrypt_from_extension(&wallet.encrypted_share, password)?;
        
        // Convert to CLI format
        let (wallet_data, mut wallet_info) = key_share_data.to_cli_wallet()?;
        
        // Update wallet name from metadata
        wallet_info.name = wallet.metadata.name.clone();
        
        // Serialize wallet data for storage
        // Serialize key share data based on curve type
        let (key_package_json, public_key_json) = if wallet_info.curve_type == "secp256k1" {
            let kp = wallet_data.secp256k1_key_package.as_ref()
                .ok_or_else(|| crate::keystore::KeystoreError::General("Missing secp256k1 key package".into()))?;
            let pk = wallet_data.secp256k1_public_key.as_ref()
                .ok_or_else(|| crate::keystore::KeystoreError::General("Missing secp256k1 public key".into()))?;
            (
                serde_json::to_string(kp).map_err(|e| crate::keystore::KeystoreError::SerializationError(e.to_string()))?,
                serde_json::to_string(pk).map_err(|e| crate::keystore::KeystoreError::SerializationError(e.to_string()))?
            )
        } else {
            let kp = wallet_data.ed25519_key_package.as_ref()
                .ok_or_else(|| crate::keystore::KeystoreError::General("Missing ed25519 key package".into()))?;
            let pk = wallet_data.ed25519_public_key.as_ref()
                .ok_or_else(|| crate::keystore::KeystoreError::General("Missing ed25519 public key".into()))?;
            (
                serde_json::to_string(kp).map_err(|e| crate::keystore::KeystoreError::SerializationError(e.to_string()))?,
                serde_json::to_string(pk).map_err(|e| crate::keystore::KeystoreError::SerializationError(e.to_string()))?
            )
        };
        
        let key_share_json = serde_json::json!({
            "key_package": key_package_json,
            "group_public_key": public_key_json,
            "session_id": wallet_data.session_id,
            "device_id": wallet_data.device_id,
        });
        
        // Get participant index from key share data before entering unsafe block
        let participant_index = key_share_data.participant_index;
        
        // Create wallet in keystore
        let keystore_ptr = Arc::into_raw(keystore.clone()) as *mut Keystore;
        let wallet_id = unsafe {
            let keystore_mut = &mut *keystore_ptr;
            
            // Get blockchain info from wallet
            let blockchains = if !wallet_info.blockchains.is_empty() {
                wallet_info.blockchains.clone()
            } else {
                // Create from legacy fields if needed
                vec![crate::keystore::BlockchainInfo {
                    blockchain: wallet.metadata.blockchain.clone(),
                    network: "mainnet".to_string(),
                    chain_id: if wallet.metadata.blockchain == "ethereum" { Some(1) } else { None },
                    address: wallet.metadata.address.clone(),
                    address_format: if wallet.metadata.blockchain == "ethereum" { "EIP-55".to_string() } else { "base58".to_string() },
                    enabled: true,
                    rpc_endpoint: None,
                    metadata: None,
                }]
            };
            
            keystore_mut.create_wallet_multi_chain(
                &wallet_info.name,
                &wallet_info.curve_type,
                blockchains,
                wallet_info.threshold as u16,
                wallet_info.total_participants as u16,
                &wallet_info.group_public_key,
                key_share_json.to_string().as_bytes(),
                new_password,
                vec![], // tags
                Some(format!("Imported from Chrome extension backup")),
                participant_index,
            )?
        };
        
        // Re-wrap the pointer
        let _keystore = unsafe { Arc::from_raw(keystore_ptr) };
        
        imported_wallet_ids.push(wallet_id);
    }
    
    Ok(imported_wallet_ids)
}

fn write_backup_to_file(
    backup: &ExtensionKeystoreBackup,
    path: &str,
) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(backup)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn read_backup_from_file(path: &str) -> Result<ExtensionKeystoreBackup, std::io::Error> {
    let file = File::open(path)?;
    let backup = serde_json::from_reader(file)?;
    Ok(backup)
}

// Base64 helper module
mod base64_helper {
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn encode(data: Vec<u8>) -> String {
        general_purpose::STANDARD.encode(data)
    }
}