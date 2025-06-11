import { describe, it, expect, beforeEach, afterEach } from 'bun:test';
import NetworkService from './networkService';
import { mainnet, sepolia } from 'viem/chains';
import type { Chain } from '../types/network';

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

describe('NetworkService', () => {
    let networkService: NetworkService;

    beforeEach(async () => {
        // Clear storage before each test
        await mockStorage.clear();
        // Reset singleton instance before each test
        NetworkService.resetInstance();

        // Get fresh instance
        networkService = NetworkService.getInstance();
        // Make sure it's initialized
        await networkService.ensureInitialized();

        // The service will automatically load default networks (mainnet and sepolia)
        // so we don't need to add them manually
    });

    afterEach(async () => {
        await mockStorage.clear();
        NetworkService.resetInstance();
    });

    it('should initialize with default ethereum networks', async () => {
        const networks = await networkService.getNetworks() as Record<string, Chain[]>;

        expect(networks.ethereum).toBeDefined();
        expect(Array.isArray(networks.ethereum)).toBe(true);
        expect(networks.ethereum.length).toBeGreaterThan(0);

        // Should include mainnet and sepolia by default
        expect(networks.ethereum.find((n: Chain) => n.id === 1)).toBeDefined(); // mainnet
        expect(networks.ethereum.find((n: Chain) => n.id === 11155111)).toBeDefined(); // sepolia
    });

    it('should return singleton instance', () => {
        const instance1 = NetworkService.getInstance();
        const instance2 = NetworkService.getInstance();

        expect(instance1).toBe(instance2);
    });

    it('should add custom Ethereum network', async () => {
        const customNetwork: Chain = {
            id: 999,
            name: 'Custom Testnet',
            network: 'custom-testnet',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://custom-testnet.example.com'] }
            },
            blockExplorers: {
                default: { name: 'CustomScan', url: 'https://customscan.example.com' }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);

        const networks = await networkService.getNetworks() as Record<string, Chain[]>;
        const addedNetwork = networks.ethereum.find((n: Chain) => n.id === 999);

        expect(addedNetwork).toBeDefined();
        expect(addedNetwork?.name).toBe('Custom Testnet');
    });

    it('should not allow duplicate network IDs', async () => {
        const network1: Chain = {
            id: 999,
            name: 'Test Network 1',
            network: 'test-network-1',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test1.example.com'] }
            }
        };

        const network2: Chain = {
            id: 999,  // Same ID
            name: 'Test Network 2',
            network: 'test-network-2',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test2.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', network1);

        await expect(networkService.addNetwork('ethereum', network2))
            .rejects.toThrow('Network with this ID already exists');

        const networks = await networkService.getNetworks() as Record<string, Chain[]>;
        const networksWithId999 = networks.ethereum.filter((n: Chain) => n.id === 999);
        expect(networksWithId999.length).toBe(1);
        expect(networksWithId999[0].name).toBe('Test Network 1');
    });

    it('should remove custom networks', async () => {
        const customNetwork: Chain = {
            id: 888,
            name: 'Removable Network',
            network: 'removable-network',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://removable.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);
        await networkService.removeNetwork('ethereum', 888);

        const networks = await networkService.getNetworks() as Record<string, Chain[]>;
        const removedNetwork = networks.ethereum.find((n: Chain) => n.id === 888);

        expect(removedNetwork).toBeUndefined();
    });

    it('should set and get current network', async () => {
        await networkService.setCurrentNetwork('ethereum', 1); // Set to mainnet

        const currentNetwork = await networkService.getCurrentNetwork('ethereum');

        expect(currentNetwork).toBeDefined();
        expect(currentNetwork?.id).toBe(1);
        expect(currentNetwork?.name).toBe('Ethereum');
    });

    it('should handle blockchain switching', async () => {
        await networkService.setCurrentNetwork('ethereum', 1);
        await networkService.setCurrentNetwork('solana', undefined);

        const ethereumCurrent = await networkService.getCurrentNetwork('ethereum');
        const solanaCurrent = await networkService.getCurrentNetwork('solana');

        expect(ethereumCurrent?.id).toBe(1);
        expect(solanaCurrent).toBeUndefined();
    });

    it('should register and notify change callbacks', async () => {
        let notificationCount = 0;
        let lastBlockchain = '';
        let lastNetwork: Chain | undefined;

        const callback = (blockchain: string, network: Chain | undefined) => {
            notificationCount++;
            lastBlockchain = blockchain;
            lastNetwork = network;
        };

        networkService.onNetworkChange(callback);
        await networkService.setCurrentNetwork('ethereum', 1);

        expect(notificationCount).toBe(1);
        expect(lastBlockchain).toBe('ethereum');
        expect(lastNetwork?.id).toBe(1);
    });

    it('should validate network structure', async () => {
        const invalidNetwork = {
            // Missing required fields
        } as Chain;

        await expect(networkService.addNetwork('ethereum', invalidNetwork))
            .rejects.toThrow();
    });

    it('should handle Solana networks', async () => {
        const solanaNetwork: Chain = {
            id: 'mainnet-beta',
            name: 'Solana Mainnet Beta',
            network: 'mainnet-beta',
            nativeCurrency: { name: 'SOL', symbol: 'SOL', decimals: 9 },
            rpcUrls: {
                default: { http: ['https://api.mainnet-beta.solana.com'] }
            }
        };

        await networkService.addNetwork('solana', solanaNetwork);

        const networks = await networkService.getNetworks() as Record<string, Chain[]>;
        const addedNetwork = networks.solana.find((n: Chain) => n.id === 'mainnet-beta');

        expect(addedNetwork).toBeDefined();
        expect(addedNetwork?.name).toBe('Solana Mainnet Beta');
    });

    it('should persist networks to storage', async () => {
        const testNetwork: Chain = {
            id: 888,
            name: 'Test Persistence',
            network: 'test-persistence',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', testNetwork);

        // Check if stored in chrome storage
        const stored = await mockStorage.get('wallet_networks');
        expect(stored.wallet_networks.ethereum.some((n: Chain) => n.id === 888)).toBe(true);
    });

    it('should load networks from storage on initialization', async () => {
        // Create base service and add custom network
        const testNetwork: Chain = {
            id: 777,
            name: 'Persisted Network',
            network: 'persisted-network',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://persisted.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', testNetwork);

        // Create new instance and verify it loads the stored network
        NetworkService.resetInstance();
        const newService = NetworkService.getInstance();
        await newService.ensureInitialized();

        const networks = await newService.getNetworks() as Record<string, Chain[]>;
        const loadedNetwork = networks.ethereum.find((n: Chain) => n.id === 777);

        expect(loadedNetwork).toBeDefined();
        expect(loadedNetwork?.name).toBe('Persisted Network');
    });

    it('should handle malformed storage data gracefully', async () => {
        // Set invalid data in storage
        await mockStorage.set({ 'wallet_networks': 'invalid_data' });

        // Reset and recreate service
        NetworkService.resetInstance();
        const newService = NetworkService.getInstance();
        await newService.ensureInitialized();

        // Should still work with defaults
        const networks = await newService.getNetworks() as Record<string, Chain[]>;
        expect(networks.ethereum).toBeDefined();
        expect(Array.isArray(networks.ethereum)).toBe(true);
    });

    it('should handle RPC client creation', async () => {
        const currentNetwork = await networkService.getCurrentNetwork('ethereum');
        expect(currentNetwork).toBeDefined();
        expect(currentNetwork?.rpcUrls?.default?.http).toBeDefined();
        expect(Array.isArray(currentNetwork?.rpcUrls?.default?.http)).toBe(true);
    });

    it('should handle network switching with validation', async () => {
        // Try to switch to non-existent network
        await expect(networkService.setCurrentNetwork('ethereum', 99999))
            .rejects.toThrow();

        // Valid network switch should work
        await networkService.setCurrentNetwork('ethereum', 1);
        const current = await networkService.getCurrentNetwork('ethereum');
        expect(current?.id).toBe(1);
    });

    it('should update existing custom networks', async () => {
        const originalNetwork: Chain = {
            id: 888,
            name: 'Original Network',
            network: 'original-network',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://original.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', originalNetwork);

        // Update the network
        const updatedNetwork: Chain = {
            id: 888,
            name: 'Updated Network',
            network: 'updated-network',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://updated.example.com'] }
            }
        };

        await networkService.updateNetwork('ethereum', updatedNetwork);

        const networks = await networkService.getNetworks() as Record<string, Chain[]>;
        const network = networks.ethereum.find((n: Chain) => n.id === 888);
        expect(network?.name).toBe('Updated Network');
    });

    it('should throw error when updating non-existent network', async () => {
        const nonExistentNetwork: Chain = {
            id: 99999,
            name: 'Non-existent',
            network: 'non-existent',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://example.com'] }
            }
        };

        await expect(networkService.updateNetwork('ethereum', nonExistentNetwork))
            .rejects.toThrow();
    });

    it('should throw error when updating protected networks', async () => {
        const mainnetUpdate: Chain = {
            id: 1,
            name: 'Modified Mainnet',
            network: 'modified-mainnet',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://modified.example.com'] }
            }
        };

        await expect(networkService.updateNetwork('ethereum', mainnetUpdate))
            .rejects.toThrow();
    });

    it('should clear networks while preserving protected ones', async () => {
        // Add custom network
        const customNetwork: Chain = {
            id: 888,
            name: 'Custom Network',
            network: 'custom-network',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://custom.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);

        // Clear networks
        await networkService.clearCustomNetworks('ethereum');

        const networks = await networkService.getNetworks() as Record<string, Chain[]>;

        // Custom network should be removed
        expect(networks.ethereum.find((n: Chain) => n.id === 888)).toBeUndefined();

        // Protected networks should remain
        expect(networks.ethereum.find((n: Chain) => n.id === 1)).toBeDefined(); // mainnet
        expect(networks.ethereum.find((n: Chain) => n.id === 11155111)).toBeDefined(); // sepolia
    });

    it('should remove network change listeners', async () => {
        let callbackCount = 0;
        const callback = () => callbackCount++;

        networkService.onNetworkChange(callback);
        await networkService.setCurrentNetwork('ethereum', 1);
        expect(callbackCount).toBe(1);

        networkService.offNetworkChange(callback);
        await networkService.setCurrentNetwork('ethereum', 11155111);
        expect(callbackCount).toBe(1); // Should not increment
    });

    it('should get specific network by blockchain and chain ID', async () => {
        const network = await networkService.getNetwork('ethereum', 1);
        expect(network).toBeDefined();
        expect(network?.id).toBe(1);
        expect(network?.name).toBe('Ethereum');
    });

    it('should get current network without blockchain parameter', async () => {
        await networkService.setCurrentNetwork('ethereum', 1);
        const currentNetwork = await networkService.getCurrentNetwork();
        expect(currentNetwork).toBeDefined();
        expect(currentNetwork?.id).toBe(1);
    });

    it('should get public client for ethereum networks', async () => {
        await networkService.setCurrentNetwork('ethereum', 1);
        const client = networkService.getPublicClient();
        expect(client).toBeDefined();
    });

    it('should throw error when getting public client for non-ethereum blockchains', async () => {
        // Add and set a Solana network
        const solanaNetwork: Chain = {
            id: 'mainnet-beta',
            name: 'Solana Mainnet',
            network: 'mainnet-beta',
            nativeCurrency: { name: 'SOL', symbol: 'SOL', decimals: 9 },
            rpcUrls: {
                default: { http: ['https://api.mainnet-beta.solana.com'] }
            }
        };

        await networkService.addNetwork('solana', solanaNetwork);
        await networkService.setCurrentNetwork('solana', 'mainnet-beta');

        expect(() => {
            networkService.getPublicClient();
        }).toThrow('Public client is only available for Ethereum networks');
    });

    it('should throw error when getting public client without current ethereum network', async () => {
        // Clear current network and ensure we're not on ethereum
        await networkService.setCurrentNetwork('solana', undefined);

        // Should throw error
        expect(() => {
            networkService.getPublicClient();
        }).toThrow('No current Ethereum network selected');
    });

    it('should get all networks when no blockchain specified', async () => {
        const allNetworks = await networkService.getNetworks();
        expect(allNetworks).toBeDefined();
        expect(typeof allNetworks).toBe('object');

        const networksAsRecord = allNetworks as Record<string, Chain[]>;
        expect(networksAsRecord.ethereum).toBeDefined();
        expect(networksAsRecord.solana).toBeDefined();
    });

    it('should handle network removal with current network fallback', async () => {
        // Add custom network and set as current
        const customNetwork: Chain = {
            id: 888,
            name: 'Custom Network',
            network: 'custom-network',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://custom.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);
        await networkService.setCurrentNetwork('ethereum', 888);

        // Verify current network is set
        let currentNetwork = await networkService.getCurrentNetwork('ethereum');
        expect(currentNetwork?.id).toBe(888);

        // Remove the custom network
        await networkService.removeNetwork('ethereum', 888);

        // Should fallback to mainnet
        currentNetwork = await networkService.getCurrentNetwork('ethereum');
        expect(currentNetwork?.id).toBe(1); // mainnet
    });
});
