import { Chain, mainnet, sepolia } from 'viem/chains';

type NetworkChangeCallback = (network: Chain | undefined) => void;

class NetworkService {
    private static instance: NetworkService;
    private networks: Chain[] = [];
    private currentNetwork: Chain | undefined;
    private readonly STORAGE_KEY = 'wallet_networks';
    private readonly CURRENT_NETWORK_KEY = 'wallet_current_network';
    private changeCallbacks: NetworkChangeCallback[] = [];

    // 受保护的默认网络 chainId 列表
    private static readonly PROTECTED_NETWORK_CHAIN_IDS: number[] = [mainnet.id, sepolia.id];

    private constructor() {
        this.loadNetworks();
    }

    public static getInstance(): NetworkService {
        if (!NetworkService.instance) {
            NetworkService.instance = new NetworkService();
        }
        return NetworkService.instance;
    }

    private async loadNetworks(): Promise<void> {
        try {
            // 加载存储的网络
            const result = await chrome.storage.local.get(this.STORAGE_KEY);
            this.networks = result[this.STORAGE_KEY] || [];

            // 如果没有网络，添加默认网络
            if (this.networks.length === 0) {
                const defaultNetworks = this.getDefaultNetworks();
                for (const network of defaultNetworks) {
                    await this.addNetwork(network);
                }
            }

            // 加载当前网络
            const currentNetworkResult = await chrome.storage.local.get(this.CURRENT_NETWORK_KEY);
            this.currentNetwork = currentNetworkResult[this.CURRENT_NETWORK_KEY];

            // 如果没有当前网络，设置为以太坊主网
            if (!this.currentNetwork) {
                const mainnetNetwork = this.networks.find(n => n.id === mainnet.id);
                if (mainnetNetwork) {
                    await this.setCurrentNetwork(mainnetNetwork.id);
                }
            }
        } catch (error) {
            console.error('Failed to load networks:', error);
            this.networks = [];
        }
    }

    private getDefaultNetworks(): Chain[] {
        return [mainnet, sepolia];
    }

    private async saveNetworks(): Promise<void> {
        try {
            await chrome.storage.local.set({ [this.STORAGE_KEY]: this.networks });
        } catch (error) {
            console.error('Failed to save networks:', error);
        }
    }

    private async saveCurrentNetwork(): Promise<void> {
        try {
            await chrome.storage.local.set({ [this.CURRENT_NETWORK_KEY]: this.currentNetwork });
        } catch (error) {
            console.error('Failed to save current network:', error);
        }
    }

    public async addNetwork(network: Chain): Promise<void> {
        if (this.networks.some(n => n.id === network.id)) {
            throw new Error('Network already exists');
        }

        this.networks.push(network);
        await this.saveNetworks();
    }

    public async removeNetwork(chainId: number): Promise<void> {
        const network = this.networks.find(n => n.id === chainId);
        if (!network) {
            throw new Error('Network not found');
        }

        // 不能删除受保护的默认网络
        if (NetworkService.PROTECTED_NETWORK_CHAIN_IDS.includes(chainId)) {
            throw new Error('Cannot remove protected network');
        }

        // 如果删除的是当前网络，切换到默认网络
        if (this.currentNetwork?.id === chainId) {
            const mainnetNetwork = this.networks.find(n => n.id === mainnet.id);
            if (mainnetNetwork) {
                await this.setCurrentNetwork(mainnetNetwork.id);
            }
        }

        this.networks = this.networks.filter(n => n.id !== chainId);
        await this.saveNetworks();
    }

    public async updateNetwork(network: Chain): Promise<void> {
        const index = this.networks.findIndex(n => n.id === network.id);
        if (index === -1) {
            throw new Error('Network not found');
        }

        // 不能修改受保护网络的 chainId
        if (NetworkService.PROTECTED_NETWORK_CHAIN_IDS.includes(network.id)) {
            throw new Error('Cannot modify protected network');
        }

        this.networks[index] = network;
        await this.saveNetworks();
    }

    public getNetworks(): Chain[] {
        return this.networks;
    }

    public getNetwork(chainId: number): Chain | undefined {
        return this.networks.find(n => n.id === chainId);
    }

    public getCurrentNetwork(): Chain | undefined {
        return this.currentNetwork;
    }

    public async setCurrentNetwork(chainId: number): Promise<void> {
        const network = this.networks.find(n => n.id === chainId);
        if (!network) {
            // 如果找不到网络，设置为以太坊主网
            const mainnetNetwork = this.networks.find(n => n.id === mainnet.id);
            if (!mainnetNetwork) {
                throw new Error('No default network found');
            }
            this.currentNetwork = mainnetNetwork;
        } else {
            this.currentNetwork = network;
        }
        await this.saveCurrentNetwork();
        this.notifyNetworkChange(this.currentNetwork);
    }

    public async clearNetworks(): Promise<void> {
        // 只保留受保护的默认网络
        this.networks = this.networks.filter(n => NetworkService.PROTECTED_NETWORK_CHAIN_IDS.includes(n.id));
        await this.saveNetworks();
    }

    public onNetworkChange(callback: NetworkChangeCallback): void {
        this.changeCallbacks.push(callback);
    }

    public removeNetworkChangeListener(callback: NetworkChangeCallback): void {
        this.changeCallbacks = this.changeCallbacks.filter(cb => cb !== callback);
    }

    private notifyNetworkChange(network: Chain | undefined): void {
        this.changeCallbacks.forEach(callback => callback(network));
    }
}

export default NetworkService; 