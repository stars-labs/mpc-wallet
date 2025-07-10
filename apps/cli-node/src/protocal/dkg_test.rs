#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocal::dkg::{handle_trigger_dkg_round1};
    use crate::utils::state::{AppState, DkgState, MeshStatus, InternalCommand};
    use frost_secp256k1::{Secp256K1Sha256, Identifier};
    use frost_core::Ciphersuite;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::collections::BTreeMap;

    async fn create_test_state() -> Arc<Mutex<AppState<Secp256K1Sha256>>> {
        let state = AppState::<Secp256K1Sha256> {
            device_id: "test-device-1".to_string(),
            devices: vec![
                "test-device-1".to_string(),
                "test-device-2".to_string(),
                "test-device-3".to_string(),
            ],
            log: vec![],
            log_scroll: 0,
            session: Some(crate::protocal::signal::SessionInfo {
                session_id: "test-session".to_string(),
                proposer_id: "test-device-1".to_string(),
                participants: vec![
                    "test-device-1".to_string(),
                    "test-device-2".to_string(),
                    "test-device-3".to_string(),
                ],
                threshold: 2,
                total: 3,
                accepted_devices: vec![
                    "test-device-1".to_string(),
                    "test-device-2".to_string(),
                    "test-device-3".to_string(),
                ],
            }),
            invites: vec![],
            device_connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
            device_statuses: std::collections::HashMap::new(),
            reconnection_tracker: crate::utils::state::ReconnectionTracker::new(),
            making_offer: std::collections::HashMap::new(),
            pending_ice_candidates: std::collections::HashMap::new(),
            dkg_state: DkgState::Idle,
            identifier_map: Some({
                let mut map = BTreeMap::new();
                map.insert("test-device-1".to_string(), Identifier::try_from(1).unwrap());
                map.insert("test-device-2".to_string(), Identifier::try_from(2).unwrap());
                map.insert("test-device-3".to_string(), Identifier::try_from(3).unwrap());
                map
            }),
            dkg_part1_public_package: None,
            dkg_part1_secret_package: None,
            received_dkg_packages: BTreeMap::new(),
            round2_secret_package: None,
            received_dkg_round2_packages: BTreeMap::new(),
            key_package: None,
            group_public_key: None,
            data_channels: std::collections::HashMap::new(),
            solana_public_key: None,
            etherum_public_key: None,
            blockchain_addresses: Vec::new(),
            mesh_status: MeshStatus::Ready,
            pending_mesh_ready_signals: vec![],
            own_mesh_ready_sent: true,
            keystore: None,
            current_wallet_id: None,
            signing_state: crate::utils::state::SigningState::Idle,
        };
        
        Arc::new(Mutex::new(state))
    }

    #[tokio::test]
    async fn test_dkg_round1_initiation() {
        let state = create_test_state().await;
        
        // Set initial state to Round1InProgress as expected by the handler
        {
            let mut state_guard = state.lock().await;
            state_guard.dkg_state = DkgState::Round1InProgress;
        }
        
        // Trigger DKG Round 1
        handle_trigger_dkg_round1(state.clone(), "test-device-1".to_string()).await;
        
        let state_guard = state.lock().await;
        
        // Verify Round 1 packages were generated
        assert!(state_guard.dkg_part1_public_package.is_some());
        assert!(state_guard.dkg_part1_secret_package.is_some());
        
        // Check logs
        assert!(state_guard.log.iter().any(|log| log.contains("Starting DKG Round 1")));
    }

    /* TODO: Re-enable when we have proper mock FROST types
    #[tokio::test]
    async fn test_dkg_round1_package_collection() {
        let state = create_test_state().await;
        
        // First, trigger Round 1 to generate our own package
        dkg::handle_trigger_dkg_round1(state.clone(), "test-device-1".to_string()).await;
        
        // Simulate receiving Round 1 packages from other devices
        let mock_packages = vec![
            ("test-device-2", frost_core::keys::dkg::round1::Package::new(vec![], vec![])),
            ("test-device-3", frost_core::keys::dkg::round1::Package::new(vec![], vec![])),
        ];
        
        {
            let mut state_guard = state.lock().await;
            
            // Add our own package
            if let Some(our_package) = &state_guard.dkg_part1_public_package {
                state_guard.received_dkg_packages.insert(
                    Identifier::try_from(1).unwrap(),
                    our_package.clone()
                );
            }
            
            // Add mock packages from other devices
            for (device_id, package) in mock_packages {
                if let Some(identifier) = state_guard.identifier_map.as_ref()
                    .and_then(|map| map.get(device_id).cloned()) {
                    state_guard.received_dkg_packages.insert(identifier, package);
                }
            }
        }
        
        // Verify all packages are collected
        let state_guard = state.lock().await;
        assert_eq!(state_guard.received_dkg_packages.len(), 3);
    }

    #[tokio::test]
    async fn test_dkg_round2_transition() {
        let state = create_test_state().await;
        
        // Setup Round 1 completion state
        {
            let mut state_guard = state.lock().await;
            state_guard.dkg_state = DkgState::Round1Complete;
            
            // Add mock Round 1 packages
            for i in 1..=3 {
                state_guard.received_dkg_packages.insert(
                    Identifier::try_from(i as u16).unwrap(),
                    frost_core::keys::dkg::round1::Package::new(vec![], vec![])
                );
            }
        }
        
        // Trigger Round 2
        let result = dkg::handle_trigger_dkg_round2(state.clone()).await;
        assert!(result.is_ok());
        
        let state_guard = state.lock().await;
        assert!(matches!(state_guard.dkg_state, DkgState::Round2InProgress));
    }

    #[tokio::test]
    async fn test_dkg_completion() {
        let state = create_test_state().await;
        
        // Setup completed Round 2 state
        {
            let mut state_guard = state.lock().await;
            state_guard.dkg_state = DkgState::Round2Complete;
            
            // Add mock Round 2 packages
            for i in 1..=3 {
                state_guard.received_dkg_round2_packages.insert(
                    Identifier::try_from(i as u16).unwrap(),
                    frost_core::keys::dkg::round2::Package::new(vec![])
                );
            }
        }
        
        // Finalize DKG
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<InternalCommand<Secp256K1Sha256>>();
        dkg::handle_finalize_dkg(state.clone(), tx).await;
        
        // Check if finalize command was sent
        if let Some(cmd) = rx.recv().await {
            assert!(matches!(cmd, InternalCommand::FinalizeDkg));
        }
        
        let state_guard = state.lock().await;
        // In real implementation, this would be Complete after processing
        assert!(state_guard.log.iter().any(|log| log.contains("Finalizing DKG")));
    }

    #[tokio::test]
    async fn test_dkg_error_handling() {
        let state = create_test_state().await;
        
        // Try to trigger Round 2 without completing Round 1
        {
            let mut state_guard = state.lock().await;
            state_guard.dkg_state = DkgState::Round1InProgress;
            state_guard.received_dkg_packages.clear(); // No packages received
        }
        
        let result = dkg::handle_trigger_dkg_round2(state.clone()).await;
        assert!(result.is_err());
        
        let state_guard = state.lock().await;
        assert!(state_guard.log.iter().any(|log| log.contains("Error") || log.contains("not in correct state")));
    }

    #[tokio::test]
    async fn test_identifier_mapping() {
        let state = create_test_state().await;
        
        let state_guard = state.lock().await;
        let identifier_map = state_guard.identifier_map.as_ref().unwrap();
        
        // Verify correct mapping
        assert_eq!(*identifier_map.get("test-device-1").unwrap(), Identifier::try_from(1).unwrap());
        assert_eq!(*identifier_map.get("test-device-2").unwrap(), Identifier::try_from(2).unwrap());
        assert_eq!(*identifier_map.get("test-device-3").unwrap(), Identifier::try_from(3).unwrap());
    }

    #[tokio::test]
    async fn test_concurrent_dkg_operations() {
        use tokio::task;
        
        let state = create_test_state().await;
        
        // Spawn multiple tasks trying to process DKG operations
        let handles: Vec<_> = (0..5).map(|i| {
            let state_clone = state.clone();
            task::spawn(async move {
                if i % 2 == 0 {
                    dkg::handle_trigger_dkg_round1(state_clone, format!("device-{}", i)).await;
                } else {
                    let mut guard = state_clone.lock().await;
                    guard.log.push(format!("Concurrent operation {}", i));
                }
            })
        }).collect();
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify state consistency
        let state_guard = state.lock().await;
        assert!(!state_guard.log.is_empty());
    }

    #[tokio::test]
    async fn test_session_validation() {
        let state = create_test_state().await;
        
        // Remove session to test validation
        {
            let mut state_guard = state.lock().await;
            state_guard.session = None;
        }
        
        // Try to trigger DKG - should fail without session
        dkg::handle_trigger_dkg_round1(state.clone(), "test-device-1".to_string()).await;
        
        let state_guard = state.lock().await;
        assert!(state_guard.log.iter().any(|log| log.contains("No active session")));
    }

    #[tokio::test]
    async fn test_threshold_validation() {
        let state = create_test_state().await;
        
        // Test with invalid threshold (greater than total)
        {
            let mut state_guard = state.lock().await;
            if let Some(ref mut session) = state_guard.session {
                session.threshold = 4; // Greater than total (3)
            }
        }
        
        // DKG should handle invalid threshold appropriately
        dkg::handle_trigger_dkg_round1(state.clone(), "test-device-1".to_string()).await;
        
        let state_guard = state.lock().await;
        // Implementation should either fix or error on invalid threshold
        assert!(state_guard.dkg_part1_public_package.is_some() || 
               state_guard.log.iter().any(|log| log.contains("threshold")));
    }
    */
}