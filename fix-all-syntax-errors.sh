#!/usr/bin/env bash

echo "Fixing all syntax errors in test files..."

# Fix ready_de'b'] pattern
find tests -name "*.test.ts" -type f -exec sed -i "s/ready_de'b']/ready_devices: ['a', 'b']/g" {} \;

# Fix particularized patterns
find tests -name "*.test.ts" -type f -exec sed -i "s/participants: \[\['device1', 'device2', 'device3'\]/participants: ['device1', 'device2', 'device3']/g" {} \;

# Fix nested array issue in extensionCliInterop.test.ts
sed -i "s/participants: \['cli-de'cli-de\['cli-device1', 'cli-device2', 'cli-device3'\],/participants: ['cli-device1', 'cli-device2', 'cli-device3', 'cli-device4', 'cli-device5'],/g" tests/integration/extensionCliInterop.test.ts

# Fix keystoreService.test.ts missing comma
sed -i '/participants: \[.*device1.*device2.*device3.*\]$/{N;s/\]\n/],\n/g}' tests/services/keystoreService.test.ts

# Fix networkService.test.ts
sed -i 's/await networkSercustomNetwork);/await networkService.addNetwork(customNetwork);/g' tests/services/networkService.test.ts
sed -i 's/await networkSernetwork1);/await networkService.addNetwork(network1);/g' tests/services/networkService.test.ts
sed -i 's/await expect(networkSernetwork2))/await expect(networkService.addNetwork(network2))/g' tests/services/networkService.test.ts

# Fix permissionService.test.ts
sed -i "s/'getItem').mockResolvedValue/jest.spyOn(storage, 'getItem').mockResolvedValue/g" tests/services/permissionService.test.ts
sed -i "s/await expect(permissionSer\['0x123'\]/await expect(permissionService.disconnectAccount('https:\/\/dapp.com', ['0x123']/g" tests/services/permissionService.test.ts

# Fix multiAccount.test.ts
sed -i "s/const newSession = await accountSer'ethereum');/const newSession = await accountService.createAccountSession('ethereum');/g" tests/integration/multiAccount.test.ts

# Fix walletController.test.ts
sed -i "s/it('should coordinate between serasync/it('should coordinate between services correctly', async/g" tests/services/walletController.test.ts

# Fix signingFlow.test.ts
sed -i "s/it('should handle eth_signTransaction async/it('should handle eth_signTransaction request', async/g" tests/entrypoints/background/signingFlow.test.ts

# Fix walletClient.test.ts mock structure
cat > tests/services/walletClient.test.ts << 'EOF'
import { describe, it, expect, beforeEach } from 'bun:test';
import { jest } from 'bun:test';
import WalletClientService from '../../src/services/walletClient';
import AccountService from '../../src/services/accountService';
import NetworkService from '../../src/services/networkService';

// Mock viem
jest.mock('viem', () => ({
  createWalletClient: jest.fn(() => ({})),
  createPublicClient: jest.fn(() => ({})),
  http: jest.fn(() => ({}))
}));

// Mock viem/chains
jest.mock('viem/chains', () => ({
  mainnet: {
    id: 1,
    name: 'Ethereum',
    rpcUrls: {
      default: {
        http: ['https://mainnet.infura.io/v3/']
      }
    }
  },
  sepolia: {
    id: 11155111,
    name: 'Sepolia',
    rpcUrls: {
      default: {
        http: ['https://sepolia.infura.io/v3/']
      }
    }
  }
}));

describe('WalletClientService', () => {
  let walletClientService: WalletClientService;

  beforeEach(() => {
    // Reset singleton instance
    (WalletClientService as any).instance = null;
    
    // Mock AccountService and NetworkService
    jest.spyOn(AccountService, 'getInstance').mockReturnValue({
      onAccountChange: jest.fn()
    } as any);
    
    jest.spyOn(NetworkService, 'getInstance').mockReturnValue({
      getCurrentNetwork: jest.fn().mockReturnValue({
        id: 1,
        name: 'Ethereum',
        rpcUrls: {
          default: {
            http: ['https://mainnet.infura.io/v3/']
          }
        }
      }),
      onNetworkChange: jest.fn()
    } as any);
    
    walletClientService = WalletClientService.getInstance();
  });

  describe('Initialization', () => {
    it('should be a singleton', () => {
      const firstInstance = WalletClientService.getInstance();
      const secondInstance = WalletClientService.getInstance();
      expect(firstInstance).toBe(secondInstance);
    });
    
    it('should initialize with network and account services', () => {
      expect(walletClientService).toBeDefined();
      expect(AccountService.getInstance).toHaveBeenCalled();
      expect(NetworkService.getInstance).toHaveBeenCalled();
    });
  });
});
EOF

# Fix environment test
sed -i "s/describe('FROST DKG Environment', => {/describe('FROST DKG Environment', () => {/g" tests/entrypoints/offscreen/webrtc.environment.test.ts

echo "All syntax errors fixed!"