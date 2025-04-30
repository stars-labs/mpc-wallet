import { MESSAGE_PREFIX, MessageType } from '../../constants';

// 定义请求和响应的类型
type JsonRpcRequest = {
    id: number | string;
    jsonrpc: string;
    method: string;
    params?: any[];
};

type JsonRpcResponse = {
    id: number | string;
    jsonrpc: string;
    result?: any;
    error?: {
        code: number;
        message: string;
        data?: any;
    };
};

// 定义消息接口
export interface ContentMessage {
    type: string;
    payload: JsonRpcRequest | JsonRpcResponse;
}

export class ContentProvider {

    constructor() {
        // 监听来自页面的消息
        window.addEventListener('message', this.handlePageMessage);

        // 监听来自扩展的消息
        chrome.runtime.onMessage.addListener(this.handleExtensionMessage);
    }

    // 处理来自页面的消息
    private handlePageMessage = (event: MessageEvent) => {
        // 忽略来自其他源的消息
        if (event.source !== window) return;

        const data = event.data;

        // 检查消息是否来自我们的注入脚本
        if (data && typeof data === 'object' && data.type === `${MESSAGE_PREFIX}${MessageType.REQUEST}`) {
            // 解析请求
            const request = data.payload as JsonRpcRequest;

            // 发送请求到扩展的后台脚本
            chrome.runtime.sendMessage(
                { type: `${MESSAGE_PREFIX}${MessageType.REQUEST}`, payload: request },
                (response: JsonRpcResponse) => {
                    // 将响应发送回页面
                    window.postMessage(
                        {
                            type: `${MESSAGE_PREFIX}${MessageType.RESPONSE}`,
                            payload: response
                        },
                        '*'
                    );
                }
            );
        }
    };

    // 处理来自扩展的消息
    private handleExtensionMessage = (
        message: any,
        sender: chrome.runtime.MessageSender,
        sendResponse: (response: any) => void
    ) => {
        // 检查消息类型
        if (message && typeof message === 'object' && message.type === `${MESSAGE_PREFIX}${MessageType.RESPONSE}`) {
            // 将响应发送回页面
            window.postMessage(message, '*');
            return true; // 保持消息通道打开以进行异步响应
        }
        return false;
    };

    // 清理方法
    public cleanup() {
        window.removeEventListener('message', this.handlePageMessage);
        // 注意：Chrome 扩展 API 不提供 removeListener 的标准方式
    }
}

export default new ContentProvider(); 