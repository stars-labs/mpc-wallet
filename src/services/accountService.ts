import { Account } from '../types/account';

type AccountChangeCallback = (account: Account | undefined) => void;

class AccountService {
    private static instance: AccountService;
    private accounts: Account[] = [];
    private readonly STORAGE_KEY = 'wallet_accounts';
    private readonly CURRENT_ACCOUNT_KEY = 'wallet_current_account';
    private currentAccountAddress: string | undefined;
    private changeCallbacks: AccountChangeCallback[] = [];

    private constructor() {
        this.loadAccounts();
    }

    public static getInstance(): AccountService {
        if (!AccountService.instance) {
            AccountService.instance = new AccountService();
        }
        return AccountService.instance;
    }

    private async loadAccounts(): Promise<void> {
        try {
            const result = await chrome.storage.local.get([this.STORAGE_KEY, this.CURRENT_ACCOUNT_KEY]);
            this.accounts = result[this.STORAGE_KEY] || [];
            this.currentAccountAddress = result[this.CURRENT_ACCOUNT_KEY];
        } catch (error) {
            console.error('Failed to load accounts:', error);
            this.accounts = [];
        }
    }

    private async saveAccounts(): Promise<void> {
        try {
            await chrome.storage.local.set({ [this.STORAGE_KEY]: this.accounts });
        } catch (error) {
            console.error('Failed to save accounts:', error);
        }
    }

    private async saveCurrentAccount(): Promise<void> {
        try {
            await chrome.storage.local.set({ [this.CURRENT_ACCOUNT_KEY]: this.currentAccountAddress });
        } catch (error) {
            console.error('Failed to save current account:', error);
        }
    }

    public async addAccount(account: Account): Promise<Account> {
        if (this.accounts.length > 0 && this.accounts.some(acc => acc.address === account.address)) {
            throw new Error('Account already exists');
        }
        this.accounts.push(account);
        await this.saveAccounts();
        await this.setCurrentAccount(account.address);
        return account;
    }

    public async removeAccount(address: string): Promise<void> {
        this.accounts = this.accounts.filter(acc => acc.address !== address);
        await this.saveAccounts();
    }

    public async updateAccount(account: Account): Promise<void> {
        const index = this.accounts.findIndex(acc => acc.address === account.address);
        if (index === -1) {
            throw new Error('Account not found');
        }
        this.accounts[index] = account;
        await this.saveAccounts();
    }

    public getAccounts(): Account[] {
        return this.accounts;
    }

    public getAccount(address: string): Account | undefined {
        return this.accounts.find(acc => acc.address === address);
    }

    public getCurrentAccount(): Account | undefined {
        console.log(this);
        if (!this.currentAccountAddress) {
            return undefined;
        }
        return this.accounts.find(acc => acc.address === this.currentAccountAddress);
    }

    public async setCurrentAccount(address: string): Promise<void> {
        const account = this.accounts.find(acc => acc.address === address);
        if (!account) {
            throw new Error('Account not found');
        }
        this.currentAccountAddress = address;
        await this.saveCurrentAccount();
        this.notifyAccountChange(account);
    }

    public async clearAccounts(): Promise<void> {
        this.accounts = [];
        this.currentAccountAddress = undefined;
        await this.saveAccounts();
        await this.saveCurrentAccount();
        this.notifyAccountChange(undefined);
    }

    public onAccountChange(callback: AccountChangeCallback): void {
        this.changeCallbacks.push(callback);
    }

    public removeAccountChangeListener(callback: AccountChangeCallback): void {
        this.changeCallbacks = this.changeCallbacks.filter(cb => cb !== callback);
    }

    private notifyAccountChange(account: Account | undefined): void {
        this.changeCallbacks.forEach(callback => callback(account));
    }
}

export default AccountService; 