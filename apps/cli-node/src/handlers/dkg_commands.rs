use crate::utils::state::{AppState, InternalCommand, DkgState, MeshStatus};
use crate::protocal::dkg;
use frost_core::Ciphersuite;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// Handles checking conditions and triggering DKG if appropriate
pub async fn handle_check_and_trigger_dkg<C>(
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
        let device_id_for_log = guard.device_id.clone();

        let mesh_ready = guard.mesh_status == MeshStatus::Ready;
        let map_exists = guard.identifier_map.is_some();
        let session_active = guard.session.is_some();
        let dkg_idle = guard.dkg_state == DkgState::Idle;
        
        // Check if this is a DKG session
        let is_dkg_session = if let Some(session) = &guard.session {
            matches!(session.session_type, crate::protocal::signal::SessionType::DKG)
        } else {
            false
        };

        if mesh_ready && map_exists && session_active && dkg_idle && is_dkg_session {
            guard.log.push(format!(
                "[CheckAndTriggerDkg-{}] All conditions met. Triggering DKG Round 1.",
                device_id_for_log
            ));
            if internal_cmd_tx_clone.send(InternalCommand::TriggerDkgRound1).is_ok() {
                guard.dkg_state = DkgState::Round1InProgress; 
            } else {
                guard.log.push(format!(
                    "[CheckAndTriggerDkg-{}] Failed to send TriggerDkgRound1 command.",
                     device_id_for_log
                ));
            }
        } else {
            guard.log.push(format!(
                "[CheckAndTriggerDkg-{}] Conditions not met. MeshReady: {}, IdentifiersMapped: {}, SessionActive: {}, DkgIdle: {}, IsDkgSession: {}",
                device_id_for_log,
                mesh_ready,
                map_exists,
                session_active,
                dkg_idle,
                is_dkg_session
            ));
        }
    });
}

/// Handles triggering DKG Round 1
pub async fn handle_trigger_dkg_round1<C>(
    state: Arc<Mutex<AppState<C>>>,
    self_device_id: String,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    let self_device_id_clone = self_device_id.clone();
    
    tokio::spawn(async move {
        state_clone.lock().await.log.push(
            "DKG Round 1: Generating and sending commitments to all devices...".to_string(),
        );
        dkg::handle_trigger_dkg_round1(state_clone, self_device_id_clone).await;
    });
}

/// Handles triggering DKG Round 2
pub async fn handle_trigger_dkg_round2<C>(
    state: Arc<Mutex<AppState<C>>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        match dkg::handle_trigger_dkg_round2(state_clone.clone()).await {
            Ok(_) => {
                state_clone.lock().await.log.push("Successfully completed handle_trigger_dkg_round2".to_string());
            },
            Err(e) => {
                state_clone.lock().await.log.push(format!(
                    "Error in handle_trigger_dkg_round2: {}", e
                ));
            }
        }
    });
}

/// Handles processing a DKG Round 1 package from a device
pub async fn handle_process_dkg_round1<C>(
    from_device_id: String,
    package: frost_core::keys::dkg::round1::Package<C>,
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
        guard.log.push(format!(
            "Processing DKG Round 1 package from {}",
            from_device_id
        ));

        let current_dkg_state = guard.dkg_state.clone();
        if current_dkg_state != DkgState::Round1InProgress {
            guard.log.push(format!(
                "Error: Received DKG Round 1 package from {} but DKG state is {:?}, not Round1InProgress.",
                from_device_id, current_dkg_state
            ));
            return;
        }

        drop(guard);

        dkg::process_dkg_round1(
            state_clone.clone(), 
            from_device_id.clone(),
            package,
        )
        .await;

        let mut guard = state_clone.lock().await;
        let all_packages_received = if let Some(session) = &guard.session {
            guard.received_dkg_packages.len() == session.participants.len()
        } else {
            false
        };

        if all_packages_received {
            guard.log.push(
                "All DKG Round 1 packages received. Setting state to Round1Complete and triggering DKG Round 2."
                    .to_string(),
            );
            guard.dkg_state = DkgState::Round1Complete;
            drop(guard); 
            if let Err(e) = internal_cmd_tx_clone.send(InternalCommand::TriggerDkgRound2) {
                state_clone.lock().await.log.push(format!(
                    "Failed to send TriggerDkgRound2 command: {}",
                    e
                ));
            }
        } else {
            guard.log.push(format!(
                "DKG Round 1: After processing package from {}, still waiting for more packages.",
                from_device_id
            ));
        }
    });
}

/// Handles processing a DKG Round 2 package from a device
pub async fn handle_process_dkg_round2<C>(
    from_device_id: String,
    package: frost_core::keys::dkg::round2::Package<C>,
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
        guard.log.push(format!(
            "Processing DKG Round 2 package from {}",
            from_device_id
        ));
        drop(guard);

        match dkg::process_dkg_round2(
            state_clone.clone(), 
            from_device_id.clone(),
            package,
        ).await {
            Ok(_) => {
                state_clone.lock().await.log.push(format!(
                    "DKG Round 2: Successfully processed package from {}",
                    from_device_id
                ));
            },
            Err(e) => {
                state_clone.lock().await.log.push(format!(
                    "DKG Round 2: Error processing package from {}: {}",
                    from_device_id, e
                ));
                return;
            }
        }

        let mut guard = state_clone.lock().await;
        
        let package_count = guard.received_dkg_round2_packages.len();
        let package_keys = guard.received_dkg_round2_packages.keys().collect::<Vec<_>>();
        let self_identifier = guard.identifier_map.as_ref()
            .and_then(|map| map.get(&guard.device_id)).cloned();
        
        let log_message = format!(
            "DKG Round 2: Current received_dkg_round2_packages count: {}, keys: {:?}, self identifier: {:?}",
            package_count,
            package_keys,
            self_identifier
        );
        guard.log.push(log_message);
        
        let own_package_counted = if let Some(self_id) = self_identifier {
            guard.received_dkg_round2_packages.contains_key(&self_id)
        } else {
            false
        };
        
        guard.log.push(format!(
            "DKG Round 2: Own package in received map: {}", own_package_counted
        ));
        
        let all_packages_received = if let Some(session) = &guard.session {
            let current_count = guard.received_dkg_round2_packages.len();
            let expected_count = session.participants.len() - 1;
            let result = current_count == expected_count;
            
            guard.log.push(format!(
                "DKG Round 2: Checking completion: {}/{} packages received",
                current_count, expected_count
            ));
            
            result
        } else {
            guard.log.push("DKG Round 2: No active session found when checking for completion".to_string());
            false
        };

        if all_packages_received {
            guard.log.push(
                "All DKG Round 2 packages received. Setting state to Round2Complete and triggering FinalizeDkg."
                    .to_string(),
            );
            guard.dkg_state = DkgState::Round2Complete;
            drop(guard); 
            
            state_clone.lock().await.log.push("Sending FinalizeDkg command now...".to_string());
            
            if let Err(e) = internal_cmd_tx_clone.send(InternalCommand::FinalizeDkg) {
                state_clone.lock().await.log.push(format!(
                    "Failed to send FinalizeDkg command: {}",
                    e
                ));
            } else {
                state_clone.lock().await.log.push("Successfully sent FinalizeDkg command".to_string());
            }
        } else {
            guard.log.push(format!(
                "DKG Round 2: After processing package from {}, still waiting for more packages.",
                from_device_id
            ));
        }
    });
}

/// Handles finalizing the DKG process
pub async fn handle_finalize_dkg<C>(
    state: Arc<Mutex<AppState<C>>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        let mut guard = state_clone.lock().await;
        guard.log.push("FinalizeDkg: Processing command.".to_string());

        let current_dkg_state = guard.dkg_state.clone();
        if current_dkg_state != DkgState::Round2Complete {
            guard.log.push(format!(
                "Error: Triggered FinalizeDkg but DKG state is {:?}, not Round2Complete.",
                current_dkg_state
            ));
            return;
        }
        
        guard.dkg_state = DkgState::Finalizing;
        
        guard.log.push(format!(
            "FinalizeDkg: All prerequisites met. Current state: {:?}. Moving to Finalizing state.",
            current_dkg_state
        ));
        
        let package_count = guard.received_dkg_round2_packages.len();
        guard.log.push(format!(
            "FinalizeDkg: Preparing to finalize with {} round2 packages", 
            package_count
        ));
        
        drop(guard);

        state_clone.lock().await.log.push("FinalizeDkg: Calling ed25519_dkg::handle_finalize_dkg function...".to_string());

        dkg::handle_finalize_dkg(state_clone.clone()).await;
        
        state_clone.lock().await.log.push(
            "FinalizeDkg: Completed DKG finalization process".to_string()
        );

        let mut final_guard = state_clone.lock().await;
        let final_dkg_state = final_guard.dkg_state.clone();
        final_guard.log.push(format!(
            "FinalizeDkg: Completion attempt finished. DKG state is now: {:?}",
            final_dkg_state
        ));
    });
}
