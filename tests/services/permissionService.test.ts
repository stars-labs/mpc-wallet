import { describe, it, expect, beforeEach, vi } from 'vitest';
import { PermissionService } from '../../src/services/permissionService';
import type { DAppPermission } from '../../src/services/permissionService';

// Mock chrome.storage API
const mockStorage = {
    local: {
        get: vi.fn(),
        set: vi.fn(),
        remove: vi.fn()
    }
};

global.chrome = { storage: mockStorage } as any;

describe('PermissionService', () => {
    let permissionService: PermissionService;
    
    beforeEach(() => {
        permissionService = new PermissionService();
        vi.clearAllMocks();
    });

    describe('initialization', () => {
        it('should load permissions on creation', async () => {
            const mockPermissions: Record<string, DAppPermission> = {
                'https://example.com': {
                    origin: 'https://example.com',
                    connectedAccounts: ['0x123', '0x456'],
                    lastConnected: Date.now(),
                    chainId: '1'
                }
            };
            
            mockStorage.local.get.mockResolvedValue({
                'permissions:dapps': mockPermissions
            });
            
            // Create new instance to trigger loading
            const service = new PermissionService();
            
            // Give it time to load
            await new Promise(resolve => setTimeout(resolve, 10));
            
            const permissions = await service.getPermission('https://example.com');
            expect(permissions).toEqual(mockPermissions['https://example.com']);
        });
    });

    describe('permission management', () => {
        const testOrigin = 'https://dapp.example.com';
        const testAccounts = ['0x123', '0x456'];
        const testChainId = '1';

        it('should grant permission to connect accounts', async () => {
            await permissionService.grantPermission(testOrigin, testAccounts, testChainId);
            
            expect(mockStorage.local.set).toHaveBeenCalledWith({
                'permissions:dapps': expect.objectContaining({
                    [testOrigin]: {
                        origin: testOrigin,
                        connectedAccounts: testAccounts,
                        lastConnected: expect.any(Number),
                        chainId: testChainId
                    }
                })
            });
        });

        it('should get permission for origin', async () => {
            // Setup existing permission
            await permissionService.grantPermission(testOrigin, testAccounts, testChainId);
            
            const permission = await permissionService.getPermission(testOrigin);
            expect(permission).toBeTruthy();
            expect(permission?.connectedAccounts).toEqual(testAccounts);
            expect(permission?.chainId).toBe(testChainId);
        });

        it('should return null for non-existent permission', async () => {
            const permission = await permissionService.getPermission('https://unknown.com');
            expect(permission).toBeNull();
        });

        it('should update existing permission', async () => {
            // Grant initial permission
            await permissionService.grantPermission(testOrigin, ['0x123'], testChainId);
            
            // Update with new accounts
            const newAccounts = ['0x123', '0x789'];
            await permissionService.grantPermission(testOrigin, newAccounts, testChainId);
            
            const permission = await permissionService.getPermission(testOrigin);
            expect(permission?.connectedAccounts).toEqual(newAccounts);
        });

        it('should revoke permission', async () => {
            // Grant permission first
            await permissionService.grantPermission(testOrigin, testAccounts, testChainId);
            
            // Revoke it
            await permissionService.revokePermission(testOrigin);
            
            const permission = await permissionService.getPermission(testOrigin);
            expect(permission).toBeNull();
        });

        it('should revoke all permissions', async () => {
            // Grant multiple permissions
            await permissionService.grantPermission('https://dapp1.com', ['0x123'], '1');
            await permissionService.grantPermission('https://dapp2.com', ['0x456'], '1');
            
            // Revoke all
            await permissionService.revokeAll();
            
            expect(mockStorage.local.set).toHaveBeenLastCalledWith({
                'permissions:dapps': {}
            });
        });
    });

    describe('account management', () => {
        const origin = 'https://dapp.example.com';
        const initialAccounts = ['0x123', '0x456'];
        
        beforeEach(async () => {
            await permissionService.grantPermission(origin, initialAccounts, '1');
        });

        it('should add account to existing permission', async () => {
            const newAccount = '0x789';
            await permissionService.addAccountToPermission(origin, newAccount);
            
            const permission = await permissionService.getPermission(origin);
            expect(permission?.connectedAccounts).toContain(newAccount);
            expect(permission?.connectedAccounts).toHaveLength(3);
        });

        it('should not duplicate accounts', async () => {
            await permissionService.addAccountToPermission(origin, '0x123');
            
            const permission = await permissionService.getPermission(origin);
            expect(permission?.connectedAccounts).toHaveLength(2);
            expect(permission?.connectedAccounts.filter(a => a === '0x123')).toHaveLength(1);
        });

        it('should remove account from permission', async () => {
            await permissionService.removeAccountFromPermission(origin, '0x123');
            
            const permission = await permissionService.getPermission(origin);
            expect(permission?.connectedAccounts).toEqual(['0x456']);
        });

        it('should revoke permission when last account is removed', async () => {
            await permissionService.removeAccountFromPermission(origin, '0x123');
            await permissionService.removeAccountFromPermission(origin, '0x456');
            
            const permission = await permissionService.getPermission(origin);
            expect(permission).toBeNull();
        });

        it('should handle removing non-existent account', async () => {
            await permissionService.removeAccountFromPermission(origin, '0xNONEXISTENT');
            
            const permission = await permissionService.getPermission(origin);
            expect(permission?.connectedAccounts).toEqual(initialAccounts);
        });
    });

    describe('permission queries', () => {
        beforeEach(async () => {
            // Setup test permissions
            await permissionService.grantPermission('https://dapp1.com', ['0x123', '0x456'], '1');
            await permissionService.grantPermission('https://dapp2.com', ['0x456', '0x789'], '137');
            await permissionService.grantPermission('https://dapp3.com', ['0x123'], '1');
        });

        it('should get all permissions', async () => {
            const all = await permissionService.getAllPermissions();
            expect(Object.keys(all)).toHaveLength(3);
            expect(all['https://dapp1.com']).toBeTruthy();
            expect(all['https://dapp2.com']).toBeTruthy();
            expect(all['https://dapp3.com']).toBeTruthy();
        });

        it('should get connected accounts for origin', async () => {
            const accounts = await permissionService.getConnectedAccounts('https://dapp1.com');
            expect(accounts).toEqual(['0x123', '0x456']);
        });

        it('should return empty array for non-connected origin', async () => {
            const accounts = await permissionService.getConnectedAccounts('https://unknown.com');
            expect(accounts).toEqual([]);
        });

        it('should check if origin has permission', async () => {
            expect(await permissionService.hasPermission('https://dapp1.com')).toBe(true);
            expect(await permissionService.hasPermission('https://unknown.com')).toBe(false);
        });

        it('should check if specific account is connected', async () => {
            expect(await permissionService.isAccountConnected('https://dapp1.com', '0x123')).toBe(true);
            expect(await permissionService.isAccountConnected('https://dapp1.com', '0x789')).toBe(false);
            expect(await permissionService.isAccountConnected('https://unknown.com', '0x123')).toBe(false);
        });

        it('should get all dapps connected to an account', async () => {
            const dapps = await permissionService.getConnectedDapps('0x456');
            expect(dapps).toHaveLength(2);
            expect(dapps).toContain('https://dapp1.com');
            expect(dapps).toContain('https://dapp2.com');
        });

        it('should return empty array for unconnected account', async () => {
            const dapps = await permissionService.getConnectedDapps('0xUNCONNECTED');
            expect(dapps).toEqual([]);
        });
    });

    describe('chain ID updates', () => {
        const origin = 'https://dapp.example.com';
        
        beforeEach(async () => {
            await permissionService.grantPermission(origin, ['0x123'], '1');
        });

        it('should update chain ID for permission', async () => {
            await permissionService.updateChainId(origin, '137');
            
            const permission = await permissionService.getPermission(origin);
            expect(permission?.chainId).toBe('137');
        });

        it('should update last connected time when updating chain', async () => {
            const beforeUpdate = Date.now();
            
            // Wait a bit to ensure timestamp difference
            await new Promise(resolve => setTimeout(resolve, 10));
            
            await permissionService.updateChainId(origin, '137');
            
            const permission = await permissionService.getPermission(origin);
            expect(permission?.lastConnected).toBeGreaterThan(beforeUpdate);
        });

        it('should not create permission when updating non-existent origin', async () => {
            await permissionService.updateChainId('https://unknown.com', '137');
            
            const permission = await permissionService.getPermission('https://unknown.com');
            expect(permission).toBeNull();
        });
    });

    describe('popup context', () => {
        it('should handle null origin gracefully', async () => {
            // Grant permission with null origin (from popup)
            await expect(permissionService.grantPermission(null as any, ['0x123'], '1'))
                .resolves.not.toThrow();
            
            // Should not store null origin
            const all = await permissionService.getAllPermissions();
            expect(all[null as any]).toBeUndefined();
        });

        it('should return empty accounts for null origin', async () => {
            const accounts = await permissionService.getConnectedAccounts(null as any);
            expect(accounts).toEqual([]);
        });
    });

    describe('persistence', () => {
        it('should save permissions to storage on changes', async () => {
            const origin = 'https://dapp.com';
            await permissionService.grantPermission(origin, ['0x123'], '1');
            
            expect(mockStorage.local.set).toHaveBeenCalledWith({
                'permissions:dapps': expect.objectContaining({
                    [origin]: expect.any(Object)
                })
            });
        });

        it('should handle storage errors gracefully', async () => {
            mockStorage.local.set.mockRejectedValue(new Error('Storage error'));
            
            // Should not throw
            await expect(permissionService.grantPermission('https://dapp.com', ['0x123'], '1'))
                .resolves.not.toThrow();
        });
    });
});