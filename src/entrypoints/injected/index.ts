import { defineContentScript } from '#imports';
import { MESSAGE_PREFIX, MessageType, WALLET_INFO } from '../../constants';
import { v4 as uuidv4 } from 'uuid';
import Icon from '../../../public/assets/icon-128.png';
class PageProvider {
    private requestIdCounter = 0;
    private pendingRequests = new Map<number | string, (response: any) => void>();

    constructor() {
        // 添加消息监听器，用于接收来自 content script 的响应
        window.addEventListener('message', this.handleContentScriptMessage);
    }

    // 处理来自 content script 的消息
    private handleContentScriptMessage = (event: MessageEvent) => {
        // 安全检查：确保消息来自当前窗口
        if (event.source !== window) return;

        const data = event.data;

        // 验证消息格式
        if (
            data &&
            typeof data === 'object' &&
            data.type === `${MESSAGE_PREFIX}${MessageType.RESPONSE}` &&
            data.payload
        ) {
            const response = data.payload;
            const callback = this.pendingRequests.get(response.id);

            if (callback) {
                callback(response);
                this.pendingRequests.delete(response.id);
            }
        }
    };

    request = (args: any): Promise<any> => {
        return new Promise((resolve, reject) => {
            const id = this.requestIdCounter++;

            // 创建 JSON-RPC 请求
            const request = {
                id,
                jsonrpc: '2.0',
                method: args.method,
                params: args.params
            };

            // 存储回调
            this.pendingRequests.set(id, (response) => {
                if (response.error) {
                    reject(new Error(response.error.message));
                } else {
                    resolve(response.result);
                }
            });

            // 发送消息到 content script
            window.postMessage(
                {
                    type: `${MESSAGE_PREFIX}${MessageType.REQUEST}`,
                    payload: request
                },
                '*'
            );
        });
    }

    // 当 PageProvider 销毁时清理监听器
    disconnect() {
        window.removeEventListener('message', this.handleContentScriptMessage);
    }
}

export default defineUnlistedScript(() => {
    const info = {
        uuid: uuidv4(),
        name: WALLET_INFO.name,
        icon: Icon,
        rdns: WALLET_INFO.name,
    };

    const injectedProvider = new Proxy(new PageProvider(), {
        deleteProperty: (target, prop) => {
            if (typeof prop === 'string' && ['on'].includes(prop)) {
                // @ts-ignore
                delete target[prop];
            }
            return true;
        },
        get: (target, prop, receiver) => {
            const method = target[prop as keyof PageProvider];
            if (typeof method === 'function') {
                return (...args: any[]) => {
                    // @ts-ignore
                    return method.apply(target, args);
                };
            }

            return Reflect.get(target, prop, receiver);
        },
    });

    const createAndDispatchEvent = () => {
        try {
            const event = new CustomEvent(
                'eip6963:announceProvider',
                { detail: Object.freeze({ info, provider: injectedProvider }) }
            );
            window.dispatchEvent(event);
        } catch (error) {
            console.error('Failed to dispatch event:', error);
        }
    };

    window.addEventListener('eip6963:requestProvider', createAndDispatchEvent);
    createAndDispatchEvent();
});
