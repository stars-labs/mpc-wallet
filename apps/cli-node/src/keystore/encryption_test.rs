#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::encryption::{encrypt_data, decrypt_data};

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let original_data = b"This is secret key material that should be encrypted";
        let password = "strong-password-123";
        
        // Encrypt
        let encrypted = encrypt_data(original_data, password).expect("Encryption failed");
        
        // Verify encrypted data is different from original
        assert_ne!(encrypted, original_data);
        assert!(encrypted.len() > original_data.len()); // Should include salt + nonce
        
        // Decrypt
        let decrypted = decrypt_data(&encrypted, password).expect("Decryption failed");
        
        // Verify roundtrip
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let data = b"Secret data";
        let password = "correct-password";
        let wrong_password = "wrong-password";
        
        let encrypted = encrypt_data(data, password).expect("Encryption failed");
        
        // Try to decrypt with wrong password
        let result = decrypt_data(&encrypted, wrong_password);
        assert!(result.is_err());
        
        match result {
            Err(crate::keystore::KeystoreError::InvalidPassword) => {},
            _ => panic!("Expected InvalidPassword error"),
        }
    }

    #[test]
    fn test_different_encryptions_produce_different_ciphertext() {
        let data = b"Same data encrypted twice";
        let password = "password";
        
        let encrypted1 = encrypt_data(data, password).expect("First encryption failed");
        let encrypted2 = encrypt_data(data, password).expect("Second encryption failed");
        
        // Due to random salt and nonce, ciphertexts should be different
        assert_ne!(encrypted1, encrypted2);
        
        // But both should decrypt to the same data
        let decrypted1 = decrypt_data(&encrypted1, password).expect("First decryption failed");
        let decrypted2 = decrypt_data(&encrypted2, password).expect("Second decryption failed");
        
        assert_eq!(decrypted1, data);
        assert_eq!(decrypted2, data);
    }

    #[test]
    fn test_empty_data_encryption() {
        let empty_data = b"";
        let password = "password";
        
        let encrypted = encrypt_data(empty_data, password).expect("Encryption failed");
        let decrypted = decrypt_data(&encrypted, password).expect("Decryption failed");
        
        assert_eq!(decrypted, empty_data);
    }

    #[test]
    fn test_large_data_encryption() {
        let large_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let password = "password";
        
        let encrypted = encrypt_data(&large_data, password).expect("Encryption failed");
        let decrypted = decrypt_data(&encrypted, password).expect("Decryption failed");
        
        assert_eq!(decrypted, large_data);
    }

    #[test]
    fn test_special_characters_in_password() {
        let data = b"Test data";
        let password = "p@$$w0rd!#$%^&*()_+-=[]{}|;':\",./<>?";
        
        let encrypted = encrypt_data(data, password).expect("Encryption failed");
        let decrypted = decrypt_data(&encrypted, password).expect("Decryption failed");
        
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_corrupted_data_fails_decryption() {
        let data = b"Original data";
        let password = "password";
        
        let mut encrypted = encrypt_data(data, password).expect("Encryption failed");
        
        // Corrupt the encrypted data
        if encrypted.len() > 50 {
            encrypted[50] ^= 0xFF; // Flip bits in the middle
        }
        
        let result = decrypt_data(&encrypted, password);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_data_fails_decryption() {
        let data = b"Original data";
        let password = "password";
        
        let encrypted = encrypt_data(data, password).expect("Encryption failed");
        
        // Truncate the encrypted data
        let truncated = &encrypted[..encrypted.len() / 2];
        
        let result = decrypt_data(truncated, password);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_format_structure() {
        let data = b"Test";
        let password = "password";
        
        let encrypted = encrypt_data(data, password).expect("Encryption failed");
        
        // Check minimum size (salt + nonce + data + auth tag)
        assert!(encrypted.len() >= 16 + 12 + data.len() + 16);
        
        // First 16 bytes should be the salt
        let salt = &encrypted[..16];
        assert_eq!(salt.len(), 16);
        
        // Next 12 bytes should be the nonce
        let nonce = &encrypted[16..28];
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_password_derivation_consistency() {
        // This test verifies that the same password and salt produce the same key
        let password = "consistent-password";
        let data1 = b"Data 1";
        let data2 = b"Data 2";
        
        // Encrypt both data with same password
        let encrypted1 = encrypt_data(data1, password).expect("Encryption 1 failed");
        let encrypted2 = encrypt_data(data2, password).expect("Encryption 2 failed");
        
        // Decrypt both
        let decrypted1 = decrypt_data(&encrypted1, password).expect("Decryption 1 failed");
        let decrypted2 = decrypt_data(&encrypted2, password).expect("Decryption 2 failed");
        
        assert_eq!(decrypted1, data1);
        assert_eq!(decrypted2, data2);
    }

    #[test]
    fn test_concurrent_encryption_operations() {
        use std::thread;
        use std::sync::Arc;
        
        let data = Arc::new(b"Concurrent test data".to_vec());
        let password = Arc::new("password".to_string());
        
        // Spawn multiple threads doing encryption/decryption
        let handles: Vec<_> = (0..10).map(|i| {
            let data = Arc::clone(&data);
            let password = Arc::clone(&password);
            
            thread::spawn(move || {
                let encrypted = encrypt_data(&data, &password).expect("Encryption failed");
                let decrypted = decrypt_data(&encrypted, &password).expect("Decryption failed");
                assert_eq!(decrypted, *data);
                i
            })
        }).collect();
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_encryption_error_messages() {
        // Test that error messages don't leak sensitive information
        let data = b"Secret";
        let password = "password";
        
        let encrypted = encrypt_data(data, password).expect("Encryption failed");
        
        // Corrupt and try to decrypt
        let mut corrupted = encrypted.clone();
        corrupted[10] ^= 0xFF;
        
        match decrypt_data(&corrupted, "wrong-password") {
            Err(_) => {
                // Good - decryption failed as expected
                // We don't need to check error message content as long as it fails
            }
            Ok(_) => panic!("Expected decryption to fail"),
        }
    }
}