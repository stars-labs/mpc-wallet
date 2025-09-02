// STUB: DKG command handlers - temporarily stubbed for deployment
use crate::utils::appstate_compat::AppState;
use crate::utils::state::{InternalCommand, DkgState};
use frost_core::Ciphersuite;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// STUB: Handles checking conditions and triggering DKG if appropriate
pub async fn handle_check_and_trigger_dkg<C>(
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut guard = state.lock().await;
    guard.dkg_state = DkgState::Failed("DKG trigger check is temporarily stubbed for deployment".to_string());
    tracing::warn!("DKG trigger check is stubbed");
}

/// STUB: Handle DKG Round 1 trigger
pub async fn handle_trigger_dkg_round1<C>(
    state: Arc<Mutex<AppState<C>>>,
    device_id: String,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut guard = state.lock().await;
    guard.dkg_state = DkgState::Failed("DKG Round 1 trigger is temporarily stubbed for deployment".to_string());
    tracing::warn!("DKG Round 1 trigger is stubbed - device_id: {}", device_id);
}

/// STUB: Handle DKG Round 2 trigger
pub async fn handle_trigger_dkg_round2<C>(
    state: Arc<Mutex<AppState<C>>>,
    device_id: String,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut guard = state.lock().await;
    guard.dkg_state = DkgState::Failed("DKG Round 2 trigger is temporarily stubbed for deployment".to_string());
    tracing::warn!("DKG Round 2 trigger is stubbed - device_id: {}", device_id);
}

/// STUB: Handle DKG finalization
pub async fn handle_dkg_finalize<C>(
    state: Arc<Mutex<AppState<C>>>,
    device_id: String,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut guard = state.lock().await;
    guard.dkg_state = DkgState::Failed("DKG finalization is temporarily stubbed for deployment".to_string());
    tracing::warn!("DKG finalization is stubbed - device_id: {}", device_id);
}

/// STUB: Handle DKG Round 1 processing
pub async fn handle_process_dkg_round1<C>(
    from_device_id: String,
    _package: frost_core::keys::dkg::round1::Package<C>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut guard = state.lock().await;
    guard.dkg_state = DkgState::Failed("DKG Round 1 processing is temporarily stubbed for deployment".to_string());
    tracing::warn!("DKG Round 1 processing is stubbed - from: {}", from_device_id);
}

/// STUB: Handle DKG Round 2 processing
pub async fn handle_process_dkg_round2<C>(
    from_device_id: String,
    _package: frost_core::keys::dkg::round2::Package<C>,
    state: Arc<Mutex<AppState<C>>>,
    _internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut guard = state.lock().await;
    guard.dkg_state = DkgState::Failed("DKG Round 2 processing is temporarily stubbed for deployment".to_string());
    tracing::warn!("DKG Round 2 processing is stubbed - from: {}", from_device_id);
}

/// STUB: Handle finalize DKG
pub async fn handle_finalize_dkg<C>(
    state: Arc<Mutex<AppState<C>>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let mut guard = state.lock().await;
    guard.dkg_state = DkgState::Failed("DKG finalization is temporarily stubbed for deployment".to_string());
    tracing::warn!("DKG finalization is stubbed");
}