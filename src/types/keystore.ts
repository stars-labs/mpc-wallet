// ===================================================================
// KEYSTORE TYPES - FROST KEY SHARE STORAGE
// ===================================================================
//
// This module defines types for securely storing FROST key shares
// and wallet metadata in the browser extension. Each account has its
// own key share from a separate DKG session.
// ===================================================================

export interface KeyShareData {
    // Core FROST key material
    keyPackage: string; // Serialized FROST KeyPackage (encrypted)
    publicKeyPackage: string; // Serialized PublicKeyPackage
    groupPublicKey: string; // The group's public key
    
    // Session information
    sessionId: string; // DKG session identifier
    deviceId: string; // This device's identifier in the group
    participantIndex: number; // This participant's index (1-based)
    
    // Threshold configuration
    threshold: number; // Required signers (t)
    totalParticipants: number; // Total participants (n)
    participants: string[]; // List of all participant device IDs
    
    // Blockchain specific
    curve: 'secp256k1' | 'ed25519'; // Ethereum or Solana
    ethereumAddress?: string; // Derived Ethereum address
    solanaAddress?: string; // Derived Solana address
    
    // Metadata
    createdAt: number; // Timestamp
    lastUsed?: number; // Last signing operation
    backupDate?: number; // Last backup timestamp
}

export interface WalletMetadata {
    id: string; // Unique wallet ID (matches account ID)
    name: string; // User-friendly name
    blockchain: 'ethereum' | 'solana';
    address: string; // The primary address
    sessionId: string; // Links to KeyShareData
    
    // Visual
    color?: string; // For UI identification
    icon?: string; // Custom icon
    
    // Status
    isActive: boolean; // Whether this wallet is currently usable
    hasBackup: boolean; // Whether user has backed up this wallet
}

export interface KeystoreIndex {
    version: string; // Keystore format version
    wallets: WalletMetadata[]; // All wallets
    activeWalletId?: string; // Currently selected wallet
    deviceId: string; // This device's global identifier
    
    // Security
    isEncrypted: boolean; // Whether key shares are encrypted
    encryptionMethod?: 'password' | 'biometric' | 'none';
    lastModified: number;
}

export interface EncryptedKeyShare {
    walletId: string;
    algorithm: 'AES-GCM'; // Encryption algorithm
    salt: string; // Salt for key derivation (base64)
    iv: string; // Initialization vector (base64)
    ciphertext: string; // Encrypted KeyShareData (base64)
    authTag?: string; // Authentication tag for GCM (base64)
}

export interface KeystoreBackup {
    version: string;
    deviceId: string;
    exportedAt: number;
    wallets: Array<{
        metadata: WalletMetadata;
        encryptedShare: EncryptedKeyShare;
    }>;
}

// Key derivation parameters (similar to CLI implementation)
export interface KeyDerivationParams {
    algorithm: 'argon2id' | 'pbkdf2';
    salt: Uint8Array;
    iterations?: number; // For PBKDF2
    memory?: number; // For Argon2
    parallelism?: number; // For Argon2
    keyLength: number; // Output key size
}

// Session info for creating new accounts
export interface NewAccountSession {
    sessionId: string;
    name: string;
    blockchain: 'ethereum' | 'solana';
    threshold: number;
    totalParticipants: number;
    participants: string[];
    status: 'proposing' | 'waiting_acceptance' | 'dkg_in_progress' | 'completed' | 'failed';
    createdAt: number;
}

// Events emitted by keystore
export interface KeystoreEvent {
    type: 'wallet_added' | 'wallet_removed' | 'wallet_updated' | 'keystore_locked' | 'keystore_unlocked';
    walletId?: string;
    timestamp: number;
}