import { createWalletClient, WalletClient, createPublicClient, http, PublicClient } from 'viem';
import { mainnet } from 'viem/chains';
import AccountService from './accountService';
import NetworkService from './networkService';

class WalletClientService {
    private static instance: WalletClientService;
    private walletClient: WalletClient;
    private publicClient: PublicClient;
    private accountService: AccountService;
    private networkService: NetworkService;

    private constructor() {
        this.accountService = AccountService.getInstance();
        this.networkService = NetworkService.getInstance();
        this.walletClient = this.initializeWalletClient();
        this.publicClient = this.initializePublicClient();

        // 监听账户变化
        this.accountService.onAccountChange(this.handleAccountChange.bind(this));
        // 监听网络变化
        this.networkService.onNetworkChange(this.handleNetworkChange.bind(this));
    }

    public static getInstance(): WalletClientService {
        if (!WalletClientService.instance) {
            WalletClientService.instance = new WalletClientService();
        }
        return WalletClientService.instance;
    }

    private initializeWalletClient(): WalletClient {
        const currentNetwork = this.networkService.getCurrentNetwork() || mainnet;
        return createWalletClient({
            chain: currentNetwork,
            transport: http(currentNetwork.rpcUrls.default.http[0])
        });
    }

    private initializePublicClient(): PublicClient {
        const currentNetwork = this.networkService.getCurrentNetwork() || mainnet;
        return createPublicClient({
            chain: currentNetwork,
            transport: http(currentNetwork.rpcUrls.default.http[0])
        });
    }

    private handleAccountChange(): void {
        // 当账户变化时，更新 wallet client
        this.walletClient = this.initializeWalletClient();
    }

    private handleNetworkChange(): void {
        // 当网络变化时，更新 wallet client 和 public client
        this.walletClient = this.initializeWalletClient();
        this.publicClient = this.initializePublicClient();
    }

    public getWalletClient(): WalletClient {
        return this.walletClient;
    }

    public getPublicClient(): PublicClient {
        return this.publicClient;
    }

    public async sendTransaction(transaction: any): Promise<string> {
        const currentAccount = this.accountService.getCurrentAccount();
        if (!currentAccount) {
            throw new Error('No account selected');
        }

        return this.walletClient.sendTransaction({
            ...transaction,
            from: currentAccount.address as `0x${string}`
        });
    }

    public async signMessage(message: string): Promise<string> {
        const currentAccount = this.accountService.getCurrentAccount();
        if (!currentAccount) {
            throw new Error('No account selected');
        }

        return this.walletClient.signMessage({
            account: currentAccount.address as `0x${string}`,
            message
        });
    }

    public async signTypedData(typedData: any): Promise<string> {
        const currentAccount = this.accountService.getCurrentAccount();
        if (!currentAccount) {
            throw new Error('No account selected');
        }

        return this.walletClient.signTypedData({
            account: currentAccount.address as `0x${string}`,
            ...typedData
        });
    }

    public async getBalance(address?: string): Promise<bigint> {
        const currentAccount = this.accountService.getCurrentAccount();
        if (!currentAccount) {
            throw new Error('No account selected');
        }

        return this.publicClient.getBalance({
            address: (address || currentAccount.address) as `0x${string}`
        });
    }

    public async getTransactionCount(address?: string): Promise<number> {
        const currentAccount = this.accountService.getCurrentAccount();
        if (!currentAccount) {
            throw new Error('No account selected');
        }

        return this.publicClient.getTransactionCount({
            address: (address || currentAccount.address) as `0x${string}`
        });
    }
}

export default WalletClientService; 