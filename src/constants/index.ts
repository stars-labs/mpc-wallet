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