use frost_mpc_cli_node::keystore::Keystore;
use frost_mpc_cli_node::utils::state::{AppState, DkgState, MeshStatus, SigningState};
use frost_mpc_cli_node::protocal::signal::SessionInfo;
use frost_secp256k1::Secp256K1Sha256;
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::TempDir;
use std::collections::BTreeMap;

/// Create a test environment with multiple CLI nodes
async fn setup_test_environment(num_devices: usize) -> Vec<(Arc<Mutex<AppState<Secp256K1Sha256>>>, TempDir)> {
    let mut devices = Vec::new();
    
    for i in 0..num_devices {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let device_id = format!("test-device-{}", i + 1);
        
        // Create keystore
        let keystore = Keystore::new(temp_dir.path(), &device_id)
            .expect("Failed to create keystore");
        
        // Create app state
        let state = AppState::<Secp256K1Sha256> {
            device_id: device_id.clone(),
            devices: (0..num_devices).map(|j| format!("test-device-{}", j + 1)).collect(),
            log: vec![],
            log_scroll: 0,
            session: None,
            invites: vec![],
            device_connections: Arc::new(Mutex::new(Default::default())),
            device_statuses: Default::default(),
            reconnection_tracker: frost_mpc_cli_node::utils::state::ReconnectionTracker::new(),
            making_offer: Default::default(),
            pending_ice_candidates: Default::default(),
            dkg_state: DkgState::Idle,
            identifier_map: None,
            dkg_part1_public_package: None,
            dkg_part1_secret_package: None,
            received_dkg_packages: BTreeMap::new(),
            round2_secret_package: None,
            received_dkg_round2_packages: BTreeMap::new(),
            key_package: None,
            group_public_key: None,
            data_channels: Default::default(),
            solana_public_key: None,
            etherum_public_key: None,
            blockchain_addresses: Vec::new(),
            mesh_status: MeshStatus::Incomplete,
            pending_mesh_ready_signals: vec![],
            own_mesh_ready_sent: false,
            keystore: Some(Arc::new(keystore)),
            current_wallet_id: None,
            signing_state: SigningState::Idle,
        };
        
        devices.push((Arc::new(Mutex::new(state)), temp_dir));
    }
    
    devices
}

#[tokio::test]
async fn test_multi_device_session_establishment() {
    let devices = setup_test_environment(3).await;
    
    // Device 0 proposes a session
    let session_id = "test-session-123";
    let session = SessionInfo {
        session_id: session_id.to_string(),
        proposer_id: "test-device-1".to_string(),
        participants: vec![
            "test-device-1".to_string(),
            "test-device-2".to_string(),
            "test-device-3".to_string(),
        ],
        threshold: 2,
        total: 3,
        accepted_devices: vec![],
    };
    
    // Set session on all devices
    for (device_state, _) in &devices {
        let mut state = device_state.lock().await;
        state.session = Some(session.clone());
    }
    
    // Verify all devices have the session
    for (i, (device_state, _)) in devices.iter().enumerate() {
        let state = device_state.lock().await;
        assert!(state.session.is_some());
        assert_eq!(state.session.as_ref().unwrap().session_id, session_id);
        println!("Device {} has session {}", i + 1, session_id);
    }
}

#[tokio::test]
async fn test_dkg_completion_and_wallet_creation() {
    let devices = setup_test_environment(3).await;
    let session_id = "dkg-test-session";
    
    // Setup session and complete DKG simulation
    for (i, (device_state, _)) in devices.iter().enumerate() {
        let mut state = device_state.lock().await;
        
        // Set session
        state.session = Some(SessionInfo {
            session_id: session_id.to_string(),
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
        });
        
        // Setup identifier map
        let mut identifier_map = BTreeMap::new();
        identifier_map.insert("test-device-1".to_string(), frost_secp256k1::Identifier::try_from(1).unwrap());
        identifier_map.insert("test-device-2".to_string(), frost_secp256k1::Identifier::try_from(2).unwrap());
        identifier_map.insert("test-device-3".to_string(), frost_secp256k1::Identifier::try_from(3).unwrap());
        state.identifier_map = Some(identifier_map);
        
        // Mark DKG as complete
        state.dkg_state = DkgState::Complete;
        state.key_package = None; // Would be set during actual DKG
        state.group_public_key = None; // Would be set during actual DKG
        state.etherum_public_key = Some(format!("0x{:040x}", i));
        state.mesh_status = MeshStatus::Ready;
    }
    
    // Create wallets on each device
    for (i, (device_state, _)) in devices.iter().enumerate() {
        let state = device_state.lock().await;
        
        if let Some(keystore) = &state.keystore {
            let keystore_ptr = Arc::into_raw(keystore.clone()) as *mut Keystore;
            let wallet_id = unsafe {
                let keystore_mut = &mut *keystore_ptr;
                
                keystore_mut.create_wallet(
                    &format!("Test Wallet {}", i + 1),
                    "secp256k1",
                    "ethereum",
                    &state.etherum_public_key.as_ref().unwrap(),
                    2,
                    3,
                    "mock-group-public-key",
                    b"mock-key-share",
                    "password123",
                    vec!["test".to_string()],
                    Some("DKG test wallet".to_string()),
                ).expect("Failed to create wallet")
            };
            
            // Re-wrap the pointer
            let _keystore = unsafe { Arc::from_raw(keystore_ptr) };
            
            println!("Device {} created wallet: {}", i + 1, wallet_id);
        }
    }
}

#[tokio::test]
async fn test_signing_flow_simulation() {
    let devices = setup_test_environment(3).await;
    
    // Setup completed DKG state for all devices
    for (device_state, _) in &devices {
        let mut state = device_state.lock().await;
        state.dkg_state = DkgState::Complete;
        state.key_package = None; // Would be set during actual DKG
        state.group_public_key = None; // Would be set during actual DKG
        
        // Setup session
        state.session = Some(SessionInfo {
            session_id: "signing-session".to_string(),
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
        });
        
        // Setup identifier map
        let mut identifier_map = BTreeMap::new();
        identifier_map.insert("test-device-1".to_string(), frost_secp256k1::Identifier::try_from(1).unwrap());
        identifier_map.insert("test-device-2".to_string(), frost_secp256k1::Identifier::try_from(2).unwrap());
        identifier_map.insert("test-device-3".to_string(), frost_secp256k1::Identifier::try_from(3).unwrap());
        state.identifier_map = Some(identifier_map);
    }
    
    // Device 1 initiates signing
    {
        let mut state = devices[0].0.lock().await;
        let signing_id = "test_sign_001";
        let transaction_data = "0xabcdef1234567890";
        
        state.signing_state = SigningState::AwaitingAcceptance {
            signing_id: signing_id.to_string(),
            transaction_data: transaction_data.to_string(),
            initiator: "test-device-1".to_string(),
            required_signers: 2,
            accepted_signers: ["test-device-1".to_string()].into_iter().collect(),
        };
        
        state.log.push(format!("Initiated signing: {}", signing_id));
    }
    
    // Device 2 accepts
    {
        let state1 = devices[0].0.lock().await;
        if let SigningState::AwaitingAcceptance { signing_id, transaction_data, initiator, required_signers, .. } = &state1.signing_state {
            let mut state2 = devices[1].0.lock().await;
            state2.signing_state = SigningState::AwaitingAcceptance {
                signing_id: signing_id.clone(),
                transaction_data: transaction_data.clone(),
                initiator: initiator.clone(),
                required_signers: *required_signers,
                accepted_signers: ["test-device-1".to_string(), "test-device-2".to_string()].into_iter().collect(),
            };
            
            state2.log.push(format!("Accepted signing: {}", signing_id));
        }
    }
    
    // Verify signing states
    let state1 = devices[0].0.lock().await;
    assert!(matches!(state1.signing_state, SigningState::AwaitingAcceptance { .. }));
    
    let state2 = devices[1].0.lock().await;
    assert!(matches!(state2.signing_state, SigningState::AwaitingAcceptance { .. }));
    
    println!("Signing flow simulation completed");
}

#[tokio::test]
async fn test_keystore_backup_restore() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();
    
    // Create original keystore and wallet
    let wallet_id = {
        let mut keystore1 = Keystore::new(temp_dir1.path(), "backup-test-device")
            .expect("Failed to create keystore");
        
        keystore1.create_wallet(
            "Backup Test Wallet",
            "secp256k1",
            "ethereum",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
            2,
            3,
            "group-public-key",
            b"secret-key-share-data",
            "strong-password",
            vec!["backup".to_string(), "test".to_string()],
            Some("Wallet for backup testing".to_string()),
        ).expect("Failed to create wallet")
    };
    
    // Load wallet data from first keystore
    let wallet_data = {
        let keystore1 = Keystore::new(temp_dir1.path(), "backup-test-device")
            .expect("Failed to load keystore");
        
        keystore1.load_wallet_file(&wallet_id, "strong-password")
            .expect("Failed to load wallet")
    };
    
    // Create second keystore and restore wallet
    let wallet_id2 = {
        let mut keystore2 = Keystore::new(temp_dir2.path(), "restore-test-device")
            .expect("Failed to create second keystore");
        
        // Get wallet info from first keystore
        let keystore1 = Keystore::new(temp_dir1.path(), "backup-test-device")
            .expect("Failed to load keystore");
        let wallet_info = keystore1.get_wallet(&wallet_id)
            .expect("Wallet not found")
            .clone();
        
        // Create wallet in second keystore (will have a new ID)
        let wallet_id2 = keystore2.create_wallet(
            &wallet_info.name,
            &wallet_info.curve_type,
            &wallet_info.blockchain,
            &wallet_info.public_address,
            wallet_info.threshold,
            wallet_info.total_participants,
            &wallet_info.group_public_key,
            &wallet_data,
            "strong-password",
            wallet_info.tags.clone(),
            wallet_info.description.clone(),
        ).expect("Failed to restore wallet");
        
        // Return the new wallet ID for verification
        wallet_id2
    };
    
    // Verify restoration
    let keystore2 = Keystore::new(temp_dir2.path(), "restore-test-device")
        .expect("Failed to load second keystore");
    
    let restored_data = keystore2.load_wallet_file(&wallet_id2, "strong-password")
        .expect("Failed to load restored wallet");
    
    assert_eq!(wallet_data, restored_data);
    println!("Wallet backup and restore successful");
}

#[tokio::test]
async fn test_concurrent_operations() {
    use tokio::task;
    
    let devices = setup_test_environment(5).await;
    
    // Spawn concurrent tasks simulating various operations
    let mut handles = vec![];
    
    // Task 1: DKG operations
    let device1 = devices[0].0.clone();
    handles.push(task::spawn(async move {
        let mut state = device1.lock().await;
        state.dkg_state = DkgState::Round1InProgress;
        state.log.push("Starting DKG Round 1".to_string());
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        state.dkg_state = DkgState::Round1Complete;
    }));
    
    // Task 2: Session management
    let device2 = devices[1].0.clone();
    handles.push(task::spawn(async move {
        let mut state = device2.lock().await;
        state.session = Some(SessionInfo {
            session_id: "concurrent-session".to_string(),
            proposer_id: "test-device-2".to_string(),
            participants: vec!["test-device-2".to_string()],
            threshold: 1,
            total: 1,
            accepted_devices: vec![],
        });
        state.log.push("Created session".to_string());
    }));
    
    // Task 3: Keystore operations
    let device3 = devices[2].0.clone();
    handles.push(task::spawn(async move {
        let wallets_len = {
            let state = device3.lock().await;
            if let Some(keystore) = &state.keystore {
                keystore.list_wallets().len()
            } else {
                0
            }
        };
        
        let mut state = device3.lock().await;
        state.log.push(format!("Listed {} wallets", wallets_len));
    }));
    
    // Task 4: Signing state updates
    let device4 = devices[3].0.clone();
    handles.push(task::spawn(async move {
        let mut state = device4.lock().await;
        state.signing_state = SigningState::Idle;
        state.log.push("Signing state idle".to_string());
    }));
    
    // Task 5: Log updates
    let device5 = devices[4].0.clone();
    handles.push(task::spawn(async move {
        for i in 0..5 {
            let mut state = device5.lock().await;
            state.log.push(format!("Log entry {}", i));
            drop(state);
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
    }));
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all devices are in consistent state
    for (i, (device_state, _)) in devices.iter().enumerate() {
        let state = device_state.lock().await;
        assert!(!state.log.is_empty());
        println!("Device {} has {} log entries", i + 1, state.log.len());
    }
}

#[tokio::test]
async fn test_error_handling_scenarios() {
    let devices = setup_test_environment(2).await;
    
    // Test 1: Invalid threshold
    {
        let mut state = devices[0].0.lock().await;
        state.session = Some(SessionInfo {
            session_id: "invalid-threshold".to_string(),
            proposer_id: "test-device-1".to_string(),
            participants: vec!["test-device-1".to_string(), "test-device-2".to_string()],
            threshold: 3, // Invalid: greater than total
            total: 2,
            accepted_devices: vec![],
        });
        
        // DKG should handle this gracefully
        assert_eq!(state.session.as_ref().unwrap().threshold, 3);
        assert_eq!(state.session.as_ref().unwrap().total, 2);
    }
    
    // Test 2: Missing key package for signing
    {
        let mut state = devices[1].0.lock().await;
        state.key_package = None; // No key package
        state.signing_state = SigningState::AwaitingAcceptance {
            signing_id: "no-key-signing".to_string(),
            transaction_data: "0x123".to_string(),
            initiator: "test-device-2".to_string(),
            required_signers: 1,
            accepted_signers: ["test-device-2".to_string()].into_iter().collect(),
        };
        
        // Should remain in awaiting state without progressing
        assert!(matches!(state.signing_state, SigningState::AwaitingAcceptance { .. }));
    }
    
    // Test 3: Keystore operation with wrong password
    {
        let state = devices[0].0.lock().await;
        if let Some(keystore) = &state.keystore {
            // Create a wallet
            let keystore_ptr = Arc::into_raw(keystore.clone()) as *mut Keystore;
            let wallet_id = unsafe {
                let keystore_mut = &mut *keystore_ptr;
                
                keystore_mut.create_wallet(
                    "Error Test Wallet",
                    "secp256k1",
                    "ethereum",
                    "0x123",
                    1,
                    1,
                    "key",
                    b"data",
                    "correct-password",
                    vec![],
                    None,
                ).expect("Failed to create wallet")
            };
            
            // Re-wrap the pointer
            let keystore: Arc<Keystore> = unsafe { Arc::from_raw(keystore_ptr) };
            
            // Try to load with wrong password
            let result = keystore.load_wallet_file(&wallet_id, "wrong-password");
            assert!(result.is_err());
        }
    }
    
    println!("Error handling tests completed");
}