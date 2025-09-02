import { describe, it, expect } from 'bun:test';

// Simple message validation utilities to test
interface ExtensionMessage {
    type: string;
    id: string;
    timestamp: number;
    payload?: any;
}

interface ExtensionResponse {
    success: boolean;
    data?: any;
    error?: string;
}

class MessageValidator {
    static validateMessage(message: any): message is ExtensionMessage {
        if (!message || typeof message !== 'object') {
            return false;
        }

        if (typeof message.type !== 'string' || message.type.trim() === '') {
            return false;
        }

        if (typeof message.id !== 'string' || message.id.trim() === '') {
            return false;
        }

        if (typeof message.timestamp !== 'number' || message.timestamp <= 0) {
            return false;
        }

        return true;
    }

    static validatePayload(message: ExtensionMessage, requiredFields: string[]): boolean {
        if (!message.payload && requiredFields.length > 0) {
            return false;
        }

        for (const field of requiredFields) {
            if (!(field in message.payload)) {
                return false;
            }
        }

        return true;
    }

    static createResponse(success: boolean, data?: any, error?: string): ExtensionResponse {
        const response: ExtensionResponse = { success };
        
        if (success && data !== undefined) {
            response.data = data;
        }
        
        if (!success && error) {
            response.error = error;
        }
        
        return response;
    }

    static validateAddress(address: string, blockchain: 'ethereum' | 'solana'): boolean {
        if (blockchain === 'ethereum') {
            return /^0x[a-fA-F0-9]{40}$/.test(address);
        } else if (blockchain === 'solana') {
            // Basic Solana address format validation (base58, roughly 32-44 characters)
            return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address);
        }
        return false;
    }

    static validateAmount(amount: string): boolean {
        // Check if string represents a valid positive number
        if (typeof amount !== 'string' || amount.trim() === '') {
            return false;
        }
        
        // Check for multiple decimal points
        if ((amount.match(/\./g) || []).length > 1) {
            return false;
        }
        
        const num = parseFloat(amount);
        return !isNaN(num) && num >= 0 && isFinite(num) && amount !== 'Infinity' && amount !== 'NaN';
    }

    static validateChainId(chainId: any): boolean {
        return typeof chainId === 'number' && 
               chainId > 0 && 
               isFinite(chainId) && 
               !isNaN(chainId) && 
               Number.isInteger(chainId);
    }
}

describe('MessageValidator', () => {
    describe('Message Validation', () => {
        it('should validate correct message format', () => {
            const validMessage = {
                type: 'GET_STATE',
                id: 'msg-123',
                timestamp: Date.now()
            };

            expect(MessageValidator.validateMessage(validMessage)).toBe(true);
        });

        it('should reject messages with missing type', () => {
            const invalidMessage = {
                id: 'msg-123',
                timestamp: Date.now()
            };

            expect(MessageValidator.validateMessage(invalidMessage)).toBe(false);
        });

        it('should reject messages with empty type', () => {
            const invalidMessage = {
                type: '',
                id: 'msg-123',
                timestamp: Date.now()
            };

            expect(MessageValidator.validateMessage(invalidMessage)).toBe(false);
        });

        it('should reject messages with missing id', () => {
            const invalidMessage = {
                type: 'GET_STATE',
                timestamp: Date.now()
            };

            expect(MessageValidator.validateMessage(invalidMessage)).toBe(false);
        });

        it('should reject messages with empty id', () => {
            const invalidMessage = {
                type: 'GET_STATE',
                id: '',
                timestamp: Date.now()
            };

            expect(MessageValidator.validateMessage(invalidMessage)).toBe(false);
        });

        it('should reject messages with invalid timestamp', () => {
            const invalidMessage1 = {
                type: 'GET_STATE',
                id: 'msg-123',
                timestamp: 0
            };

            const invalidMessage2 = {
                type: 'GET_STATE',
                id: 'msg-123',
                timestamp: -1
            };

            const invalidMessage3 = {
                type: 'GET_STATE',
                id: 'msg-123',
                timestamp: 'invalid'
            };

            expect(MessageValidator.validateMessage(invalidMessage1)).toBe(false);
            expect(MessageValidator.validateMessage(invalidMessage2)).toBe(false);
            expect(MessageValidator.validateMessage(invalidMessage3)).toBe(false);
        });

        it('should reject null or undefined messages', () => {
            expect(MessageValidator.validateMessage(null)).toBe(false);
            expect(MessageValidator.validateMessage(undefined)).toBe(false);
        });

        it('should reject non-object messages', () => {
            expect(MessageValidator.validateMessage('string')).toBe(false);
            expect(MessageValidator.validateMessage(123)).toBe(false);
            expect(MessageValidator.validateMessage(true)).toBe(false);
        });
    });

    describe('Payload Validation', () => {
        const validMessage: ExtensionMessage = {
            type: 'CREATE_ACCOUNT',
            id: 'msg-456',
            timestamp: Date.now(),
            payload: {
                name: 'Test Account',
                blockchain: 'ethereum',
                address: '0x742d35Cc6641C4532B4d2a3F44ae7f35E0D29636'
            }
        };

        it('should validate payload with required fields', () => {
            const requiredFields = ['name', 'blockchain'];
            expect(MessageValidator.validatePayload(validMessage, requiredFields)).toBe(true);
        });

        it('should reject payload missing required fields', () => {
            const requiredFields = ['name', 'blockchain', 'missingField'];
            expect(MessageValidator.validatePayload(validMessage, requiredFields)).toBe(false);
        });

        it('should handle empty required fields', () => {
            expect(MessageValidator.validatePayload(validMessage, [])).toBe(true);
        });

        it('should reject message without payload when fields required', () => {
            const messageWithoutPayload: ExtensionMessage = {
                type: 'GET_STATE',
                id: 'msg-789',
                timestamp: Date.now()
            };

            expect(MessageValidator.validatePayload(messageWithoutPayload, ['someField'])).toBe(false);
        });

        it('should accept message without payload when no fields required', () => {
            const messageWithoutPayload: ExtensionMessage = {
                type: 'GET_STATE',
                id: 'msg-789',
                timestamp: Date.now()
            };

            expect(MessageValidator.validatePayload(messageWithoutPayload, [])).toBe(true);
        });
    });

    describe('Response Creation', () => {
        it('should create successful response with data', () => {
            const data = { accounts: [], currentAccount: null };
            const response = MessageValidator.createResponse(true, data);

            expect(response.success).toBe(true);
            expect(response.data).toEqual(data);
            expect(response.error).toBeUndefined();
        });

        it('should create successful response without data', () => {
            const response = MessageValidator.createResponse(true);

            expect(response.success).toBe(true);
            expect(response.data).toBeUndefined();
            expect(response.error).toBeUndefined();
        });

        it('should create error response with message', () => {
            const errorMessage = 'Something went wrong';
            const response = MessageValidator.createResponse(false, undefined, errorMessage);

            expect(response.success).toBe(false);
            expect(response.data).toBeUndefined();
            expect(response.error).toBe(errorMessage);
        });

        it('should create error response without message', () => {
            const response = MessageValidator.createResponse(false);

            expect(response.success).toBe(false);
            expect(response.data).toBeUndefined();
            expect(response.error).toBeUndefined();
        });
    });

    describe('Address Validation', () => {
        it('should validate correct Ethereum addresses', () => {
            const validEthAddresses = [
                '0x742d35Cc6641C4532B4d2a3F44ae7f35E0D29636',
                '0x0000000000000000000000000000000000000000',
                '0xFFfFfFffFFfffFFfFFfFFFFFffFFFffffFfFFFfF'
            ];

            validEthAddresses.forEach(address => {
                expect(MessageValidator.validateAddress(address, 'ethereum')).toBe(true);
            });
        });

        it('should reject invalid Ethereum addresses', () => {
            const invalidEthAddresses = [
                '742d35Cc6641C4532B4d2a3F44ae7f35E0D29636', // Missing 0x
                '0x742d35Cc6641C4532B4d2a3F44ae7f35E0D2963', // Too short
                '0x742d35Cc6641C4532B4d2a3F44ae7f35E0D296360', // Too long
                '0xGGGd35Cc6641C4532B4d2a3F44ae7f35E0D29636', // Invalid characters
                '', // Empty
                '0x' // Just prefix
            ];

            invalidEthAddresses.forEach(address => {
                expect(MessageValidator.validateAddress(address, 'ethereum')).toBe(false);
            });
        });

        it('should validate correct Solana addresses', () => {
            const validSolAddresses = [
                '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM',
                'DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1',
                '11111111111111111111111111111112' // System program
            ];

            validSolAddresses.forEach(address => {
                expect(MessageValidator.validateAddress(address, 'solana')).toBe(true);
            });
        });

        it('should reject invalid Solana addresses', () => {
            const invalidSolAddresses = [
                '0x742d35Cc6641C4532B4d2a3F44ae7f35E0D29636', // Ethereum format
                '9WzDX', // Too short
                '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM', // Too long
                '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM0', // Invalid character
                '', // Empty
                '000000000000000000000000000000000' // Invalid format
            ];

            invalidSolAddresses.forEach(address => {
                expect(MessageValidator.validateAddress(address, 'solana')).toBe(false);
            });
        });
    });

    describe('Amount Validation', () => {
        it('should validate correct amounts', () => {
            const validAmounts = [
                '0',
                '1',
                '100.5',
                '0.000000000000000001',
                '1000000000000000000',
                '123.456789'
            ];

            validAmounts.forEach(amount => {
                expect(MessageValidator.validateAmount(amount)).toBe(true);
            });
        });

        it('should reject invalid amounts', () => {
            const invalidAmounts = [
                '-1', // Negative
                'abc', // Non-numeric
                '', // Empty
                'NaN', // NaN string
                'Infinity', // Infinity string
                '1.2.3', // Multiple decimals
                '1e999', // Too large
            ];

            invalidAmounts.forEach(amount => {
                expect(MessageValidator.validateAmount(amount)).toBe(false);
            });
        });
    });

    describe('Chain ID Validation', () => {
        it('should validate correct chain IDs', () => {
            const validChainIds = [1, 5, 56, 137, 42161];

            validChainIds.forEach(chainId => {
                expect(MessageValidator.validateChainId(chainId)).toBe(true);
            });
        });

        it('should reject invalid chain IDs', () => {
            const invalidChainIds = [0, -1, 'ethereum', null, undefined, 1.5, NaN, Infinity];

            invalidChainIds.forEach(chainId => {
                expect(MessageValidator.validateChainId(chainId)).toBe(false);
            });
        });
    });

    describe('Integration Tests', () => {
        it('should validate complete transaction message', () => {
            const transactionMessage = {
                type: 'INITIATE_TRANSACTION',
                id: 'txn-123',
                timestamp: Date.now(),
                payload: {
                    to: '0x742d35Cc6641C4532B4d2a3F44ae7f35E0D29636',
                    value: '1000000000000000000',
                    chainId: 1,
                    accountId: 'account-456'
                }
            };

            // Validate message structure
            expect(MessageValidator.validateMessage(transactionMessage)).toBe(true);

            // Validate required payload fields
            expect(MessageValidator.validatePayload(transactionMessage, ['to', 'value', 'chainId', 'accountId'])).toBe(true);

            // Validate specific field formats
            expect(MessageValidator.validateAddress(transactionMessage.payload.to, 'ethereum')).toBe(true);
            expect(MessageValidator.validateAmount(transactionMessage.payload.value)).toBe(true);
            expect(MessageValidator.validateChainId(transactionMessage.payload.chainId)).toBe(true);
        });

        it('should reject incomplete transaction message', () => {
            const incompleteMessage = {
                type: 'INITIATE_TRANSACTION',
                id: 'txn-456',
                timestamp: Date.now(),
                payload: {
                    to: 'invalid-address',
                    value: '-100',
                    chainId: 'ethereum'
                }
                // Missing accountId
            };

            // Message structure is valid
            expect(MessageValidator.validateMessage(incompleteMessage)).toBe(true);

            // But payload validation should fail
            expect(MessageValidator.validatePayload(incompleteMessage, ['to', 'value', 'chainId', 'accountId'])).toBe(false);

            // And field format validation should fail
            expect(MessageValidator.validateAddress(incompleteMessage.payload.to, 'ethereum')).toBe(false);
            expect(MessageValidator.validateAmount(incompleteMessage.payload.value)).toBe(false);
            expect(MessageValidator.validateChainId(incompleteMessage.payload.chainId)).toBe(false);
        });

        it('should handle edge cases gracefully', () => {
            // Very large timestamp
            const futureDateMessage = {
                type: 'TEST',
                id: 'future-123',
                timestamp: Date.now() + 1000000000
            };
            expect(MessageValidator.validateMessage(futureDateMessage)).toBe(true);

            // Very long message type
            const longTypeMessage = {
                type: 'A'.repeat(1000),
                id: 'long-type-123',
                timestamp: Date.now()
            };
            expect(MessageValidator.validateMessage(longTypeMessage)).toBe(true);

            // Very long ID
            const longIdMessage = {
                type: 'TEST',
                id: 'X'.repeat(1000),
                timestamp: Date.now()
            };
            expect(MessageValidator.validateMessage(longIdMessage)).toBe(true);
        });
    });
});