//! Handler functions for offline mode commands

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use crate::{
    utils::appstate_compat::AppState,
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
        return;
    }
    
    app_state.offline_mode = enabled;
    
    if enabled {
    } else {
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
            return;
        }
    };
    
    let wallet = match keystore.get_wallet(&wallet_id) {
        Some(w) => w,
        None => {
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
        app_state.offline_config.as_ref().map(|c| c.default_expiration_minutes).unwrap_or(60),
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
    if let Err(_e) = session.add_signing_request(signing_request.clone()) {
        return;
    }
    
    // Store session
    app_state.offline_sessions.insert(session_id.clone(), session);
    
}

/// Handle exporting signing request
pub async fn handle_export_signing_request<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    output_path: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let app_state = state.lock().await;
    
    let session = match app_state.offline_sessions.get(&session_id) {
        Some(s) => s,
        None => {
            return;
        }
    };
    
    let request = match &session.signing_request {
        Some(r) => r,
        None => {
            return;
        }
    };
    
    let path = PathBuf::from(&output_path);
    match export_signing_request(
        request,
        &session_id,
        &path,
        app_state.offline_config.as_ref().map(|c| c.default_expiration_minutes).unwrap_or(60),
    ) {
        Ok(_) => {
        }
        Err(_e) => {
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
    let config = match &app_state.offline_config {
        Some(c) => c,
        None => {
            return; // No config available
        }
    };
    let data = match import_offline_data(&path, config) {
        Ok(d) => d,
        Err(_e) => {
            return;
        }
    };
    
    let request: SigningRequest = match data.extract() {
        Ok(r) => r,
        Err(_e) => {
            return;
        }
    };
    
    // Create or update session
    let session_id = data.session_id.clone();
    
    if let Some(session) = app_state.offline_sessions.get_mut(&session_id) {
        // Update existing session
        if let Err(_e) = session.add_signing_request(request) {
            return;
        }
    } else {
        // Create new session
        let mut session = OfflineSession::new(
            session_id.clone(),
            request.wallet_id.clone(),
            request.required_signers.clone(),
            request.threshold,
            app_state.offline_config.as_ref().map(|c| c.default_expiration_minutes).unwrap_or(60),
        );
        
        if let Err(_e) = session.add_signing_request(request) {
            return;
        }
        
        app_state.offline_sessions.insert(session_id.clone(), session);
    }
    
}

/// Handle reviewing a signing request
pub async fn handle_review_signing_request<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    state: Arc<Mutex<AppState<C>>>,
) {
    let app_state = state.lock().await;
    
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
                    return;
                }
            }
        },
        None => {
            return;
        }
    };
    
    if let Some((_wallet_id, _message, _hash, _chain_type, _threshold, _total_signers, _progress)) = request_info {
        // Offline signing session info available
    }
}

/// Handle listing offline sessions
pub async fn handle_list_offline_sessions<C: frost_core::Ciphersuite + Send + Sync + 'static>(
    state: Arc<Mutex<AppState<C>>>,
) {
    let app_state = state.lock().await;
    
    if app_state.offline_sessions.is_empty() {
        return;
    }
    
    // Collect session info to avoid borrowing issues
    let sessions_info: Vec<_> = app_state.offline_sessions.iter()
        .map(|(id, session)| {
            let progress = session.get_progress();
            (id.clone(), session.wallet_id.clone(), progress)
        })
        .collect();
    
    
    for (_id, _wallet_id, _progress) in sessions_info {
        // Session info: id, wallet_id, progress details
    }
    
}