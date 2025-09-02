//! Wallet creation command handlers for the DKG wallet creation flow
//!
//! This module implements the backend logic for the complete wallet creation workflow
//! as documented in the UI wireframes, supporting online, offline, and hybrid modes.

use crate::utils::appstate_compat::AppState;
use crate::utils::state::InternalCommand;
use crate::handlers::session_handler::{
    WalletSessionConfig, WalletCreationProgress, WalletCreationStage, 
    BlockchainConfig, WalletCreationMode, handle_propose_wallet_session,
    handle_session_discovery, handle_progress_update
};
use frost_core::Ciphersuite;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::info;

/// Quick DKG session creation (2-of-3 standard wallet)
pub async fn handle_create_quick_wallet<C: Ciphersuite + Send + Sync + 'static>(
    wallet_name: String,
    curve_type: String, // "secp256k1" or "ed25519"
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    device_id: String,
) -> Result<String, String>
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    info!("Creating quick wallet: {} with curve: {}", wallet_name, curve_type);

    // Default blockchain configuration based on curve type
    let blockchain_config = match curve_type.as_str() {
        "secp256k1" => vec![
            BlockchainConfig {
                blockchain: "ethereum".to_string(),
                network: "mainnet".to_string(),
                enabled: true,
                chain_id: Some(1),
            },
            BlockchainConfig {
                blockchain: "bitcoin".to_string(),
                network: "mainnet".to_string(),
                enabled: false, // Disabled by default
                chain_id: None,
            },
        ],
        "ed25519" => vec![
            BlockchainConfig {
                blockchain: "solana".to_string(),
                network: "mainnet".to_string(),
                enabled: true,
                chain_id: None,
            },
        ],
        _ => return Err(format!("Unsupported curve type: {}", curve_type)),
    };

    let config = WalletSessionConfig {
        wallet_name: wallet_name.clone(),
        description: Some(format!("Quick 2-of-3 {} wallet", curve_type)),
        total: 3,
        threshold: 2,
        curve_type,
        mode: WalletCreationMode::Online,
        timeout_hours: 24,
        auto_discovery: true,
        blockchain_config,
    };

    // Update progress
    let progress = WalletCreationProgress {
        stage: WalletCreationStage::Configuration,
        current_step: 1,
        total_steps: 6,
        message: "Configuring wallet parameters".to_string(),
        details: Some(format!("Creating {}", wallet_name)),
    };
    handle_progress_update(app_state.clone(), progress).await;

    // Create the session
    handle_propose_wallet_session(config, app_state, internal_cmd_tx, device_id).await?;
    
    Ok(wallet_name)
}

/// Custom DKG setup with advanced configuration
pub async fn handle_create_custom_wallet<C: Ciphersuite + Send + Sync + 'static>(
    config: WalletSessionConfig,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    device_id: String,
) -> Result<String, String>
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    info!("Creating custom wallet: {} with {}/{} threshold", 
        config.wallet_name, config.threshold, config.total);

    // Validate configuration
    validate_wallet_config(&config)?;

    // Update progress
    let progress = WalletCreationProgress {
        stage: WalletCreationStage::Configuration,
        current_step: 1,
        total_steps: 8,
        message: "Validating custom configuration".to_string(),
        details: Some(format!("Advanced setup for {}", config.wallet_name)),
    };
    handle_progress_update(app_state.clone(), progress).await;

    // Create the session
    handle_propose_wallet_session(config.clone(), app_state, internal_cmd_tx, device_id).await?;
    
    Ok(config.wallet_name)
}

/// Multi-chain wallet creation
pub async fn handle_create_multichain_wallet<C: Ciphersuite + Send + Sync + 'static>(
    wallet_name: String,
    selected_blockchains: Vec<String>,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    device_id: String,
) -> Result<String, String>
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    info!("Creating multi-chain wallet: {} for blockchains: {:?}", 
        wallet_name, selected_blockchains);

    // Determine required curve types
    let (secp256k1_chains, ed25519_chains): (Vec<_>, Vec<_>) = selected_blockchains
        .iter()
        .partition(|chain| matches!(chain.as_str(), "ethereum" | "bitcoin" | "bsc" | "polygon"));

    if !secp256k1_chains.is_empty() && !ed25519_chains.is_empty() {
        return Err("Multi-curve wallets require separate DKG sessions for each curve type".to_string());
    }

    let curve_type = if !secp256k1_chains.is_empty() {
        "secp256k1".to_string()
    } else {
        "ed25519".to_string()
    };

    // Build blockchain configuration
    let blockchain_config = selected_blockchains.iter().map(|blockchain| {
        let (network, chain_id) = match blockchain.as_str() {
            "ethereum" => ("mainnet", Some(1)),
            "bsc" => ("mainnet", Some(56)),
            "polygon" => ("mainnet", Some(137)),
            "solana" => ("mainnet", None),
            "bitcoin" => ("mainnet", None),
            _ => ("mainnet", None),
        };

        BlockchainConfig {
            blockchain: blockchain.clone(),
            network: network.to_string(),
            enabled: true,
            chain_id,
        }
    }).collect();

    let config = WalletSessionConfig {
        wallet_name: wallet_name.clone(),
        description: Some(format!("Multi-chain {} wallet", curve_type)),
        total: 3,
        threshold: 2,
        curve_type,
        mode: WalletCreationMode::Online,
        timeout_hours: 24,
        auto_discovery: true,
        blockchain_config,
    };

    // Update progress
    let progress = WalletCreationProgress {
        stage: WalletCreationStage::Configuration,
        current_step: 1,
        total_steps: 6,
        message: "Configuring multi-chain support".to_string(),
        details: Some(format!("Chains: {}", selected_blockchains.join(", "))),
    };
    handle_progress_update(app_state.clone(), progress).await;

    // Create the session
    handle_propose_wallet_session(config, app_state, internal_cmd_tx, device_id).await?;
    
    Ok(wallet_name)
}

/// Offline DKG wallet creation
pub async fn handle_create_offline_wallet<C: Ciphersuite + Send + Sync + 'static>(
    config: WalletSessionConfig,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    device_id: String,
) -> Result<String, String>
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    info!("Creating offline wallet: {} with air-gapped security", config.wallet_name);

    // Ensure offline mode
    let mut offline_config = config.clone();
    offline_config.mode = WalletCreationMode::Offline;

    // Update progress
    let progress = WalletCreationProgress {
        stage: WalletCreationStage::Configuration,
        current_step: 1,
        total_steps: 8,
        message: "Preparing offline DKG package".to_string(),
        details: Some("Air-gapped security mode enabled".to_string()),
    };
    handle_progress_update(app_state.clone(), progress).await;

    // Create offline session
    handle_propose_wallet_session(offline_config.clone(), app_state, internal_cmd_tx, device_id).await?;
    
    Ok(offline_config.wallet_name)
}

/// Handle participant discovery for wallet creation
pub async fn handle_participant_discovery<C: Ciphersuite + Send + Sync + 'static>(
    session_id: String,
    required_participants: u16,
    app_state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) -> Result<Vec<String>, String>
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    info!("Starting participant discovery for session: {}", session_id);

    let state = app_state.lock().await;
    // Starting participant discovery
    
    // Update progress
    drop(state);
    let progress = WalletCreationProgress {
        stage: WalletCreationStage::ParticipantDiscovery,
        current_step: 2,
        total_steps: 6,
        message: format!("Searching for {} participants", required_participants),
        details: Some("Broadcasting session availability".to_string()),
    };
    handle_progress_update(app_state.clone(), progress).await;

    // Use existing session discovery mechanism
    let sessions = handle_session_discovery(app_state, internal_cmd_tx).await?;
    
    // Filter for our session and return participant list
    if let Some(our_session) = sessions.iter().find(|s| s.session_code == session_id) {
        Ok(vec![our_session.creator_device.clone()]) // Start with creator
    } else {
        Ok(vec![]) // No participants yet
    }
}

/// Handle DKG progress updates and state transitions
pub async fn handle_dkg_progress<C: Ciphersuite + Send + Sync + 'static>(
    app_state: Arc<Mutex<AppState<C>>>,
    dkg_state: crate::utils::state::DkgState,
) {
    let progress = match dkg_state {
        crate::utils::state::DkgState::Round1InProgress => WalletCreationProgress {
            stage: WalletCreationStage::DkgRound1,
            current_step: 4,
            total_steps: 6,
            message: "Generating cryptographic commitments".to_string(),
            details: Some("FROST Round 1 in progress".to_string()),
        },
        crate::utils::state::DkgState::Round2InProgress => WalletCreationProgress {
            stage: WalletCreationStage::DkgRound2,
            current_step: 5,
            total_steps: 6,
            message: "Distributing key shares".to_string(),
            details: Some("FROST Round 2 in progress".to_string()),
        },
        crate::utils::state::DkgState::Finalizing => WalletCreationProgress {
            stage: WalletCreationStage::Finalization,
            current_step: 6,
            total_steps: 6,
            message: "Finalizing wallet creation".to_string(),
            details: Some("Deriving addresses and saving wallet".to_string()),
        },
        crate::utils::state::DkgState::Complete => WalletCreationProgress {
            stage: WalletCreationStage::Complete,
            current_step: 6,
            total_steps: 6,
            message: "Wallet created successfully!".to_string(),
            details: Some("Ready for transactions".to_string()),
        },
        crate::utils::state::DkgState::Failed(ref reason) => WalletCreationProgress {
            stage: WalletCreationStage::Failed,
            current_step: 0,
            total_steps: 6,
            message: "Wallet creation failed".to_string(),
            details: Some(reason.clone()),
        },
        _ => return, // No progress update needed for other states
    };

    handle_progress_update(app_state, progress).await;
}

/// Validate wallet configuration before creation
fn validate_wallet_config(config: &WalletSessionConfig) -> Result<(), String> {
    if config.wallet_name.trim().is_empty() {
        return Err("Wallet name cannot be empty".to_string());
    }

    if config.total < 2 {
        return Err("Total participants must be at least 2".to_string());
    }

    if config.threshold == 0 {
        return Err("Threshold must be at least 1".to_string());
    }

    if config.threshold > config.total {
        return Err("Threshold cannot exceed total participants".to_string());
    }

    if config.timeout_hours == 0 || config.timeout_hours > 168 {
        return Err("Timeout must be between 1 and 168 hours".to_string());
    }

    if !matches!(config.curve_type.as_str(), "secp256k1" | "ed25519") {
        return Err("Curve type must be 'secp256k1' or 'ed25519'".to_string());
    }

    if config.blockchain_config.is_empty() {
        return Err("At least one blockchain must be configured".to_string());
    }

    Ok(())
}

/// Handle wallet creation completion
pub async fn handle_wallet_creation_complete<C: Ciphersuite + Send + Sync + 'static>(
    wallet_id: String,
    addresses: Vec<crate::keystore::BlockchainInfo>,
    app_state: Arc<Mutex<AppState<C>>>,
) {
    let mut state = app_state.lock().await;
    
    state.current_wallet_id = Some(wallet_id.clone());
    state.blockchain_addresses = addresses.clone();
    
    // Log completion
    for address in &addresses {
        if address.enabled {
        }
    }
    
    // Update progress to complete
    let progress = WalletCreationProgress {
        stage: WalletCreationStage::Complete,
        current_step: 6,
        total_steps: 6,
        message: "Wallet ready for use".to_string(),
        details: Some(format!("Wallet ID: {}", wallet_id)),
    };
    
    state.wallet_creation_progress = Some(progress);
    
    info!("Wallet creation completed: {}", wallet_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_wallet_config() {
        let config = WalletSessionConfig {
            wallet_name: "test-wallet".to_string(),
            description: None,
            total: 3,
            threshold: 2,
            curve_type: "secp256k1".to_string(),
            mode: WalletCreationMode::Online,
            timeout_hours: 24,
            auto_discovery: true,
            blockchain_config: vec![BlockchainConfig {
                blockchain: "ethereum".to_string(),
                network: "mainnet".to_string(),
                enabled: true,
                chain_id: Some(1),
            }],
        };

        assert!(validate_wallet_config(&config).is_ok());

        // Test invalid threshold
        let mut invalid_config = config.clone();
        invalid_config.threshold = 4;
        assert!(validate_wallet_config(&invalid_config).is_err());

        // Test empty name
        invalid_config = config.clone();
        invalid_config.wallet_name = "".to_string();
        assert!(validate_wallet_config(&invalid_config).is_err());
    }
}