// ===================================================================
// RPC HANDLER MODULE
// ===================================================================
//
// This module handles JSON-RPC requests from web applications and
// UI requests from the popup interface. It provides standardized
// wallet functionality including account management, network queries,
// and transaction handling.
// ===================================================================

import AccountService from '../../services/accountService';
import NetworkService from '../../services/networkService';
import WalletClientService from '../../services/walletClient';
import WalletController from "../../services/walletController";
import { getPermissionService } from '../../services/permissionService';
import { toHex } from 'viem';
import type { JsonRpcRequest } from "../../types/messages";

/**
 * Handles JSON-RPC requests from web applications
 */
export class RpcHandler {
    private accountService: AccountService;
    private networkService: NetworkService;
    private walletClientService: WalletClientService;
    private permissionService = getPermissionService();
    private origin: string = '';

    constructor() {
        this.accountService = AccountService.getInstance();
        this.networkService = NetworkService.getInstance();
        this.walletClientService = WalletClientService.getInstance();
    }

    /**
     * Set the origin of the request for permission checking
     */
    setOrigin(origin: string): void {
        this.origin = origin;
    }

    /**
     * Process a JSON-RPC request and return the response
     */
    async handleRpcRequest(request: JsonRpcRequest): Promise<unknown> {
        try {
            console.log(`[RpcHandler] Processing RPC request: ${request.method}`);

            switch (request.method) {
                case 'eth_accounts':
                case 'eth_requestAccounts':
                    return await this.handleAccountsRequest(request.method);

                case 'eth_chainId':
                    return await this.handleChainIdRequest();

                case 'net_version':
                    return await this.handleNetVersionRequest();

                case 'eth_getBalance':
                    return await this.handleGetBalanceRequest(request.params as unknown[]);

                case 'eth_sendTransaction':
                    return await this.handleSendTransactionRequest(request.params as unknown[]);

                case 'eth_signMessage':
                case 'personal_sign':
                    return await this.handleSignMessageRequest(request.params as unknown[]);

                case 'eth_getTransactionCount':
                    return await this.handleGetTransactionCountRequest(request.params as unknown[]);

                case 'eth_gasPrice':
                    return await this.handleGasPriceRequest();

                case 'eth_estimateGas':
                    return await this.handleEstimateGasRequest(request.params as unknown[]);

                default:
                    // Forward read-only methods to RPC provider
                    if (this.isReadOnlyMethod(request.method)) {
                        return await this.forwardToRpcProvider(request);
                    }
                    throw new Error(`Unsupported method: ${request.method}`);
            }
        } catch (error) {
            console.error(`[RpcHandler] RPC request failed: ${request.method}`, error);
            throw error;
        }
    }

    /**
     * Handle eth_accounts and eth_requestAccounts
     */
    private async handleAccountsRequest(method: string): Promise<string[]> {
        // For eth_accounts, return already connected accounts
        if (method === 'eth_accounts') {
            // If no origin (e.g., from popup), return current account
            if (!this.origin) {
                const currentAccount = this.accountService.getCurrentAccount();
                return currentAccount ? [currentAccount.address] : [];
            }
            
            const connectedAccounts = this.permissionService.getConnectedAccounts(this.origin);
            console.log(`[RpcHandler] eth_accounts for ${this.origin}: ${connectedAccounts.length} connected`);
            return connectedAccounts;
        }

        // For eth_requestAccounts, we need to prompt user for permission
        if (method === 'eth_requestAccounts') {
            // First ensure we have at least one account
            await this.accountService.ensureInitialized();
            let accounts = this.accountService.getAccountsByBlockchain('ethereum');
            
            if (accounts.length === 0) {
                console.log('[RpcHandler] No accounts exist, creating default account');
                const defaultAccount = await this.accountService.ensureDefaultAccount();
                if (!defaultAccount) {
                    throw new Error('Failed to create default account');
                }
                accounts = [defaultAccount];
            }

            // Check if we already have permissions
            const connectedAccounts = this.permissionService.getConnectedAccounts(this.origin);
            if (connectedAccounts.length > 0) {
                console.log(`[RpcHandler] Returning existing connections for ${this.origin}`);
                return connectedAccounts;
            }

            // For now, auto-connect all accounts (in production, show UI selector)
            // TODO: Implement UI account selector
            const accountAddresses = accounts.map(acc => acc.address);
            const currentNetwork = this.networkService.getCurrentNetwork();
            const chainId = currentNetwork ? toHex(currentNetwork.id) : '0x1';
            
            await this.permissionService.connectAccounts(
                this.origin, 
                accountAddresses,
                chainId
            );

            console.log(`[RpcHandler] Connected ${accountAddresses.length} accounts to ${this.origin}`);
            return accountAddresses;
        }

        return [];
    }

    /**
     * Handle eth_chainId request
     */
    private async handleChainIdRequest(): Promise<string> {
        const currentNetwork = this.networkService.getCurrentNetwork();
        if (!currentNetwork) {
            throw new Error('No current network found');
        }
        return toHex(currentNetwork.id);
    }

    /**
     * Handle net_version request
     */
    private async handleNetVersionRequest(): Promise<string> {
        const network = this.networkService.getCurrentNetwork();
        if (!network) {
            throw new Error('No current network found');
        }
        return network.id.toString();
    }

    /**
     * Handle eth_getBalance request
     */
    private async handleGetBalanceRequest(params: unknown[]): Promise<string> {
        if (!params || params.length < 1) {
            throw new Error('Missing address parameter');
        }

        const address = params[0] as string;
        // WalletClientService.getBalance() only takes address as optional parameter
        return await this.walletClientService.getBalance(address);
    }

    /**
     * Handle eth_sendTransaction request
     */
    private async handleSendTransactionRequest(params: unknown[]): Promise<string> {
        if (!params || params.length < 1) {
            throw new Error('Missing transaction parameters');
        }

        const transaction = params[0] as any;

        // Validate transaction parameters
        if (!transaction.to || !transaction.value) {
            throw new Error('Invalid transaction parameters');
        }

        // Use wallet client service to send transaction
        return await this.walletClientService.sendTransaction(transaction);
    }

    /**
     * Handle message signing requests
     */
    private async handleSignMessageRequest(params: unknown[]): Promise<string> {
        if (!params || params.length < 1) {
            throw new Error('Missing message parameter');
        }

        const message = params[0] as string;
        // WalletClientService.signMessage() only takes message parameter
        return await this.walletClientService.signMessage(message);
    }

    /**
     * Handle eth_getTransactionCount request
     */
    private async handleGetTransactionCountRequest(params: unknown[]): Promise<string> {
        if (!params || params.length < 1) {
            throw new Error('Missing address parameter');
        }

        const address = params[0] as string;
        // WalletClientService.getTransactionCount() returns number, convert to string
        const count = await this.walletClientService.getTransactionCount(address);
        return count.toString();
    }

    /**
     * Handle eth_gasPrice request
     */
    private async handleGasPriceRequest(): Promise<string> {
        return await this.walletClientService.getGasPrice();
    }

    /**
     * Handle eth_estimateGas request
     */
    private async handleEstimateGasRequest(params: unknown[]): Promise<string> {
        if (!params || params.length < 1) {
            throw new Error('Missing transaction parameters');
        }

        const transaction = params[0] as any;
        return await this.walletClientService.estimateGas(transaction);
    }

    /**
     * Generic RPC request method that can be used by other modules
     */
    async makeRpcRequest(request: JsonRpcRequest): Promise<unknown> {
        // For now, just delegate to handleRpcRequest
        return this.handleRpcRequest(request);
    }
}

/**
 * Handles UI requests from the popup interface
 */
export class UIRequestHandler {
    private walletController: WalletController;

    constructor() {
        this.walletController = WalletController.getInstance();
    }

    /**
     * Handle UI requests from popup
     */
    async handleUIRequest(request: { method: string; params: unknown[] }): Promise<{ success: boolean; data?: unknown; error?: string }> {
        const { method, params } = request;

        console.log(`[UIRequestHandler] Processing UI request: ${method}`);

        if (typeof this.walletController[method as keyof WalletController] === 'function') {
            try {
                const result = await (this.walletController[method as keyof WalletController] as (...args: unknown[]) => unknown)(...params);
                return { success: true, data: result };
            } catch (error) {
                console.error(`[UIRequestHandler] UI request failed: ${method}`, error);
                return { success: false, error: error instanceof Error ? error.message : 'Unknown error' };
            }
        }

        return { success: false, error: `Method ${method} not found on WalletController` };
    }

    /**
     * Check if a method is read-only and should be forwarded to RPC provider
     */
    private isReadOnlyMethod(method: string): boolean {
        const readOnlyMethods = [
            'eth_blockNumber',
            'eth_getBlockByHash',
            'eth_getBlockByNumber',
            'eth_getTransactionByHash',
            'eth_getTransactionReceipt',
            'eth_getBlockTransactionCountByHash',
            'eth_getBlockTransactionCountByNumber',
            'eth_getUncleCountByBlockHash',
            'eth_getUncleCountByBlockNumber',
            'eth_getCode',
            'eth_call',
            'eth_getLogs',
            'eth_getFilterChanges',
            'eth_getFilterLogs',
            'eth_newFilter',
            'eth_newBlockFilter',
            'eth_newPendingTransactionFilter',
            'eth_uninstallFilter',
            'eth_getStorageAt',
            'eth_getProof',
            'eth_feeHistory',
            'eth_maxPriorityFeePerGas'
        ];
        
        return readOnlyMethods.includes(method);
    }

    /**
     * Forward RPC request to the network's RPC provider
     */
    private async forwardToRpcProvider(request: JsonRpcRequest): Promise<any> {
        try {
            const network = this.networkService.getCurrentNetwork();
            if (!network || !network.rpcUrls?.default?.http?.[0]) {
                throw new Error('No RPC URL available for current network');
            }

            const rpcUrl = network.rpcUrls.default.http[0];
            console.log(`[RpcHandler] Forwarding ${request.method} to RPC provider: ${rpcUrl}`);

            const response = await fetch(rpcUrl, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    id: request.id || 1,
                    method: request.method,
                    params: request.params || []
                })
            });

            if (!response.ok) {
                throw new Error(`RPC request failed with status ${response.status}`);
            }

            const result = await response.json();
            
            if (result.error) {
                throw new Error(result.error.message || 'RPC request failed');
            }

            return result.result;
        } catch (error) {
            console.error(`[RpcHandler] Failed to forward to RPC provider:`, error);
            throw error;
        }
    }
}
