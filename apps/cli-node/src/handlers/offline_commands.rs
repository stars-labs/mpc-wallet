//! Handler functions for offline mode commands

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use crate::{
    utils::state::AppState,
    offline::{
        OfflineSession,
        types::*,
        export::*, import::*,
    },
};

/// Handle offline mode toggle
pub async fn handle_offline_mode<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    enabled: bool,
    state: Arc<Mutex<AppState<C>>>,
) {
    let mut app_state = state.lock().await;
    
    if enabled == app_state.offline_mode {
        app_state.log.push(format!("Offline mode is already {}", if enabled { "ON" } else { "OFF" }));
        return;
    }
    
    app_state.offline_mode = enabled;
    
    if enabled {
        app_state.log.push("🔒 OFFLINE MODE ENABLED".to_string());
        app_state.log.push("Network operations are disabled".to_string());
        app_state.log.push("Use SD card commands to transfer data".to_string());
    } else {
        app_state.log.push("🌐 OFFLINE MODE DISABLED".to_string());
        app_state.log.push("Network operations resumed".to_string());
    }
}

/// Handle creating a signing request for offline distribution
pub async fn handle_create_signing_request<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    wallet_id: String,
    message: String,
    transaction_hex: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let mut app_state = state.lock().await;
    
    // Get wallet info
    let keystore = match &app_state.keystore {
        Some(ks) => ks.clone(),
        None => {
            app_state.log.push("❌ Keystore not initialized".to_string());
            return;
        }
    };
    
    let wallet = match keystore.get_wallet(&wallet_id) {
        Some(w) => w,
        None => {
            app_state.log.push(format!("❌ Wallet '{}' not found", wallet_id));
            return;
        }
    };
    
    // Create session
    let session_id = format!("signing_{}", chrono::Utc::now().timestamp());
    
    // Generate participant list based on total participants
    // For now, we'll use device IDs like "device-1", "device-2", etc.
    let participants: Vec<String> = (1..=wallet.total_participants)
        .map(|i| format!("device-{}", i))
        .collect();
    
    let mut session = OfflineSession::new(
        session_id.clone(),
        wallet_id.clone(),
        participants.clone(),
        wallet.threshold,
        app_state.offline_config.default_expiration_minutes,
    );
    
    // Determine blockchain type
    let chain_type = if wallet.curve_type == "secp256k1" {
        "ethereum"
    } else {
        "solana"
    };
    
    // Create signing request
    let signing_request = SigningRequest {
        wallet_id: wallet_id.clone(),
        transaction: TransactionData {
            chain_type: chain_type.to_string(),
            payload: {
                use base64::{Engine as _, engine::general_purpose};
                general_purpose::STANDARD.encode(&hex::decode(&transaction_hex).unwrap_or_default())
            },
            hash: transaction_hex.clone(),
            chain_data: None,
        },
        message: message.clone(),
        required_signers: participants,
        threshold: wallet.threshold,
        metadata: None,
    };
    
    // Add to session
    if let Err(e) = session.add_signing_request(signing_request.clone()) {
        app_state.log.push(format!("❌ Failed to create signing request: {}", e));
        return;
    }
    
    // Store session
    app_state.offline_sessions.insert(session_id.clone(), session);
    
    app_state.log.push("✅ Signing request created".to_string());
    app_state.log.push(format!("Session ID: {}", session_id));
    app_state.log.push(format!("Export with: /export_signing_request {}", session_id));
}

/// Handle exporting signing request
pub async fn handle_export_signing_request<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    output_path: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let mut app_state = state.lock().await;
    
    let session = match app_state.offline_sessions.get(&session_id) {
        Some(s) => s,
        None => {
            app_state.log.push(format!("❌ Session '{}' not found", session_id));
            return;
        }
    };
    
    let request = match &session.signing_request {
        Some(r) => r,
        None => {
            app_state.log.push("❌ No signing request in session".to_string());
            return;
        }
    };
    
    let path = PathBuf::from(&output_path);
    match export_signing_request(
        request,
        &session_id,
        &path,
        app_state.offline_config.default_expiration_minutes,
    ) {
        Ok(_) => {
            app_state.log.push(format!("✅ Exported signing request to {}", output_path));
        }
        Err(e) => {
            app_state.log.push(format!("❌ Export failed: {}", e));
        }
    }
}

/// Handle importing signing request
pub async fn handle_import_signing_request<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    input_path: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let mut app_state = state.lock().await;
    
    let path = PathBuf::from(&input_path);
    let data = match import_offline_data(&path, &app_state.offline_config) {
        Ok(d) => d,
        Err(e) => {
            app_state.log.push(format!("❌ Import failed: {}", e));
            return;
        }
    };
    
    let request: SigningRequest = match data.extract() {
        Ok(r) => r,
        Err(e) => {
            app_state.log.push(format!("❌ Invalid signing request: {}", e));
            return;
        }
    };
    
    // Create or update session
    let session_id = data.session_id.clone();
    
    if let Some(session) = app_state.offline_sessions.get_mut(&session_id) {
        // Update existing session
        if let Err(e) = session.add_signing_request(request) {
            app_state.log.push(format!("❌ Failed to update session: {}", e));
            return;
        }
    } else {
        // Create new session
        let mut session = OfflineSession::new(
            session_id.clone(),
            request.wallet_id.clone(),
            request.required_signers.clone(),
            request.threshold,
            app_state.offline_config.default_expiration_minutes,
        );
        
        if let Err(e) = session.add_signing_request(request) {
            app_state.log.push(format!("❌ Failed to create session: {}", e));
            return;
        }
        
        app_state.offline_sessions.insert(session_id.clone(), session);
    }
    
    app_state.log.push(format!("✅ Imported signing request for session {}", session_id));
    app_state.log.push("Review with: /review_signing_request <session_id>".to_string());
}

/// Handle reviewing a signing request
pub async fn handle_review_signing_request<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let mut app_state = state.lock().await;
    
    // Collect the request information to avoid borrowing conflicts
    let request_info = match app_state.offline_sessions.get(&session_id) {
        Some(session) => {
            match &session.signing_request {
                Some(request) => {
                    let progress = session.get_progress();
                    Some((
                        request.wallet_id.clone(),
                        request.message.clone(),
                        request.transaction.hash.clone(),
                        request.transaction.chain_type.clone(),
                        request.threshold,
                        request.required_signers.len(),
                        progress,
                    ))
                },
                None => {
                    app_state.log.push("❌ No signing request in session".to_string());
                    return;
                }
            }
        },
        None => {
            app_state.log.push(format!("❌ Session '{}' not found", session_id));
            return;
        }
    };
    
    if let Some((wallet_id, message, hash, chain_type, threshold, total_signers, progress)) = request_info {
        app_state.log.push("═══════════════════════════════════════════════════".to_string());
        app_state.log.push("SIGNING REQUEST".to_string());
        app_state.log.push("═══════════════════════════════════════════════════".to_string());
        app_state.log.push(format!("Session: {}", session_id));
        app_state.log.push(format!("Wallet: {}", wallet_id));
        app_state.log.push(format!("Message: {}", message));
        app_state.log.push(format!("Transaction: {} ({})", hash, chain_type));
        app_state.log.push(format!("Threshold: {} of {}", threshold, total_signers));
        app_state.log.push("═══════════════════════════════════════════════════".to_string());
        
        app_state.log.push(format!("State: {:?}", progress.state));
        app_state.log.push(format!("Expires in: {} minutes", 
            progress.expires_in.num_minutes()
        ));
    }
}

/// Handle listing offline sessions
pub async fn handle_list_offline_sessions<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    state: Arc<Mutex<AppState<C>>>,
) {
    let mut app_state = state.lock().await;
    
    if app_state.offline_sessions.is_empty() {
        app_state.log.push("No offline sessions".to_string());
        return;
    }
    
    // Collect session info to avoid borrowing issues
    let sessions_info: Vec<_> = app_state.offline_sessions.iter()
        .map(|(id, session)| {
            let progress = session.get_progress();
            (id.clone(), session.wallet_id.clone(), progress)
        })
        .collect();
    
    app_state.log.push("═══════════════════════════════════════════════════".to_string());
    app_state.log.push("OFFLINE SESSIONS".to_string());
    app_state.log.push("═══════════════════════════════════════════════════".to_string());
    
    for (id, wallet_id, progress) in sessions_info {
        app_state.log.push(format!("• {} ({})", id, wallet_id));
        app_state.log.push(format!("  State: {:?}", progress.state));
        app_state.log.push(format!("  Commitments: {}/{}", 
            progress.commitments_received, 
            progress.commitments_needed
        ));
        app_state.log.push(format!("  Shares: {}/{}", 
            progress.shares_received, 
            progress.shares_needed
        ));
        app_state.log.push(format!("  Expires: {} min", 
            progress.expires_in.num_minutes()
        ));
    }
    
    app_state.log.push("═══════════════════════════════════════════════════".to_string());
}