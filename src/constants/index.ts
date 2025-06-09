// 消息传递相关常量
export const MESSAGE_PREFIX = 'starlab_wallet';

// 消息类型
export enum MessageType {
    REQUEST = 'REQUEST',
    RESPONSE = 'RESPONSE',
    ACCOUNT_MANAGEMENT = 'ACCOUNT_MANAGEMENT',
    NETWORK_MANAGEMENT = 'NETWORK_MANAGEMENT',
    UI_REQUEST = 'UI_REQUEST'
}

// 钱包信息
export const WALLET_INFO = {
    name: 'Starlab Wallet',
    version: '1.0.0',
    description: 'A browser extension wallet for starlab'
};

// Default addresses - these are generated if no real DKG address is available
export const DEFAULT_ADDRESSES = {
    // Deterministically generate addresses based on a seed or return fallback constants
    ethereum: (seed?: string) => {
        // Fallback constant for contexts where no proper seed is available (like unlisted scripts)
        if (!seed) {
            return '0x0000000000000000000000000000000000000001';
        }
        
        try {
            // Use a hash of the seed to make it more random-looking but deterministic
            const hash = Array.from(seed).reduce((acc, char) => (acc * 31 + char.charCodeAt(0)) & 0xFFFFFFFF, 0).toString(16);
            // Create an address-like string
            return `0x${hash.padStart(8, '0')}000000000000000000000000000000000000000`.substring(0, 42);
        } catch (e) {
            console.error('Error generating Ethereum address from seed:', e);
            return '0x0000000000000000000000000000000000000001';
        }
    },
    solana: (seed?: string) => {
        // Fallback constant for contexts where no proper seed is available
        if (!seed) {
            return 'unpredictablesolanaaddress00000000000000001';
        }
        
        try {
            // Make a Solana-style address
            const hash = Array.from(seed).reduce((acc, char) => (acc * 31 + char.charCodeAt(0)) & 0xFFFFFFFF, 0).toString(16);
            return `${hash.padStart(16, '0')}00000000000000000000000000000000`.substring(0, 44);
        } catch (e) {
            console.error('Error generating Solana address from seed:', e);
            return 'unpredictablesolanaaddress00000000000000001';
        }
    }
};