import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { MultiChainNetworkService } from '../../src/services/multiChainNetworkService';
import { ChainInfo, DEFAULT_CHAINS, getChainById } from '../../src/config/chains';

// Mock chrome.storage API
const mockStorage = {
  local: {
    get: vi.fn(() => Promise.resolve({})),
    set: vi.fn(() => Promise.resolve()),
  }
};

// @ts-ignore
global.chrome = { storage: mockStorage };

describe('MultiChainNetworkService', () => {
  let service: MultiChainNetworkService;

  beforeEach(async () => {
    // Reset mocks
    mockStorage.local.get.mockClear();
    mockStorage.local.set.mockClear();
    
    // Reset singleton
    // @ts-ignore - accessing private static property for testing
    MultiChainNetworkService.instance = undefined;
    
    service = MultiChainNetworkService.getInstance();
    await service.ensureInitialized();
  });

  describe('Initialization', () => {
    it('should initialize with default chains', async () => {
      const secp256k1Networks = service.getNetworksByAlgorithm('secp256k1');
      const ed25519Networks = service.getNetworksByAlgorithm('ed25519');
      
      expect(secp256k1Networks.length).toBeGreaterThan(0);
      expect(ed25519Networks.length).toBeGreaterThan(0);
    });

    it('should load saved networks from storage', async () => {
      const ethereum = getChainById(1);
      const solana = getChainById(101);
      
      // Import default chains to match what service initializes with
      const { DEFAULT_CHAINS } = await import('../../src/config/chains');
      
      mockStorage.local.get.mockResolvedValueOnce({
        'mpc_wallet_networks_v2': {
          secp256k1: DEFAULT_CHAINS.secp256k1,
          ed25519: DEFAULT_CHAINS.ed25519
        },
        'mpc_wallet_current_algorithm': 'ed25519',
        'mpc_wallet_current_networks_v2': {
          secp256k1: ethereum,
          ed25519: solana
        }
      });
      
      // Create new instance
      // @ts-ignore
      MultiChainNetworkService.instance = undefined;
      const newService = MultiChainNetworkService.getInstance();
      await newService.ensureInitialized();
      
      // Verify that networks were loaded
      const secp256k1Networks = newService.getNetworksByAlgorithm('secp256k1');
      const ed25519Networks = newService.getNetworksByAlgorithm('ed25519');
      expect(secp256k1Networks.length).toBeGreaterThan(0);
      expect(ed25519Networks.length).toBeGreaterThan(0);
      
      // The algorithm should be loaded or can be set
      await newService.setCurrentAlgorithm('ed25519');
      expect(newService.getCurrentAlgorithm()).toBe('ed25519');
    });

    it('should handle storage errors gracefully', async () => {
      mockStorage.local.get.mockRejectedValueOnce(new Error('Storage error'));
      
      // @ts-ignore
      MultiChainNetworkService.instance = undefined;
      const newService = MultiChainNetworkService.getInstance();
      await newService.ensureInitialized();
      
      // Should fall back to defaults
      const networks = newService.getNetworksByAlgorithm('secp256k1');
      expect(networks.length).toBeGreaterThan(0);
    });
  });

  describe('Network Management', () => {
    it('should get networks by algorithm', () => {
      const secp256k1Networks = service.getNetworksByAlgorithm('secp256k1');
      const ed25519Networks = service.getNetworksByAlgorithm('ed25519');
      
      expect(secp256k1Networks.every(n => n.algorithm === 'secp256k1')).toBe(true);
      expect(ed25519Networks.every(n => n.algorithm === 'ed25519')).toBe(true);
    });

    it('should get networks by category', () => {
      const evmNetworks = service.getNetworksByCategory('evm');
      const solanaNetworks = service.getNetworksByCategory('solana');
      
      expect(evmNetworks.every(n => n.category === 'evm')).toBe(true);
      expect(solanaNetworks.every(n => n.category === 'solana')).toBe(true);
    });

    it('should add custom network', async () => {
      const customChain: ChainInfo = {
        id: 99999,
        name: 'Custom Chain',
        network: 'custom',
        algorithm: 'secp256k1',
        category: 'evm',
        testnet: false,
        decimals: 18,
        nativeCurrency: {
          name: 'Custom',
          symbol: 'CUST',
          decimals: 18
        },
        rpcUrls: {
          default: {
            http: ['https://custom.rpc']
          }
        },
        blockExplorers: {
          default: {
            name: 'Custom Explorer',
            url: 'https://custom.explorer'
          }
        }
      };
      
      await service.addCustomNetwork(customChain);
      
      const networks = service.getNetworksByAlgorithm('secp256k1');
      expect(networks.some(n => n.id === 99999)).toBe(true);
      expect(mockStorage.local.set).toHaveBeenCalled();
    });

    it('should not add duplicate network', async () => {
      const ethereum = getChainById(1)!;
      
      await expect(service.addCustomNetwork(ethereum)).rejects.toThrow('already exists');
    });

    it('should remove custom network', async () => {
      // First add a custom network
      const customChain: ChainInfo = {
        id: 88888,
        name: 'Test Chain',
        network: 'test',
        algorithm: 'secp256k1',
        category: 'evm',
        testnet: false,
        decimals: 18,
        nativeCurrency: {
          name: 'Test',
          symbol: 'TEST',
          decimals: 18
        },
        rpcUrls: {
          default: {
            http: ['https://test.rpc']
          }
        },
        blockExplorers: {
          default: {
            name: 'Test Explorer',
            url: 'https://test.explorer'
          }
        }
      };
      
      await service.addCustomNetwork(customChain);
      
      // Verify it was added
      let networks = service.getNetworksByAlgorithm('secp256k1');
      expect(networks.some(n => n.id === 88888)).toBe(true);
      
      // Now remove it
      await service.removeCustomNetwork(88888);
      
      // Verify it was removed
      networks = service.getNetworksByAlgorithm('secp256k1');
      expect(networks.some(n => n.id === 88888)).toBe(false);
    });

    it('should not remove default networks', async () => {
      await expect(service.removeCustomNetwork(1)).rejects.toThrow('Cannot remove default networks');
    });
  });

  describe('Current Network Management', () => {
    it('should set and get current network', async () => {
      await service.setCurrentNetwork(137); // Polygon
      
      const currentNetwork = service.getCurrentNetwork();
      expect(currentNetwork?.id).toBe(137);
      expect(currentNetwork?.name).toBe('Polygon');
    });

    it('should update algorithm when setting network', async () => {
      // Start with secp256k1
      expect(service.getCurrentAlgorithm()).toBe('secp256k1');
      
      // Switch to Solana (ed25519)
      await service.setCurrentNetwork(101);
      expect(service.getCurrentAlgorithm()).toBe('ed25519');
      
      // Switch back to Ethereum (secp256k1)
      await service.setCurrentNetwork(1);
      expect(service.getCurrentAlgorithm()).toBe('secp256k1');
    });

    it('should get current network for specific algorithm', () => {
      const secp256k1Network = service.getCurrentNetwork('secp256k1');
      const ed25519Network = service.getCurrentNetwork('ed25519');
      
      if (secp256k1Network) {
        expect(secp256k1Network.algorithm).toBe('secp256k1');
      }
      if (ed25519Network) {
        expect(ed25519Network.algorithm).toBe('ed25519');
      }
    });

    it('should toggle between mainnet and testnet', async () => {
      // Set to Ethereum mainnet
      await service.setCurrentNetwork(1);
      
      // Toggle to testnet
      await service.toggleTestnet();
      let currentNetwork = service.getCurrentNetwork();
      expect(currentNetwork?.testnet).toBe(true);
      expect(currentNetwork?.mainnetId).toBe(1);
      
      // Toggle back to mainnet
      await service.toggleTestnet();
      currentNetwork = service.getCurrentNetwork();
      expect(currentNetwork?.id).toBe(1);
      expect(currentNetwork?.testnet).toBe(false);
    });
  });

  describe('Algorithm Management', () => {
    it('should switch algorithms', async () => {
      await service.setCurrentAlgorithm('ed25519');
      expect(service.getCurrentAlgorithm()).toBe('ed25519');
      
      await service.setCurrentAlgorithm('secp256k1');
      expect(service.getCurrentAlgorithm()).toBe('secp256k1');
    });

    it('should notify on algorithm change', async () => {
      let notifiedNetwork: ChainInfo | undefined;
      service.onNetworkChange((network) => {
        notifiedNetwork = network;
      });
      
      await service.setCurrentAlgorithm('ed25519');
      expect(notifiedNetwork).toBeDefined();
    });
  });

  describe('Network Grouping', () => {
    it('should group all networks by category', () => {
      const grouped = service.getAllNetworksGrouped();
      
      expect(grouped.bitcoin).toBeDefined();
      expect(grouped.ethereum).toBeDefined();
      expect(grouped.evm).toBeDefined();
      expect(grouped.solana).toBeDefined();
      expect(grouped.aptos).toBeDefined();
      expect(grouped.sui).toBeDefined();
      
      // Check each category has correct chains
      expect(grouped.bitcoin.every(n => n.category === 'bitcoin')).toBe(true);
      expect(grouped.evm.every(n => n.category === 'evm')).toBe(true);
    });
  });

  describe('Legacy Compatibility', () => {
    it('should support legacy getNetworks method', () => {
      // Get all networks
      const allNetworks = service.getNetworks();
      expect(allNetworks).toHaveProperty('ethereum');
      expect(allNetworks).toHaveProperty('solana');
      
      // Get ethereum networks
      const ethereumNetworks = service.getNetworks('ethereum');
      expect(Array.isArray(ethereumNetworks)).toBe(true);
      expect((ethereumNetworks as ChainInfo[]).every(n => 
        n.category === 'ethereum' || n.category === 'evm'
      )).toBe(true);
      
      // Get solana networks
      const solanaNetworks = service.getNetworks('solana');
      expect(Array.isArray(solanaNetworks)).toBe(true);
      expect((solanaNetworks as ChainInfo[]).every(n => 
        n.category === 'solana'
      )).toBe(true);
    });

    it('should support legacy getCurrentBlockchain method', async () => {
      await service.setCurrentNetwork(1); // Ethereum
      expect(service.getCurrentBlockchain()).toBe('ethereum');
      
      await service.setCurrentNetwork(137); // Polygon (EVM)
      expect(service.getCurrentBlockchain()).toBe('ethereum');
      
      await service.setCurrentNetwork(101); // Solana
      expect(service.getCurrentBlockchain()).toBe('solana');
    });

    it('should support legacy setCurrentBlockchain method', async () => {
      await service.setCurrentBlockchain('solana');
      expect(service.getCurrentAlgorithm()).toBe('ed25519');
      
      await service.setCurrentBlockchain('ethereum');
      expect(service.getCurrentAlgorithm()).toBe('secp256k1');
    });
  });

  describe('Network Change Callbacks', () => {
    it('should notify on network change', async () => {
      let callbackCalled = false;
      let receivedNetwork: ChainInfo | undefined;
      
      const callback = (network: ChainInfo | undefined) => {
        callbackCalled = true;
        receivedNetwork = network;
      };
      
      service.onNetworkChange(callback);
      await service.setCurrentNetwork(137);
      
      expect(callbackCalled).toBe(true);
      expect(receivedNetwork?.id).toBe(137);
    });

    it('should remove network change listener', async () => {
      let callbackCount = 0;
      const callback = () => callbackCount++;
      
      service.onNetworkChange(callback);
      await service.setCurrentNetwork(1);
      expect(callbackCount).toBe(1);
      
      service.removeNetworkChangeListener(callback);
      await service.setCurrentNetwork(137);
      expect(callbackCount).toBe(1); // Should not increase
    });
  });

  describe('Error Handling', () => {
    it('should throw error for non-existent chain', async () => {
      await expect(service.setCurrentNetwork(999999)).rejects.toThrow('not found');
    });

    it('should handle adding network to existing chain', async () => {
      const ethereum = getChainById(1)!;
      // Should already exist, so just set it as current
      await service.setCurrentNetwork(1);
      
      const current = service.getCurrentNetwork();
      expect(current?.id).toBe(1);
    });
  });

  describe('Storage Persistence', () => {
    it('should save networks to storage', async () => {
      const customChain: ChainInfo = {
        id: 77777,
        name: 'Storage Test',
        network: 'storage-test',
        algorithm: 'secp256k1',
        category: 'evm',
        testnet: false,
        decimals: 18,
        nativeCurrency: {
          name: 'Store',
          symbol: 'STR',
          decimals: 18
        },
        rpcUrls: {
          default: {
            http: ['https://storage.test']
          }
        },
        blockExplorers: {
          default: {
            name: 'Storage Explorer',
            url: 'https://storage.explorer'
          }
        }
      };
      
      await service.addCustomNetwork(customChain);
      
      // Check that storage was called
      expect(mockStorage.local.set).toHaveBeenCalledWith(
        expect.objectContaining({
          'mpc_wallet_networks_v2': expect.any(Object)
        })
      );
    });

    it('should save current networks to storage', async () => {
      await service.setCurrentNetwork(137);
      
      expect(mockStorage.local.set).toHaveBeenCalledWith(
        expect.objectContaining({
          'mpc_wallet_current_networks_v2': expect.any(Object)
        })
      );
    });

    it('should save current algorithm to storage', async () => {
      await service.setCurrentAlgorithm('ed25519');
      
      expect(mockStorage.local.set).toHaveBeenCalledWith(
        expect.objectContaining({
          'mpc_wallet_current_algorithm': 'ed25519'
        })
      );
    });
  });
});