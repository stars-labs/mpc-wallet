import { Account } from '../types/account';

type AccountChangeCallback = (account: Account | null) => void;

class AccountService {
    private static instance: AccountService;
    private accounts: Account[] = [];
    private readonly STORAGE_KEY = 'wallet_accounts';
    private readonly CURRENT_ACCOUNT_KEY = 'wallet_current_account';
    private currentAccountAddress: string | undefined;
    private changeCallbacks: AccountChangeCallback[] = [];
    private initialized: boolean = false;

    private constructor() {
        // Don't call async methods in constructor
        this.initializeAsync();
    }

    public static getInstance(): AccountService {
        if (!AccountService.instance) {
            AccountService.instance = new AccountService();
        }
        return AccountService.instance;
    }

    private async initializeAsync(): Promise<void> {
        if (!this.initialized) {
            await this.loadAccounts();
            this.initialized = true;
        }
    }

    public async ensureInitialized(): Promise<void> {
        if (!this.initialized) {
            await this.initializeAsync();
        }
    }

    // Testing utility method to reset singleton instance
    public static resetInstance(): void {
        AccountService.instance = undefined as any;
    }

    private async loadAccounts(): Promise<void> {
        try {
            // In test environment, chrome.storage might not be available
            if (typeof chrome !== 'undefined' && chrome.storage) {
                const result = await chrome.storage.local.get([this.STORAGE_KEY, this.CURRENT_ACCOUNT_KEY]);
                this.accounts = result[this.STORAGE_KEY] || [];
                this.currentAccountAddress = result[this.CURRENT_ACCOUNT_KEY];
            } else {
                // Fallback for test environment
                this.accounts = [];
                this.currentAccountAddress = undefined;
            }
        } catch (error) {
            console.error('Failed to load accounts:', error);
            this.accounts = [];
        }
    }

    private async saveAccounts(): Promise<void> {
        try {
            if (typeof chrome !== 'undefined' && chrome.storage) {
                await chrome.storage.local.set({ [this.STORAGE_KEY]: this.accounts });
            }
        } catch (error) {
            console.error('Failed to save accounts:', error);
        }
    }

    private async saveCurrentAccount(): Promise<void> {
        try {
            if (typeof chrome !== 'undefined' && chrome.storage) {
                await chrome.storage.local.set({ [this.CURRENT_ACCOUNT_KEY]: this.currentAccountAddress });
            }
        } catch (error) {
            console.error('Failed to save current account:', error);
        }
    }

    public async addAccount(account: Account): Promise<Account> {
        // Validate account structure
        if (!account || typeof account !== 'object') {
            throw new Error('Invalid account: must be an object');
        }
        
        if (!account.id || typeof account.id !== 'string') {
            throw new Error('Invalid account: id is required and must be a string');
        }
        
        if (!account.name || typeof account.name !== 'string') {
            throw new Error('Invalid account: name is required and must be a string');
        }
        
        if (!account.address || typeof account.address !== 'string') {
            throw new Error('Invalid account: address is required and must be a string');
        }
        
        if (!account.blockchain || typeof account.blockchain !== 'string') {
            throw new Error('Invalid account: blockchain is required and must be a string');
        }

        // Check for duplicate ID
        if (this.accounts.some(acc => acc.id === account.id)) {
            throw new Error('Account with this ID already exists');
        }

        // Check for duplicate address on same blockchain
        if (this.accounts.some(acc => acc.address === account.address && acc.blockchain === account.blockchain)) {
            throw new Error('Account with this address already exists for this blockchain');
        }

        this.accounts.push(account);
        await this.saveAccounts();
        await this.setCurrentAccount(account.id);
        return account;
    }

    public async removeAccount(accountId: string): Promise<void> {
        const accountIndex = this.accounts.findIndex(acc => acc.id === accountId);
        if (accountIndex === -1) {
            throw new Error('Account not found');
        }

        const removedAccount = this.accounts[accountIndex];
        this.accounts.splice(accountIndex, 1);

        // If this was the current account, clear current account
        if (this.currentAccountAddress === removedAccount.address) {
            this.currentAccountAddress = undefined;
        }

        await this.saveAccounts();
        await this.saveCurrentAccount();
        this.notifyAccountChange(this.getCurrentAccount());
    }

    public async updateAccount(account: Account): Promise<Account> {
        const accountIndex = this.accounts.findIndex(acc => acc.id === account.id);
        if (accountIndex === -1) {
            throw new Error('Account not found');
        }

        this.accounts[accountIndex] = account;
        await this.saveAccounts();
        this.notifyAccountChange(this.getCurrentAccount());
        return account;
    }

    public getAccounts(): Account[] {
        return this.accounts;
    }

    public getAccount(address: string): Account | undefined {
        return this.accounts.find(acc => acc.address === address);
    }

    public getAccountById(id: string): Account | null {
        return this.accounts.find(acc => acc.id === id) || null;
    }

    public getAccountsByBlockchain(blockchain: 'ethereum' | 'solana'): Account[] {
        return this.accounts.filter(acc => acc.blockchain === blockchain);
    }

    public getCurrentAccount(): Account | null {
        if (!this.currentAccountAddress) {
            // If no current account is set but we have accounts, use the first one
            if (this.accounts.length > 0) {
                this.currentAccountAddress = this.accounts[0].address;
                this.saveCurrentAccount().catch(error => {
                    console.error('Failed to save current account:', error);
                });
                return this.accounts[0];
            }
            return null;
        }
        return this.accounts.find(acc => acc.address === this.currentAccountAddress) || null;
    }

    public async ensureDefaultAccount(): Promise<Account | null> {
        // If we already have an account, use it
        if (this.accounts.length > 0) {
            return this.getCurrentAccount();
        }

        try {
            // Try to get address from WebRTC DKG if possible
            const address = await this.getAddressFromDKG();
            if (address) {
                const account: Account = {
                    id: `mpc-${Date.now()}`,
                    address,
                    name: 'MPC Wallet',
                    balance: '0',
                    blockchain: 'ethereum',
                    created: Date.now()
                };
                await this.addAccount(account);
                return account;
            }
        } catch (error) {
            console.error('Failed to create default account:', error);
        }

        return null;
    }

    private async getAddressFromDKG(): Promise<string | null> {
        try {
            // Send message to offscreen document to get Ethereum address
            const response = await chrome.runtime.sendMessage({
                type: "getEthereumAddress"
            });

            if (response && response.success && response.data && response.data.ethereumAddress) {
                return response.data.ethereumAddress;
            }

            return null;
        } catch (error) {
            console.error('Error getting address from DKG:', error);
            return null;
        }
    }

    public async setCurrentAccount(accountId: string): Promise<void> {
        const account = this.accounts.find(acc => acc.id === accountId);
        if (!account) {
            throw new Error('Account not found');
        }
        this.currentAccountAddress = account.address;
        await this.saveCurrentAccount();
        this.notifyAccountChange(account);
    }

    public async clearAccounts(): Promise<void> {
        this.accounts = [];
        this.currentAccountAddress = undefined;
        await this.saveAccounts();
        await this.saveCurrentAccount();
        this.notifyAccountChange(null);
    }

    public onAccountChange(callback: AccountChangeCallback): void {
        this.changeCallbacks.push(callback);
    }

    public removeAccountChangeListener(callback: AccountChangeCallback): void {
        this.changeCallbacks = this.changeCallbacks.filter(cb => cb !== callback);
    }

    private notifyAccountChange(account: Account | null): void {
        this.changeCallbacks.forEach(callback => callback(account));
    }
}

export default AccountService; 