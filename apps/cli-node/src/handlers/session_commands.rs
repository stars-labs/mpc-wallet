use crate::network::webrtc::initiate_webrtc_connections;
use crate::protocal::signal::{SessionInfo, SessionProposal, SessionResponse, WebSocketMessage, SessionType};
use crate::utils::state::{AppState, InternalCommand};
use frost_core::{Ciphersuite, Identifier};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use webrtc_signal_server::ClientMsg;

/// Helper function to check if conditions are met for automatic mesh ready
async fn check_and_auto_mesh_ready<C>(
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar:
        Send + Sync,
{
    let should_send_mesh_ready = {
        let state_guard = state.lock().await;

        // Check if we have an active session
        if let Some(session) = &state_guard.session {
            // Condition 1: All session responses received (all participants accepted)
            let all_responses_received =
                session.accepted_devices.len() == session.participants.len();

            // Condition 2: All WebRTC connections to session devices are connected
            let session_devices_except_self: Vec<String> = session
                .participants
                .iter()
                .filter(|p| **p != state_guard.device_id)
                .cloned()
                .collect();

            let all_webrtc_connected = session_devices_except_self.iter().all(|device_id| {
                state_guard.device_statuses.get(device_id)
                    .map(|status| matches!(status, webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected))
                    .unwrap_or(false)
            });

            // Check if we haven't already sent our mesh ready signal
            let not_already_sent = !matches!(
                state_guard.mesh_status,
                crate::utils::state::MeshStatus::Ready
            );

            all_responses_received && all_webrtc_connected && not_already_sent
        } else {
            false
        }
    };

    if should_send_mesh_ready {
        state.lock().await.log.push("Auto-triggering mesh ready: All session responses received and WebRTC connections established".to_string());
        if let Err(e) = internal_cmd_tx.send(InternalCommand::SendOwnMeshReadySignal) {
            state
                .lock()
                .await
                .log
                .push(format!("Failed to auto-send mesh ready signal: {}", e));
        }
    }
}

/// Handles proposing a new session
pub async fn handle_propose_session<C>(
    session_id: String,
    total: u16,
    threshold: u16,
    participants: Vec<String>,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    self_device_id: String,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar:
        Send + Sync,
{
    let state_clone = state.clone();
    let internal_cmd_tx_clone = internal_cmd_tx.clone();
    let self_device_id_clone = self_device_id.clone();

    tokio::spawn(async move {
        let mut state_guard = state_clone.lock().await;

        // Auto-detect session type based on wallet name
        let keystore_arc = state_guard.keystore.clone();
        let session_type = if let Some(keystore) = keystore_arc {
            // Check if a wallet with this name exists
            let wallets = keystore.list_wallets();
            if let Some(wallet) = wallets.iter().find(|w| w.session_id == session_id) {
                // Wallet exists - validate parameters
                if wallet.total_participants != total {
                    state_guard.log.push(format!(
                        "❌ Cannot proceed: Parameter mismatch",
                    ));
                    state_guard.log.push(format!(
                        "Wallet '{}' requires {} participants (you specified: {})",
                        session_id, wallet.total_participants, total
                    ));
                    state_guard.log.push(format!(
                        "Correct usage: /propose {} {} {} <devices>",
                        session_id, wallet.total_participants, wallet.threshold
                    ));
                    return;
                }
                if wallet.threshold != threshold {
                    state_guard.log.push(format!(
                        "❌ Cannot proceed: Threshold mismatch",
                    ));
                    state_guard.log.push(format!(
                        "Wallet '{}' has threshold {} (you specified: {})",
                        session_id, wallet.threshold, threshold
                    ));
                    state_guard.log.push(format!(
                        "Correct usage: /propose {} {} {} <devices>",
                        session_id, wallet.total_participants, wallet.threshold
                    ));
                    return;
                }
                
                state_guard.log.push(format!(
                    "Found wallet '{}' ({}/{}, {})",
                    wallet.session_id, wallet.threshold, wallet.total_participants, wallet.curve_type
                ));
                state_guard.log.push("Starting signing session...".to_string());
                
                // Load wallet cryptographic materials for signing session
                let device_id = state_guard.device_id.clone();
                match keystore.load_wallet_file(&wallet.session_id, &device_id) {
                    Ok(wallet_data) => {
                        // Parse the wallet data JSON
                        match std::str::from_utf8(&wallet_data) {
                            Ok(wallet_json) => {
                                match serde_json::from_str::<serde_json::Value>(wallet_json) {
                                    Ok(wallet_obj) => {
                                        // Extract key_package and group_public_key
                                        if let (Some(key_package_str), Some(group_public_key_str)) = (
                                            wallet_obj.get("key_package").and_then(|v| v.as_str()),
                                            wallet_obj.get("group_public_key").and_then(|v| v.as_str())
                                        ) {
                                            // Deserialize the cryptographic materials
                                            match (
                                                serde_json::from_str::<frost_core::keys::KeyPackage<C>>(key_package_str),
                                                serde_json::from_str::<frost_core::keys::PublicKeyPackage<C>>(group_public_key_str)
                                            ) {
                                                (Ok(key_package), Ok(group_public_key)) => {
                                                    // Store the cryptographic materials in AppState
                                                    state_guard.key_package = Some(key_package);
                                                    state_guard.group_public_key = Some(group_public_key.clone());
                                                    
                                                    // Set DKG state to Complete since wallet already exists
                                                    state_guard.dkg_state = crate::utils::state::DkgState::Complete;
                                                    
                                                    // Generate blockchain addresses from the group public key
                                                    crate::protocal::dkg::generate_public_key_addresses(&mut state_guard, &group_public_key);
                                                    
                                                    state_guard.log.push("✅ Wallet cryptographic materials loaded successfully".to_string());
                                                    state_guard.log.push("🔓 DKG state set to Complete - ready for signing".to_string());
                                                },
                                                (Err(e1), _) => {
                                                    state_guard.log.push(format!("❌ Failed to deserialize key_package: {}", e1));
                                                },
                                                (_, Err(e2)) => {
                                                    state_guard.log.push(format!("❌ Failed to deserialize group_public_key: {}", e2));
                                                }
                                            }
                                        } else {
                                            state_guard.log.push("❌ Missing key_package or group_public_key in wallet data".to_string());
                                        }
                                    },
                                    Err(e) => {
                                        state_guard.log.push(format!("❌ Failed to parse wallet JSON: {}", e));
                                    }
                                }
                            },
                            Err(e) => {
                                state_guard.log.push(format!("❌ Failed to decode wallet data as UTF-8: {}", e));
                            }
                        }
                    },
                    Err(e) => {
                        state_guard.log.push(format!("❌ Failed to load wallet file: {}", e));
                    }
                }
                
                // Get primary blockchain from WalletMetadata
                let blockchain = if !wallet.blockchains.is_empty() {
                    wallet.blockchains.iter()
                        .find(|b| b.enabled)
                        .or_else(|| wallet.blockchains.first())
                        .map(|b| b.blockchain.clone())
                        .unwrap_or_else(|| "unknown".to_string())
                } else {
                    wallet.blockchain.clone().unwrap_or_else(|| "unknown".to_string())
                };
                    
                SessionType::Signing {
                    wallet_name: wallet.session_id.clone(),
                    curve_type: wallet.curve_type.clone(),
                    blockchain,
                    group_public_key: wallet.group_public_key.clone(),
                }
            } else {
                // No wallet found - create DKG session
                state_guard.log.push(format!(
                    "No wallet '{}' found.",
                    session_id
                ));
                state_guard.log.push(format!(
                    "Starting DKG to create new {}-of-{} wallet...",
                    threshold, total
                ));
                SessionType::DKG
            }
        } else {
            // No keystore - must be DKG
            state_guard.log.push("No keystore initialized. Starting DKG session...".to_string());
            SessionType::DKG
        };

        let session_proposal = SessionProposal {
            session_id: session_id.clone(),
            total,
            threshold,
            participants: participants.clone(),
            session_type: session_type.clone(),
        };

        state_guard.log.push(format!(
            "Proposing session '{}' with {} participants and threshold {}",
            session_id, total, threshold
        ));

        let current_device_id = state_guard.device_id.clone();
        state_guard.session = Some(SessionInfo {
            session_id: session_id.clone(),
            proposer_id: current_device_id.clone(),
            total,
            threshold,
            participants: participants.clone(),
            accepted_devices: vec![current_device_id.clone()],
            session_type: session_type.clone(),
        });

        let mut map_created_and_check_dkg = false;
        if participants.len() == 1 && participants.contains(&current_device_id) {
            if state_guard.identifier_map.is_none() {
                let mut participants_sorted = participants.clone();
                participants_sorted.sort();

                let mut new_identifier_map = BTreeMap::new();
                for (i, device_id_str) in participants_sorted.iter().enumerate() {
                    match Identifier::try_from((i + 1) as u16) {
                        Ok(identifier) => {
                            new_identifier_map.insert(device_id_str.clone(), identifier);
                        }
                        Err(e) => {
                            state_guard.log.push(format!(
                                "Error creating identifier for {}: {}",
                                device_id_str, e
                            ));
                            state_guard.session = None;
                            state_guard.identifier_map = None;
                            return;
                        }
                    }
                }
                state_guard.log.push(format!(
                    "Identifier map created for single-participant session: {:?}",
                    new_identifier_map
                ));
                state_guard.identifier_map = Some(new_identifier_map);
                map_created_and_check_dkg = true;
            }
        }

        let local_device_id_for_filter = state_guard.device_id.clone();
        drop(state_guard);

        if map_created_and_check_dkg {
            if let Err(e) = internal_cmd_tx_clone.send(InternalCommand::CheckAndTriggerDkg) {
                state_clone.lock().await.log.push(format!(
                    "Failed to send CheckAndTriggerDkg after proposing single-user session: {}",
                    e
                ));
            }
        }

        let mut state_guard_for_broadcast = state_clone.lock().await;
        for device in participants
            .iter()
            .filter(|p| **p != local_device_id_for_filter)
        {
            let proposal_msg = WebSocketMessage::SessionProposal(session_proposal.clone());
            match serde_json::to_value(proposal_msg) {
                Ok(json_val) => {
                    let relay_msg = ClientMsg::Relay {
                        to: device.clone(),
                        data: json_val,
                    };
                    if let Err(e) =
                        internal_cmd_tx_clone.send(InternalCommand::SendToServer(relay_msg))
                    {
                        state_guard_for_broadcast.log.push(format!(
                            "Failed to send session proposal to {}: {}",
                            device, e
                        ));
                    } else {
                        state_guard_for_broadcast
                            .log
                            .push(format!("Sent session proposal to {}", device));
                    }
                }
                Err(e) => {
                    state_guard_for_broadcast.log.push(format!(
                        "Error serializing session proposal for {}: {}",
                        device, e
                    ));
                }
            }
        }
        drop(state_guard_for_broadcast);
        initiate_webrtc_connections(
            participants,
            self_device_id_clone,
            state_clone.clone(),
            internal_cmd_tx_clone.clone(),
        )
        .await;

        // Check for auto mesh ready after WebRTC initiation
        check_and_auto_mesh_ready(state_clone, internal_cmd_tx_clone).await;
    });
}

/// Handles accepting a session proposal
pub async fn handle_accept_session_proposal<C>(
    session_id: String,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar:
        Send + Sync,
{
    let state_clone = state.clone();
    let internal_cmd_tx_clone = internal_cmd_tx.clone();

    tokio::spawn(async move {
        let (participants_for_webrtc, device_id_for_webrtc, other_participants, wallet_status) = {
            let mut state_guard = state_clone.lock().await;
            let current_device_id = state_guard.device_id.clone();

            if let Some(invite_index) = state_guard
                .invites
                .iter()
                .position(|invite| invite.session_id == session_id)
            {
                let invite = state_guard.invites.remove(invite_index);

                // Check if this is a signing session and validate wallet
                let mut wallet_status = None;
                match &invite.session_type {
                    crate::protocal::signal::SessionType::Signing { wallet_name, curve_type: _, blockchain: _, group_public_key: _ } => {
                        state_guard.log.push(format!(
                            "Signing session request for wallet: {}",
                            wallet_name
                        ));
                        
                        // Check if we have the wallet
                        let has_wallet = if let Some(keystore) = &state_guard.keystore {
                            keystore.list_wallets().iter().any(|w| &w.session_id == wallet_name)
                        } else {
                            false
                        };
                        
                        if !has_wallet {
                            state_guard.log.push(format!(
                                "⚠️ Wallet '{}' not found in local keystore",
                                wallet_name
                            ));
                            state_guard.log.push("Options:".to_string());
                            state_guard.log.push("[1] Request wallet from participants (not implemented yet)".to_string());
                            state_guard.log.push("[2] Import wallet from backup (use /import_wallet)".to_string());
                            state_guard.log.push("[3] Join as observer (not implemented yet)".to_string());
                            
                            // For now, we'll still accept but mark wallet as missing
                            wallet_status = Some(crate::protocal::signal::WalletStatus {
                                has_wallet: false,
                                wallet_valid: false,
                                identifier: None,
                                error_reason: Some("Wallet not found".to_string()),
                            });
                        } else {
                            state_guard.log.push("✓ Wallet found in keystore".to_string());
                            
                            // Load wallet cryptographic materials for signing session
                            if let Some(keystore) = &state_guard.keystore {
                                let device_id = state_guard.device_id.clone();
                                match keystore.load_wallet_file(wallet_name, &device_id) {
                                    Ok(wallet_data) => {
                                        // Parse the wallet data JSON
                                        match std::str::from_utf8(&wallet_data) {
                                            Ok(wallet_json) => {
                                                match serde_json::from_str::<serde_json::Value>(wallet_json) {
                                                    Ok(wallet_obj) => {
                                                        // Extract key_package and group_public_key
                                                        if let (Some(key_package_str), Some(group_public_key_str)) = (
                                                            wallet_obj.get("key_package").and_then(|v| v.as_str()),
                                                            wallet_obj.get("group_public_key").and_then(|v| v.as_str())
                                                        ) {
                                                            // Deserialize the cryptographic materials
                                                            match (
                                                                serde_json::from_str::<frost_core::keys::KeyPackage<C>>(key_package_str),
                                                                serde_json::from_str::<frost_core::keys::PublicKeyPackage<C>>(group_public_key_str)
                                                            ) {
                                                                (Ok(key_package), Ok(group_public_key)) => {
                                                                    // Store the cryptographic materials in AppState
                                                                    state_guard.key_package = Some(key_package);
                                                                    state_guard.group_public_key = Some(group_public_key.clone());
                                                                    
                                                                    // Set DKG state to Complete since wallet already exists
                                                                    state_guard.dkg_state = crate::utils::state::DkgState::Complete;
                                                                    
                                                                    // Generate blockchain addresses from the group public key
                                                                    crate::protocal::dkg::generate_public_key_addresses(&mut state_guard, &group_public_key);
                                                                    
                                                                    state_guard.log.push("✅ Wallet cryptographic materials loaded successfully".to_string());
                                                                    state_guard.log.push("🔓 DKG state set to Complete - ready for signing".to_string());
                                                                },
                                                                (Err(e1), _) => {
                                                                    state_guard.log.push(format!("❌ Failed to deserialize key_package: {}", e1));
                                                                },
                                                                (_, Err(e2)) => {
                                                                    state_guard.log.push(format!("❌ Failed to deserialize group_public_key: {}", e2));
                                                                }
                                                            }
                                                        } else {
                                                            state_guard.log.push("❌ Missing key_package or group_public_key in wallet data".to_string());
                                                        }
                                                    },
                                                    Err(e) => {
                                                        state_guard.log.push(format!("❌ Failed to parse wallet JSON: {}", e));
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                state_guard.log.push(format!("❌ Failed to decode wallet data as UTF-8: {}", e));
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        state_guard.log.push(format!("❌ Failed to load wallet file: {}", e));
                                    }
                                }
                            }
                            
                            wallet_status = Some(crate::protocal::signal::WalletStatus {
                                has_wallet: true,
                                wallet_valid: true,
                                identifier: Some(1), // TODO: Get actual identifier from wallet
                                error_reason: None,
                            });
                        }
                    }
                    crate::protocal::signal::SessionType::DKG => {
                        state_guard.log.push("DKG session - no wallet required".to_string());
                    }
                }
                
                state_guard
                    .log
                    .push(format!("You accepted session proposal '{}'", session_id));

                // Set up the session - include self in accepted_devices and any early responses
                let mut accepted_devices = vec![current_device_id.clone()];

                // Always include the proposer as they accepted when they proposed
                if !accepted_devices.contains(&invite.proposer_id) {
                    accepted_devices.push(invite.proposer_id.clone());
                }

                // Add any early accepted devices from the invite
                for device in &invite.accepted_devices {
                    if !accepted_devices.contains(device) {
                        accepted_devices.push(device.clone());
                    }
                }

                state_guard.log.push(format!(
                    "Session setup: accepted_devices after including proposer and early responses: {:?}",
                    accepted_devices
                ));

                let session_info = SessionInfo {
                    session_id: invite.session_id.clone(),
                    proposer_id: invite.proposer_id.clone(),
                    total: invite.total,
                    threshold: invite.threshold,
                    participants: invite.participants.clone(),
                    accepted_devices,
                    session_type: invite.session_type.clone(),
                };
                state_guard.session = Some(session_info);

                // Create identifier map immediately when accepting session
                if state_guard.identifier_map.is_none() {
                    let mut participants_sorted = invite.participants.clone();
                    participants_sorted.sort();
                    let mut new_identifier_map = BTreeMap::new();

                    let mut creation_successful = true;
                    for (i, device_id_str) in participants_sorted.iter().enumerate() {
                        match Identifier::try_from((i + 1) as u16) {
                            Ok(identifier) => {
                                new_identifier_map.insert(device_id_str.clone(), identifier);
                            }
                            Err(e) => {
                                state_guard.log.push(format!(
                                    "Error creating identifier for {}: {}",
                                    device_id_str, e
                                ));
                                state_guard.session = None;
                                creation_successful = false;
                                break;
                            }
                        }
                    }

                    if creation_successful {
                        state_guard.log.push(format!(
                            "Identifier map created (on accept): {:?}",
                            new_identifier_map
                        ));
                        state_guard.identifier_map = Some(new_identifier_map);

                        // Schedule CheckAndTriggerDkg after identifier map creation
                        let internal_cmd_tx_for_dkg = internal_cmd_tx_clone.clone();
                        let state_for_error_log = state_clone.clone();
                        tokio::spawn(async move {
                            if let Err(e) =
                                internal_cmd_tx_for_dkg.send(InternalCommand::CheckAndTriggerDkg)
                            {
                                state_for_error_log.lock().await.log.push(format!(
                                    "Failed to send CheckAndTriggerDkg after accepting session: {}",
                                    e
                                ));
                            }
                        });
                    }
                }

                // Get list of other participants to broadcast to
                let other_participants: Vec<String> = invite
                    .participants
                    .iter()
                    .filter(|p| **p != current_device_id)
                    .cloned()
                    .collect();

                (invite.participants, current_device_id, other_participants, wallet_status)
            } else {
                state_guard.log.push(format!(
                    "No pending invite found for session '{}'",
                    session_id
                ));
                (Vec::new(), String::new(), Vec::new(), None)
            }
        };

        // Broadcast SessionResponse to all other participants
        if !other_participants.is_empty() {
            let response = SessionResponse {
                session_id: session_id.clone(),
                accepted: true,
                wallet_status: wallet_status.clone(),
            };

            for device in other_participants {
                let websocket_message = WebSocketMessage::SessionResponse(response.clone());
                match serde_json::to_value(websocket_message) {
                    Ok(json_val) => {
                        let relay_msg = ClientMsg::Relay {
                            to: device.clone(),
                            data: json_val,
                        };
                        if let Err(e) =
                            internal_cmd_tx_clone.send(InternalCommand::SendToServer(relay_msg))
                        {
                            state_clone.lock().await.log.push(format!(
                                "Failed to send session response to {}: {}",
                                device, e
                            ));
                        } else {
                            state_clone
                                .lock()
                                .await
                                .log
                                .push(format!("Sent session acceptance response to {}", device));
                        }
                    }
                    Err(e) => {
                        state_clone.lock().await.log.push(format!(
                            "Failed to serialize session response for {}: {}",
                            device, e
                        ));
                    }
                }
            }
        }

        if !participants_for_webrtc.is_empty() {
            // Begin WebRTC connection process
            initiate_webrtc_connections(
                participants_for_webrtc,
                device_id_for_webrtc,
                state_clone.clone(),
                internal_cmd_tx_clone.clone(),
            )
            .await;

            // Check for auto mesh ready after WebRTC initiation
            check_and_auto_mesh_ready(state_clone, internal_cmd_tx_clone).await;
        }
    });
}

/// Handles processing a session response
pub async fn handle_process_session_response<C>(
    from_device_id: String,
    response: SessionResponse,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar:
        Send + Sync,
{
    let state_clone = state.clone();
    let internal_cmd_tx_clone = internal_cmd_tx.clone();

    tokio::spawn(async move {
        let mut log_msgs = Vec::new();
        let mut session_cancelled = false;
        let mut session_id_to_cancel = None;
        let mut map_created_and_check_dkg = false;
        let mut participants_for_map_creation: Option<Vec<String>> = None;

        {
            let mut state_guard = state_clone.lock().await;
            log_msgs.push(format!(
                "Received session response from {}: Accepted={}",
                from_device_id, response.accepted
            ));

            if response.accepted {
                let mut handled_in_active_session = false;
                if let Some(session) = state_guard.session.as_mut() {
                    if session.session_id == response.session_id {
                        handled_in_active_session = true;
                        if !session.accepted_devices.contains(&from_device_id) {
                            session.accepted_devices.push(from_device_id.clone());
                            log_msgs.push(format!(
                                "Device {} accepted session '{}'. Active session accepted devices: {}/{}",
                                from_device_id, session.session_id, session.accepted_devices.len(), session.participants.len()
                            ));
                            if session.accepted_devices.len() == session.participants.len() {
                                log_msgs.push(format!(
                                    "All {} participants accepted session '{}' (active session). Preparing identifier map.",
                                    session.participants.len(), session.session_id
                                ));
                                participants_for_map_creation = Some(session.participants.clone());
                            }
                        } else {
                            log_msgs.push(format!(
                                "Device {} already recorded in active session '{}' accepted_devices.",
                                from_device_id, session.session_id
                            ));
                        }
                    }
                }

                // Check pending invites if not handled in active session
                if !handled_in_active_session {
                    if let Some(invite_to_update) = state_guard
                        .invites
                        .iter_mut()
                        .find(|i| i.session_id == response.session_id)
                    {
                        if !invite_to_update.accepted_devices.contains(&from_device_id) {
                            invite_to_update
                                .accepted_devices
                                .push(from_device_id.clone());
                            log_msgs.push(format!(
                                "Recorded early acceptance from {} for pending invite '{}'. Invite accepted_devices count: {}.",
                                from_device_id, invite_to_update.session_id, invite_to_update.accepted_devices.len()
                            ));
                        } else {
                            log_msgs.push(format!(
                                "Device {} already recorded in pending invite '{}' accepted_devices.",
                                from_device_id, invite_to_update.session_id
                            ));
                        }
                    } else {
                        log_msgs.push(format!(
                            "No active session or pending invite found for session ID '{}' from device {}.",
                            response.session_id, from_device_id
                        ));
                    }
                }

                // Create identifier map if all participants have accepted
                if let Some(participants_list) = participants_for_map_creation {
                    if state_guard.identifier_map.is_none() {
                        let mut participants_sorted = participants_list;
                        participants_sorted.sort();

                        let mut new_identifier_map = BTreeMap::new();
                        for (i, device_id_str) in participants_sorted.iter().enumerate() {
                            match Identifier::try_from((i + 1) as u16) {
                                Ok(identifier) => {
                                    new_identifier_map.insert(device_id_str.clone(), identifier);
                                }
                                Err(e) => {
                                    log_msgs.push(format!(
                                        "Error creating identifier for {}: {}",
                                        device_id_str, e
                                    ));
                                    state_guard.session = None;
                                    state_guard.identifier_map = None;
                                    // Push logs and return
                                    for msg_item in log_msgs {
                                        state_guard.log.push(msg_item);
                                    }
                                    return;
                                }
                            }
                        }
                        log_msgs.push(format!("Identifier map created: {:?}", new_identifier_map));
                        state_guard.identifier_map = Some(new_identifier_map);
                        map_created_and_check_dkg = true;
                    }

                    // Process any buffered mesh ready signals now that all session responses are received
                    let buffered_signals =
                        std::mem::take(&mut state_guard.pending_mesh_ready_signals);
                    if !buffered_signals.is_empty() {
                        log_msgs.push(format!(
                            "Processing {} buffered mesh ready signals now that all session responses are received",
                            buffered_signals.len()
                        ));
                        drop(state_guard); // Drop the lock before sending commands

                        for buffered_device_id in buffered_signals {
                            log_msgs.push(format!(
                                "Processing buffered mesh ready from device: {}",
                                buffered_device_id
                            ));
                            if let Err(e) =
                                internal_cmd_tx_clone.send(InternalCommand::ProcessMeshReady {
                                    device_id: buffered_device_id.clone(),
                                })
                            {
                                log_msgs.push(format!("Failed to send ProcessMeshReady for buffered signal from {}: {}", buffered_device_id, e));
                            }
                        }

                        // Note: We don't need to reacquire the lock here since we'll handle
                        // the rest of the processing outside the lock scope
                    } else {
                        drop(state_guard); // Drop the lock even if no buffered signals
                    }
                } else {
                    drop(state_guard); // Drop the lock if no participants_for_map_creation
                }
            } else {
                // Handle rejection
                log_msgs.push(format!(
                    "Device {} rejected session '{}'.",
                    from_device_id, response.session_id
                ));
                if let Some(session) = &state_guard.session {
                    if session.session_id == response.session_id {
                        session_cancelled = true;
                        session_id_to_cancel = Some(response.session_id.clone());
                    }
                }
            }
        }

        // Trigger DKG if needed
        if map_created_and_check_dkg {
            if let Err(e) = internal_cmd_tx_clone.send(InternalCommand::CheckAndTriggerDkg) {
                log_msgs.push(format!("Failed to send CheckAndTriggerDkg command: {}", e));
            }
        }

        // Cancel session if rejected
        if session_cancelled {
            let mut guard = state_clone.lock().await;
            guard.session = None;
            guard.identifier_map = None;
            if let Some(sid) = session_id_to_cancel {
                log_msgs.push(format!(
                    "Session '{}' cancelled due to rejection by {}.",
                    sid, from_device_id
                ));
            }
        }

        // Log all messages
        let mut guard = state_clone.lock().await;
        for msg in log_msgs {
            guard.log.push(msg);
        }
        drop(guard);

        // Check for auto mesh ready after processing session response
        check_and_auto_mesh_ready(state_clone, internal_cmd_tx_clone).await;
    });
}
