import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import AccountService from '../../src/services/accountService';
import { Account } from '../../src/types/account';

// Helper function to create test accounts with required fields
function createTestAccount(partial: Partial<Account> & { id: string; address: string; name: string; blockchain: 'ethereum' | 'solana' }): Account {
    return {
        balance: '0',
        publicKey: 'test-public-key',
        created: Date.now(),
        ...partial
    };
}

// Create a proper mock for chrome.storage.local
const mockStorage = {
    data: {} as Record<string, any>,
    get: async (keys: string | string[] | Record<string, any>) => {
        // Handle different get overloads
        if (typeof keys === 'string') {
            return { [keys]: mockStorage.data[keys] };
        } else if (Array.isArray(keys)) {
            const result: Record<string, any> = {};
            keys.forEach(key => {
                result[key] = mockStorage.data[key];
            });
            return result;
        } else {
            const result: Record<string, any> = {};
            Object.keys(keys).forEach(key => {
                result[key] = mockStorage.data[key] || keys[key];
            });
            return result;
        }
    },
    set: async (data: Record<string, any>) => {
        Object.assign(mockStorage.data, data);
    },
    clear: async () => {
        mockStorage.data = {};
    }
};

// Install the mock
// Mock chrome API
(global as any).chrome = {
    storage: {
        local: mockStorage
    }
};

describe('AccountService', () => {
    let accountService: AccountService;

    beforeEach(async () => {
        // Clear mock storage
        await mockStorage.clear();
        // Reset singleton instance before each test
        AccountService.resetInstance();
        // Get fresh instance
        accountService = AccountService.getInstance();
        // Make sure it's initialized so that storage gets a chance to load
        await accountService.ensureInitialized();
    });

    afterEach(async () => {
        await mockStorage.clear();
        AccountService.resetInstance();
    });

    it('should return singleton instance', () => {
        const instance1 = AccountService.getInstance();
        const instance2 = AccountService.getInstance();

        expect(instance1).toBe(instance2);
    });

    it('should initialize with empty accounts list', async () => {
        const accounts = await accountService.getAccounts();
        expect(accounts).toEqual([]);
    });

    it('should add new account', async () => {
        const newAccount = {
            id: 'account-1',
            name: 'Test Account',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'test-public-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(newAccount);

        const accounts = await accountService.getAccounts();
        expect(accounts).toHaveLength(1);
        expect(accounts[0]).toEqual(newAccount);
    });

    it('should not allow duplicate account IDs', async () => {
        const account1 = {
            id: 'account-1',
            name: 'First Account',
            address: '0x1111111111111111111111111111111111111111',
            balance: '0',
            publicKey: 'pub-key-1',
            blockchain: 'ethereum' as const
        };

        const account2 = {
            id: 'account-1', // Same ID
            name: 'Second Account',
            address: '0x2222222222222222222222222222222222222222',
            balance: '0',
            publicKey: 'pub-key-2',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account1);

        await expect(accountService.addAccount(account2))
            .rejects.toThrow('Account with this ID already exists');
    });

    it('should not allow duplicate addresses within same blockchain', async () => {
        const account1 = {
            id: 'account-1',
            name: 'First Account',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'pub-key-1',
            blockchain: 'ethereum' as const
        };

        const account2 = {
            id: 'account-2',
            name: 'Second Account',
            address: '0x1234567890123456789012345678901234567890', // Same address
            balance: '0',
            publicKey: 'pub-key-2',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account1);

        await expect(accountService.addAccount(account2))
            .rejects.toThrow('Account with this address already exists for this blockchain');
    });

    it('should allow same address on different blockchains', async () => {
        const ethereumAccount = {
            id: 'eth-account',
            name: 'Ethereum Account',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'eth-pub-key',
            blockchain: 'ethereum' as const,
            created: Date.now()
        };

        const solanaAccount = {
            id: 'sol-account',
            name: 'Solana Account',
            address: '0x1234567890123456789012345678901234567890', // Same address format for test
            balance: '0',
            publicKey: 'sol-pub-key',
            blockchain: 'solana' as const,
            created: Date.now()
        };

        await accountService.addAccount(ethereumAccount);

        // Should not throw - different blockchains
        await expect(accountService.addAccount(solanaAccount)).resolves.toEqual(solanaAccount);

        const accounts = await accountService.getAccounts();
        expect(accounts).toHaveLength(2);
    });

    it('should update existing account', async () => {
        const originalAccount = {
            id: 'account-1',
            name: 'Original Name',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(originalAccount);

        const updatedAccount = {
            ...originalAccount,
            name: 'Updated Name',
            balance: '1000000000000000000' // 1 ETH in wei
        };

        await accountService.updateAccount(updatedAccount);

        const accounts = await accountService.getAccounts();
        expect(accounts).toHaveLength(1);
        expect(accounts[0].name).toBe('Updated Name');
        expect(accounts[0].balance).toBe('1000000000000000000');
    });

    it('should throw error when updating non-existent account', async () => {
        const nonExistentAccount = {
            id: 'non-existent',
            name: 'Non-existent Account',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await expect(accountService.updateAccount(nonExistentAccount))
            .rejects.toThrow('Account not found');
    });

    it('should remove account', async () => {
        const account = {
            id: 'account-to-remove',
            name: 'Account to Remove',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account);

        const accountsBefore = await accountService.getAccounts();
        expect(accountsBefore).toHaveLength(1);

        await accountService.removeAccount('account-to-remove');

        const accountsAfter = await accountService.getAccounts();
        expect(accountsAfter).toHaveLength(0);
    });

    it('should throw error when removing non-existent account', async () => {
        await expect(accountService.removeAccount('non-existent'))
            .rejects.toThrow('Account not found');
    });

    it('should set and get current account', async () => {
        const account1 = {
            id: 'account-1',
            name: 'First Account',
            address: '0x1111111111111111111111111111111111111111',
            balance: '0',
            publicKey: 'pub-key-1',
            blockchain: 'ethereum' as const
        };

        const account2 = {
            id: 'account-2',
            name: 'Second Account',
            address: '0x2222222222222222222222222222222222222222',
            balance: '0',
            publicKey: 'pub-key-2',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account1);
        await accountService.addAccount(account2);

        // Set current account
        await accountService.setCurrentAccount('account-2');

        const currentAccount = await accountService.getCurrentAccount();
        expect(currentAccount?.id).toBe('account-2');
        expect(currentAccount?.name).toBe('Second Account');
    });

    it('should return null for current account when none set', async () => {
        const currentAccount = await accountService.getCurrentAccount();
        expect(currentAccount).toBeNull();
    });

    it('should throw error when setting non-existent current account', async () => {
        await expect(accountService.setCurrentAccount('non-existent'))
            .rejects.toThrow('Account not found');
    });

    it('should get accounts by blockchain', async () => {
        const ethAccount = {
            id: 'eth-account',
            name: 'Ethereum Account',
            address: '0x1111111111111111111111111111111111111111',
            balance: '0',
            publicKey: 'eth-pub-key',
            blockchain: 'ethereum' as const
        };

        const solAccount = {
            id: 'sol-account',
            name: 'Solana Account',
            address: 'Sol1111111111111111111111111111111111111111',
            balance: '0',
            publicKey: 'sol-pub-key',
            blockchain: 'solana' as const
        };

        await accountService.addAccount(ethAccount);
        await accountService.addAccount(solAccount);

        const ethereumAccounts = await accountService.getAccountsByBlockchain('ethereum');
        const solanaAccounts = await accountService.getAccountsByBlockchain('solana');

        expect(ethereumAccounts).toHaveLength(1);
        expect(ethereumAccounts[0].blockchain).toBe('ethereum');

        expect(solanaAccounts).toHaveLength(1);
        expect(solanaAccounts[0].blockchain).toBe('solana');
    });

    it('should get account by ID', async () => {
        const account = {
            id: 'specific-account',
            name: 'Specific Account',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account);

        const foundAccount = await accountService.getAccountById('specific-account');
        expect(foundAccount).toEqual(account);

        const notFoundAccount = await accountService.getAccountById('non-existent');
        expect(notFoundAccount).toBeNull();
    });

    it('should persist accounts to storage', async () => {
        const account = {
            id: 'persistent-account',
            name: 'Persistent Account',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account);

        // Check storage directly
        const stored = await mockStorage.get('wallet_accounts');
        expect(stored.wallet_accounts).toHaveLength(1);
        expect(stored.wallet_accounts[0]).toEqual(account);
    });

    it('should load accounts from storage on initialization', async () => {
        // Create test account
        const testAccount = {
            id: 'pre-stored',
            name: 'Pre-stored Account',
            address: '0x9999999999999999999999999999999999999999',
            balance: '5000000000000000000',
            publicKey: 'pre-stored-key',
            blockchain: 'ethereum' as const,
            created: Date.now()
        };

        // Clear storage and reset singleton
        await mockStorage.clear();
        AccountService.resetInstance();

        // Create a service and add the test account
        const tempService = AccountService.getInstance();
        await tempService.ensureInitialized();
        await tempService.addAccount(testAccount);
        await tempService.setCurrentAccount('pre-stored');

        // Reset singleton to simulate app restart
        AccountService.resetInstance();

        // Create new service instance
        const newService = AccountService.getInstance();
        await newService.ensureInitialized();
        const accounts = await newService.getAccounts();
        const currentAccount = await newService.getCurrentAccount();

        expect(accounts).toHaveLength(1);
        expect(accounts[0].id).toBe('pre-stored');
        expect(currentAccount?.id).toBe('pre-stored');
    });

    it('should handle malformed storage data gracefully', async () => {
        // Set malformed data in storage
        await mockStorage.set({ wallet_accounts: 'invalid-data' });

        // Should still work and initialize empty
        const accounts = await accountService.getAccounts();
        expect(accounts).toEqual([]);
    });

    it('should validate account structure', async () => {
        const invalidAccount = {
            // Missing required fields
            name: 'Invalid Account'
        } as any;

        await expect(accountService.addAccount(invalidAccount))
            .rejects.toThrow();
    });

    it('should handle account balance updates', async () => {
        const account = {
            id: 'balance-test',
            name: 'Balance Test Account',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account);

        // Update balance
        await accountService.updateAccount({
            ...account,
            balance: '2500000000000000000' // 2.5 ETH
        });

        const updatedAccount = await accountService.getAccountById('balance-test');
        expect(updatedAccount?.balance).toBe('2500000000000000000');
    });

    it('should get account by address', async () => {
        const account = {
            id: 'address-lookup-test',
            name: 'Address Lookup Account',
            address: '0xabcdef1234567890abcdef1234567890abcdef12',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account);

        const foundAccount = accountService.getAccount('0xabcdef1234567890abcdef1234567890abcdef12');
        expect(foundAccount).toEqual(account);

        const notFoundAccount = accountService.getAccount('0x0000000000000000000000000000000000000000');
        expect(notFoundAccount).toBeUndefined();
    });

    it('should auto-select first account when no current account is set but accounts exist', async () => {
        const account1 = {
            id: 'auto-select-1',
            name: 'Auto Select Account 1',
            address: '0x1111111111111111111111111111111111111111',
            balance: '0',
            publicKey: 'pub-key-1',
            blockchain: 'ethereum' as const
        };

        const account2 = {
            id: 'auto-select-2',
            name: 'Auto Select Account 2',
            address: '0x2222222222222222222222222222222222222222',
            balance: '0',
            publicKey: 'pub-key-2',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account1);
        await accountService.addAccount(account2);

        // Clear current account setting
        (accountService as any).currentAccountAddress = undefined;

        // Should auto-select first account
        const currentAccount = accountService.getCurrentAccount();
        expect(currentAccount?.id).toBe('auto-select-1');
        expect((accountService as any).currentAccountAddress).toBe(account1.address);
    });

    it('should handle storage errors during auto-selection gracefully', async () => {
        // Spy on console.error
        const originalConsoleError = console.error;
        let errorLogged = false;
        console.error = (...args: any[]) => {
            if (args[0] === 'Failed to save current account:') {
                errorLogged = true;
            }
        };

        const account = {
            id: 'auto-select-error',
            name: 'Auto Select Error Account',
            address: '0x1111111111111111111111111111111111111111',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account);

        // Mock storage to fail during save
        const originalSet = mockStorage.set;
        mockStorage.set = async () => {
            throw new Error('Storage failed');
        };

        // Clear current account setting
        (accountService as any).currentAccountAddress = undefined;

        // Should still auto-select first account despite storage error
        const currentAccount = accountService.getCurrentAccount();
        expect(currentAccount?.id).toBe('auto-select-error');
        expect((accountService as any).currentAccountAddress).toBe(account.address);

        // Allow time for async error handling
        await new Promise(resolve => setTimeout(resolve, 10));

        // Check that error was logged
        expect(errorLogged).toBe(true);

        // Restore mocks
        mockStorage.set = originalSet;
        console.error = originalConsoleError;
    });

    it('should clear all accounts', async () => {
        const account1 = {
            id: 'clear-test-1',
            name: 'Clear Test Account 1',
            address: '0x1111111111111111111111111111111111111111',
            balance: '0',
            publicKey: 'pub-key-1',
            blockchain: 'ethereum' as const
        };

        const account2 = {
            id: 'clear-test-2',
            name: 'Clear Test Account 2',
            address: '0x2222222222222222222222222222222222222222',
            balance: '0',
            publicKey: 'pub-key-2',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account1);
        await accountService.addAccount(account2);
        await accountService.setCurrentAccount('clear-test-1');

        expect(accountService.getAccounts()).toHaveLength(2);
        expect(accountService.getCurrentAccount()?.id).toBe('clear-test-1');

        await accountService.clearAccounts();

        expect(accountService.getAccounts()).toHaveLength(0);
        expect(accountService.getCurrentAccount()).toBeNull();
        expect((accountService as any).currentAccountAddress).toBeUndefined();

        // Check storage is cleared
        const stored = await mockStorage.get('wallet_accounts');
        expect(stored.wallet_accounts).toEqual([]);
    });

    it('should handle account change listeners', async () => {
        let changeCallbackCount = 0;
        let lastChangedAccount: any = null;

        const callback1 = (account: any) => {
            changeCallbackCount++;
            lastChangedAccount = account;
        };

        const callback2 = (account: any) => {
            changeCallbackCount++;
        };

        // Add listeners
        accountService.onAccountChange(callback1);
        accountService.onAccountChange(callback2);

        const account = {
            id: 'listener-test',
            name: 'Listener Test Account',
            address: '0x1234567890123456789012345678901234567890',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(account);

        // Reset counters after adding account
        changeCallbackCount = 0;
        lastChangedAccount = null;

        await accountService.setCurrentAccount('listener-test');

        expect(changeCallbackCount).toBe(2); // Both callbacks should be called
        expect(lastChangedAccount?.id).toBe('listener-test');

        // Remove one listener
        accountService.removeAccountChangeListener(callback2);

        // Reset counters
        changeCallbackCount = 0;
        lastChangedAccount = null;

        // Change account again
        await accountService.clearAccounts();

        expect(changeCallbackCount).toBe(1); // Only one callback should be called
        expect(lastChangedAccount).toBeNull(); // Account was cleared
    });

    it('should ensure default account when no accounts exist', async () => {
        // Mock chrome.runtime.sendMessage for DKG address
        const originalSendMessage = (global as any).chrome.runtime?.sendMessage;
        (global as any).chrome = {
            ...(global as any).chrome,
            runtime: {
                sendMessage: async (message: any) => {
                    if (message.type === 'getEthereumAddress') {
                        return {
                            success: true,
                            data: {
                                ethereumAddress: '0xabcdef1234567890abcdef1234567890abcdef12'
                            }
                        };
                    }
                    return { success: false };
                }
            }
        };

        expect(accountService.getAccounts()).toHaveLength(0);

        const defaultAccount = await accountService.ensureDefaultAccount();

        expect(defaultAccount).not.toBeNull();
        expect(defaultAccount?.address).toBe('0xabcdef1234567890abcdef1234567890abcdef12');
        expect(defaultAccount?.name).toBe('MPC Wallet');
        expect(defaultAccount?.blockchain).toBe('ethereum');
        expect(defaultAccount?.id).toMatch(/^mpc-/);

        const accounts = accountService.getAccounts();
        expect(accounts).toHaveLength(1);
        expect(accounts[0]).toEqual(defaultAccount);

        // Restore original mock
        if (originalSendMessage) {
            (global as any).chrome.runtime.sendMessage = originalSendMessage;
        }
    });

    it('should return existing account when ensureDefaultAccount called with existing accounts', async () => {
        const existingAccount = {
            id: 'existing-account',
            name: 'Existing Account',
            address: '0x1111111111111111111111111111111111111111',
            balance: '0',
            publicKey: 'pub-key',
            blockchain: 'ethereum' as const
        };

        await accountService.addAccount(existingAccount);

        const defaultAccount = await accountService.ensureDefaultAccount();

        expect(defaultAccount?.id).toBe('existing-account');
        expect(accountService.getAccounts()).toHaveLength(1); // No new account created
    });

    it('should handle DKG address retrieval failure gracefully', async () => {
        // Mock chrome.runtime.sendMessage to fail
        (global as any).chrome = {
            ...(global as any).chrome,
            runtime: {
                sendMessage: async (message: any) => {
                    if (message.type === 'getEthereumAddress') {
                        return { success: false };
                    }
                    return { success: false };
                }
            }
        };

        expect(accountService.getAccounts()).toHaveLength(0);

        const defaultAccount = await accountService.ensureDefaultAccount();

        expect(defaultAccount).toBeNull();
        expect(accountService.getAccounts()).toHaveLength(0);
    });

    it('should handle DKG communication errors gracefully', async () => {
        // Temporarily suppress console.error for this test
        const originalConsoleError = console.error;
        console.error = () => { }; // Suppress error output

        // Mock chrome.runtime.sendMessage to throw
        (global as any).chrome = {
            ...(global as any).chrome,
            runtime: {
                sendMessage: async (message: any) => {
                    throw new Error('Communication failed');
                }
            }
        };

        expect(accountService.getAccounts()).toHaveLength(0);

        const defaultAccount = await accountService.ensureDefaultAccount();

        expect(defaultAccount).toBeNull();
        expect(accountService.getAccounts()).toHaveLength(0);

        // Restore console.error
        console.error = originalConsoleError;
    });

    it('should handle invalid DKG response format gracefully', async () => {
        // Mock chrome.runtime.sendMessage to return invalid format
        (global as any).chrome = {
            ...(global as any).chrome,
            runtime: {
                sendMessage: async (message: any) => {
                    if (message.type === 'getEthereumAddress') {
                        return {
                            success: true,
                            data: null // Invalid data
                        };
                    }
                    return { success: false };
                }
            }
        };

        expect(accountService.getAccounts()).toHaveLength(0);

        const defaultAccount = await accountService.ensureDefaultAccount();

        expect(defaultAccount).toBeNull();
        expect(accountService.getAccounts()).toHaveLength(0);
    });
});
