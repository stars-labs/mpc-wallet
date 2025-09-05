//! Command - Side effects to be executed
//!
//! Commands represent operations that have side effects and need to be executed
//! outside of the pure update function. They handle async operations, I/O, and
//! interactions with external systems.

use crate::elm::message::{Message, SigningRequest};
use crate::elm::model::WalletConfig;
use tokio::sync::mpsc::UnboundedSender;
use std::path::PathBuf;
use tracing::{info, error};

/// Commands represent side effects to be executed
#[derive(Debug, Clone)]
pub enum Command {
    // Data loading commands
    LoadWallets,
    LoadSessions,
    LoadWalletDetails { wallet_id: String },
    LoadSigningRequests,
    
    // Network operations
    ConnectWebSocket { url: String },
    ReconnectWebSocket,
    DisconnectWebSocket,
    SendNetworkMessage { to: String, data: Vec<u8> },
    BroadcastMessage { data: Vec<u8> },
    
    // Keystore operations
    InitializeKeystore { path: String, device_id: String },
    SaveWallet { wallet_data: Vec<u8> },
    DeleteWallet { wallet_id: String },
    ExportWallet { wallet_id: String, path: PathBuf },
    ImportWallet { path: PathBuf },
    
    // DKG operations
    StartDKG { config: WalletConfig },
    JoinDKG { session_id: String },
    CancelDKG,
    
    // Signing operations
    StartSigning { request: SigningRequest },
    ApproveSignature { request_id: String },
    RejectSignature { request_id: String },
    
    // UI operations
    SendMessage(Message),
    ScheduleMessage { delay_ms: u64, message: Box<Message> },
    RefreshUI,
    
    // Settings operations
    SaveSettings { websocket_url: String, device_id: String },
    LoadSettings,
    
    // System operations
    Quit,
    None,
}

impl Command {
    /// Execute the command and send resulting messages back to the update loop
    pub async fn execute<C: frost_core::Ciphersuite>(
        self, 
        tx: UnboundedSender<Message>,
        app_state: &std::sync::Arc<tokio::sync::Mutex<crate::utils::appstate_compat::AppState<C>>>,
    ) -> anyhow::Result<()> {
        match self {
            Command::LoadWallets => {
                info!("Loading wallets from keystore");
                
                let state = app_state.lock().await;
                if let Some(ref keystore) = state.keystore {
                    let wallets = keystore.list_wallets();
                    // Convert Vec<&WalletMetadata> to Vec<WalletMetadata> by cloning
                    let wallets: Vec<crate::keystore::WalletMetadata> = wallets.into_iter()
                        .cloned()
                        .collect();
                    let _ = tx.send(Message::WalletsLoaded { wallets });
                } else {
                    let _ = tx.send(Message::Error { 
                        message: "Keystore not initialized".to_string() 
                    });
                }
            }
            
            Command::LoadSessions => {
                info!("Loading available sessions");
                
                let state = app_state.lock().await;
                let sessions = state.invites.clone();
                let _ = tx.send(Message::SessionsLoaded { sessions });
            }
            
            Command::LoadWalletDetails { wallet_id } => {
                info!("Loading details for wallet: {}", wallet_id);
                
                let state = app_state.lock().await;
                if let Some(ref keystore) = state.keystore {
                    if let Some(_wallet) = keystore.get_wallet(&wallet_id) {
                        // Wallet details loaded, update UI
                        let _ = tx.send(Message::Success { 
                            message: format!("Wallet {} loaded", wallet_id) 
                        });
                    } else {
                        let _ = tx.send(Message::Error { 
                            message: format!("Wallet {} not found", wallet_id) 
                        });
                    }
                }
            }
            
            Command::InitializeKeystore { path, device_id } => {
                info!("Initializing keystore at: {}", path);
                
                use crate::keystore::Keystore;
                match Keystore::new(&path, &device_id) {
                    Ok(keystore) => {
                        let mut state = app_state.lock().await;
                        state.keystore = Some(std::sync::Arc::new(keystore));
                        let _ = tx.send(Message::KeystoreInitialized { path });
                    }
                    Err(e) => {
                        error!("Failed to initialize keystore: {}", e);
                        let _ = tx.send(Message::KeystoreError { 
                            error: e.to_string() 
                        });
                    }
                }
            }
            
            Command::StartDKG { config } => {
                info!("Starting DKG with config: {:?}", config);
                
                // Send initial progress update
                let _ = tx.send(Message::UpdateDKGProgress { 
                    round: crate::elm::message::DKGRound::Initialization,
                    progress: 0.1,
                });
                
                // Start DKG task
                let tx_clone = tx.clone();
                let _app_state_clone = app_state.clone();
                tokio::spawn(async move {
                    // Simulate DKG process (replace with actual implementation)
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = tx_clone.send(Message::UpdateDKGProgress { 
                        round: crate::elm::message::DKGRound::Round1,
                        progress: 0.3,
                    });
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = tx_clone.send(Message::UpdateDKGProgress { 
                        round: crate::elm::message::DKGRound::Round2,
                        progress: 0.6,
                    });
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = tx_clone.send(Message::UpdateDKGProgress { 
                        round: crate::elm::message::DKGRound::Finalization,
                        progress: 0.9,
                    });
                    
                    // Complete DKG
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = tx_clone.send(Message::DKGComplete { 
                        result: crate::elm::message::DKGResult {
                            wallet_id: format!("wallet_{}", uuid::Uuid::new_v4()),
                            group_public_key: "mock_public_key".to_string(),
                            participant_index: 1,
                            addresses: vec![
                                ("Ethereum".to_string(), "0x1234...".to_string()),
                                ("Solana".to_string(), "Sol123...".to_string()),
                            ],
                        }
                    });
                });
            }
            
            Command::DeleteWallet { wallet_id } => {
                info!("Deleting wallet: {}", wallet_id);
                
                // TODO: Implement wallet deletion in keystore
                // For now, just send an error message
                let _ = tx.send(Message::Error { 
                    message: "Wallet deletion not yet implemented".to_string() 
                });
            }
            
            Command::ConnectWebSocket { url } => {
                info!("Connecting to WebSocket: {}", url);
                // WebSocket connection will be handled by AppRunner
                // Just send a message to indicate connection attempt
                let _ = tx.send(Message::Info { 
                    message: format!("Connecting to {}", url) 
                });
            }
            
            Command::ReconnectWebSocket => {
                info!("Attempting to reconnect WebSocket");
                // Trigger reconnection logic
                let _ = tx.send(Message::Info { 
                    message: "Reconnecting...".to_string() 
                });
            }
            
            Command::SendMessage(msg) => {
                // Forward the message
                let _ = tx.send(msg);
            }
            
            Command::ScheduleMessage { delay_ms, message } => {
                // Schedule a message to be sent after a delay
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    let _ = tx.send(*message);
                });
            }
            
            Command::RefreshUI => {
                // UI refresh handled by the view layer
                info!("UI refresh requested");
            }
            
            Command::Quit => {
                info!("Application quit requested");
                // Send quit message to trigger app shutdown
                let _ = tx.send(Message::Quit);
            }
            
            Command::None => {
                // No operation
            }
            
            _ => {
                info!("Command not yet implemented: {:?}", self);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_creation() {
        let cmd = Command::LoadWallets;
        assert!(matches!(cmd, Command::LoadWallets));
        
        let cmd = Command::StartDKG { 
            config: WalletConfig {
                name: "Test".to_string(),
                total_participants: 3,
                threshold: 2,
                curve: crate::elm::model::CurveType::Secp256k1,
                mode: crate::elm::model::WalletMode::Online,
            }
        };
        assert!(matches!(cmd, Command::StartDKG { .. }));
    }
}