// DKG command handlers - uses simplified DKG implementation
use crate::utils::appstate_compat::AppState;
use crate::utils::state::{InternalCommand, DkgState};
use crate::protocal::dkg_simple;
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
    let guard = state.lock().await;
    
    // Check if we have a session and enough participants
    if let Some(session) = &guard.session {
        if session.participants.len() >= 2 {
            let device_id = guard.device_id.clone();
            drop(guard);
            
            tracing::info!("Conditions met for DKG, triggering Round 1");
            // Trigger DKG Round 1
            let _ = internal_cmd_tx.send(InternalCommand::TriggerDkgRound1);
        } else {
            tracing::info!("Not enough participants for DKG yet");
        }
    } else {
        tracing::info!("No session available for DKG");
    }
}

/// Handle DKG Round 1 trigger - delegates to simplified implementation
pub async fn handle_trigger_dkg_round1<C>(
    state: Arc<Mutex<AppState<C>>>,
    device_id: String,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    tracing::info!("Starting DKG Round 1 for device: {}", device_id);
    
    // Use the simplified DKG implementation
    dkg_simple::handle_trigger_dkg_round1(state, device_id, internal_cmd_tx).await;
}

/// Handle DKG Round 2 trigger - delegates to simplified implementation
pub async fn handle_trigger_dkg_round2<C>(
    state: Arc<Mutex<AppState<C>>>,
    device_id: String,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    tracing::info!("Starting DKG Round 2 for device: {}", device_id);
    
    // Use the simplified DKG implementation
    dkg_simple::handle_trigger_dkg_round2(state, device_id).await;
}

/// Handle DKG finalization - delegates to simplified implementation
pub async fn handle_dkg_finalize<C>(
    state: Arc<Mutex<AppState<C>>>,
    device_id: String,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    tracing::info!("Finalizing DKG for device: {}", device_id);
    
    // Use the simplified DKG implementation
    dkg_simple::finalize_dkg(state, device_id).await;
}

/// Handle DKG Round 1 processing
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
    tracing::info!("Processing DKG Round 1 package from: {}", from_device_id);
    
    let mut guard = state.lock().await;
    
    // Store the round 1 package
    if let Ok(id_num) = from_device_id.parse::<u16>() {
        if let Ok(id) = frost_core::Identifier::<C>::try_from(id_num) {
            guard.dkg_round1_packages.insert(id, package);
            
            // Check if we have all round 1 packages
            if let Some(session) = &guard.session {
                if guard.dkg_round1_packages.len() == session.participants.len() {
                    tracing::info!("All Round 1 packages received, triggering Round 2");
                    let device_id = guard.device_id.clone();
                    drop(guard);
                    
                    // Trigger Round 2
                    let _ = internal_cmd_tx.send(InternalCommand::TriggerDkgRound2);
                }
            }
        } else {
            guard.dkg_state = DkgState::Failed(format!("Invalid participant identifier: {}", from_device_id));
        }
    } else {
        guard.dkg_state = DkgState::Failed(format!("Failed to parse device ID: {}", from_device_id));
    }
}

/// Handle DKG Round 2 processing
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
    tracing::info!("Processing DKG Round 2 package from: {}", from_device_id);
    
    let mut guard = state.lock().await;
    
    // Store the round 2 package
    if let Ok(id_num) = from_device_id.parse::<u16>() {
        if let Ok(id) = frost_core::Identifier::<C>::try_from(id_num) {
            guard.dkg_round2_packages.insert(id, package);
            
            // Check if we have all round 2 packages
            if let Some(session) = &guard.session {
                if guard.dkg_round2_packages.len() == session.participants.len() - 1 {
                    tracing::info!("All Round 2 packages received, finalizing DKG");
                    let device_id = guard.device_id.clone();
                    drop(guard);
                    
                    // Trigger finalization
                    let _ = internal_cmd_tx.send(InternalCommand::FinalizeDkg);
                }
            }
        } else {
            guard.dkg_state = DkgState::Failed(format!("Invalid participant identifier: {}", from_device_id));
        }
    } else {
        guard.dkg_state = DkgState::Failed(format!("Failed to parse device ID: {}", from_device_id));
    }
}

/// Handle finalize DKG
pub async fn handle_finalize_dkg<C>(
    state: Arc<Mutex<AppState<C>>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let guard = state.lock().await;
    let device_id = guard.device_id.clone();
    drop(guard);
    
    tracing::info!("Finalizing DKG process");
    
    // Delegate to the simplified implementation
    dkg_simple::finalize_dkg(state, device_id).await;
}