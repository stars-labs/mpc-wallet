import { describe, it, expect, beforeEach, afterEach, mock } from 'bun:test';
import { render, fireEvent, waitFor, screen } from '@testing-library/svelte';
import { vi } from 'vitest';
import AccountManager from '../../src/components/AccountManager.svelte';
import type { Account } from '@mpc-wallet/types/account';

// Mock dependencies
const mockAccountService = {
    getAccounts: mock(async () => [] as Account[]),
    getCurrentAccount: mock(async () => null as Account | null),
    addAccount: mock(async (account: Account) => account),
    updateAccount: mock(async (account: Account) => account),
    removeAccount: mock(async (id: string) => {}),
    setCurrentAccount: mock(async (id: string | null) => {}),
    onAccountChange: mock((callback: (account: Account | null) => void) => {}),
    offAccountChange: mock((callback: (account: Account | null) => void) => {})
};

const mockKeystoreManager = {
    createKeystore: mock(async () => ({ id: 'test-keystore', keys: 'encrypted' })),
    importKeystore: mock(async () => ({ id: 'imported-keystore', keys: 'encrypted' })),
    exportKeystore: mock(async () => 'exported-keystore-data'),
    hasKeystore: mock(() => true)
};

// Mock the services
vi.mock('../../src/services/accountService', () => ({
    default: {
        getInstance: () => mockAccountService
    }
}));

vi.mock('../../src/services/keystoreManager', () => ({
    KeystoreManager: mockKeystoreManager
}));

// Test data
const createTestAccount = (overrides: Partial<Account> = {}): Account => ({
    id: 'test-account-1',
    name: 'Test Account',
    address: '0x742d35Cc6641C4532B4d2a3F44ae7f35E0D29636',
    balance: '1000000000000000000', // 1 ETH
    blockchain: 'ethereum',
    publicKey: 'test-public-key',
    created: Date.now(),
    lastUsed: Date.now(),
    isActive: false,
    metadata: {
        source: 'generated',
        derivationPath: "m/44'/60'/0'/0/0"
    },
    ...overrides
});

describe('AccountManager Component', () => {
    beforeEach(() => {
        // Clear all mocks
        Object.values(mockAccountService).forEach(mock => mock.mockClear());
        Object.values(mockKeystoreManager).forEach(mock => mock.mockClear());
    });

    describe('Component Rendering', () => {
        it('should render empty state when no accounts exist', async () => {
            mockAccountService.getAccounts.mockResolvedValueOnce([]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.textContent).toContain('No accounts found');
            });
        });

        it('should render account list when accounts exist', async () => {
            const testAccounts = [
                createTestAccount(),
                createTestAccount({
                    id: 'test-account-2',
                    name: 'Second Account',
                    address: '0x8ba1f109551bD432803012645Hac136c',
                    blockchain: 'solana'
                })
            ];
            
            mockAccountService.getAccounts.mockResolvedValueOnce(testAccounts);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.textContent).toContain('Test Account');
                expect(container.textContent).toContain('Second Account');
                expect(container.textContent).toContain('0x742d35Cc...E0D29636');
            });
        });

        it('should show current account indicator', async () => {
            const testAccount = createTestAccount({ isActive: true });
            
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            mockAccountService.getCurrentAccount.mockResolvedValueOnce(testAccount);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.querySelector('.account-active')).toBeTruthy();
            });
        });

        it('should display account balances correctly', async () => {
            const testAccount = createTestAccount({
                balance: '2500000000000000000', // 2.5 ETH
                blockchain: 'ethereum'
            });
            
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.textContent).toContain('2.5'); // Should format ETH balance
            });
        });
    });

    describe('Account Creation', () => {
        it('should show create account modal when create button clicked', async () => {
            mockAccountService.getAccounts.mockResolvedValueOnce([]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                const createButton = container.querySelector('[data-testid="create-account-btn"]');
                expect(createButton).toBeTruthy();
            });
            
            const createButton = container.querySelector('[data-testid="create-account-btn"]') as HTMLElement;
            await fireEvent.click(createButton);
            
            await waitFor(() => {
                expect(container.textContent).toContain('Create New Account');
            });
        });

        it('should create new account with valid data', async () => {
            mockAccountService.getAccounts.mockResolvedValueOnce([]);
            const newAccount = createTestAccount({ name: 'New Account' });
            mockAccountService.addAccount.mockResolvedValueOnce(newAccount);
            
            const { container } = render(AccountManager);
            
            // Open create modal
            const createButton = container.querySelector('[data-testid="create-account-btn"]') as HTMLElement;
            await fireEvent.click(createButton);
            
            // Fill form
            const nameInput = container.querySelector('[data-testid="account-name-input"]') as HTMLInputElement;
            const blockchainSelect = container.querySelector('[data-testid="blockchain-select"]') as HTMLSelectElement;
            
            await fireEvent.input(nameInput, { target: { value: 'New Account' } });
            await fireEvent.change(blockchainSelect, { target: { value: 'ethereum' } });
            
            // Submit form
            const submitButton = container.querySelector('[data-testid="create-submit-btn"]') as HTMLElement;
            await fireEvent.click(submitButton);
            
            await waitFor(() => {
                expect(mockAccountService.addAccount).toHaveBeenCalledWith(
                    expect.objectContaining({
                        name: 'New Account',
                        blockchain: 'ethereum'
                    })
                );
            });
        });

        it('should validate account name input', async () => {
            mockAccountService.getAccounts.mockResolvedValueOnce([]);
            
            const { container } = render(AccountManager);
            
            // Open create modal
            const createButton = container.querySelector('[data-testid="create-account-btn"]') as HTMLElement;
            await fireEvent.click(createButton);
            
            // Try to submit with empty name
            const submitButton = container.querySelector('[data-testid="create-submit-btn"]') as HTMLElement;
            await fireEvent.click(submitButton);
            
            await waitFor(() => {
                expect(container.textContent).toContain('Account name is required');
            });
        });

        it('should handle create account errors', async () => {
            mockAccountService.getAccounts.mockResolvedValueOnce([]);
            mockAccountService.addAccount.mockRejectedValueOnce(new Error('Account creation failed'));
            
            const { container } = render(AccountManager);
            
            // Open create modal and fill form
            const createButton = container.querySelector('[data-testid="create-account-btn"]') as HTMLElement;
            await fireEvent.click(createButton);
            
            const nameInput = container.querySelector('[data-testid="account-name-input"]') as HTMLInputElement;
            await fireEvent.input(nameInput, { target: { value: 'Test Account' } });
            
            const submitButton = container.querySelector('[data-testid="create-submit-btn"]') as HTMLElement;
            await fireEvent.click(submitButton);
            
            await waitFor(() => {
                expect(container.textContent).toContain('Account creation failed');
            });
        });
    });

    describe('Account Switching', () => {
        it('should switch to selected account', async () => {
            const testAccounts = [
                createTestAccount(),
                createTestAccount({
                    id: 'account-2',
                    name: 'Account 2',
                    address: '0x8ba1f109551bD432803012645Hac136c'
                })
            ];
            
            mockAccountService.getAccounts.mockResolvedValueOnce(testAccounts);
            mockAccountService.getCurrentAccount.mockResolvedValueOnce(testAccounts[0]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                const switchButton = container.querySelector('[data-testid="switch-account-2"]');
                expect(switchButton).toBeTruthy();
            });
            
            const switchButton = container.querySelector('[data-testid="switch-account-2"]') as HTMLElement;
            await fireEvent.click(switchButton);
            
            expect(mockAccountService.setCurrentAccount).toHaveBeenCalledWith('account-2');
        });

        it('should update UI when account changes via service', async () => {
            const testAccount = createTestAccount();
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            mockAccountService.getCurrentAccount.mockResolvedValueOnce(null);
            
            let accountChangeCallback: (account: Account | null) => void;
            mockAccountService.onAccountChange.mockImplementationOnce((callback) => {
                accountChangeCallback = callback;
            });
            
            const { container } = render(AccountManager);
            
            // Simulate account change from service
            await waitFor(() => {
                accountChangeCallback!(testAccount);
            });
            
            await waitFor(() => {
                expect(container.querySelector('.account-active')).toBeTruthy();
            });
        });
    });

    describe('Account Management', () => {
        it('should show account details modal', async () => {
            const testAccount = createTestAccount();
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                const detailsButton = container.querySelector('[data-testid="account-details-test-account-1"]');
                expect(detailsButton).toBeTruthy();
            });
            
            const detailsButton = container.querySelector('[data-testid="account-details-test-account-1"]') as HTMLElement;
            await fireEvent.click(detailsButton);
            
            await waitFor(() => {
                expect(container.textContent).toContain('Account Details');
                expect(container.textContent).toContain(testAccount.address);
            });
        });

        it('should edit account name', async () => {
            const testAccount = createTestAccount();
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            mockAccountService.updateAccount.mockResolvedValueOnce({
                ...testAccount,
                name: 'Updated Name'
            });
            
            const { container } = render(AccountManager);
            
            // Open details modal
            const detailsButton = container.querySelector('[data-testid="account-details-test-account-1"]') as HTMLElement;
            await fireEvent.click(detailsButton);
            
            // Click edit button
            const editButton = container.querySelector('[data-testid="edit-account-btn"]') as HTMLElement;
            await fireEvent.click(editButton);
            
            // Update name
            const nameInput = container.querySelector('[data-testid="edit-name-input"]') as HTMLInputElement;
            await fireEvent.input(nameInput, { target: { value: 'Updated Name' } });
            
            // Save changes
            const saveButton = container.querySelector('[data-testid="save-changes-btn"]') as HTMLElement;
            await fireEvent.click(saveButton);
            
            expect(mockAccountService.updateAccount).toHaveBeenCalledWith(
                expect.objectContaining({
                    name: 'Updated Name'
                })
            );
        });

        it('should remove account with confirmation', async () => {
            const testAccount = createTestAccount();
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            // Open details modal
            const detailsButton = container.querySelector('[data-testid="account-details-test-account-1"]') as HTMLElement;
            await fireEvent.click(detailsButton);
            
            // Click remove button
            const removeButton = container.querySelector('[data-testid="remove-account-btn"]') as HTMLElement;
            await fireEvent.click(removeButton);
            
            // Confirm removal
            await waitFor(() => {
                expect(container.textContent).toContain('Are you sure you want to remove this account?');
            });
            
            const confirmButton = container.querySelector('[data-testid="confirm-remove-btn"]') as HTMLElement;
            await fireEvent.click(confirmButton);
            
            expect(mockAccountService.removeAccount).toHaveBeenCalledWith('test-account-1');
        });
    });

    describe('Keystore Integration', () => {
        it('should show import keystore option', async () => {
            mockAccountService.getAccounts.mockResolvedValueOnce([]);
            
            const { container } = render(AccountManager);
            
            const importButton = container.querySelector('[data-testid="import-keystore-btn"]');
            expect(importButton).toBeTruthy();
        });

        it('should import keystore file', async () => {
            mockAccountService.getAccounts.mockResolvedValueOnce([]);
            const importedAccount = createTestAccount({ name: 'Imported Account' });
            mockKeystoreManager.importKeystore.mockResolvedValueOnce({
                account: importedAccount,
                keystore: { id: 'imported', keys: 'encrypted' }
            });
            
            const { container } = render(AccountManager);
            
            const importButton = container.querySelector('[data-testid="import-keystore-btn"]') as HTMLElement;
            await fireEvent.click(importButton);
            
            // Mock file input
            const fileInput = container.querySelector('[data-testid="keystore-file-input"]') as HTMLInputElement;
            const mockFile = new File(['{"test": "keystore"}'], 'keystore.json', { type: 'application/json' });
            
            Object.defineProperty(fileInput, 'files', {
                value: [mockFile],
                writable: false,
            });
            
            await fireEvent.change(fileInput);
            
            const passwordInput = container.querySelector('[data-testid="keystore-password-input"]') as HTMLInputElement;
            await fireEvent.input(passwordInput, { target: { value: 'test-password' } });
            
            const importSubmitButton = container.querySelector('[data-testid="import-submit-btn"]') as HTMLElement;
            await fireEvent.click(importSubmitButton);
            
            await waitFor(() => {
                expect(mockKeystoreManager.importKeystore).toHaveBeenCalled();
            });
        });

        it('should export account keystore', async () => {
            const testAccount = createTestAccount();
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            mockKeystoreManager.exportKeystore.mockResolvedValueOnce('exported-keystore-data');
            
            const { container } = render(AccountManager);
            
            // Open account details
            const detailsButton = container.querySelector('[data-testid="account-details-test-account-1"]') as HTMLElement;
            await fireEvent.click(detailsButton);
            
            // Click export button
            const exportButton = container.querySelector('[data-testid="export-keystore-btn"]') as HTMLElement;
            await fireEvent.click(exportButton);
            
            expect(mockKeystoreManager.exportKeystore).toHaveBeenCalledWith(testAccount.id);
        });
    });

    describe('Balance Display', () => {
        it('should format ETH balance correctly', async () => {
            const testAccount = createTestAccount({
                balance: '1500000000000000000', // 1.5 ETH
                blockchain: 'ethereum'
            });
            
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.textContent).toContain('1.5 ETH');
            });
        });

        it('should format SOL balance correctly', async () => {
            const testAccount = createTestAccount({
                balance: '2500000000', // 2.5 SOL (9 decimals)
                blockchain: 'solana'
            });
            
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.textContent).toContain('2.5 SOL');
            });
        });

        it('should handle zero balance', async () => {
            const testAccount = createTestAccount({
                balance: '0',
                blockchain: 'ethereum'
            });
            
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.textContent).toContain('0 ETH');
            });
        });

        it('should handle very large balances', async () => {
            const testAccount = createTestAccount({
                balance: '1000000000000000000000', // 1000 ETH
                blockchain: 'ethereum'
            });
            
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.textContent).toContain('1,000 ETH');
            });
        });
    });

    describe('Error Handling', () => {
        it('should handle service errors gracefully', async () => {
            mockAccountService.getAccounts.mockRejectedValueOnce(new Error('Service unavailable'));
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                expect(container.textContent).toContain('Error loading accounts');
            });
        });

        it('should handle network errors during account operations', async () => {
            const testAccount = createTestAccount();
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            mockAccountService.updateAccount.mockRejectedValueOnce(new Error('Network error'));
            
            const { container } = render(AccountManager);
            
            // Try to update account
            const detailsButton = container.querySelector('[data-testid="account-details-test-account-1"]') as HTMLElement;
            await fireEvent.click(detailsButton);
            
            const editButton = container.querySelector('[data-testid="edit-account-btn"]') as HTMLElement;
            await fireEvent.click(editButton);
            
            const nameInput = container.querySelector('[data-testid="edit-name-input"]') as HTMLInputElement;
            await fireEvent.input(nameInput, { target: { value: 'New Name' } });
            
            const saveButton = container.querySelector('[data-testid="save-changes-btn"]') as HTMLElement;
            await fireEvent.click(saveButton);
            
            await waitFor(() => {
                expect(container.textContent).toContain('Failed to update account');
            });
        });

        it('should handle invalid keystore files', async () => {
            mockAccountService.getAccounts.mockResolvedValueOnce([]);
            mockKeystoreManager.importKeystore.mockRejectedValueOnce(new Error('Invalid keystore format'));
            
            const { container } = render(AccountManager);
            
            const importButton = container.querySelector('[data-testid="import-keystore-btn"]') as HTMLElement;
            await fireEvent.click(importButton);
            
            const fileInput = container.querySelector('[data-testid="keystore-file-input"]') as HTMLInputElement;
            const mockFile = new File(['invalid data'], 'invalid.txt', { type: 'text/plain' });
            
            Object.defineProperty(fileInput, 'files', {
                value: [mockFile],
                writable: false,
            });
            
            await fireEvent.change(fileInput);
            
            const passwordInput = container.querySelector('[data-testid="keystore-password-input"]') as HTMLInputElement;
            await fireEvent.input(passwordInput, { target: { value: 'password' } });
            
            const importSubmitButton = container.querySelector('[data-testid="import-submit-btn"]') as HTMLElement;
            await fireEvent.click(importSubmitButton);
            
            await waitFor(() => {
                expect(container.textContent).toContain('Invalid keystore format');
            });
        });
    });

    describe('Accessibility', () => {
        it('should have proper ARIA labels', async () => {
            const testAccount = createTestAccount();
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                const accountItem = container.querySelector('[role="listitem"]');
                expect(accountItem).toBeTruthy();
                expect(accountItem?.getAttribute('aria-label')).toContain('Test Account');
            });
        });

        it('should support keyboard navigation', async () => {
            const testAccount = createTestAccount();
            mockAccountService.getAccounts.mockResolvedValueOnce([testAccount]);
            
            const { container } = render(AccountManager);
            
            await waitFor(() => {
                const switchButton = container.querySelector('[data-testid="switch-account-test-account-1"]') as HTMLElement;
                expect(switchButton.tabIndex).toBeGreaterThanOrEqual(0);
            });
        });
    });
});