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
        set: () => Promise.resolve()
      }
    };
    
    global.chrome = { storage: mockStorage } as any;
    
    // Reset singleton
    // @ts-ignore - accessing private static property for testing
    MultiChainNetworkService.instance = undefined;
    
    networkService = MultiChainNetworkService.getInstance();
    await networkService.ensureInitialized();
  });

  describe('Network Service', () => {
    it('should provide network groups', () => {
      const grouped = networkService.getAllNetworksGrouped();
      
      expect(grouped.bitcoin).toBeDefined();
      expect(grouped.ethereum).toBeDefined();
      expect(grouped.evm).toBeDefined();
      expect(grouped.solana).toBeDefined();
      expect(grouped.aptos).toBeDefined();
      expect(grouped.sui).toBeDefined();
    });
  });
});
