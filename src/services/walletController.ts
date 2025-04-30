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

    // Network Service Proxies
    public async getNetworks() {
        return this.networkService.getNetworks();
    }

    public async getCurrentNetwork() {
        return this.networkService.getCurrentNetwork();
    }

    public async addNetwork(network: any) {
        return this.networkService.addNetwork(network);
    }

    public async removeNetwork(networkId: number) {
        return this.networkService.removeNetwork(networkId);
    }

    public async setCurrentNetwork(networkId: number) {
        return this.networkService.setCurrentNetwork(networkId);
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

    public async removeAccount(address: string) {
        return this.accountService.removeAccount(address);
    }

    public async setCurrentAccount(address: string) {
        return this.accountService.setCurrentAccount(address);
    }

    // Wallet Client Proxies
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
