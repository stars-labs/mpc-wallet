import { defineBackground } from '#imports';
import { MESSAGE_PREFIX, MessageType } from '../../constants';
import AccountService from '../../services/accountService';
import NetworkService from '../../services/networkService';
import WalletClientService from '../../services/walletClient';
import { toHex } from 'viem';
import WalletController from "../../services/walletController";

// Initialize services
const accountService = AccountService.getInstance();
const networkService = NetworkService.getInstance();
const walletClientService = WalletClientService.getInstance();

// 处理 RPC 请求
async function handleRpcRequest(request: any) {
    try {
        // 根据请求的方法执行相应的操作
        switch (request.method) {
            case 'eth_accounts':
            case 'eth_requestAccounts':
                // 返回当前选中的账户地址
                const currentAccount = accountService.getCurrentAccount();
                if (!currentAccount) {
                    throw new Error('No account selected');
                }
                return [currentAccount.address];

            case 'eth_chainId':
                // 返回当前网络的 chainId
                const currentNetwork = networkService.getCurrentNetwork();
                if (!currentNetwork) {
                    throw new Error('No current network found');
                }
                return toHex(currentNetwork.id);

            case 'net_version':
                // 返回当前网络的 chainId
                const network = networkService.getCurrentNetwork();
                if (!network) {
                    throw new Error('No current network found');
                }
                return toHex(network.id);

            default:
                // 使用 walletClient 处理其他 RPC 请求
                return await rpcRequest(request);
        }
    } catch (error) {
        console.error('RPC request error:', error);
        throw error;
    }
}

// 通用的 RPC 请求处理函数
async function rpcRequest(request: any) {
    const { method, params } = request;

    // 检查是否是只读操作
    const isReadOnly = [
        'eth_getBalance',
        'eth_getTransactionCount',
        'eth_getBlockByNumber',
        'eth_getBlockByHash',
        'eth_getTransactionByHash',
        'eth_getTransactionReceipt',
        'eth_call',
        'eth_estimateGas',
        'eth_getLogs'
    ].includes(method);

    try {
        if (isReadOnly) {
            // 使用 publicClient 处理只读操作
            return await walletClientService.getPublicClient().request({
                method,
                params
            });
        } else {
            // 使用 walletClient 处理需要签名的操作
            return await walletClientService.getWalletClient().request({
                method,
                params
            });
        }
    } catch (error) {
        console.error(`RPC request failed: ${method}`, error);
        throw error;
    }
}

// 处理账户管理请求
async function handleAccountManagement(action: string, payload: any) {
    try {
        console.log(action, payload);
        switch (action) {
            case 'addAccount':
                const newAccount = await accountService.addAccount(payload);
                return { success: true, data: newAccount };
            case 'removeAccount':
                await accountService.removeAccount(payload.address);
                return { success: true };
            case 'updateAccount':
                await accountService.updateAccount(payload);
                return { success: true };
            case 'getAccounts':
                return { success: true, data: accountService.getAccounts() };
            case 'getAccount':
                const account = accountService.getAccount(payload.address);
                return { success: true, data: account };
            case 'getCurrentAccount':
                const currentAccount = accountService.getCurrentAccount();
                return { success: true, data: currentAccount };
            default:
                return { success: false, error: 'Unknown action' };
        }
    } catch (error) {
        console.error('Account management error:', error);
        return { success: false, error: (error as Error).message };
    }
}

// 处理网络管理请求
async function handleNetworkManagement(action: string, payload: any) {
    try {
        switch (action) {
            case 'addNetwork':
                await networkService.addNetwork(payload);
                return { success: true };
            case 'removeNetwork':
                await networkService.removeNetwork(payload.chainId);
                return { success: true };
            case 'updateNetwork':
                await networkService.updateNetwork(payload);
                return { success: true };
            case 'getNetworks':
                return { success: true, data: networkService.getNetworks() };
            case 'getNetwork':
                const network = networkService.getNetwork(payload.chainId);
                return { success: true, data: network };
            case 'getCurrentNetwork':
                const currentNetwork = networkService.getCurrentNetwork();
                return { success: true, data: currentNetwork };
            case 'setCurrentNetwork':
                await networkService.setCurrentNetwork(payload.chainId);
                return { success: true };
            default:
                return { success: false, error: 'Unknown action' };
        }
    } catch (error) {
        console.error('Network management error:', error);
        return { success: false, error: (error as Error).message };
    }
}

async function handleUIRequest(request: { method: string; params: unknown[] }) {
    const { method, params } = request;
    const walletController = WalletController.getInstance();

    if (typeof walletController[method as keyof WalletController] === 'function') {
        try {
            const result = await (walletController[method as keyof WalletController] as (...args: unknown[]) => unknown)(...params);
            return { success: true, data: result };
        } catch (error) {
            return { success: false, error: error instanceof Error ? error.message : 'Unknown error' };
        }
    }

    return { success: false, error: `Method ${method} not found on WalletController` };
}

export default defineBackground({
    main() {
        // 监听来自 content script 的消息
        chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
            // 检查消息是否来自我们的 content script
            if (
                message &&
                typeof message === 'object' &&
                message.type === `${MESSAGE_PREFIX}${MessageType.REQUEST}`
            ) {
                const request = message.payload;
                console.log('onMessage', message, sender, sendResponse);
                // 处理 RPC 请求
                handleRpcRequest(request)
                    .then(result => {
                        // 发送成功响应
                        sendResponse({
                            id: request.id,
                            jsonrpc: request.jsonrpc,
                            result
                        });
                    })
                    .catch(error => {
                        // 发送错误响应
                        sendResponse({
                            id: request.id,
                            jsonrpc: request.jsonrpc,
                            error: {
                                code: -32603, // Internal error
                                message: error.message || 'Internal error'
                            }
                        });
                    });

                // 返回 true 表示我们将异步发送响应
                return true;
            }

            // 处理账户管理请求
            if (message.type === MessageType.ACCOUNT_MANAGEMENT) {
                const { action, payload } = message;
                handleAccountManagement(action, payload)
                    .then(response => sendResponse(response));
                return true; // Keep the message channel open for async response
            }

            // 处理网络管理请求
            if (message.type === MessageType.NETWORK_MANAGEMENT) {
                const { action, payload } = message;
                handleNetworkManagement(action, payload)
                    .then(response => sendResponse(response));
                return true; // Keep the message channel open for async response
            }

            if (message.type === MessageType.UI_REQUEST) {
                handleUIRequest(message.payload)
                    .then(sendResponse)
                    .catch(error => sendResponse({ success: false, error: error.message }));
                return true; // 保持消息通道开放以支持异步响应
            }
        });
    },
}); 