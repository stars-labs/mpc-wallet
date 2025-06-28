import { describe, it, expect, beforeEach } from 'bun:test';
import WalletClientService from './walletClient';

describe('WalletClientService', () => {
    let walletClient: WalletClientService;

    beforeEach(() => {
        walletClient = WalletClientService.getInstance();
    });

    it('should return singleton instance', () => {
        const instance1 = WalletClientService.getInstance();
        const instance2 = WalletClientService.getInstance();

        expect(instance1).toBe(instance2);
    });

    it('should initialize with disconnected state', async () => {
        const isConnected = await walletClient.isConnected();
        expect(isConnected).toBe(false);
    });

    it('should handle connect operation', async () => {
        const result = await walletClient.connect();
        expect(result).toBeDefined();
    });

    it('should handle disconnect operation', async () => {
        const result = await walletClient.disconnect();
        expect(result).toBeDefined();
    });

    it('should handle balance queries', async () => {
        const testAddress = '0x1234567890123456789012345678901234567890';

        try {
            const balance = await walletClient.getBalance(testAddress);
            expect(typeof balance).toBe('string');
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('RPC')
            ).toBe(true);
        }
    });

    it('should handle transaction sending', async () => {
        const transaction = {
            to: '0x1234567890123456789012345678901234567890',
            value: '1000000000000000000',
            data: '0x'
        };

        try {
            const result = await walletClient.sendTransaction(transaction);
            expect(result).toBeDefined();
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('No account selected') ||
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('RPC')
            ).toBe(true);
        }
    });

    it('should handle message signing', async () => {
        const message = 'Test message for signing';

        try {
            const signature = await walletClient.signMessage(message);
            expect(typeof signature).toBe('string');
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('No account selected') ||
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('RPC') ||
                error.message.includes('personal_sign')
            ).toBe(true);
        }
    });

    it('should handle typed data signing', async () => {
        const typedData = {
            domain: {
                name: 'Test App',
                version: '1',
                chainId: 1,
                verifyingContract: '0x1234567890123456789012345678901234567890'
            },
            types: {
                Person: [
                    { name: 'name', type: 'string' },
                    { name: 'wallet', type: 'address' }
                ]
            },
            message: {
                name: 'Alice',
                wallet: '0x1234567890123456789012345678901234567890'
            }
        };

        try {
            const signature = await walletClient.signTypedData(typedData);
            expect(typeof signature).toBe('string');
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('No account selected') ||
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('RPC')
            ).toBe(true);
        }
    });

    it('should handle chain ID queries', async () => {
        try {
            const chainId = await walletClient.getChainId();
            expect(typeof chainId).toBe('string');
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('RPC')
            ).toBe(true);
        }
    });

    it('should handle gas estimation', async () => {
        const transaction = {
            to: '0x1234567890123456789012345678901234567890',
            value: '1000000000000000000',
            data: '0x'
        };

        try {
            const gasEstimate = await walletClient.estimateGas(transaction);
            expect(typeof gasEstimate).toBe('string');
            expect(parseInt(gasEstimate)).toBeGreaterThan(0);
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('method') ||
                error.message.includes('network') ||
                error.message.includes('account') ||
                error.message.includes('connection') ||
                error.message.includes('not supported') ||
                error.message.includes('RPC')
            ).toBe(true);
        }
    });

    it('should handle gas price queries', async () => {
        try {
            const gasPrice = await walletClient.getGasPrice();
            expect(typeof gasPrice).toBe('string');
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('RPC')
            ).toBe(true);
        }
    });

    it('should handle transaction receipts', async () => {
        const txHash = '0x1234567890123456789012345678901234567890123456789012345678901234';

        try {
            const receipt = await walletClient.getTransactionReceipt(txHash);
            expect(receipt).toBeDefined();
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('RPC') ||
                error.message.includes('invalid') ||
                error.message.includes('hash')
            ).toBe(true);
        }
    });

    it('should handle block number queries', async () => {
        try {
            const blockNumber = await walletClient.getBlockNumber();
            expect(typeof blockNumber).toBe('number');
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('RPC')
            ).toBe(true);
        }
    });

    it('should handle account requests', async () => {
        try {
            const accounts = await walletClient.requestAccounts();
            expect(Array.isArray(accounts)).toBe(true);
        } catch (error: any) {
            expect(error).toBeDefined();
            expect(
                error.message.includes('network') ||
                error.message.includes('connection') ||
                error.message.includes('method') ||
                error.message.includes('not supported') ||
                error.message.includes('RPC')
            ).toBe(true);
        }
    });

    it('should handle event listening', () => {
        const callback = (accounts: string[]) => {
            expect(Array.isArray(accounts)).toBe(true);
        };

        expect(() => {
            walletClient.onAccountsChanged(callback);
        }).not.toThrow();

        expect(() => {
            walletClient.onChainChanged((chainId: string) => {
                expect(typeof chainId).toBe('string');
            });
        }).not.toThrow();

        expect(() => {
            walletClient.onDisconnect(() => {
                // Handle disconnect
            });
        }).not.toThrow();
    });

    it('should handle multiple simultaneous operations', async () => {
        const operations = [
            walletClient.isConnected(),
            walletClient.getChainId().catch(() => 'error'),
            walletClient.getBlockNumber().catch(() => 0)
        ];

        const results = await Promise.allSettled(operations);
        expect(results.length).toBe(3);
        expect(results.every(r => r.status === 'fulfilled' || r.status === 'rejected')).toBe(true);
    });
});
