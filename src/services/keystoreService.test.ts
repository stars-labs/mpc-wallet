import { describe, it, expect, beforeEach, vi } from 'vitest';
import { KeystoreService } from './keystoreService';
import type { KeyShareData, WalletMetadata } from '../types/keystore';

// Mock chrome.storage API
const mockStorage = {
    local: {
        get: vi.fn(),
        set: vi.fn(),
        remove: vi.fn()
    }
};

// Mock crypto.subtle
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
        // Fill with deterministic values for testing
        for (let i = 0; i < arr.length; i++) {
            arr[i] = i % 256;
        }
        return arr;
    })
};

global.chrome = { storage: mockStorage } as any;
global.crypto = mockCrypto as any;

describe('KeystoreService', () => {
    let keystore: KeystoreService;
    
    beforeEach(() => {
        keystore = new KeystoreService();
        vi.clearAllMocks();
    });

    describe('initialization', () => {
        it('should start in locked state', () => {
            expect(keystore.isLocked()).toBe(true);
        });

        it('should have no wallets initially', async () => {
            mockStorage.local.get.mockResolvedValue({});
            const wallets = await keystore.listWallets();
            expect(wallets).toEqual([]);
        });
    });

    describe('setPassword and unlock', () => {
        const password = 'test-password-123';

        beforeEach(() => {
            // Mock crypto operations for password derivation
            mockCrypto.subtle.importKey.mockResolvedValue('mock-key');
            mockCrypto.subtle.deriveBits.mockResolvedValue(new ArrayBuffer(32));
            mockCrypto.subtle.digest.mockResolvedValue(new ArrayBuffer(32));
            mockStorage.local.set.mockResolvedValue(undefined);
        });

        it('should set password and unlock keystore', async () => {
            await keystore.setPassword(password);
            expect(keystore.isLocked()).toBe(false);
            expect(mockStorage.local.set).toHaveBeenCalledWith(
                expect.objectContaining({
                    'keystore:passwordHash': expect.any(String),
                    'keystore:salt': expect.any(String)
                })
            );
        });

        it('should derive encryption key from password', async () => {
            await keystore.setPassword(password);
            expect(mockCrypto.subtle.importKey).toHaveBeenCalledWith(
                'raw',
                expect.any(Uint8Array),
                { name: 'PBKDF2' },
                false,
                ['deriveBits']
            );
            expect(mockCrypto.subtle.deriveBits).toHaveBeenCalled();
        });

        it('should unlock with correct password', async () => {
            // Set password first
            await keystore.setPassword(password);
            
            // Lock it
            keystore.lock();
            expect(keystore.isLocked()).toBe(true);
            
            // Mock stored hash
            const mockHash = 'mock-hash';
            mockStorage.local.get.mockResolvedValue({
                'keystore:passwordHash': mockHash,
                'keystore:salt': 'mock-salt'
            });
            mockCrypto.subtle.digest.mockResolvedValue(
                new TextEncoder().encode(mockHash).buffer
            );
            
            // Unlock with correct password
            const result = await keystore.unlock(password);
            expect(result).toBe(true);
            expect(keystore.isLocked()).toBe(false);
        });

        it('should fail to unlock with incorrect password', async () => {
            // Set password first
            await keystore.setPassword(password);
            keystore.lock();
            
            // Mock stored hash
            mockStorage.local.get.mockResolvedValue({
                'keystore:passwordHash': 'correct-hash',
                'keystore:salt': 'mock-salt'
            });
            mockCrypto.subtle.digest.mockResolvedValue(
                new TextEncoder().encode('wrong-hash').buffer
            );
            
            // Try to unlock with wrong password
            const result = await keystore.unlock('wrong-password');
            expect(result).toBe(false);
            expect(keystore.isLocked()).toBe(true);
        });
    });

    describe('wallet operations', () => {
        const password = 'test-password';
        const mockKeyShareData: KeyShareData = {
            keyPackage: 'mock-key-package',
            publicKeyPackage: 'mock-public-key',
            groupPublicKey: '0x1234567890abcdef',
            sessionId: 'session-123',
            deviceId: 'device-1',
            participantIndex: 1,
            threshold: 2,
            totalParticipants: 3,
            participants: ['device-1', 'device-2', 'device-3'],
            curve: 'secp256k1',
            createdAt: Date.now()
        };
        
        const mockMetadata: WalletMetadata = {
            id: 'wallet-1',
            name: 'Test Wallet',
            blockchain: 'ethereum',
            address: '0x742d35Cc6634C0532925a3b844Bc9e7595f4279',
            sessionId: 'session-123',
            isActive: true,
            hasBackup: false
        };

        beforeEach(async () => {
            // Setup encryption mocks
            mockCrypto.subtle.importKey.mockResolvedValue('mock-key');
            mockCrypto.subtle.deriveBits.mockResolvedValue(new ArrayBuffer(32));
            mockCrypto.subtle.digest.mockResolvedValue(new ArrayBuffer(32));
            mockCrypto.subtle.encrypt.mockResolvedValue(new ArrayBuffer(100));
            mockCrypto.subtle.decrypt.mockResolvedValue(
                new TextEncoder().encode(JSON.stringify(mockKeyShareData)).buffer
            );
            mockStorage.local.set.mockResolvedValue(undefined);
            mockStorage.local.get.mockResolvedValue({});
            
            // Unlock keystore
            await keystore.setPassword(password);
        });

        it('should add wallet with encrypted key share', async () => {
            await keystore.addWallet('wallet-1', mockKeyShareData, mockMetadata);
            
            expect(mockCrypto.subtle.encrypt).toHaveBeenCalledWith(
                expect.objectContaining({ name: 'AES-GCM' }),
                expect.any(CryptoKey),
                expect.any(Uint8Array)
            );
            
            expect(mockStorage.local.set).toHaveBeenCalledWith(
                expect.objectContaining({
                    'keystore:wallet:wallet-1': expect.objectContaining({
                        algorithm: 'AES-GCM',
                        salt: expect.any(String),
                        iv: expect.any(String),
                        ciphertext: expect.any(String)
                    }),
                    'keystore:metadata:wallet-1': mockMetadata
                })
            );
        });

        it('should list wallets', async () => {
            mockStorage.local.get.mockResolvedValue({
                'keystore:metadata:wallet-1': mockMetadata,
                'keystore:metadata:wallet-2': { ...mockMetadata, id: 'wallet-2', name: 'Wallet 2' }
            });
            
            const wallets = await keystore.listWallets();
            expect(wallets).toHaveLength(2);
            expect(wallets[0]).toEqual(mockMetadata);
            expect(wallets[1].id).toBe('wallet-2');
        });

        it('should get wallet key share when unlocked', async () => {
            // Setup stored encrypted data
            mockStorage.local.get.mockResolvedValue({
                'keystore:wallet:wallet-1': {
                    algorithm: 'AES-GCM',
                    salt: 'mock-salt',
                    iv: 'mock-iv',
                    ciphertext: 'mock-ciphertext'
                }
            });
            
            const keyShare = await keystore.getWallet('wallet-1');
            expect(keyShare).toEqual(mockKeyShareData);
            expect(mockCrypto.subtle.decrypt).toHaveBeenCalled();
        });

        it('should throw error when getting wallet while locked', async () => {
            keystore.lock();
            await expect(keystore.getWallet('wallet-1')).rejects.toThrow('Keystore is locked');
        });

        it('should remove wallet', async () => {
            await keystore.removeWallet('wallet-1');
            
            expect(mockStorage.local.remove).toHaveBeenCalledWith([
                'keystore:wallet:wallet-1',
                'keystore:metadata:wallet-1'
            ]);
        });

        it('should update wallet metadata', async () => {
            const updatedMetadata = { ...mockMetadata, name: 'Updated Name' };
            await keystore.updateWalletMetadata('wallet-1', updatedMetadata);
            
            expect(mockStorage.local.set).toHaveBeenCalledWith({
                'keystore:metadata:wallet-1': updatedMetadata
            });
        });
    });

    describe('backup and restore', () => {
        const password = 'backup-password';
        const mockWallets = [
            {
                metadata: {
                    id: 'wallet-1',
                    name: 'Wallet 1',
                    blockchain: 'ethereum',
                    address: '0x123',
                    sessionId: 'session-1',
                    isActive: true,
                    hasBackup: false
                },
                keyShare: {
                    keyPackage: 'key-1',
                    publicKeyPackage: 'pub-1',
                    groupPublicKey: '0xabc',
                    sessionId: 'session-1',
                    deviceId: 'device-1',
                    participantIndex: 1,
                    threshold: 2,
                    totalParticipants: 3,
                    participants: ['device-1', 'device-2', 'device-3'],
                    curve: 'secp256k1' as const,
                    createdAt: Date.now()
                }
            }
        ];

        beforeEach(async () => {
            // Setup mocks
            mockCrypto.subtle.importKey.mockResolvedValue('mock-key');
            mockCrypto.subtle.deriveBits.mockResolvedValue(new ArrayBuffer(32));
            mockCrypto.subtle.digest.mockResolvedValue(new ArrayBuffer(32));
            mockCrypto.subtle.encrypt.mockResolvedValue(new ArrayBuffer(100));
            mockCrypto.subtle.decrypt.mockResolvedValue(
                new TextEncoder().encode(JSON.stringify(mockWallets[0].keyShare)).buffer
            );
            mockStorage.local.set.mockResolvedValue(undefined);
            
            await keystore.setPassword(password);
        });

        it('should create encrypted backup', async () => {
            // Setup wallet data
            mockStorage.local.get.mockResolvedValue({
                'keystore:metadata:wallet-1': mockWallets[0].metadata,
                'keystore:wallet:wallet-1': {
                    algorithm: 'AES-GCM',
                    salt: 'mock-salt',
                    iv: 'mock-iv',
                    ciphertext: 'mock-ciphertext'
                }
            });
            
            const backup = await keystore.createBackup();
            
            expect(backup).toHaveProperty('version', '1.0.0');
            expect(backup).toHaveProperty('deviceId');
            expect(backup).toHaveProperty('exportedAt');
            expect(backup.wallets).toHaveLength(1);
            expect(backup.wallets[0].metadata).toEqual(mockWallets[0].metadata);
            expect(backup.wallets[0].encryptedShare).toHaveProperty('algorithm', 'AES-GCM');
        });

        it('should restore from backup', async () => {
            const backup = {
                version: '1.0.0',
                deviceId: 'device-1',
                exportedAt: Date.now(),
                wallets: [{
                    metadata: mockWallets[0].metadata,
                    encryptedShare: {
                        walletId: 'wallet-1',
                        algorithm: 'AES-GCM',
                        salt: btoa('salt'),
                        iv: btoa('iv'),
                        ciphertext: btoa('ciphertext')
                    }
                }]
            };
            
            await keystore.restoreFromBackup(backup, password);
            
            expect(mockStorage.local.set).toHaveBeenCalledWith(
                expect.objectContaining({
                    'keystore:metadata:wallet-1': mockWallets[0].metadata
                })
            );
        });

        it('should clear all data', async () => {
            mockStorage.local.get.mockResolvedValue({
                'keystore:wallet:wallet-1': {},
                'keystore:metadata:wallet-1': {},
                'keystore:passwordHash': 'hash',
                'keystore:salt': 'salt',
                'keystore:index': {}
            });
            
            await keystore.clear();
            
            expect(mockStorage.local.remove).toHaveBeenCalledWith([
                'keystore:wallet:wallet-1',
                'keystore:metadata:wallet-1',
                'keystore:passwordHash',
                'keystore:salt',
                'keystore:index'
            ]);
            expect(keystore.isLocked()).toBe(true);
        });
    });

    describe('security', () => {
        it('should use different salt for each encryption', async () => {
            await keystore.setPassword('password');
            
            const keyShare: KeyShareData = {
                keyPackage: 'test',
                publicKeyPackage: 'test',
                groupPublicKey: '0x123',
                sessionId: 'session-1',
                deviceId: 'device-1',
                participantIndex: 1,
                threshold: 2,
                totalParticipants: 3,
                participants: ['device-1', 'device-2', 'device-3'],
                curve: 'secp256k1',
                createdAt: Date.now()
            };
            
            const salts = new Set<string>();
            
            for (let i = 0; i < 3; i++) {
                await keystore.addWallet(`wallet-${i}`, keyShare, {
                    id: `wallet-${i}`,
                    name: `Wallet ${i}`,
                    blockchain: 'ethereum',
                    address: '0x123',
                    sessionId: 'session-1',
                    isActive: true,
                    hasBackup: false
                });
                
                const encryptCall = mockStorage.local.set.mock.calls[i][0];
                const encryptedData = encryptCall[`keystore:wallet:wallet-${i}`];
                salts.add(encryptedData.salt);
            }
            
            // Each encryption should use a unique salt
            expect(salts.size).toBe(3);
        });

        it('should not expose sensitive data in errors', async () => {
            keystore.lock();
            
            try {
                await keystore.getWallet('wallet-1');
            } catch (error: any) {
                expect(error.message).not.toContain('password');
                expect(error.message).not.toContain('key');
                expect(error.message).toBe('Keystore is locked');
            }
        });
    });
});