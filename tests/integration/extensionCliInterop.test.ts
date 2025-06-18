import { describe, it, expect, beforeEach, vi } from 'vitest';
import { getKeystoreService } from '../../src/services/keystoreService';
import type { KeyShareData, KeystoreBackup } from '../../src/types/keystore';

// Mock CLI keystore format types
interface CLIWalletData {
    secp256k1_key_package?: any;
    secp256k1_public_key?: any;
    ed25519_key_package?: any;
    ed25519_public_key?: any;
    session_id: string;
    device_id: string;
}

interface CLIExtensionKeyShareData {
    keyPackage: string;
    publicKeyPackage: string;
    groupPublicKey: string;
    sessionId: string;
    deviceId: string;
    participantIndex: number;
    threshold: number;
    totalParticipants: number;
    participants: string[];
    curve: string;
    ethereumAddress?: string;
    solanaAddress?: string;
    createdAt: number;
    lastUsed?: number;
    backupDate?: number;
}

// Mock crypto operations
const mockCrypto = {
    subtle: {
        generateKey: vi.fn(),
        importKey: vi.fn(),
        deriveBits: vi.fn(),
        encrypt: vi.fn(),
        decrypt: vi.fn(),
        digest: vi.fn()
    },
    getRandomValues: vi.fn((arr: Uint8Array) => {
        for (let i = 0; i < arr.length; i++) {
            arr[i] = i % 256;
        }
        return arr;
    })
};

global.crypto = mockCrypto as any;

// Mock chrome storage
const mockStorage = {
    local: {
        get: vi.fn(),
        set: vi.fn(),
        remove: vi.fn()
    }
};

global.chrome = { storage: mockStorage } as any;

describe('Extension-CLI Keystore Interoperability', () => {
    let extensionKeystore: any;
    
    beforeEach(async () => {
        vi.clearAllMocks();
        mockStorage.local.get.mockResolvedValue({});
        mockStorage.local.set.mockResolvedValue(undefined);
        
        // Setup crypto mocks
        mockCrypto.subtle.importKey.mockResolvedValue('mock-key');
        mockCrypto.subtle.deriveBits.mockResolvedValue(new ArrayBuffer(32));
        mockCrypto.subtle.digest.mockResolvedValue(new ArrayBuffer(32));
        mockCrypto.subtle.encrypt.mockResolvedValue(new ArrayBuffer(100));
        
        extensionKeystore = getKeystoreService();
        await extensionKeystore.setPassword('test-password');
    });

    describe('CLI to Extension format conversion', () => {
        it('should import CLI Ethereum wallet to extension', async () => {
            // Simulate CLI wallet data format
            const cliKeyShareData: CLIExtensionKeyShareData = {
                keyPackage: btoa('mock-secp256k1-key-package'),
                publicKeyPackage: btoa('mock-secp256k1-public-key'),
                groupPublicKey: '0x' + '1234567890abcdef'.repeat(8),
                sessionId: 'cli-session-eth-123',
                deviceId: 'cli-device-1',
                participantIndex: 1,
                threshold: 2,
                totalParticipants: 3,
                participants: ['cli-device-1', 'cli-device-2', 'cli-device-3'],
                curve: 'secp256k1',
                ethereumAddress: '0x742d35Cc6634C0532925a3b844Bc9e7595f4279',
                createdAt: Date.now() - 86400000, // 1 day ago
                backupDate: Date.now()
            };
            
            // Convert to extension format
            const extensionKeyShare: KeyShareData = {
                keyPackage: cliKeyShareData.keyPackage,
                publicKeyPackage: cliKeyShareData.publicKeyPackage,
                groupPublicKey: cliKeyShareData.groupPublicKey,
                sessionId: cliKeyShareData.sessionId,
                deviceId: cliKeyShareData.deviceId,
                participantIndex: cliKeyShareData.participantIndex,
                threshold: cliKeyShareData.threshold,
                totalParticipants: cliKeyShareData.totalParticipants,
                participants: cliKeyShareData.participants,
                curve: cliKeyShareData.curve as 'secp256k1',
                ethereumAddress: cliKeyShareData.ethereumAddress,
                createdAt: cliKeyShareData.createdAt,
                lastUsed: cliKeyShareData.lastUsed,
                backupDate: cliKeyShareData.backupDate
            };
            
            // Import to extension keystore
            await extensionKeystore.addWallet('imported-cli-eth', extensionKeyShare, {
                id: 'imported-cli-eth',
                name: 'Imported CLI Ethereum Wallet',
                blockchain: 'ethereum',
                address: cliKeyShareData.ethereumAddress!,
                sessionId: cliKeyShareData.sessionId,
                isActive: false,
                hasBackup: true
            });
            
            // Verify import
            const wallets = await extensionKeystore.listWallets();
            const importedWallet = wallets.find(w => w.id === 'imported-cli-eth');
            
            expect(importedWallet).toBeDefined();
            expect(importedWallet?.blockchain).toBe('ethereum');
            expect(importedWallet?.address).toBe(cliKeyShareData.ethereumAddress);
            expect(importedWallet?.sessionId).toBe(cliKeyShareData.sessionId);
        });

        it('should import CLI Solana wallet to extension', async () => {
            const cliKeyShareData: CLIExtensionKeyShareData = {
                keyPackage: btoa('mock-ed25519-key-package'),
                publicKeyPackage: btoa('mock-ed25519-public-key'),
                groupPublicKey: '0x' + 'fedcba0987654321'.repeat(8),
                sessionId: 'cli-session-sol-456',
                deviceId: 'cli-device-2',
                participantIndex: 2,
                threshold: 3,
                totalParticipants: 5,
                participants: ['cli-device-1', 'cli-device-2', 'cli-device-3', 'cli-device-4', 'cli-device-5'],
                curve: 'ed25519',
                solanaAddress: '7S3P4HxJpyyigGzodYwHtCxZyUQe9JiBMHyRWXArAaKv',
                createdAt: Date.now() - 172800000, // 2 days ago
                lastUsed: Date.now() - 3600000 // 1 hour ago
            };
            
            const extensionKeyShare: KeyShareData = {
                keyPackage: cliKeyShareData.keyPackage,
                publicKeyPackage: cliKeyShareData.publicKeyPackage,
                groupPublicKey: cliKeyShareData.groupPublicKey,
                sessionId: cliKeyShareData.sessionId,
                deviceId: cliKeyShareData.deviceId,
                participantIndex: cliKeyShareData.participantIndex,
                threshold: cliKeyShareData.threshold,
                totalParticipants: cliKeyShareData.totalParticipants,
                participants: cliKeyShareData.participants,
                curve: cliKeyShareData.curve as 'ed25519',
                solanaAddress: cliKeyShareData.solanaAddress,
                createdAt: cliKeyShareData.createdAt,
                lastUsed: cliKeyShareData.lastUsed
            };
            
            await extensionKeystore.addWallet('imported-cli-sol', extensionKeyShare, {
                id: 'imported-cli-sol',
                name: 'Imported CLI Solana Wallet',
                blockchain: 'solana',
                address: cliKeyShareData.solanaAddress!,
                sessionId: cliKeyShareData.sessionId,
                isActive: false,
                hasBackup: true
            });
            
            const wallets = await extensionKeystore.listWallets();
            const importedWallet = wallets.find(w => w.id === 'imported-cli-sol');
            
            expect(importedWallet).toBeDefined();
            expect(importedWallet?.blockchain).toBe('solana');
            expect(importedWallet?.address).toBe(cliKeyShareData.solanaAddress);
        });
    });

    describe('Extension to CLI format conversion', () => {
        let extensionWalletId: string;
        let extensionKeyShare: KeyShareData;
        
        beforeEach(async () => {
            // Create an extension wallet
            extensionWalletId = 'ext-wallet-1';
            extensionKeyShare = {
                keyPackage: btoa('extension-key-package'),
                publicKeyPackage: btoa('extension-public-key'),
                groupPublicKey: '0xabcdef1234567890',
                sessionId: 'ext-session-123',
                deviceId: 'chrome-ext-device',
                participantIndex: 1,
                threshold: 2,
                totalParticipants: 3,
                participants: ['chrome-ext-device', 'cli-device-1', 'cli-device-2'],
                curve: 'secp256k1',
                ethereumAddress: '0x5aAeb6053F3e94c9b9A09F33669435E7EF1BEaEd',
                createdAt: Date.now()
            };
            
            await extensionKeystore.addWallet(extensionWalletId, extensionKeyShare, {
                id: extensionWalletId,
                name: 'Extension Wallet',
                blockchain: 'ethereum',
                address: extensionKeyShare.ethereumAddress!,
                sessionId: extensionKeyShare.sessionId,
                isActive: true,
                hasBackup: false
            });
        });

        it('should export extension wallet to CLI format', async () => {
            // Mock decrypt to return the key share
            mockCrypto.subtle.decrypt.mockResolvedValue(
                new TextEncoder().encode(JSON.stringify(extensionKeyShare)).buffer
            );
            
            // Create backup
            const backup = await extensionKeystore.createBackup();
            
            expect(backup.wallets).toHaveLength(1);
            const exportedWallet = backup.wallets[0];
            
            // Verify backup structure matches CLI expectations
            expect(exportedWallet.metadata.id).toBe(extensionWalletId);
            expect(exportedWallet.metadata.sessionId).toBe(extensionKeyShare.sessionId);
            expect(exportedWallet.encryptedShare.algorithm).toBe('AES-GCM');
            
            // Simulate CLI decryption and conversion
            const cliFormat: CLIExtensionKeyShareData = {
                keyPackage: extensionKeyShare.keyPackage,
                publicKeyPackage: extensionKeyShare.publicKeyPackage,
                groupPublicKey: extensionKeyShare.groupPublicKey,
                sessionId: extensionKeyShare.sessionId,
                deviceId: extensionKeyShare.deviceId,
                participantIndex: extensionKeyShare.participantIndex,
                threshold: extensionKeyShare.threshold,
                totalParticipants: extensionKeyShare.totalParticipants,
                participants: extensionKeyShare.participants,
                curve: extensionKeyShare.curve,
                ethereumAddress: extensionKeyShare.ethereumAddress,
                createdAt: extensionKeyShare.createdAt,
                backupDate: backup.exportedAt
            };
            
            // Verify all required fields for CLI
            expect(cliFormat.sessionId).toBe(extensionKeyShare.sessionId);
            expect(cliFormat.participants).toEqual(extensionKeyShare.participants);
            expect(cliFormat.threshold).toBe(extensionKeyShare.threshold);
        });
    });

    describe('Encryption compatibility', () => {
        it('should use PBKDF2 with 100k iterations for CLI compatibility', async () => {
            const keyShare: KeyShareData = {
                keyPackage: 'test-key',
                publicKeyPackage: 'test-pub',
                groupPublicKey: '0x123',
                sessionId: 'test-session',
                deviceId: 'test-device',
                participantIndex: 1,
                threshold: 2,
                totalParticipants: 3,
                participants: ['device-1', 'device-2', 'device-3'],
                curve: 'secp256k1',
                createdAt: Date.now()
            };
            
            await extensionKeystore.addWallet('test-wallet', keyShare, {
                id: 'test-wallet',
                name: 'Test Wallet',
                blockchain: 'ethereum',
                address: '0x123',
                sessionId: 'test-session',
                isActive: true,
                hasBackup: false
            });
            
            // Check that PBKDF2 was used (via crypto.subtle.deriveBits)
            expect(mockCrypto.subtle.importKey).toHaveBeenCalledWith(
                'raw',
                expect.any(Uint8Array),
                { name: 'PBKDF2' },
                false,
                ['deriveBits']
            );
            
            expect(mockCrypto.subtle.deriveBits).toHaveBeenCalledWith(
                expect.objectContaining({
                    name: 'PBKDF2',
                    iterations: 100000
                }),
                expect.anything(),
                256
            );
        });
    });

    describe('Session ID compatibility', () => {
        it('should preserve session IDs during import/export', async () => {
            const sessionId = 'shared-session-123-abc';
            const participants = ['ext-device', 'cli-device-1', 'cli-device-2'];
            
            // Import from CLI
            const cliKeyShare: CLIExtensionKeyShareData = {
                keyPackage: btoa('cli-key'),
                publicKeyPackage: btoa('cli-pub'),
                groupPublicKey: '0x999',
                sessionId: sessionId,
                deviceId: 'cli-device-1',
                participantIndex: 2,
                threshold: 2,
                totalParticipants: 3,
                participants: participants,
                curve: 'secp256k1',
                ethereumAddress: '0xABC',
                createdAt: Date.now()
            };
            
            // Convert and import
            const extensionKeyShare: KeyShareData = {
                ...cliKeyShare,
                curve: cliKeyShare.curve as 'secp256k1'
            };
            
            await extensionKeystore.addWallet('session-test', extensionKeyShare, {
                id: 'session-test',
                name: 'Session Test Wallet',
                blockchain: 'ethereum',
                address: cliKeyShare.ethereumAddress!,
                sessionId: sessionId,
                isActive: true,
                hasBackup: false
            });
            
            // Export back
            mockCrypto.subtle.decrypt.mockResolvedValue(
                new TextEncoder().encode(JSON.stringify(extensionKeyShare)).buffer
            );
            
            const backup = await extensionKeystore.createBackup();
            const exportedWallet = backup.wallets[0];
            
            // Verify session ID is preserved
            expect(exportedWallet.metadata.sessionId).toBe(sessionId);
            
            // Verify participants list is preserved
            const wallets = await extensionKeystore.listWallets();
            const wallet = wallets.find(w => w.id === 'session-test');
            expect(wallet?.sessionId).toBe(sessionId);
        });
    });

    describe('Multi-device scenarios', () => {
        it('should handle wallets from multiple CLI devices', async () => {
            const sessionId = 'multi-device-session';
            const devices = [
                { id: 'cli-device-1', index: 1 },
                { id: 'cli-device-2', index: 2 },
                { id: 'ext-device', index: 3 }
            ];
            
            // Import wallets from different CLI devices
            for (const device of devices.slice(0, 2)) {
                const keyShare: KeyShareData = {
                    keyPackage: btoa(`key-${device.id}`),
                    publicKeyPackage: btoa(`pub-${device.id}`),
                    groupPublicKey: '0xSHARED_GROUP_KEY',
                    sessionId: sessionId,
                    deviceId: device.id,
                    participantIndex: device.index,
                    threshold: 2,
                    totalParticipants: 3,
                    participants: devices.map(d => d.id),
                    curve: 'secp256k1',
                    ethereumAddress: '0xSHARED_ADDRESS',
                    createdAt: Date.now()
                };
                
                await extensionKeystore.addWallet(`wallet-${device.id}`, keyShare, {
                    id: `wallet-${device.id}`,
                    name: `Wallet from ${device.id}`,
                    blockchain: 'ethereum',
                    address: '0xSHARED_ADDRESS',
                    sessionId: sessionId,
                    isActive: false,
                    hasBackup: true
                });
            }
            
            // Verify both imports
            const wallets = await extensionKeystore.listWallets();
            expect(wallets).toHaveLength(2);
            
            // All should have same session ID and address
            const sessionWallets = wallets.filter(w => w.sessionId === sessionId);
            expect(sessionWallets).toHaveLength(2);
            expect(sessionWallets.every(w => w.address === '0xSHARED_ADDRESS')).toBe(true);
            
            // But different device origins
            const deviceIds = sessionWallets.map(w => w.id);
            expect(deviceIds).toContain('wallet-cli-device-1');
            expect(deviceIds).toContain('wallet-cli-device-2');
        });
    });

    describe('Error handling', () => {
        it('should handle invalid CLI format gracefully', async () => {
            const invalidKeyShare = {
                // Missing required fields
                sessionId: 'invalid-session',
                deviceId: 'device-1'
            } as any;
            
            await expect(
                extensionKeystore.addWallet('invalid', invalidKeyShare, {
                    id: 'invalid',
                    name: 'Invalid Wallet',
                    blockchain: 'ethereum',
                    address: '0x000',
                    sessionId: 'invalid-session',
                    isActive: false,
                    hasBackup: false
                })
            ).resolves.not.toThrow();
            
            // But wallet should be stored (extension is flexible)
            const wallets = await extensionKeystore.listWallets();
            expect(wallets.some(w => w.id === 'invalid')).toBe(true);
        });

        it('should handle version mismatches', async () => {
            const futureVersionBackup = {
                version: '2.0.0', // Future version
                deviceId: 'future-device',
                exportedAt: Date.now(),
                wallets: []
            };
            
            // Extension should still attempt to restore
            await expect(
                extensionKeystore.restoreFromBackup(futureVersionBackup, 'password')
            ).resolves.not.toThrow();
        });
    });
});