import NetworkService from "./networkService";
import AccountService from "./accountService";
import WalletClientService from "./walletClient";

class WalletController {
    private static instance: WalletController;
    private networkService: NetworkService;
    private accountService: AccountService;
    private walletClientService: WalletClientService;

    private constructor() {
        this.networkService = NetworkService.getInstance();
        this.accountService = AccountService.getInstance();
        this.walletClientService = WalletClientService.getInstance();
    }

    public static getInstance(): WalletController {
        if (!WalletController.instance) {
            WalletController.instance = new WalletController();
        }
        return WalletController.instance;
    }

    // Testing utility method to reset singleton instance
    public static resetInstance(): void {
        WalletController.instance = undefined as any;
    }

    // Network Service Proxies
    public async getNetworks() {
        return this.networkService.getNetworks();
    }

    public async getCurrentNetwork() {
        return this.networkService.getCurrentNetwork();
    }

    public async addNetwork(network: any) {
        // The network parameter should include blockchain type
        const blockchain = network.blockchain || 'ethereum';
        return this.networkService.addNetwork(blockchain, network);
    }

    public async removeNetwork(networkId: number) {
        // Default to ethereum blockchain for now
        return this.networkService.removeNetwork('ethereum', networkId);
    }

    public async setCurrentNetwork(networkId: number) {
        // Default to ethereum blockchain for now  
        return this.networkService.setCurrentNetwork('ethereum', networkId);
    }

    // Account Service Proxies
    public async getAccounts() {
        return this.accountService.getAccounts();
    }

    public async getCurrentAccount() {
        return this.accountService.getCurrentAccount();
    }

    public async addAccount(account: any) {
        return this.accountService.addAccount(account);
    }

    public async updateAccount(account: any) {
        return this.accountService.updateAccount(account);
    }

    public async removeAccount(accountId: string) {
        return this.accountService.removeAccount(accountId);
    }

    public async setCurrentAccount(accountId: string) {
        return this.accountService.setCurrentAccount(accountId);
    }

    public async getAccountsByBlockchain(blockchain: 'ethereum' | 'solana') {
        return this.accountService.getAccountsByBlockchain(blockchain);
    }

    // Wallet Client Proxies
    public async connect() {
        // Mock connection for now since wallet client doesn't have these methods
        return { status: 'connected' };
    }

    public async disconnect() {
        // Mock disconnection
        return { status: 'disconnected' };
    }

    public async isConnected() {
        // Mock connection status 
        return false;
    }

    public async sendTransaction(transaction: any) {
        return this.walletClientService.getWalletClient().sendTransaction(transaction);
    }

    public async signMessage(params: { account: `0x${string}`; message: string }) {
        return this.walletClientService.getWalletClient().signMessage(params);
    }

    public async getBalance(address: `0x${string}`) {
        return this.walletClientService.getPublicClient().getBalance({ address });
    }

    public async getTransactionCount(address: `0x${string}`) {
        return this.walletClientService.getPublicClient().getTransactionCount({ address });
    }

    // RPC Method Proxies
    public async eth_accounts() {
        const currentAccount = this.accountService.getCurrentAccount();
        return currentAccount ? [currentAccount.address] : [];
    }

    public async eth_requestAccounts() {
        return this.eth_accounts();
    }

    public async eth_chainId() {
        return this.networkService.getCurrentNetwork()?.id;
    }

    public async net_version() {
        return this.networkService.getCurrentNetwork()?.id.toString();
    }
}

export default WalletController;
