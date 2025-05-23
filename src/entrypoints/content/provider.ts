import { MESSAGE_PREFIX, MessageType } from '../../constants';
import type {
    ContentToBackgroundMsg,
    BackgroundToContentMsg,
    InjectedToContentMsg,
    ContentToInjectedMsg,
    JsonRpcRequest,
    JsonRpcResponse
} from '../../types/messages';

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

        const data = event.data as InjectedToContentMsg;

        // 检查消息是否来自我们的注入脚本
        if (data && typeof data === 'object' && data.type === `${MESSAGE_PREFIX}${MessageType.REQUEST}`) {
            // 解析请求
            const request = data.payload as JsonRpcRequest;

            // 转发到 background script
            this.forwardToBackground(request);
        }
    };

    // 处理来自扩展的消息
    private handleExtensionMessage = (
        message: BackgroundToContentMsg,
        sender: chrome.runtime.MessageSender,
        sendResponse: (response?: any) => void
    ) => {
        // 转发响应到页面
        if (message.type === 'REQUEST_RESPONSE') {
            this.sendToPage({
                type: 'WALLET_RESPONSE',
                payload: message.payload
            });
        }
        // Handle other message types...
    };

    private forwardToBackground(request: JsonRpcRequest) {
        const message: ContentToBackgroundMsg = {
            type: 'FORWARD_REQUEST',
            payload: request,
            timestamp: Date.now()
        };

        chrome.runtime.sendMessage(message);
    }

    private sendToPage(message: ContentToInjectedMsg) {
        window.postMessage({
            type: `${MESSAGE_PREFIX}${MessageType.RESPONSE}`,
            ...message
        }, '*');
    }

    // 清理方法
    public cleanup() {
        window.removeEventListener('message', this.handlePageMessage);
        // 注意：Chrome 扩展 API 不提供 removeListener 的标准方式
    }
}

export default new ContentProvider();