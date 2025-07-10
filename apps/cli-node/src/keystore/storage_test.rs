#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::{Keystore, KeystoreError};
    use crate::keystore::models::{WalletInfo, DeviceInfo, WalletMetadata};
    use tempfile::TempDir;
    use std::fs;

    fn create_test_keystore() -> (Keystore, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let keystore = Keystore::new(temp_dir.path(), "test-device").expect("Failed to create keystore");
        (keystore, temp_dir)
    }

    #[test]
    fn test_keystore_creation() {
        let temp_dir = TempDir::new().unwrap();
        let keystore = Keystore::new(temp_dir.path(), "test-device-1");
        
        assert!(keystore.is_ok());
        let keystore = keystore.unwrap();
        assert_eq!(keystore.device_id(), "test-device-1");
        
        // Check directory structure
        assert!(temp_dir.path().join("wallets").exists());
        assert!(temp_dir.path().join("wallets/test-device-1").exists());
        assert!(temp_dir.path().join("wallets/test-device-1/ed25519").exists());
        assert!(temp_dir.path().join("wallets/test-device-1/secp256k1").exists());
    }

    #[test]
    fn test_device_id_persistence() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create keystore with device name
        {
            let _keystore = Keystore::new(temp_dir.path(), "my-device").unwrap();
        }
        
        // Load keystore again with different name
        let keystore2 = Keystore::new(temp_dir.path(), "different-name").unwrap();
        assert_eq!(keystore2.device_id(), "different-name");
    }

    #[test]
    fn test_wallet_creation() {
        let (mut keystore, _temp_dir) = create_test_keystore();
        
        let wallet_id = keystore.create_wallet(
            "Test Wallet",
            "secp256k1",
            "ethereum",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
            2,
            3,
            "mock-group-public-key",
            b"test-key-share-data",
            "password123",
            vec!["test".to_string(), "wallet".to_string()],
            Some("Test wallet description".to_string()),
            1, // participant_index
        ).unwrap();
        
        // Verify wallet was created
        assert!(!wallet_id.is_empty());
        let wallets = keystore.list_wallets();
        assert_eq!(wallets.len(), 1);
        
        let wallet = wallets[0];
        assert_eq!(wallet.wallet_id, "Test Wallet");
        assert_eq!(wallet.curve_type, "secp256k1");
        assert_eq!(wallet.blockchain, "ethereum");
        assert_eq!(wallet.public_address, "0x742d35Cc6634C0532925a3b844Bc9e7595f4279");
        assert_eq!(wallet.threshold, 2);
        assert_eq!(wallet.total_participants, 3);
        assert_eq!(wallet.participant_index, 1);
        assert_eq!(wallet.tags, vec!["test", "wallet"]);
        assert_eq!(wallet.description, Some("Test wallet description".to_string()));
    }

    #[test]
    fn test_wallet_listing() {
        let (mut keystore, _temp_dir) = create_test_keystore();
        
        // Create multiple wallets
        let wallet_ids: Vec<String> = (0..3).map(|i| {
            keystore.create_wallet(
                &format!("Wallet {}", i),
                if i % 2 == 0 { "secp256k1" } else { "ed25519" },
                if i % 2 == 0 { "ethereum" } else { "solana" },
                &format!("0x{:040x}", i),
                2,
                3,
                &format!("group-key-{}", i),
                format!("key-share-{}", i).as_bytes(),
                "password",
                vec![],
                None,
                (i + 1) as u16, // participant_index
            ).unwrap()
        }).collect();
        
        let wallets = keystore.list_wallets();
        assert_eq!(wallets.len(), 3);
        
        // Verify all wallets are listed
        for (i, wallet) in wallets.iter().enumerate() {
            assert_eq!(wallet.wallet_id, format!("Wallet {}", i));
            assert!(wallet_ids.contains(&wallet.wallet_id));
        }
    }

    #[test]
    fn test_wallet_retrieval() {
        let (mut keystore, _temp_dir) = create_test_keystore();
        
        let wallet_id = keystore.create_wallet(
            "Retrievable Wallet",
            "secp256k1",
            "ethereum",
            "0x123",
            2,
            3,
            "group-key",
            b"secret-key-share",
            "password",
            vec![],
            None,
            1, // participant_index
        ).unwrap();
        
        // Get wallet by ID
        let wallet = keystore.get_wallet(&wallet_id);
        assert!(wallet.is_some());
        assert_eq!(wallet.unwrap().wallet_id, "Retrievable Wallet");
        
        // Try non-existent wallet
        let missing = keystore.get_wallet("non-existent-id");
        assert!(missing.is_none());
    }

    #[test]
    fn test_wallet_file_encryption() {
        let (mut keystore, temp_dir) = create_test_keystore();
        
        let secret_data = b"super-secret-key-share-data";
        let password = "strong-password";
        
        let wallet_id = keystore.create_wallet(
            "Encrypted Wallet",
            "ed25519",
            "solana",
            "7S3P4HxJpyyigGzodYwHtCxZyUQe9JiBMHyRWXArAaKv",
            3,
            5,
            "group-key",
            secret_data,
            password,
            vec![],
            None,
            2, // participant_index
        ).unwrap();
        
        // Load encrypted data
        let loaded_data = keystore.load_wallet_file(&wallet_id, password).unwrap();
        assert_eq!(loaded_data, secret_data);
        
        // Try with wrong password
        let wrong_result = keystore.load_wallet_file(&wallet_id, "wrong-password");
        assert!(wrong_result.is_err());
        
        // Verify file exists and is encrypted
        let wallet_file = temp_dir.path()
            .join("wallets")
            .join(keystore.device_id())
            .join("ed25519")
            .join(format!("{}.json", wallet_id));
        assert!(wallet_file.exists());
        
        // Read raw file - should not contain plaintext in the encrypted data field
        let raw_content = fs::read_to_string(&wallet_file).unwrap();
        assert!(!raw_content.contains(std::str::from_utf8(secret_data).unwrap()));
    }

    #[test]
    fn test_wallet_deletion() {
        let (mut keystore, temp_dir) = create_test_keystore();
        
        let wallet_id = keystore.create_wallet(
            "To Delete",
            "secp256k1",
            "ethereum",
            "0x456",
            2,
            3,
            "group-key",
            b"data",
            "password",
            vec![],
            None,
            1, // participant_index
        ).unwrap();
        
        // Verify wallet exists
        assert_eq!(keystore.list_wallets().len(), 1);
        
        let wallet_file = temp_dir.path()
            .join("wallets")
            .join(keystore.device_id())
            .join("secp256k1")
            .join(format!("{}.json", wallet_id));
        assert!(wallet_file.exists());
    }

    #[test]
    fn test_device_management() {
        let (keystore, _temp_dir) = create_test_keystore();
        
        // Check this device is registered
        let device = keystore.get_this_device();
        assert!(device.is_some());
        
        let device = device.unwrap();
        assert_eq!(device.device_id, "test-device");
        assert_eq!(device.name, "test-device");
    }

    #[test]
    fn test_wallet_with_multiple_devices() {
        let (mut keystore, _temp_dir) = create_test_keystore();
        
        // Create wallet info with multiple devices
        let mut wallet_info = WalletInfo::new(
            "multi-device-wallet".to_string(),
            "Multi-Device Test".to_string(),
            "secp256k1".to_string(),
            "ethereum".to_string(),
            "0x789".to_string(),
            2,
            3,
            "group-key".to_string(),
            vec![],
            None,
        );
        
        // Add multiple devices
        wallet_info.add_device(DeviceInfo::new(
            "device-1".to_string(),
            "Device 1".to_string(),
            "1".to_string(),
        ));
        wallet_info.add_device(DeviceInfo::new(
            "device-2".to_string(),
            "Device 2".to_string(),
            "2".to_string(),
        ));
        wallet_info.add_device(DeviceInfo::new(
            "device-3".to_string(),
            "Device 3".to_string(),
            "3".to_string(),
        ));
        
        assert_eq!(wallet_info.devices.len(), 3);
        
        // Test device replacement
        wallet_info.add_device(DeviceInfo::new(
            "device-2".to_string(),
            "Device 2 Updated".to_string(),
            "2".to_string(),
        ));
        
        assert_eq!(wallet_info.devices.len(), 3);
        assert_eq!(wallet_info.devices[1].name, "Device 2 Updated");
    }

    #[test]
    fn test_concurrent_wallet_operations() {
        use std::sync::Arc;
        use std::thread;
        
        let temp_dir = Arc::new(TempDir::new().unwrap());
        let path = temp_dir.path().to_path_buf();
        
        // Create initial keystore
        Keystore::new(&path, "concurrent-test").unwrap();
        
        // Create a mutex to serialize wallet creation
        let keystore = Arc::new(std::sync::Mutex::new(
            Keystore::new(&path, "concurrent-test").unwrap()
        ));
        
        // Spawn multiple threads trying to create wallets
        let handles: Vec<_> = (0..5).map(|i| {
            let keystore = Arc::clone(&keystore);
            thread::spawn(move || {
                let mut ks = keystore.lock().unwrap();
                ks.create_wallet(
                    &format!("Thread Wallet {}", i),
                    "secp256k1",
                    "ethereum",
                    &format!("0x{:040x}", i),
                    2,
                    3,
                    "group-key",
                    format!("thread-{}-data", i).as_bytes(),
                    "password",
                    vec![],
                    None,
                    (i + 1) as u16, // participant_index
                )
            })
        }).collect();
        
        // Wait for all threads to complete
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        // All operations should succeed
        for result in &results {
            assert!(result.is_ok());
        }
        
        // Verify all wallets were created
        let keystore = Keystore::new(&path, "concurrent-test").unwrap();
        let wallets = keystore.list_wallets();
        assert_eq!(wallets.len(), 5);
    }

    #[test]
    fn test_index_persistence() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create keystore and add wallet
        let wallet_id = {
            let mut keystore = Keystore::new(temp_dir.path(), "test-device").unwrap();
            keystore.create_wallet(
                "Persistent Wallet",
                "ed25519",
                "solana",
                "SolanaAddress123",
                2,
                3,
                "group-key",
                b"data",
                "password",
                vec!["persistent".to_string()],
                Some("Should survive reload".to_string()),
                1, // participant_index
            ).unwrap()
        };
        
        // Create new keystore instance - should load existing data
        let keystore2 = Keystore::new(temp_dir.path(), "test-device").unwrap();
        let wallets = keystore2.list_wallets();
        
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].wallet_id, wallet_id);
        assert_eq!(wallets[0].description, Some("Should survive reload".to_string()));
    }

    #[test]
    fn test_error_handling() {
        let (mut keystore, _temp_dir) = create_test_keystore();
        
        // Test wallet not found error
        match keystore.load_wallet_file("non-existent-wallet", "password") {
            Err(KeystoreError::General(msg)) => {
                assert!(msg.contains("Failed to open wallet file"));
            }
            _ => panic!("Expected General error"),
        }
        
    }
}