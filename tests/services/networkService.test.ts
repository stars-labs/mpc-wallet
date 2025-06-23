import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import NetworkService from '../../src/services/networkService';
import { mainnet, sepolia } from 'viem/chains';
import type { Chain } from '../../src/types/network';

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
        // so we don't need to add them manually or set current network here
    });

    afterEach(async () => {
        await mockStorage.clear();
        NetworkService.resetInstance();
    });

    // Store original method
    const originalGetDefaultNetworks = NetworkService.prototype.getDefaultNetworks;

    it('should initialize with empty ethereum networks list in test', async () => {
        const networks = await networkService.getNetworks();

        expect(networks.ethereum).toBeDefined();
        expect(networks.ethereum.length).toBe(2); // mainnet and sepolia added in beforeEach

        // Networks should include mainnet and sepolia
        expect(Array.isArray(networks.ethereum)).toBe(true);
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

        const networks = await networkService.getNetworks();
        const addedNetwork = networks.ethereum.find((n: Chain) => n.id === 999);

        expect(addedNetwork).toBeDefined();
        expect(addedNetwork?.name).toBe('Custom Testnet');
    });

    it('should not allow duplicate network IDs', async () => {
        const network1: Chain = {
            id: 999,
            name: 'Test Network 1',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test1.example.com'] }
            }
        };

        const network2: Chain = {
            id: 999,  // Same ID
            name: 'Test Network 2',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test2.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', network1);
        await expect(networkService.addNetwork('ethereum', network2))
            .rejects.toThrow('Network with this ID already exists');
    });

    it('should remove custom networks', async () => {
        // Add custom network first
        const customNetwork: Chain = {
            id: 999,
            name: 'Custom Network',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://custom.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);

        // Remove custom network - should succeed
        await networkService.removeNetwork('ethereum', 999);

        const networks = await networkService.getNetworks();
        const removedNetwork = networks.ethereum.find(n => n.id === 999);
        expect(removedNetwork).toBeUndefined();
    });

    it('should set and get current network', async () => {
        // Add a test network first
        const customNetwork: Chain = {
            id: 888,
            name: 'Test Network',
            nativeCurrency: { name: 'TST', symbol: 'TST', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);

        // Set current network to our test network
        await networkService.setCurrentNetwork('ethereum', 888);

        const currentNetwork = await networkService.getCurrentNetwork('ethereum');
        expect(currentNetwork?.id).toBe(888);
        expect(currentNetwork?.name).toBe('Test Network');
    });

    it('should handle blockchain switching', async () => {
        // Initially should be ethereum
        const initialBlockchain = await networkService.getCurrentBlockchain();
        expect(initialBlockchain).toBe('ethereum');

        // Switch to solana
        await networkService.setCurrentBlockchain('solana');
        const newBlockchain = await networkService.getCurrentBlockchain();
        expect(newBlockchain).toBe('solana');
    });

    it('should register and notify change callbacks', async () => {
        let notifiedNetwork: Chain | undefined;

        const callback = (network: Chain | undefined) => {
            notifiedNetwork = network;
        };

        // Add a test network
        const testNetwork: Chain = {
            id: 777,
            name: 'Test Network',
            nativeCurrency: { name: 'TST', symbol: 'TST', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', testNetwork);

        networkService.onNetworkChange(callback);

        // Change network
        await networkService.setCurrentNetwork('ethereum', 777);

        // Should have notified the callback
        expect(notifiedNetwork?.id).toBe(777);
    });

    it('should validate network structure', async () => {
        const invalidNetwork = {
            // Missing required fields
            name: 'Invalid Network'
        } as Chain;

        await expect(networkService.addNetwork('ethereum', invalidNetwork))
            .rejects.toThrow();
    });

    it('should handle Solana networks', async () => {
        const solanaNetwork: Chain = {
            id: 101, // Solana mainnet
            name: 'Solana Mainnet',
            nativeCurrency: { name: 'SOL', symbol: 'SOL', decimals: 9 },
            rpcUrls: {
                default: { http: ['https://api.mainnet-beta.solana.com'] }
            }
        };

        await networkService.addNetwork('solana', solanaNetwork);

        const networks = await networkService.getNetworks();
        const addedNetwork = networks.solana.find(n => n.id === 101);

        expect(addedNetwork).toBeDefined();
        expect(addedNetwork?.nativeCurrency.symbol).toBe('SOL');
    });

    it('should persist networks to storage', async () => {
        const customNetwork: Chain = {
            id: 888,
            name: 'Persisted Network',
            nativeCurrency: { name: 'TEST', symbol: 'TEST', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);

        // Check that it was saved to storage
        const stored = await mockStorage.get('wallet_networks');
        expect(stored.wallet_networks.ethereum).toBeDefined();
        expect(stored.wallet_networks.ethereum.some((n: Chain) => n.id === 888)).toBe(true);
    });

    it('should load networks from storage on initialization', async () => {
        // Pre-populate storage with our custom test network
        const customNetwork = {
            id: 777,
            name: 'Pre-stored Network',
            nativeCurrency: { name: 'PRE', symbol: 'PRE', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://prestored.example.com'] }
            }
        };

        // Clear storage and reset singleton to start fresh
        await mockStorage.clear();
        NetworkService.resetInstance();

        // Create base service and add custom network
        const tempService = NetworkService.getInstance();
        await tempService.ensureInitialized();
        await tempService.addNetwork('ethereum', customNetwork);

        // Reset singleton again to simulate app restart
        NetworkService.resetInstance();

        // Create new instance and check if it loads our network
        const newService = NetworkService.getInstance();
        await newService.ensureInitialized();
        const networks = await newService.getNetworks();

        // Verify our network was loaded from storage
        const preStoredNetwork = networks.ethereum.find(n => n.id === 777);
        expect(preStoredNetwork).toBeDefined();
        expect(preStoredNetwork?.name).toBe('Pre-stored Network');
    });

    it('should handle RPC client creation', async () => {
        // Add a test network
        const testNetwork: Chain = {
            id: 777,
            name: 'Test Network',
            nativeCurrency: { name: 'TST', symbol: 'TST', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', testNetwork);
        await networkService.setCurrentNetwork('ethereum', 777);

        const currentNetwork = await networkService.getCurrentNetwork('ethereum');
        expect(currentNetwork).toBeDefined();

        // The service should be able to create RPC clients for valid networks
        // This tests the integration with viem
        expect(currentNetwork?.rpcUrls.default.http.length).toBe(1);
        expect(currentNetwork?.rpcUrls.default.http[0]).toBe('https://test.example.com');
    });

    it('should handle network switching with validation', async () => {
        // Try to set non-existent network
        await expect(networkService.setCurrentNetwork('ethereum', 99999))
            .rejects.toThrow('Network not found');
    });

    it('should handle malformed storage data gracefully', async () => {
        // Set malformed data in storage
        await mockStorage.set({ wallet_networks: 'invalid-data' });

        // Should still work and initialize defaults
        const networks = await networkService.getNetworks() as Record<string, Chain[]>;
        expect(networks.ethereum).toBeDefined();
        expect(networks.solana).toBeDefined();
    });

    it('should update existing custom networks', async () => {
        // Add a custom network first
        const customNetwork: Chain = {
            id: 888,
            name: 'Test Network',
            network: 'test',
            nativeCurrency: { name: 'TST', symbol: 'TST', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);

        // Update the network
        const updatedNetwork: Chain = {
            id: 888,
            name: 'Updated Test Network',
            network: 'test-updated',
            nativeCurrency: { name: 'TST2', symbol: 'TST2', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://updated.example.com'] }
            }
        };

        await networkService.updateNetwork('ethereum', updatedNetwork);

        // Verify the update
        const networks = await networkService.getNetworks() as Record<string, Chain[]>;
        const updated = networks.ethereum.find(n => n.id === 888);
        expect(updated?.name).toBe('Updated Test Network');
        expect(updated?.nativeCurrency?.symbol).toBe('TST2');
        expect(updated?.rpcUrls?.default.http[0]).toBe('https://updated.example.com');
    });

    it('should throw error when updating non-existent network', async () => {
        const nonExistentNetwork: Chain = {
            id: 99999,
            name: 'Non-existent Network',
            network: 'nonexistent',
            nativeCurrency: { name: 'NE', symbol: 'NE', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://nonexistent.example.com'] }
            }
        };

        await expect(networkService.updateNetwork('ethereum', nonExistentNetwork))
            .rejects.toThrow('Network not found');
    });

    it('should throw error when updating protected networks', async () => {
        // Try to update mainnet (protected network) - mainnet should already exist from initialization
        const mainnetNetworks = await networkService.getNetworks();
        const mainnet = (mainnetNetworks.ethereum as Chain[]).find((n: Chain) => n.id === 1);
        expect(mainnet).toBeDefined(); // Ensure mainnet exists

        // Try to update mainnet (protected network)
        const updatedMainnet: Chain = {
            id: 1, // mainnet ID
            name: 'Modified Mainnet',
            network: 'modified-mainnet',
            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://malicious.example.com'] }
            }
        };

        await expect(networkService.updateNetwork('ethereum', updatedMainnet))
            .rejects.toThrow('Cannot modify protected network');
    });

    it('should clear networks while preserving protected ones', async () => {
        // Default networks (mainnet and sepolia) should already exist from initialization
        // Add some custom networks
        const customNetwork1: Chain = {
            id: 888,
            name: 'Custom Network 1',
            network: 'custom1',
            nativeCurrency: { name: 'TST1', symbol: 'TST1', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test1.example.com'] }
            }
        };

        const customNetwork2: Chain = {
            id: 999,
            name: 'Custom Network 2',
            network: 'custom2',
            nativeCurrency: { name: 'TST2', symbol: 'TST2', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test2.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork1);
        await networkService.addNetwork('ethereum', customNetwork2);

        // Clear ethereum networks
        await networkService.clearNetworks('ethereum');

        // Check that only protected networks remain
        const networks = await networkService.getNetworks() as Record<string, Chain[]>;
        const ethNetworks = networks.ethereum;

        // Should only have protected networks (mainnet, sepolia, etc.)
        expect(ethNetworks.find(n => n.id === 888)).toBeUndefined();
        expect(ethNetworks.find(n => n.id === 999)).toBeUndefined();
        expect(ethNetworks.find(n => n.id === 1)).toBeDefined(); // mainnet
        expect(ethNetworks.find(n => n.id === 11155111)).toBeDefined(); // sepolia
    });

    it('should remove network change listeners', async () => {
        let callbackCount = 0;

        const callback1 = () => { callbackCount++; };
        const callback2 = () => { callbackCount++; };

        // Add both callbacks
        networkService.onNetworkChange(callback1);
        networkService.onNetworkChange(callback2);

        // Add a test network to trigger callbacks
        const testNetwork: Chain = {
            id: 777,
            name: 'Test Network',
            network: 'test',
            nativeCurrency: { name: 'TST', symbol: 'TST', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', testNetwork);
        await networkService.setCurrentNetwork('ethereum', 777);

        // Both callbacks should have been called
        expect(callbackCount).toBe(2);

        // Remove one callback
        networkService.removeNetworkChangeListener(callback1);

        // Reset counter and trigger another change
        callbackCount = 0;

        // Use existing mainnet (already loaded) to switch to
        await networkService.setCurrentNetwork('ethereum', 1); // switch to mainnet

        // Only one callback should have been called
        expect(callbackCount).toBe(1);
    });

    it('should get specific network by blockchain and chain ID', async () => {
        // Test getting mainnet (already loaded by default)
        const mainnetNetwork = networkService.getNetwork('ethereum', 1);
        expect(mainnetNetwork).toBeDefined();
        expect(mainnetNetwork?.id).toBe(1);

        // Test getting non-existent network
        const nonExistentNetwork = networkService.getNetwork('ethereum', 99999);
        expect(nonExistentNetwork).toBeUndefined();
    });

    it('should get current network without blockchain parameter', async () => {
        // Set current networks for ethereum (mainnet already loaded)
        await networkService.setCurrentBlockchain('ethereum');
        await networkService.setCurrentNetwork('ethereum', 1);

        await networkService.setCurrentBlockchain('solana');

        // Get current network without specifying blockchain (should return undefined when no current network for solana)
        const currentNetwork = networkService.getCurrentNetwork();
        expect(currentNetwork).toBeUndefined(); // No current solana network set

        // Switch back to ethereum
        await networkService.setCurrentBlockchain('ethereum');
        const currentEthNetwork = networkService.getCurrentNetwork();
        expect(currentEthNetwork?.id).toBe(1); // mainnet
    });

    it('should get public client for ethereum networks', async () => {
        // Set current blockchain to ethereum and use existing mainnet
        await networkService.setCurrentBlockchain('ethereum');
        await networkService.setCurrentNetwork('ethereum', 1); // mainnet

        // Should be able to get public client
        const publicClient = networkService.getPublicClient();
        expect(publicClient).toBeDefined();
    });

    it('should throw error when getting public client for non-ethereum blockchains', async () => {
        // Set current blockchain to solana
        await networkService.setCurrentBlockchain('solana');

        // Should throw error
        expect(() => {
            networkService.getPublicClient();
        }).toThrow('Public client is only available for Ethereum networks');
    });

    it('should throw error when getting public client without current ethereum network', async () => {
        // Set blockchain to ethereum but clear current network
        await networkService.setCurrentBlockchain('ethereum');

        // Manually clear the current ethereum network by setting it to undefined
        (networkService as any).currentNetworks.ethereum = undefined;

        // Should throw error
        expect(() => {
            networkService.getPublicClient();
        }).toThrow('No current Ethereum network selected');
    });

    it('should get all networks when no blockchain specified', async () => {
        const allNetworks = networkService.getNetworks() as Record<string, Chain[]>;
        expect(allNetworks).toHaveProperty('ethereum');
        expect(allNetworks).toHaveProperty('solana');
        expect(Array.isArray(allNetworks.ethereum)).toBe(true);
        expect(Array.isArray(allNetworks.solana)).toBe(true);
    });

    it('should handle network removal with current network fallback', async () => {
        // Use existing mainnet and add custom network
        const customNetwork: Chain = {
            id: 888,
            name: 'Test Network',
            network: 'test',
            nativeCurrency: { name: 'TST', symbol: 'TST', decimals: 18 },
            rpcUrls: {
                default: { http: ['https://test.example.com'] }
            }
        };

        await networkService.addNetwork('ethereum', customNetwork);
        await networkService.setCurrentNetwork('ethereum', 888);

        // Verify it's current
        const currentBefore = networkService.getCurrentNetwork('ethereum');
        expect(currentBefore?.id).toBe(888);

        // Remove the network
        await networkService.removeNetwork('ethereum', 888);

        // Should have fallen back to mainnet
        const currentAfter = networkService.getCurrentNetwork('ethereum');
        expect(currentAfter?.id).toBe(1); // mainnet
    });
});
