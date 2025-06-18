// ===================================================================
// KEYSTORE SERVICE - FROST KEY SHARE MANAGEMENT
// ===================================================================
//
// This service manages the secure storage of FROST key shares for
// multiple accounts. Each account represents a separate DKG session
// with its own key material. The service handles encryption, 
// persistence, and recovery of key shares.
// ===================================================================

import { storage } from "#imports";
import type { 
    KeyShareData, 
    WalletMetadata, 
    KeystoreIndex, 
    EncryptedKeyShare,
    KeystoreBackup,
    NewAccountSession
} from "../types/keystore";

export class KeystoreService {
    private static instance: KeystoreService;
    private keystoreIndex: KeystoreIndex | null = null;
    private keyShares: Map<string, KeyShareData> = new Map();
    private password: string | null = null;
    private isUnlocked: boolean = false;
    
    private readonly STORAGE_PREFIX = "mpc_keystore";
    private readonly INDEX_KEY = `${this.STORAGE_PREFIX}_index`;
    private readonly SHARES_KEY_PREFIX = `${this.STORAGE_PREFIX}_share_`;
    
    private constructor() {
        this.loadKeystoreIndex();
    }
    
    public static getInstance(): KeystoreService {
        if (!KeystoreService.instance) {
            KeystoreService.instance = new KeystoreService();
        }
        return KeystoreService.instance;
    }
    
    /**
     * Initialize or create keystore
     */
    public async initialize(deviceId: string): Promise<void> {
        if (!this.keystoreIndex) {
            this.keystoreIndex = {
                version: "1.0.0",
                wallets: [],
                deviceId,
                isEncrypted: true,
                encryptionMethod: 'password',
                lastModified: Date.now()
            };
            await this.saveKeystoreIndex();
        }
    }
    
    /**
     * Check if keystore is locked
     */
    public isLocked(): boolean {
        return !this.isUnlocked;
    }
    
    /**
     * Unlock keystore with password
     */
    public async unlock(password: string): Promise<boolean> {
        try {
            this.password = password;
            
            // Try to decrypt one key share to verify password
            if (this.keystoreIndex && this.keystoreIndex.wallets.length > 0) {
                const firstWallet = this.keystoreIndex.wallets[0];
                await this.loadKeyShare(firstWallet.id);
            }
            
            this.isUnlocked = true;
            return true;
        } catch (error) {
            console.error("[KeystoreService] Failed to unlock:", error);
            this.password = null;
            this.isUnlocked = false;
            return false;
        }
    }
    
    /**
     * Lock keystore
     */
    public lock(): void {
        this.password = null;
        this.isUnlocked = false;
        this.keyShares.clear();
    }
    
    /**
     * Add a new wallet after DKG completion
     */
    public async addWallet(
        walletId: string,
        keyShareData: KeyShareData,
        metadata: WalletMetadata
    ): Promise<void> {
        if (!this.isUnlocked || !this.password) {
            throw new Error("Keystore is locked");
        }
        
        // Encrypt and save key share
        const encrypted = await this.encryptKeyShare(walletId, keyShareData);
        await this.saveEncryptedShare(walletId, encrypted);
        
        // Update index
        if (!this.keystoreIndex) {
            throw new Error("Keystore not initialized");
        }
        
        this.keystoreIndex.wallets.push(metadata);
        this.keystoreIndex.lastModified = Date.now();
        await this.saveKeystoreIndex();
        
        // Cache decrypted share
        this.keyShares.set(walletId, keyShareData);
        
        console.log("[KeystoreService] Added wallet:", walletId);
    }
    
    /**
     * Get key share for a wallet
     */
    public async getKeyShare(walletId: string): Promise<KeyShareData | null> {
        if (!this.isUnlocked) {
            throw new Error("Keystore is locked");
        }
        
        // Check cache first
        if (this.keyShares.has(walletId)) {
            return this.keyShares.get(walletId)!;
        }
        
        // Load and decrypt
        try {
            const keyShare = await this.loadKeyShare(walletId);
            if (keyShare) {
                this.keyShares.set(walletId, keyShare);
            }
            return keyShare;
        } catch (error) {
            console.error("[KeystoreService] Failed to load key share:", error);
            return null;
        }
    }
    
    /**
     * Get all wallet metadata
     */
    public getWallets(): WalletMetadata[] {
        return this.keystoreIndex?.wallets || [];
    }
    
    /**
     * Get wallet metadata by ID
     */
    public getWallet(walletId: string): WalletMetadata | null {
        return this.keystoreIndex?.wallets.find(w => w.id === walletId) || null;
    }
    
    /**
     * Remove a wallet
     */
    public async removeWallet(walletId: string): Promise<void> {
        if (!this.keystoreIndex) return;
        
        // Remove from index
        this.keystoreIndex.wallets = this.keystoreIndex.wallets.filter(
            w => w.id !== walletId
        );
        this.keystoreIndex.lastModified = Date.now();
        await this.saveKeystoreIndex();
        
        // Remove encrypted share
        await storage.removeItem(`local:${this.SHARES_KEY_PREFIX}${walletId}`);
        
        // Remove from cache
        this.keyShares.delete(walletId);
    }
    
    /**
     * Export wallet for backup
     */
    public async exportWallet(walletId: string): Promise<KeystoreBackup> {
        if (!this.isUnlocked || !this.keystoreIndex) {
            throw new Error("Keystore is locked");
        }
        
        const metadata = this.getWallet(walletId);
        if (!metadata) {
            throw new Error("Wallet not found");
        }
        
        const encryptedShare = await this.loadEncryptedShare(walletId);
        if (!encryptedShare) {
            throw new Error("Encrypted share not found");
        }
        
        return {
            version: this.keystoreIndex.version,
            deviceId: this.keystoreIndex.deviceId,
            exportedAt: Date.now(),
            wallets: [{
                metadata,
                encryptedShare
            }]
        };
    }
    
    /**
     * Import wallet from backup
     */
    public async importWallet(backup: KeystoreBackup, password: string): Promise<void> {
        if (!this.isUnlocked) {
            throw new Error("Keystore is locked");
        }
        
        for (const wallet of backup.wallets) {
            // Check if wallet already exists
            if (this.getWallet(wallet.metadata.id)) {
                console.warn("[KeystoreService] Wallet already exists:", wallet.metadata.id);
                continue;
            }
            
            // Save encrypted share
            await this.saveEncryptedShare(wallet.metadata.id, wallet.encryptedShare);
            
            // Try to decrypt to verify password
            try {
                await this.loadKeyShare(wallet.metadata.id);
                
                // Add to index
                this.keystoreIndex!.wallets.push(wallet.metadata);
                this.keystoreIndex!.lastModified = Date.now();
                await this.saveKeystoreIndex();
            } catch (error) {
                // Remove if decryption failed
                await storage.removeItem(`local:${this.SHARES_KEY_PREFIX}${wallet.metadata.id}`);
                throw new Error("Invalid password for backup");
            }
        }
    }
    
    // === Private Helper Methods ===
    
    private async loadKeystoreIndex(): Promise<void> {
        try {
            const stored = await storage.getItem<KeystoreIndex>(`local:${this.INDEX_KEY}`);
            if (stored) {
                this.keystoreIndex = stored;
                console.log("[KeystoreService] Loaded keystore index with", stored.wallets.length, "wallets");
            }
        } catch (error) {
            console.error("[KeystoreService] Failed to load index:", error);
        }
    }
    
    private async saveKeystoreIndex(): Promise<void> {
        if (!this.keystoreIndex) return;
        
        try {
            await storage.setItem(`local:${this.INDEX_KEY}`, this.keystoreIndex);
        } catch (error) {
            console.error("[KeystoreService] Failed to save index:", error);
        }
    }
    
    private async encryptKeyShare(walletId: string, keyShare: KeyShareData): Promise<EncryptedKeyShare> {
        if (!this.password) {
            throw new Error("No password set");
        }
        
        // Generate salt and IV
        const salt = crypto.getRandomValues(new Uint8Array(16));
        const iv = crypto.getRandomValues(new Uint8Array(12));
        
        // Derive key from password
        const key = await this.deriveKey(this.password, salt);
        
        // Encrypt data
        const encoder = new TextEncoder();
        const data = encoder.encode(JSON.stringify(keyShare));
        
        const ciphertext = await crypto.subtle.encrypt(
            { name: 'AES-GCM', iv },
            key,
            data
        );
        
        return {
            walletId,
            algorithm: 'AES-GCM',
            salt: btoa(String.fromCharCode(...salt)),
            iv: btoa(String.fromCharCode(...iv)),
            ciphertext: btoa(String.fromCharCode(...new Uint8Array(ciphertext)))
        };
    }
    
    private async decryptKeyShare(encrypted: EncryptedKeyShare): Promise<KeyShareData> {
        if (!this.password) {
            throw new Error("No password set");
        }
        
        // Decode base64
        const salt = Uint8Array.from(atob(encrypted.salt), c => c.charCodeAt(0));
        const iv = Uint8Array.from(atob(encrypted.iv), c => c.charCodeAt(0));
        const ciphertext = Uint8Array.from(atob(encrypted.ciphertext), c => c.charCodeAt(0));
        
        // Derive key
        const key = await this.deriveKey(this.password, salt);
        
        // Decrypt
        const decrypted = await crypto.subtle.decrypt(
            { name: 'AES-GCM', iv },
            key,
            ciphertext
        );
        
        const decoder = new TextDecoder();
        const json = decoder.decode(decrypted);
        return JSON.parse(json);
    }
    
    private async deriveKey(password: string, salt: Uint8Array): Promise<CryptoKey> {
        const encoder = new TextEncoder();
        const keyMaterial = await crypto.subtle.importKey(
            'raw',
            encoder.encode(password),
            'PBKDF2',
            false,
            ['deriveKey']
        );
        
        return crypto.subtle.deriveKey(
            {
                name: 'PBKDF2',
                salt,
                iterations: 100000,
                hash: 'SHA-256'
            },
            keyMaterial,
            { name: 'AES-GCM', length: 256 },
            false,
            ['encrypt', 'decrypt']
        );
    }
    
    private async saveEncryptedShare(walletId: string, encrypted: EncryptedKeyShare): Promise<void> {
        await storage.setItem(`local:${this.SHARES_KEY_PREFIX}${walletId}`, encrypted);
    }
    
    private async loadEncryptedShare(walletId: string): Promise<EncryptedKeyShare | null> {
        return await storage.getItem<EncryptedKeyShare>(`local:${this.SHARES_KEY_PREFIX}${walletId}`);
    }
    
    private async loadKeyShare(walletId: string): Promise<KeyShareData | null> {
        const encrypted = await this.loadEncryptedShare(walletId);
        if (!encrypted) return null;
        
        return await this.decryptKeyShare(encrypted);
    }
}

// Export singleton getter
export const getKeystoreService = (): KeystoreService => {
    return KeystoreService.getInstance();
};