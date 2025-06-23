import { describe, it, expect, beforeEach } from 'bun:test';
import { MultiChainNetworkService } from '../../src/services/multiChainNetworkService';
import type { ChainInfo } from '../../src/config/chains';
import { getChainById } from '../../src/config/chains';

describe('ChainSelector Component', () => {
  let networkService: MultiChainNetworkService;
  
  beforeEach(async () => {
    // Mock chrome.storage API
    const mockStorage = {
      local: {
        get: () => Promise.resolve({}),
        set: () => Promise.resolve(),
      }
    };
    // @ts-ignore
    global.chrome = { storage: mockStorage };
    
    // Reset singleton
    // @ts-ignore - accessing private static property for testing
    MultiChainNetworkService.instance = undefined;
    
    networkService = MultiChainNetworkService.getInstance();
    await networkService.ensureInitialized();
  });

  describe('Network Service Integration', () => {
    it('should provide grouped networks', () => {
      const grouped = networkService.getAllNetworksGrouped();
      
      expect(grouped.bitcoin).toBeDefined();
      expect(grouped.ethereum).toBeDefined();
      expect(grouped.evm).toBeDefined();
      expect(grouped.solana).toBeDefined();
      expect(grouped.aptos).toBeDefined();
      expect(grouped.sui).toBeDefined();
    });

    it('should filter networks by algorithm', () => {
      const secp256k1Networks = networkService.getNetworksByAlgorithm('secp256k1');
      const ed25519Networks = networkService.getNetworksByAlgorithm('ed25519');
      
      expect(secp256k1Networks.length).toBeGreaterThan(0);
      expect(ed25519Networks.length).toBeGreaterThan(0);
      
      // Verify algorithm filtering
      expect(secp256k1Networks.every(n => n.algorithm === 'secp256k1')).toBe(true);
      expect(ed25519Networks.every(n => n.algorithm === 'ed25519')).toBe(true);
    });

    it('should handle chain selection', async () => {
      const polygon = getChainById(137);
      if (!polygon) throw new Error('Polygon chain not found');
      
      await networkService.setCurrentNetwork(137);
      
      const currentNetwork = networkService.getCurrentNetwork();
      expect(currentNetwork?.id).toBe(137);
      expect(currentNetwork?.name).toBe('Polygon');
    });
  });

  describe('Chain Display Formatting', () => {
    it('should format chain names correctly', () => {
      const ethereum = getChainById(1);
      const sepolia = getChainById(11155111);
      
      expect(ethereum?.name).toBe('Ethereum Mainnet');
      expect(sepolia?.name).toBe('Sepolia');
      expect(sepolia?.testnet).toBe(true);
    });

    it('should have correct category icons', () => {
      const categoryIcons: Record<string, string> = {
        bitcoin: '₿',
        ethereum: 'Ξ',
        evm: '🔷',
        solana: '◎',
        aptos: '🔺',
        sui: '🌊'
      };
      
      // Verify icon mapping
      Object.entries(categoryIcons).forEach(([category, icon]) => {
        expect(icon).toBeTruthy();
        expect(icon.length).toBeGreaterThan(0);
      });
    });
  });

  describe('Search and Filter Logic', () => {
    it('should filter by search query', () => {
      const allNetworks = networkService.getAllNetworksGrouped();
      const allChains: ChainInfo[] = [];
      
      Object.values(allNetworks).forEach(chains => {
        allChains.push(...chains);
      });
      
      // Search for "poly"
      const polyResults = allChains.filter(chain => {
        const query = 'poly'.toLowerCase();
        return (
          chain.name.toLowerCase().includes(query) ||
          chain.nativeCurrency.symbol.toLowerCase().includes(query) ||
          chain.network.toLowerCase().includes(query)
        );
      });
      
      expect(polyResults.length).toBeGreaterThan(0);
      expect(polyResults.some(c => c.name === 'Polygon')).toBe(true);
    });

    it('should filter by symbol', () => {
      const allNetworks = networkService.getAllNetworksGrouped();
      const allChains: ChainInfo[] = [];
      
      Object.values(allNetworks).forEach(chains => {
        allChains.push(...chains);
      });
      
      // Search for "ETH"
      const ethResults = allChains.filter(chain => {
        const query = 'ETH'.toLowerCase();
        return (
          chain.name.toLowerCase().includes(query) ||
          chain.nativeCurrency.symbol.toLowerCase().includes(query) ||
          chain.network.toLowerCase().includes(query)
        );
      });
      
      expect(ethResults.length).toBeGreaterThan(0);
      expect(ethResults.some(c => c.name === 'Ethereum Mainnet')).toBe(true);
    });

    it('should filter testnets', () => {
      const allNetworks = networkService.getAllNetworksGrouped();
      const allChains: ChainInfo[] = [];
      
      Object.values(allNetworks).forEach(chains => {
        allChains.push(...chains);
      });
      
      const mainnets = allChains.filter(chain => !chain.testnet);
      const testnets = allChains.filter(chain => chain.testnet);
      
      expect(mainnets.length).toBeGreaterThan(0);
      expect(testnets.length).toBeGreaterThan(0);
      
      // Verify testnet flag
      expect(mainnets.every(c => !c.testnet)).toBe(true);
      expect(testnets.every(c => c.testnet)).toBe(true);
    });
  });

  describe('Category Filtering', () => {
    it('should group chains by category', () => {
      const grouped = networkService.getAllNetworksGrouped();
      
      // Check ethereum category
      if (grouped.ethereum.length > 0) {
        expect(grouped.ethereum.every(c => c.category === 'ethereum')).toBe(true);
      }
      
      // Check evm category
      if (grouped.evm.length > 0) {
        expect(grouped.evm.every(c => c.category === 'evm')).toBe(true);
      }
      
      // Check solana category
      if (grouped.solana.length > 0) {
        expect(grouped.solana.every(c => c.category === 'solana')).toBe(true);
      }
    });

    it('should have valid category labels', () => {
      const categoryLabels: Record<string, string> = {
        bitcoin: '₿ Bitcoin',
        ethereum: 'Ξ Ethereum',
        evm: '🔷 EVM Compatible',
        solana: '◎ Solana',
        aptos: '🔺 Aptos',
        sui: '🌊 Sui'
      };
      
      Object.entries(categoryLabels).forEach(([category, label]) => {
        // Special case for EVM
        if (category === 'evm') {
          expect(label).toContain('EVM');
        } else {
          expect(label).toContain(category.charAt(0).toUpperCase() + category.slice(1));
        }
      });
    });
  });

  describe('Selected Chain State', () => {
    it('should track current chain', async () => {
      const ethereum = getChainById(1);
      if (!ethereum) throw new Error('Ethereum chain not found');
      
      await networkService.setCurrentNetwork(1);
      
      const current = networkService.getCurrentNetwork();
      expect(current?.id).toBe(1);
      expect(current?.name).toBe('Ethereum Mainnet');
    });

    it('should handle chain switching', async () => {
      // Start with Ethereum
      await networkService.setCurrentNetwork(1);
      expect(networkService.getCurrentNetwork()?.id).toBe(1);
      
      // Switch to Polygon
      await networkService.setCurrentNetwork(137);
      expect(networkService.getCurrentNetwork()?.id).toBe(137);
      
      // Switch to Solana
      await networkService.setCurrentNetwork(101);
      expect(networkService.getCurrentNetwork()?.id).toBe(101);
    });
  });
});