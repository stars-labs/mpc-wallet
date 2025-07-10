/**
 * Chain Configuration
 * 
 * This file contains the configuration for all supported blockchain networks.
 * Each chain is categorized by its signature algorithm:
 * - secp256k1: Bitcoin, Ethereum, and EVM-compatible chains
 * - ed25519: Solana, Aptos, Sui, and other modern chains
 */

import type { Chain } from "@mpc-wallet/types/network";

// Signature algorithm types
export type SignatureAlgorithm = 'secp256k1' | 'ed25519';

// Chain categories
export type ChainCategory = 'bitcoin' | 'ethereum' | 'evm' | 'solana' | 'aptos' | 'sui';

// Extended chain information
export interface ChainInfo extends Chain {
  algorithm: SignatureAlgorithm;
  category: ChainCategory;
  testnet?: boolean;
  mainnetId?: number; // Reference to mainnet chain ID for testnets
  derivationPath?: string; // Default derivation path
  addressPrefix?: string; // For chains with special address formats
  decimals?: number; // Native token decimals
}

// Bitcoin chains
export const BITCOIN_CHAINS: ChainInfo[] = [
  {
    id: 0,
    name: 'Bitcoin',
    network: 'bitcoin',
    algorithm: 'secp256k1',
    category: 'bitcoin',
    testnet: false,
    derivationPath: "m/84'/0'/0'/0/0", // BIP84 for native segwit
    addressPrefix: 'bc1',
    decimals: 8,
    nativeCurrency: {
      name: 'Bitcoin',
      symbol: 'BTC',
      decimals: 8
    },
    rpcUrls: {
      default: {
        http: ['https://mempool.space/api']
      }
    },
    blockExplorers: {
      default: {
        name: 'Mempool',
        url: 'https://mempool.space'
      }
    }
  },
  {
    id: 1000001,
    name: 'Bitcoin Testnet',
    network: 'bitcoin-testnet',
    algorithm: 'secp256k1',
    category: 'bitcoin',
    testnet: true,
    mainnetId: 0,
    derivationPath: "m/84'/1'/0'/0/0", // BIP84 for testnet
    addressPrefix: 'tb1',
    decimals: 8,
    nativeCurrency: {
      name: 'Test Bitcoin',
      symbol: 'tBTC',
      decimals: 8
    },
    rpcUrls: {
      default: {
        http: ['https://mempool.space/testnet/api']
      }
    },
    blockExplorers: {
      default: {
        name: 'Mempool Testnet',
        url: 'https://mempool.space/testnet'
      }
    }
  }
];

// Ethereum and EVM chains
export const ETHEREUM_CHAINS: ChainInfo[] = [
  // Ethereum
  {
    id: 1,
    name: 'Ethereum Mainnet',
    network: 'mainnet',
    algorithm: 'secp256k1',
    category: 'ethereum',
    testnet: false,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://eth.llamarpc.com']
      }
    },
    blockExplorers: {
      default: {
        name: 'Etherscan',
        url: 'https://etherscan.io'
      }
    }
  },
  {
    id: 11155111,
    name: 'Sepolia',
    network: 'sepolia',
    algorithm: 'secp256k1',
    category: 'ethereum',
    testnet: true,
    mainnetId: 1,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Sepolia Ether',
      symbol: 'SEP',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://sepolia.infura.io/v3/']
      }
    },
    blockExplorers: {
      default: {
        name: 'Sepolia Etherscan',
        url: 'https://sepolia.etherscan.io'
      }
    }
  },
  
  // Polygon
  {
    id: 137,
    name: 'Polygon',
    network: 'polygon',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: false,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'MATIC',
      symbol: 'MATIC',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://polygon-rpc.com']
      }
    },
    blockExplorers: {
      default: {
        name: 'PolygonScan',
        url: 'https://polygonscan.com'
      }
    }
  },
  {
    id: 80001,
    name: 'Polygon Mumbai',
    network: 'polygon-mumbai',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: true,
    mainnetId: 137,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'MATIC',
      symbol: 'MATIC',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://rpc-mumbai.maticvigil.com']
      }
    },
    blockExplorers: {
      default: {
        name: 'Mumbai PolygonScan',
        url: 'https://mumbai.polygonscan.com'
      }
    }
  },
  
  // BSC
  {
    id: 56,
    name: 'BNB Smart Chain',
    network: 'bsc',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: false,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'BNB',
      symbol: 'BNB',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://bsc-dataseed.binance.org']
      }
    },
    blockExplorers: {
      default: {
        name: 'BscScan',
        url: 'https://bscscan.com'
      }
    }
  },
  {
    id: 97,
    name: 'BNB Smart Chain Testnet',
    network: 'bsc-testnet',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: true,
    mainnetId: 56,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Test BNB',
      symbol: 'tBNB',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://data-seed-prebsc-1-s1.binance.org:8545']
      }
    },
    blockExplorers: {
      default: {
        name: 'BscScan Testnet',
        url: 'https://testnet.bscscan.com'
      }
    }
  },
  
  // Arbitrum
  {
    id: 42161,
    name: 'Arbitrum One',
    network: 'arbitrum',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: false,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://arb1.arbitrum.io/rpc']
      }
    },
    blockExplorers: {
      default: {
        name: 'Arbiscan',
        url: 'https://arbiscan.io'
      }
    }
  },
  {
    id: 421614,
    name: 'Arbitrum Sepolia',
    network: 'arbitrum-sepolia',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: true,
    mainnetId: 42161,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://sepolia-rollup.arbitrum.io/rpc']
      }
    },
    blockExplorers: {
      default: {
        name: 'Arbiscan Testnet',
        url: 'https://sepolia.arbiscan.io'
      }
    }
  },
  
  // Optimism
  {
    id: 10,
    name: 'Optimism',
    network: 'optimism',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: false,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://mainnet.optimism.io']
      }
    },
    blockExplorers: {
      default: {
        name: 'Optimistic Etherscan',
        url: 'https://optimistic.etherscan.io'
      }
    }
  },
  {
    id: 11155420,
    name: 'Optimism Sepolia',
    network: 'optimism-sepolia',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: true,
    mainnetId: 10,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://sepolia.optimism.io']
      }
    },
    blockExplorers: {
      default: {
        name: 'Optimism Sepolia Explorer',
        url: 'https://sepolia-optimism.etherscan.io'
      }
    }
  },
  
  // Base
  {
    id: 8453,
    name: 'Base',
    network: 'base',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: false,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://mainnet.base.org']
      }
    },
    blockExplorers: {
      default: {
        name: 'BaseScan',
        url: 'https://basescan.org'
      }
    }
  },
  {
    id: 84532,
    name: 'Base Sepolia',
    network: 'base-sepolia',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: true,
    mainnetId: 8453,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://sepolia.base.org']
      }
    },
    blockExplorers: {
      default: {
        name: 'Base Sepolia Explorer',
        url: 'https://sepolia.basescan.org'
      }
    }
  },
  
  // Avalanche
  {
    id: 43114,
    name: 'Avalanche C-Chain',
    network: 'avalanche',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: false,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'AVAX',
      symbol: 'AVAX',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://api.avax.network/ext/bc/C/rpc']
      }
    },
    blockExplorers: {
      default: {
        name: 'SnowTrace',
        url: 'https://snowtrace.io'
      }
    }
  },
  {
    id: 43113,
    name: 'Avalanche Fuji',
    network: 'avalanche-fuji',
    algorithm: 'secp256k1',
    category: 'evm',
    testnet: true,
    mainnetId: 43114,
    derivationPath: "m/44'/60'/0'/0/0",
    decimals: 18,
    nativeCurrency: {
      name: 'AVAX',
      symbol: 'AVAX',
      decimals: 18
    },
    rpcUrls: {
      default: {
        http: ['https://api.avax-test.network/ext/bc/C/rpc']
      }
    },
    blockExplorers: {
      default: {
        name: 'SnowTrace Testnet',
        url: 'https://testnet.snowtrace.io'
      }
    }
  }
];

// Solana chains
export const SOLANA_CHAINS: ChainInfo[] = [
  {
    id: 101,
    name: 'Solana Mainnet',
    network: 'solana-mainnet',
    algorithm: 'ed25519',
    category: 'solana',
    testnet: false,
    derivationPath: "m/44'/501'/0'/0'",
    decimals: 9,
    nativeCurrency: {
      name: 'SOL',
      symbol: 'SOL',
      decimals: 9
    },
    rpcUrls: {
      default: {
        http: ['https://api.mainnet-beta.solana.com']
      }
    },
    blockExplorers: {
      default: {
        name: 'Solscan',
        url: 'https://solscan.io'
      }
    }
  },
  {
    id: 102,
    name: 'Solana Devnet',
    network: 'solana-devnet',
    algorithm: 'ed25519',
    category: 'solana',
    testnet: true,
    mainnetId: 101,
    derivationPath: "m/44'/501'/0'/0'",
    decimals: 9,
    nativeCurrency: {
      name: 'SOL',
      symbol: 'SOL',
      decimals: 9
    },
    rpcUrls: {
      default: {
        http: ['https://api.devnet.solana.com']
      }
    },
    blockExplorers: {
      default: {
        name: 'Solscan Devnet',
        url: 'https://solscan.io?cluster=devnet'
      }
    }
  },
  {
    id: 103,
    name: 'Solana Testnet',
    network: 'solana-testnet',
    algorithm: 'ed25519',
    category: 'solana',
    testnet: true,
    mainnetId: 101,
    derivationPath: "m/44'/501'/0'/0'",
    decimals: 9,
    nativeCurrency: {
      name: 'SOL',
      symbol: 'SOL',
      decimals: 9
    },
    rpcUrls: {
      default: {
        http: ['https://api.testnet.solana.com']
      }
    },
    blockExplorers: {
      default: {
        name: 'Solscan Testnet',
        url: 'https://solscan.io?cluster=testnet'
      }
    }
  }
];

// Aptos chains
export const APTOS_CHAINS: ChainInfo[] = [
  {
    id: 201,
    name: 'Aptos Mainnet',
    network: 'aptos-mainnet',
    algorithm: 'ed25519',
    category: 'aptos',
    testnet: false,
    derivationPath: "m/44'/637'/0'/0'/0'",
    decimals: 8,
    nativeCurrency: {
      name: 'APT',
      symbol: 'APT',
      decimals: 8
    },
    rpcUrls: {
      default: {
        http: ['https://fullnode.mainnet.aptoslabs.com/v1']
      }
    },
    blockExplorers: {
      default: {
        name: 'Aptos Explorer',
        url: 'https://explorer.aptoslabs.com'
      }
    }
  },
  {
    id: 202,
    name: 'Aptos Testnet',
    network: 'aptos-testnet',
    algorithm: 'ed25519',
    category: 'aptos',
    testnet: true,
    mainnetId: 201,
    derivationPath: "m/44'/637'/0'/0'/0'",
    decimals: 8,
    nativeCurrency: {
      name: 'APT',
      symbol: 'APT',
      decimals: 8
    },
    rpcUrls: {
      default: {
        http: ['https://fullnode.testnet.aptoslabs.com/v1']
      }
    },
    blockExplorers: {
      default: {
        name: 'Aptos Testnet Explorer',
        url: 'https://explorer.aptoslabs.com?network=testnet'
      }
    }
  }
];

// Sui chains
export const SUI_CHAINS: ChainInfo[] = [
  {
    id: 301,
    name: 'Sui Mainnet',
    network: 'sui-mainnet',
    algorithm: 'ed25519',
    category: 'sui',
    testnet: false,
    derivationPath: "m/44'/784'/0'/0'/0'",
    addressPrefix: '0x',
    decimals: 9,
    nativeCurrency: {
      name: 'SUI',
      symbol: 'SUI',
      decimals: 9
    },
    rpcUrls: {
      default: {
        http: ['https://fullnode.mainnet.sui.io']
      }
    },
    blockExplorers: {
      default: {
        name: 'Sui Explorer',
        url: 'https://suiexplorer.com'
      }
    }
  },
  {
    id: 302,
    name: 'Sui Testnet',
    network: 'sui-testnet',
    algorithm: 'ed25519',
    category: 'sui',
    testnet: true,
    mainnetId: 301,
    derivationPath: "m/44'/784'/0'/0'/0'",
    addressPrefix: '0x',
    decimals: 9,
    nativeCurrency: {
      name: 'SUI',
      symbol: 'SUI',
      decimals: 9
    },
    rpcUrls: {
      default: {
        http: ['https://fullnode.testnet.sui.io']
      }
    },
    blockExplorers: {
      default: {
        name: 'Sui Testnet Explorer',
        url: 'https://suiexplorer.com?network=testnet'
      }
    }
  }
];

// All chains grouped by algorithm
export const CHAINS_BY_ALGORITHM: Record<SignatureAlgorithm, ChainInfo[]> = {
  secp256k1: [...BITCOIN_CHAINS, ...ETHEREUM_CHAINS],
  ed25519: [...SOLANA_CHAINS, ...APTOS_CHAINS, ...SUI_CHAINS]
};

// All chains grouped by category
export const CHAINS_BY_CATEGORY: Record<ChainCategory, ChainInfo[]> = {
  bitcoin: BITCOIN_CHAINS,
  ethereum: ETHEREUM_CHAINS.filter(c => c.category === 'ethereum'),
  evm: ETHEREUM_CHAINS.filter(c => c.category === 'evm'),
  solana: SOLANA_CHAINS,
  aptos: APTOS_CHAINS,
  sui: SUI_CHAINS
};

// All chains
export const ALL_CHAINS: ChainInfo[] = [
  ...BITCOIN_CHAINS,
  ...ETHEREUM_CHAINS,
  ...SOLANA_CHAINS,
  ...APTOS_CHAINS,
  ...SUI_CHAINS
];

// Helper functions
export function getChainById(chainId: number): ChainInfo | undefined {
  return ALL_CHAINS.find(chain => chain.id === chainId);
}

export function getChainsByAlgorithm(algorithm: SignatureAlgorithm): ChainInfo[] {
  return CHAINS_BY_ALGORITHM[algorithm] || [];
}

export function getChainsByCategory(category: ChainCategory): ChainInfo[] {
  return CHAINS_BY_CATEGORY[category] || [];
}

export function getMainnetChains(): ChainInfo[] {
  return ALL_CHAINS.filter(chain => !chain.testnet);
}

export function getTestnetChains(): ChainInfo[] {
  return ALL_CHAINS.filter(chain => chain.testnet);
}

export function getTestnetForMainnet(mainnetId: number): ChainInfo | undefined {
  return ALL_CHAINS.find(chain => chain.testnet && chain.mainnetId === mainnetId);
}

// Default chains for initial setup
export const DEFAULT_CHAINS = {
  secp256k1: [
    getChainById(1), // Ethereum Mainnet
    getChainById(11155111), // Sepolia
    getChainById(137), // Polygon
    getChainById(56), // BSC
    getChainById(42161), // Arbitrum
  ].filter(Boolean) as ChainInfo[],
  ed25519: [
    getChainById(101), // Solana Mainnet
    getChainById(102), // Solana Devnet
  ].filter(Boolean) as ChainInfo[]
};