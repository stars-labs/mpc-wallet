import { describe, it, expect, beforeEach, vi } from 'vitest';
import AccountService from '../../src/services/accountService';
import { PermissionService } from '../../src/services/permissionService';
import { getKeystoreService } from '../../src/services/keystoreService';
import type { Account } from '../../src/types/account';
import type { KeyShareData } from '../../src/types/keystore';

// Mock chrome storage
const mockStorage = {
    local: {
        get: vi.fn(),
        set: vi.fn(),
        remove: vi.fn()
    }
};

// Mock chrome runtime
const mockRuntime = {
    sendMessage: vi.fn()
};

global.chrome = { 
    storage: mockStorage,
    runtime: mockRuntime
} as any;

// Mock crypto for keystore
global.crypto = {
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
} as any;

describe('Multi-Account Integration', () => {
    let accountService: AccountService;
    let permissionService: PermissionService;
    let keystore: any;
    
    const testAccounts: Account[] = [
        {
            id: 'acc-1',
            address: '0x742d35Cc6634C0532925a3b844Bc9e7595f4279',
            name: 'Account 1',
            balance: '100',
            blockchain: 'ethereum',
            created: Date.now(),
            metadata: {
                sessionId: 'session-1',
                source: 'dkg',
                threshold: 2,
                totalParticipants: 3
            }
        },
        {
            id: 'acc-2',
            address: '0x5aAeb6053F3e94c9b9A09F33669435E7EF1BEaEd',
            name: 'Account 2',
            balance: '50',
            blockchain: 'ethereum',
            created: Date.now(),
            metadata: {
                sessionId: 'session-2',
                source: 'dkg',
                threshold: 2,
                totalParticipants: 3
            }
        },
        {
            id: 'acc-3',
            address: '7S3P4HxJpyyigGzodYwHtCxZyUQe9JiBMHyRWXArAaKv',
            name: 'Solana Account',
            balance: '10',
            blockchain: 'solana',
            created: Date.now(),
            metadata: {
                sessionId: 'session-3',
                source: 'dkg',
                threshold: 3,
                totalParticipants: 5
            }
        }
    ];

    beforeEach(async () => {
        vi.clearAllMocks();
        mockStorage.local.get.mockResolvedValue({});
        mockStorage.local.set.mockResolvedValue(undefined);
        mockRuntime.sendMessage.mockResolvedValue({ success: true });
        
        // Reset singletons
        AccountService.resetInstance();
        
        // Initialize services
        accountService = AccountService.getInstance();
        permissionService = new PermissionService();
        keystore = getKeystoreService();
        
        // Setup keystore
        await keystore.setPassword('test-password');
        
        // Wait for initialization
        await accountService.ensureInitialized();
    });

    describe('account creation with DKG', () => {
        it('should create new account through DKG session', async () => {
            // Start new account creation
            const newSession = await accountService.generateNewAccount('My New Account', 'ethereum');
            
            expect(newSession).toMatchObject({
                sessionId: expect.stringMatching(/^account_ethereum_\d+_[a-z0-9]+$/),
                name: 'My New Account',
                blockchain: 'ethereum',
                threshold: 2,
                totalParticipants: 3,
                status: 'proposing'
            });
            
            // Simulate DKG completion
            const mockKeyShareData: KeyShareData = {
                keyPackage: 'mock-key-package',
                publicKeyPackage: 'mock-public-key',
                groupPublicKey: '0xabcdef',
                sessionId: newSession.sessionId,
                deviceId: 'device-1',
                participantIndex: 1,
                threshold: 2,
                totalParticipants: 3,
                participants: ['device-1', 'device-2', 'device-3'],
                curve: 'secp256k1',
                createdAt: Date.now()
            };
            
            const newAccount = await accountService.completeAccountCreation(
                newSession.sessionId,
                '0xNEW_ADDRESS',
                mockKeyShareData
            );
            
            expect(newAccount).toMatchObject({
                id: `mpc-${newSession.sessionId}`,
                address: '0xNEW_ADDRESS',
                name: 'My New Account',
                blockchain: 'ethereum'
            });
            
            // Verify keystore was updated
            const wallets = await keystore.listWallets();
            expect(wallets.some(w => w.id === newAccount!.id)).toBe(true);
        });

        it('should handle multiple pending sessions', async () => {
            const sessions = await Promise.all([
                accountService.generateNewAccount('Account A', 'ethereum'),
                accountService.generateNewAccount('Account B', 'ethereum'),
                accountService.generateNewAccount('Solana Account', 'solana')
            ]);
            
            expect(sessions).toHaveLength(3);
            expect(sessions[0].name).toBe('Account A');
            expect(sessions[1].name).toBe('Account B');
            expect(sessions[2].blockchain).toBe('solana');
        });
    });

    describe('multi-account permission management', () => {
        beforeEach(async () => {
            // Add test accounts
            for (const account of testAccounts) {
                await accountService.addAccount(account);
            }
        });

        it('should grant permission to specific accounts', async () => {
            const origin = 'https://dapp.example.com';
            const selectedAccounts = [testAccounts[0].address, testAccounts[1].address];
            
            await permissionService.grantPermission(origin, selectedAccounts, '1');
            
            const connectedAccounts = await permissionService.getConnectedAccounts(origin);
            expect(connectedAccounts).toEqual(selectedAccounts);
            expect(connectedAccounts).not.toContain(testAccounts[2].address);
        });

        it('should handle account switching', async () => {
            const origin = 'https://dapp.example.com';
            
            // Initially connect account 1
            await permissionService.grantPermission(origin, [testAccounts[0].address], '1');
            let connected = await permissionService.getConnectedAccounts(origin);
            expect(connected).toEqual([testAccounts[0].address]);
            
            // Switch to account 2
            await permissionService.grantPermission(origin, [testAccounts[1].address], '1');
            connected = await permissionService.getConnectedAccounts(origin);
            expect(connected).toEqual([testAccounts[1].address]);
        });

        it('should support multiple accounts per dapp', async () => {
            const origin = 'https://multi-account-dapp.com';
            const allEthereumAddresses = testAccounts
                .filter(a => a.blockchain === 'ethereum')
                .map(a => a.address);
            
            await permissionService.grantPermission(origin, allEthereumAddresses, '1');
            
            const connected = await permissionService.getConnectedAccounts(origin);
            expect(connected).toEqual(allEthereumAddresses);
            expect(connected).toHaveLength(2);
        });

        it('should track accounts across multiple dapps', async () => {
            const dapp1 = 'https://dapp1.com';
            const dapp2 = 'https://dapp2.com';
            const dapp3 = 'https://dapp3.com';
            
            // Connect different accounts to different dapps
            await permissionService.grantPermission(dapp1, [testAccounts[0].address], '1');
            await permissionService.grantPermission(dapp2, [testAccounts[0].address, testAccounts[1].address], '1');
            await permissionService.grantPermission(dapp3, [testAccounts[1].address], '1');
            
            // Check account 0 connections
            const account0Dapps = await permissionService.getConnectedDapps(testAccounts[0].address);
            expect(account0Dapps).toContain(dapp1);
            expect(account0Dapps).toContain(dapp2);
            expect(account0Dapps).not.toContain(dapp3);
            
            // Check account 1 connections
            const account1Dapps = await permissionService.getConnectedDapps(testAccounts[1].address);
            expect(account1Dapps).not.toContain(dapp1);
            expect(account1Dapps).toContain(dapp2);
            expect(account1Dapps).toContain(dapp3);
        });
    });

    describe('account removal and cleanup', () => {
        beforeEach(async () => {
            // Add accounts and set up permissions
            for (const account of testAccounts) {
                await accountService.addAccount(account);
                
                // Also add to keystore
                const mockKeyShare: KeyShareData = {
                    keyPackage: 'mock-key',
                    publicKeyPackage: 'mock-pub',
                    groupPublicKey: '0x123',
                    sessionId: account.metadata?.sessionId || 'session',
                    deviceId: 'device-1',
                    participantIndex: 1,
                    threshold: 2,
                    totalParticipants: 3,
                    participants: ['device-1', 'device-2', 'device-3'],
                    curve: account.blockchain === 'ethereum' ? 'secp256k1' : 'ed25519',
                    createdAt: Date.now()
                };
                
                await keystore.addWallet(account.id, mockKeyShare, {
                    id: account.id,
                    name: account.name,
                    blockchain: account.blockchain,
                    address: account.address,
                    sessionId: account.metadata?.sessionId || 'session',
                    isActive: true,
                    hasBackup: false
                });
            }
            
            // Set up some permissions
            await permissionService.grantPermission('https://dapp1.com', [testAccounts[0].address], '1');
            await permissionService.grantPermission('https://dapp2.com', [testAccounts[0].address, testAccounts[1].address], '1');
        });

        it('should remove account and clean up permissions', async () => {
            // Remove account 0
            await accountService.removeAccount(testAccounts[0].id);
            
            // Verify account is removed
            const accounts = accountService.getAccounts();
            expect(accounts.find(a => a.id === testAccounts[0].id)).toBeUndefined();
            
            // Verify permissions are updated
            const dapp1Connected = await permissionService.getConnectedAccounts('https://dapp1.com');
            expect(dapp1Connected).toEqual([]); // Should be empty as only account was removed
            
            const dapp2Connected = await permissionService.getConnectedAccounts('https://dapp2.com');
            expect(dapp2Connected).toEqual([testAccounts[1].address]); // Should still have account 1
        });

        it('should handle removing account from keystore', async () => {
            // Verify wallet exists in keystore
            let wallets = await keystore.listWallets();
            expect(wallets.find(w => w.id === testAccounts[0].id)).toBeTruthy();
            
            // Remove from keystore
            await keystore.removeWallet(testAccounts[0].id);
            
            // Verify removed
            wallets = await keystore.listWallets();
            expect(wallets.find(w => w.id === testAccounts[0].id)).toBeUndefined();
        });
    });

    describe('blockchain-specific operations', () => {
        beforeEach(async () => {
            for (const account of testAccounts) {
                await accountService.addAccount(account);
            }
        });

        it('should filter accounts by blockchain', () => {
            const ethereumAccounts = accountService.getAccountsByBlockchain('ethereum');
            expect(ethereumAccounts).toHaveLength(2);
            expect(ethereumAccounts.every(a => a.blockchain === 'ethereum')).toBe(true);
            
            const solanaAccounts = accountService.getAccountsByBlockchain('solana');
            expect(solanaAccounts).toHaveLength(1);
            expect(solanaAccounts[0].blockchain).toBe('solana');
        });

        it('should handle chain switching for permissions', async () => {
            const origin = 'https://chain-switch-dapp.com';
            const ethereumAccount = testAccounts[0].address;
            
            // Connect on mainnet
            await permissionService.grantPermission(origin, [ethereumAccount], '1');
            let permission = await permissionService.getPermission(origin);
            expect(permission?.chainId).toBe('1');
            
            // Switch to polygon
            await permissionService.updateChainId(origin, '137');
            permission = await permissionService.getPermission(origin);
            expect(permission?.chainId).toBe('137');
            expect(permission?.connectedAccounts).toEqual([ethereumAccount]);
        });
    });

    describe('EIP-6963 provider interface', () => {
        it('should announce multiple providers', async () => {
            // Add multiple accounts
            for (const account of testAccounts.filter(a => a.blockchain === 'ethereum')) {
                await accountService.addAccount(account);
            }
            
            // Mock window.dispatchEvent
            const dispatchedEvents: any[] = [];
            global.window = {
                dispatchEvent: vi.fn((event) => {
                    dispatchedEvents.push(event);
                })
            } as any;
            
            // Simulate provider announcement
            const providers = accountService.getAccountsByBlockchain('ethereum').map(account => ({
                info: {
                    uuid: account.id,
                    name: `MPC Wallet - ${account.name}`,
                    icon: 'data:image/svg+xml;base64,...',
                    rdns: `tech.autolife.mpc.${account.id}`
                },
                provider: {
                    // Mock provider interface
                    request: vi.fn(),
                    on: vi.fn(),
                    removeListener: vi.fn(),
                    _mpcAccount: account.address
                }
            }));
            
            // Announce providers
            providers.forEach(({ info, provider }) => {
                window.dispatchEvent(new CustomEvent('eip6963:announceProvider', {
                    detail: { info, provider }
                }));
            });
            
            expect(dispatchedEvents).toHaveLength(2);
            expect(dispatchedEvents[0].type).toBe('eip6963:announceProvider');
            expect(dispatchedEvents[0].detail.info.name).toContain('Account 1');
            expect(dispatchedEvents[1].detail.info.name).toContain('Account 2');
        });
    });
});