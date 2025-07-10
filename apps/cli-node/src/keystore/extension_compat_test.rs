#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::{
        WalletInfo, DeviceInfo, WalletData, 
        ExtensionKeyShareData, ExtensionWalletMetadata,
        ExtensionEncryptedKeyShare, ExtensionKeystoreBackup, ExtensionBackupWallet,
        encrypt_for_extension, decrypt_from_extension, KeystoreError
    };
    use frost_secp256k1::{Secp256K1Sha256, keys::{KeyPackage, PublicKeyPackage}};
    use frost_ed25519::{Ed25519Sha512};
    use frost_core::Ciphersuite;
    use chrono::Utc;

    fn create_test_wallet_data_secp256k1() -> (WalletData, WalletInfo) {
        // Create mock key package and public key for secp256k1
        // Note: These would be actual FROST types in real usage, but we can't create them in tests
        let wallet_data = WalletData {
            secp256k1_key_package: None, // Would be Some(KeyPackage) in real usage
            secp256k1_public_key: None,   // Would be Some(PublicKeyPackage) in real usage
            ed25519_key_package: None,
            ed25519_public_key: None,
            session_id: "test-session-123".to_string(),
            device_id: "device-1".to_string(),
        };
        
        let mut wallet_info = WalletInfo::new(
            "wallet-123".to_string(),
            "Test Wallet".to_string(),
            "secp256k1".to_string(),
            "ethereum".to_string(),
            "0x742d35Cc6634C0532925a3b844Bc9e7595f4279".to_string(),
            2,
            3,
            "mock-group-public-key".to_string(),
            vec!["test".to_string()],
            Some("Test wallet for unit tests".to_string()),
        );
        
        // Add devices
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
        
        (wallet_data, wallet_info)
    }

    fn create_test_wallet_data_ed25519() -> (WalletData, WalletInfo) {
        let key_package_bytes = vec![11, 12, 13, 14, 15]; // Mock serialized data
        let public_key_bytes = vec![16, 17, 18, 19, 20]; // Mock serialized data
        
        let wallet_data = WalletData {
            secp256k1_key_package: None,
            secp256k1_public_key: None,
            ed25519_key_package: None, // Mock data - actual key package creation would be complex
            ed25519_public_key: None,   // Mock data - actual public key creation would be complex
            session_id: "test-session-456".to_string(),
            device_id: "device-2".to_string(),
        };
        
        let mut wallet_info = WalletInfo::new(
            "wallet-456".to_string(),
            "Solana Test Wallet".to_string(),
            "ed25519".to_string(),
            "solana".to_string(),
            "7S3P4HxJpyyigGzodYwHtCxZyUQe9JiBMHyRWXArAaKv".to_string(),
            2,
            4,
            "mock-ed25519-group-public-key".to_string(),
            vec!["solana".to_string()],
            Some("Solana test wallet".to_string()),
        );
        
        // Add devices
        for i in 1..=4 {
            wallet_info.add_device(DeviceInfo::new(
                format!("device-{}", i),
                format!("Device {}", i),
                i.to_string(),
            ));
        }
        
        (wallet_data, wallet_info)
    }

    /* TODO: Re-enable when we have proper mock FROST types
    #[test]
    fn test_extension_format_conversion_secp256k1() {
        let (wallet_data, wallet_info) = create_test_wallet_data_secp256k1();
        let device_info = DeviceInfo {
            device_id: "device-1".to_string(),
            name: "Device 1".to_string(),
            identifier: "1".to_string(),
            last_seen: 1234567890,
        };
        
        // Convert to extension format
        let extension_data = ExtensionKeyShareData::from_cli_wallet(
            &wallet_data,
            &wallet_info,
            &device_info,
            "ethereum",
        ).expect("Failed to convert to extension format");
        
        // Verify conversion
        assert_eq!(extension_data.session_id, "test-session-123");
        assert_eq!(extension_data.device_id, "device-1");
        assert_eq!(extension_data.participant_index, 1); // 1-based in extension
        assert_eq!(extension_data.threshold, 2);
        assert_eq!(extension_data.total_participants, 3);
        assert_eq!(extension_data.participants.len(), 3);
        assert_eq!(extension_data.curve, "secp256k1");
        assert_eq!(extension_data.ethereum_address, Some("0x742d35Cc6634C0532925a3b844Bc9e7595f4279".to_string()));
        assert_eq!(extension_data.solana_address, None);
        
        // Convert back to CLI format
        let (converted_wallet_data, converted_wallet_info) = extension_data.to_cli_wallet()
            .expect("Failed to convert back to CLI format");
        
        // Verify round-trip conversion
        assert_eq!(converted_wallet_data.session_id, wallet_data.session_id);
        assert_eq!(converted_wallet_data.device_id, wallet_data.device_id);
        assert!(converted_wallet_data.secp256k1_key_package.is_some());
        assert!(converted_wallet_data.ed25519_key_package.is_none());
        assert_eq!(converted_wallet_info.threshold, wallet_info.threshold);
        assert_eq!(converted_wallet_info.total_participants, wallet_info.total_participants);
        assert_eq!(converted_wallet_info.blockchain, "ethereum");
    }
    */

    /* TODO: Re-enable when we have proper mock FROST types
    #[test]
    fn test_extension_format_conversion_ed25519() {
        let (wallet_data, wallet_info) = create_test_wallet_data_ed25519();
        let device_info = DeviceInfo {
            device_id: "device-2".to_string(),
            name: "Device 2".to_string(),
            identifier: "2".to_string(),
            last_seen: 1234567890,
        };
        
        // Convert to extension format
        let extension_data = ExtensionKeyShareData::from_cli_wallet(
            &wallet_data,
            &wallet_info,
            &device_info,
            "solana",
        ).expect("Failed to convert to extension format");
        
        // Verify conversion
        assert_eq!(extension_data.session_id, "test-session-456");
        assert_eq!(extension_data.device_id, "device-2");
        assert_eq!(extension_data.participant_index, 2); // Device 2 is at index 1 (0-based), so 2 (1-based)
        assert_eq!(extension_data.threshold, 2);
        assert_eq!(extension_data.total_participants, 4);
        assert_eq!(extension_data.participants.len(), 4);
        assert_eq!(extension_data.curve, "ed25519");
        assert_eq!(extension_data.ethereum_address, None);
        assert_eq!(extension_data.solana_address, Some("7S3P4HxJpyyigGzodYwHtCxZyUQe9JiBMHyRWXArAaKv".to_string()));
    }
    */

    /* TODO: Re-enable when we have proper mock FROST types
    #[test]
    fn test_encryption_for_extension() {
        let (wallet_data, wallet_info) = create_test_wallet_data_secp256k1();
        let device_info = DeviceInfo {
            device_id: "device-1".to_string(),
            name: "Device 1".to_string(),
            identifier: "1".to_string(),
            last_seen: 1234567890,
        };
        
        let extension_data = ExtensionKeyShareData::from_cli_wallet(
            &wallet_data,
            &wallet_info,
            &device_info,
            "ethereum",
        ).expect("Failed to convert to extension format");
        
        // Encrypt the data
        let password = "test-password-123";
        let encrypted = encrypt_for_extension(&extension_data, password, "wallet-123")
            .expect("Failed to encrypt");
        
        // Verify encrypted structure
        assert_eq!(encrypted.wallet_id, "wallet-123");
        assert_eq!(encrypted.algorithm, "AES-GCM");
        assert!(!encrypted.salt.is_empty());
        assert!(!encrypted.iv.is_empty());
        assert!(!encrypted.ciphertext.is_empty());
        
        // Decrypt and verify
        let decrypted = decrypt_from_extension(&encrypted, password)
            .expect("Failed to decrypt");
        
        assert_eq!(decrypted.session_id, extension_data.session_id);
        assert_eq!(decrypted.device_id, extension_data.device_id);
        assert_eq!(decrypted.participant_index, extension_data.participant_index);
    }
    */

    /* TODO: Re-enable when we have proper mock FROST types
    #[test]
    fn test_encryption_with_wrong_password() {
        let (wallet_data, wallet_info) = create_test_wallet_data_secp256k1();
        let device_info = DeviceInfo {
            device_id: "device-1".to_string(),
            name: "Device 1".to_string(),
            identifier: "1".to_string(),
            last_seen: 1234567890,
        };
        
        let extension_data = ExtensionKeyShareData::from_cli_wallet(
            &wallet_data,
            &wallet_info,
            &device_info,
            "ethereum",
        ).expect("Failed to convert to extension format");
        
        // Encrypt with one password
        let encrypted = encrypt_for_extension(&extension_data, "correct-password", "wallet-123")
            .expect("Failed to encrypt");
        
        // Try to decrypt with wrong password
        let result = decrypt_from_extension(&encrypted, "wrong-password");
        assert!(result.is_err());
    }
    */

    #[test]
    fn test_device_not_found_error() {
        let (wallet_data, wallet_info) = create_test_wallet_data_secp256k1();
        let device_info = DeviceInfo {
            device_id: "unknown-device".to_string(),
            name: "Unknown Device".to_string(),
            identifier: "999".to_string(),
            last_seen: 1234567890,
        };
        
        let result = ExtensionKeyShareData::from_cli_wallet(
            &wallet_data,
            &wallet_info,
            &device_info,
            "ethereum",
        );
        
        assert!(result.is_err());
        match result {
            Err(KeystoreError::General(msg)) => {
                assert!(msg.contains("Device not found"));
            }
            _ => panic!("Expected General error with device not found message"),
        }
    }

    #[test]
    fn test_unknown_blockchain_error() {
        let (wallet_data, wallet_info) = create_test_wallet_data_secp256k1();
        let device_info = DeviceInfo {
            device_id: "device-1".to_string(),
            name: "Device 1".to_string(),
            identifier: "1".to_string(),
            last_seen: 1234567890,
        };
        
        let result = ExtensionKeyShareData::from_cli_wallet(
            &wallet_data,
            &wallet_info,
            &device_info,
            "unknown-chain",
        );
        
        assert!(result.is_err());
        match result {
            Err(KeystoreError::General(msg)) => {
                assert!(msg.contains("Unknown blockchain"));
            }
            _ => panic!("Expected General error with unknown blockchain message"),
        }
    }

    #[test]
    fn test_backup_format_creation() {
        let backup = ExtensionKeystoreBackup {
            version: "1.0.0".to_string(),
            device_id: "test-device".to_string(),
            exported_at: Utc::now().timestamp_millis(),
            wallets: vec![
                ExtensionBackupWallet {
                    metadata: ExtensionWalletMetadata {
                        id: "wallet-1".to_string(),
                        name: "Test Wallet".to_string(),
                        blockchain: "ethereum".to_string(),
                        address: "0x742d35Cc6634C0532925a3b844Bc9e7595f4279".to_string(),
                        session_id: "session-123".to_string(),
                        is_active: true,
                        has_backup: true,
                    },
                    encrypted_share: ExtensionEncryptedKeyShare {
                        wallet_id: "wallet-1".to_string(),
                        algorithm: "AES-GCM".to_string(),
                        salt: "mock-salt".to_string(),
                        iv: "mock-iv".to_string(),
                        ciphertext: "mock-ciphertext".to_string(),
                        auth_tag: None,
                    },
                },
            ],
        };
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&backup).expect("Failed to serialize backup");
        
        // Deserialize back
        let deserialized: ExtensionKeystoreBackup = 
            serde_json::from_str(&json).expect("Failed to deserialize backup");
        
        assert_eq!(deserialized.version, backup.version);
        assert_eq!(deserialized.device_id, backup.device_id);
        assert_eq!(deserialized.wallets.len(), 1);
        assert_eq!(deserialized.wallets[0].metadata.id, "wallet-1");
    }

    /* TODO: Re-enable when we have proper mock FROST types
    #[test]
    fn test_missing_key_package_error() {
        let wallet_data = WalletData {
            secp256k1_key_package: None, // Missing!
            secp256k1_public_key: Some(Default::default()),
            ed25519_key_package: None,
            ed25519_public_key: None,
            session_id: "test-session".to_string(),
            device_id: "device-1".to_string(),
        };
        
        let mut wallet_info = WalletInfo::new(
            "wallet-123".to_string(),
            "Test Wallet".to_string(),
            "secp256k1".to_string(),
            "ethereum".to_string(),
            "0x742d35Cc6634C0532925a3b844Bc9e7595f4279".to_string(),
            2,
            3,
            "mock-group-public-key".to_string(),
            vec![],
            None,
        );
        wallet_info.add_device(DeviceInfo::new(
            "device-1".to_string(),
            "Device 1".to_string(),
            "1".to_string(),
        ));
        
        let device_info = DeviceInfo {
            device_id: "device-1".to_string(),
            name: "Device 1".to_string(),
            identifier: "1".to_string(),
            last_seen: 1234567890,
        };
        
        let result = ExtensionKeyShareData::from_cli_wallet(
            &wallet_data,
            &wallet_info,
            &device_info,
            "ethereum",
        );
        
        assert!(result.is_err());
        match result {
            Err(KeystoreError::General(msg)) => {
                assert!(msg.contains("Missing secp256k1 key package"));
            }
            _ => panic!("Expected General error with missing key package message"),
        }
    }
    */
}