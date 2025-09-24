//! Centralized error types for the TUI node
//! 
//! This module provides structured error handling to replace panic-prone
//! unwrap() and expect() calls throughout the codebase.

use thiserror::Error;
use std::io;

/// Errors that can occur during cryptographic operations
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid parameters for cryptographic operation: {0}")]
    InvalidParams(String),
    
    #[error("Password hashing failed: {0}")]
    PasswordHashError(String),
    
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionError(String),
    
    #[error("Invalid salt length: expected {expected}, got {got}")]
    InvalidSaltLength { expected: usize, got: usize },
    
    #[error("Invalid key derivation parameters")]
    InvalidKdfParams,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// Errors that can occur during DKG protocol
#[derive(Error, Debug)]
pub enum DKGError {
    #[error("Invalid participant identifier: {0}")]
    InvalidIdentifier(u16),
    
    #[error("Invalid participant count: {0}")]
    InvalidParticipantCount(u16),
    
    #[error("Invalid threshold: {0}")]
    InvalidThreshold(u16),
    
    #[error("DKG round 1 failed: {0}")]
    Round1Error(String),
    
    #[error("DKG round 2 failed: {0}")]
    Round2Error(String),
    
    #[error("Missing participant: {0}")]
    MissingParticipant(u16),
    
    #[error("Protocol state error: {0}")]
    StateError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Errors that can occur during signing operations
#[derive(Error, Debug)]
pub enum SigningError {
    #[error("Invalid signer identifier: {0}")]
    InvalidIdentifier(u16),
    
    #[error("Insufficient signers: need {threshold}, got {actual}")]
    InsufficientSigners { threshold: u16, actual: usize },
    
    #[error("Signing round 1 failed: {0}")]
    Round1Error(String),
    
    #[error("Signing round 2 failed: {0}")]
    Round2Error(String),
    
    #[error("Invalid signature share from participant {0}")]
    InvalidShare(u16),
    
    #[error("Nonce generation failed: {0}")]
    NonceError(String),
    
    #[error("Key package not found")]
    KeyPackageNotFound,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Errors that can occur in keystore operations  
#[derive(Error, Debug)]
pub enum KeystoreError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Keystore not found at path: {0}")]
    NotFound(String),
    
    #[error("Invalid keystore format")]
    InvalidFormat,
    
    #[error("Wallet already exists: {0}")]
    WalletExists(String),
    
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(#[from] CryptoError),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid password")]
    InvalidPassword,
    
    #[error("Keystore is locked")]
    Locked,
}

/// Errors that can occur in component operations
#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("Component not found: {0:?}")]
    NotFound(String),
    
    #[error("Component already mounted: {0:?}")]
    AlreadyMounted(String),
    
    #[error("Invalid component state")]
    InvalidState,
    
    #[error("Component ID conflict: {0:?}")]
    IdConflict(String),
}

/// Main error type that encompasses all error variants
#[derive(Error, Debug)]
pub enum TuiError {
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("DKG protocol error: {0}")]
    DKG(#[from] DKGError),
    
    #[error("Signing error: {0}")]
    Signing(#[from] SigningError),
    
    #[error("Keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    
    #[error("Component error: {0}")]
    Component(#[from] ComponentError),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Type alias for Results in this crate
pub type Result<T> = std::result::Result<T, TuiError>;

/// Type alias for crypto-specific results
pub type CryptoResult<T> = std::result::Result<T, CryptoError>;

/// Type alias for DKG-specific results
pub type DKGResult<T> = std::result::Result<T, DKGError>;

/// Type alias for signing-specific results  
pub type SigningResult<T> = std::result::Result<T, SigningError>;

/// Type alias for keystore-specific results
pub type KeystoreResult<T> = std::result::Result<T, KeystoreError>;