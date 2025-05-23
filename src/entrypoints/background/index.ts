import { defineBackground } from '#imports';
import { MESSAGE_PREFIX, MessageType } from '../../constants';
import AccountService from '../../services/accountService';
import NetworkService from '../../services/networkService';
import WalletClientService from '../../services/walletClient';
import { toHex } from 'viem';
import WalletController from "../../services/walletController";
import { WebSocketClient } from "./websocket";
import type {
    ServerMsg, ClientMsg, WebSocketMessagePayload, SessionProposal, SessionResponse, SessionInfo, WebRTCSignal
} from "../../types/appstate";
import {
    type JsonRpcRequest,
    type BackgroundMessage,
    type OffscreenMessage,
    type PopupMessage,
    type BackgroundToOffscreenMessage,
    type InitialStateMessage,
    validateMessage,
    validateSessionProposal,
    validateSessionAcceptance,
    isRpcMessage,
    isAccountManagement,
    isNetworkManagement,
    isUIRequest,
    MESSAGE_TYPES
} from "../../types/messages";
import { storage } from "#imports";

// Initialize services
const accountService = AccountService.getInstance();
const networkService = NetworkService.getInstance();
const walletClientService = WalletClientService.getInstance();

// 处理 RPC 请求
async function handleRpcRequest(request: JsonRpcRequest): Promise<unknown> {
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
async function rpcRequest(request: JsonRpcRequest): Promise<unknown> {
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
                method: method as any, // viem might need specific method types, using any for broadness
                params: params as any, // viem might need specific param types
            });
        } else {
            // 使用 walletClient 处理需要签名的操作
            return await walletClientService.getWalletClient().request({
                method: method as any,
                params: params as any,
            });
        }
    } catch (error) {
        console.error(`RPC request failed: ${method}`, error);
        throw error;
    }
}

async function handleUIRequest(request: { method: string; params: unknown[] }): Promise<{ success: boolean; data?: unknown; error?: string }> {
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

let popupPorts = new Set<chrome.runtime.Port>();
let offscreenDocumentReady = false;
let pendingOffscreenInitData: { peerId: string; wsUrl: string } | null = null;
let offscreenInitSent = false;
let offscreenCreationInProgress = false;

// Refined ensureOffscreenDocument function
async function ensureOffscreenDocument(): Promise<{ success: boolean; created?: boolean; message?: string; error?: string }> {
    if (!chrome.offscreen) {
        console.warn("[Background] chrome.offscreen API not available");
        return { success: false, error: "Offscreen API not available." };
    }

    // First, check if the document already exists.
    if (await chrome.offscreen.hasDocument()) {
        console.log("[Background] Offscreen document already exists.");
        if (!offscreenDocumentReady && pendingOffscreenInitData) {
            console.log("[Background] Offscreen exists but not yet ready. Will send init data upon 'offscreenReady' signal.");
        } else if (offscreenDocumentReady && pendingOffscreenInitData && !offscreenInitSent) {
            console.log("[Background] Offscreen exists and is marked ready. Attempting to resend init data if not already sent.");
            const initResult = await safelySendOffscreenMessage({
                type: "fromBackground",
                payload: {
                    type: "init",
                    ...pendingOffscreenInitData
                }
            }, "init");
            if (initResult.success) {
                console.log("[Background] Successfully resent init data to existing (and ready) offscreen document.");
                offscreenInitSent = true;
            } else {
                console.warn("[Background] Failed to resend init data to existing (and ready) offscreen document:", initResult.error);
            }
        }
        return { success: true, created: false, message: "Offscreen document already exists." };
    }

    // If document doesn't exist, check if creation is already in progress.
    if (offscreenCreationInProgress) {
        console.log("[Background] Offscreen document creation is already in progress. Awaiting completion.");
        return { success: true, created: false, message: "Offscreen document creation already in progress." };
    }

    // If document doesn't exist and no creation is in progress, proceed to create.
    console.log("[Background] Creating offscreen document as it does not exist.");
    offscreenCreationInProgress = true; // Set flag before starting creation

    try {
        const reasons: chrome.offscreen.Reason[] = [chrome.offscreen.Reason.DOM_SCRAPING];

        if (reasons.length === 0) {
            console.error("[Background] No valid reasons found for creating offscreen document. Aborting creation.");
            return { success: false, error: "No valid offscreen reasons available." };
        }

        await chrome.offscreen.createDocument({
            url: chrome.runtime.getURL('offscreen.html'),
            reasons: reasons,
            justification: 'Manages WebRTC connections and signaling for MPC sessions using DOM capabilities.',
        });
        console.log("[Background] Offscreen document created successfully. Waiting for 'offscreenReady' signal to send init data.");
        offscreenDocumentReady = false; // Explicitly set to false as we just created it
        offscreenInitSent = false;    // Reset init sent status
        return { success: true, created: true, message: "Offscreen document created. Awaiting ready signal." };

    } catch (e: any) {
        if (e.message && e.message.includes("Only a single offscreen document may be created")) {
            console.warn("[Background] Attempted to create offscreen document, but it already exists (caught specific error during creation attempt):", e.message);
            return { success: true, created: false, message: "Offscreen document already exists (creation conflict resolved)." };
        }
        console.error("[Background] Error with offscreen document operation:", e);
        return { success: false, error: e.message || "Unknown error with offscreen document." };
    } finally {
        offscreenCreationInProgress = false; // Reset flag after creation attempt (success or failure)
    }
}

// New helper function to safely send messages to offscreen document with retries
async function safelySendOffscreenMessage(message: BackgroundToOffscreenMessage, messageDescription: string = "message", maxRetries = 3, retryDelay = 500): Promise<{ success: boolean, error?: string }> {
    // First check if offscreen document actually exists
    if (!chrome.offscreen || !await chrome.offscreen.hasDocument()) {
        console.warn(`[Background] Cannot send ${messageDescription}: offscreen document does not exist`);
        return { success: false, error: "Offscreen document does not exist" };
    }

    let lastError = "";
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
        try {
            console.log(`[Background] Sending ${messageDescription} to offscreen document (attempt ${attempt}/${maxRetries}):`, message);
            await chrome.runtime.sendMessage(message);
            console.log(`[Background] ${messageDescription} successfully sent to offscreen document on attempt ${attempt}`);
            return { success: true };
        } catch (error: any) {
            lastError = error.message || "Unknown error";
            console.warn(`[Background] Failed to send ${messageDescription} to offscreen document (attempt ${attempt}/${maxRetries}):`, lastError);

            // If the error indicates the receiving end doesn't exist, try to recreate offscreen
            if (lastError.includes("Receiving end does not exist") || lastError.includes("Could not establish connection")) {
                console.log(`[Background] Attempting to recreate offscreen document due to connection error`);
                offscreenDocumentReady = false;
                offscreenInitSent = false;

                const recreateResult = await ensureOffscreenDocument();
                if (!recreateResult.success) {
                    console.error(`[Background] Failed to recreate offscreen document:`, recreateResult.error);
                    return { success: false, error: `Failed to recreate offscreen: ${recreateResult.error}` };
                }

                // Wait a bit longer for the recreated document to be ready
                await new Promise(resolve => setTimeout(resolve, 1000));
            }

            if (attempt < maxRetries) {
                console.log(`[Background] Waiting ${retryDelay}ms before retry...`);
                await new Promise(resolve => setTimeout(resolve, retryDelay));
                // Increase delay for next retry to give more time
                retryDelay = retryDelay * 1.5;
            }
        }
    }

    return { success: false, error: `Failed after ${maxRetries} attempts. Last error: ${lastError}` };
}

// WebSocket client setup
let wsClient: WebSocketClient | null = null;
let peers: string[] = [];
let appState = {
    peerId: "",
    connectedPeers: [] as string[],
    wsConnected: false,
    sessionInfo: null as SessionInfo | null,
    invites: [] as SessionInfo[],
    meshStatus: { type: 0 /* MeshStatusType */ },
    dkgState: 0 /* DkgState */
};

// Broadcast state to all connected popup ports
function broadcastToPopupPorts(message: PopupMessage) {
    popupPorts.forEach(port => {
        try {
            port.postMessage(message);
        } catch (error) {
            console.error("[Background] Error sending message to popup port:", error);
            popupPorts.delete(port);
        }
    });
}

export default defineBackground(() => {
    // Initialize WebSocket when background starts
    console.log("[Background] Background script starting...");

    // Set up popup port connections
    chrome.runtime.onConnect.addListener((port) => {
        if (port.name === "popup") {
            console.log("[Background] Popup connected");
            popupPorts.add(port);

            // Send current state to newly connected popup
            const initialStateMessage: InitialStateMessage = {
                type: "initialState",
                ...appState
            };
            console.log("[Background] Sending initial state to popup:", initialStateMessage);
            port.postMessage(initialStateMessage);

            port.onDisconnect.addListener(() => {
                console.log("[Background] Popup disconnected");
                popupPorts.delete(port);
            });
        }
    });

    // Initialize offscreen document on startup
    ensureOffscreenDocument().then(result => {
        console.log("[Background] Initial offscreen document setup:", result);
    });

    // Simplified message handling with runtime validation
    chrome.runtime.onMessage.addListener((message: unknown, sender, sendResponse) => {
        console.log("[Background] Received message:", message, "from sender:", sender);

        // Validate basic message structure
        if (!validateMessage(message)) {
            console.warn("[Background] Invalid message structure:", message);
            sendResponse({ success: false, error: "Invalid message structure" });
            return true;
        }

        // Handle async operations
        (async () => {
            try {
                // Simple switch on message type with runtime validation
                switch (message.type) {
                    case MESSAGE_TYPES.GET_STATE:
                        sendResponse(appState);
                        break;

                    case MESSAGE_TYPES.LIST_PEERS:
                        console.log("[Background] LIST_PEERS request received. WebSocket state:", wsClient?.getReadyState());
                        if (wsClient?.getReadyState() === WebSocket.OPEN) {
                            try {
                                console.log("[Background] Sending peer list request to server");
                                wsClient.listPeers();
                                console.log("[Background] Peer list request sent successfully");
                                sendResponse({ success: true });
                            } catch (error) {
                                console.error("[Background] Error requesting peer list:", error);
                                sendResponse({ success: false, error: (error as Error).message });
                            }
                        } else {
                            console.warn("[Background] WebSocket not connected, cannot list peers. Ready state:", wsClient?.getReadyState());
                            sendResponse({ success: false, error: "WebSocket not connected" });
                        }
                        break;

                    case MESSAGE_TYPES.RELAY:
                        if ('to' in message && 'data' in message && wsClient?.getReadyState() === WebSocket.OPEN) {
                            try {
                                await wsClient.relayMessage(message.to as string, message.data);
                                sendResponse({ success: true });
                            } catch (error) {
                                console.error("[Background] Error relaying message:", error);
                                sendResponse({ success: false, error: (error as Error).message });
                            }
                        } else {
                            sendResponse({ success: false, error: "Invalid relay message or WebSocket not connected" });
                        }
                        break;

                    case MESSAGE_TYPES.CREATE_OFFSCREEN:
                        const createResult = await ensureOffscreenDocument();
                        sendResponse(createResult);
                        break;

                    case MESSAGE_TYPES.GET_OFFSCREEN_STATUS:
                        const hasDocument = chrome.offscreen ? await chrome.offscreen.hasDocument() : false;
                        sendResponse({
                            hasDocument,
                            ready: offscreenDocumentReady,
                            initSent: offscreenInitSent
                        });
                        break;

                    case MESSAGE_TYPES.FROM_OFFSCREEN:
                        if ('payload' in message) {
                            await handleOffscreenMessage(message.payload as OffscreenMessage);
                            sendResponse({ success: true });
                        } else {
                            sendResponse({ success: false, error: "FromOffscreen message missing payload" });
                        }
                        break;

                    case MESSAGE_TYPES.OFFSCREEN_READY:
                        console.log("[Background] Received offscreenReady signal");
                        offscreenDocumentReady = true;

                        if (pendingOffscreenInitData && !offscreenInitSent) {
                            console.log("[Background] Sending pending init data to newly ready offscreen document");
                            const initResult = await safelySendOffscreenMessage({
                                type: "fromBackground",
                                payload: {
                                    type: "init",
                                    ...pendingOffscreenInitData
                                }
                            }, "init");

                            if (initResult.success) {
                                console.log("[Background] Successfully sent init data to offscreen document");
                                offscreenInitSent = true;
                                pendingOffscreenInitData = null;
                            } else {
                                console.warn("[Background] Failed to send init data to offscreen document:", initResult.error);
                            }
                        } else {
                            console.log("[Background] Offscreen ready but no pending init data or already sent. PendingData:", !!pendingOffscreenInitData, "InitSent:", offscreenInitSent);
                        }
                        sendResponse({ success: true });
                        break;

                    case "requestInit":
                        console.log("[Background] Received requestInit from offscreen");
                        if (pendingOffscreenInitData || appState.peerId) {
                            const initData = pendingOffscreenInitData || {
                                peerId: appState.peerId,
                                wsUrl: "wss://auto-life.tech"
                            };

                            console.log("[Background] Sending init data in response to request:", initData);
                            const initResult = await safelySendOffscreenMessage({
                                type: "fromBackground",
                                payload: {
                                    type: "init",
                                    ...initData
                                }
                            }, "requestedInit");

                            if (initResult.success) {
                                console.log("[Background] Successfully sent requested init data");
                                offscreenInitSent = true;
                                pendingOffscreenInitData = null;
                                sendResponse({ success: true, message: "Init data sent" });
                            } else {
                                console.warn("[Background] Failed to send requested init data:", initResult.error);
                                sendResponse({ success: false, error: initResult.error });
                            }
                        } else {
                            console.warn("[Background] No init data available to send");
                            sendResponse({ success: false, error: "No init data available" });
                        }
                        break;

                    // Handle legacy message types
                    default:
                        if (isRpcMessage(message)) {
                            const result = await handleRpcRequest(message.payload);
                            sendResponse({ success: true, result });
                        } else if (isAccountManagement(message)) {
                            sendResponse({ success: false, error: "Account management not implemented" });
                        } else if (isNetworkManagement(message)) {
                            sendResponse({ success: false, error: "Network management not implemented" });
                        } else if (isUIRequest(message)) {
                            const result = await handleUIRequest(message.payload);
                            sendResponse(result);
                        } else {
                            console.warn("[Background] Unknown message type:", message.type);
                            sendResponse({ success: false, error: `Unknown message type: ${message.type}` });
                        }
                        break;
                }
            } catch (error) {
                console.error("[Background] Error handling message:", error);
                sendResponse({ success: false, error: (error as Error).message });
            }
        })();

        return true;
    });

    // Simplified offscreen message handling
    async function handleOffscreenMessage(payload: OffscreenMessage) {
        console.log("[Background] Handling offscreen message:", payload);

        switch (payload.type) {
            case "log":
                if ('message' in payload) {
                    console.log("[Offscreen Log]", payload.message);
                }
                break;

            case "relayViaWs":
                if ('to' in payload && 'data' in payload && wsClient?.getReadyState() === WebSocket.OPEN) {
                    try {
                        await wsClient.relayMessage(payload.to as string, payload.data);
                    } catch (error) {
                        console.error("[Background] Error relaying via WebSocket:", error);
                    }
                } else {
                    console.warn("[Background] Cannot relay message, WebSocket not connected or invalid payload");
                }
                break;

            default:
                // Forward other messages to popup
                broadcastToPopupPorts({
                    type: "fromOffscreen",
                    payload
                });
                break;
        }
    }

    // Initialize WebSocket connection
    const initializeWebSocket = async () => {
        try {
            const WEBSOCKET_URL = "wss://auto-life.tech";
            wsClient = new WebSocketClient(WEBSOCKET_URL);

            // Generate peer ID
            appState.peerId = "mpc-2";
            console.log("[Background] Generated peer ID:", appState.peerId);

            // Set up event handlers BEFORE connecting
            console.log("[Background] Setting up WebSocket event handlers");

            // Set up event handlers using the WebSocketClient's callback system
            wsClient.onOpen(() => {
                console.log("[Background] WebSocket onOpen event triggered - connection established");
                appState.wsConnected = true;

                // Broadcast connection status immediately to any connected popups
                console.log("[Background] Broadcasting wsConnected=true to popups. Current popup ports:", popupPorts.size);
                broadcastToPopupPorts({ type: "wsStatus", connected: true });

                // Also broadcast updated full state
                const stateUpdate = {
                    type: "initialState" as const,
                    ...appState
                };
                console.log("[Background] Broadcasting full state update:", stateUpdate);
                broadcastToPopupPorts(stateUpdate);

                // Register with server
                console.log("[Background] Registering with server as peer:", appState.peerId);
                try {
                    wsClient!.register(appState.peerId);
                    console.log("[Background] Registration sent to server");
                } catch (regError) {
                    console.error("[Background] Error during registration:", regError);
                }

                // Request initial peer list with delay to ensure registration is processed
                setTimeout(() => {
                    console.log("[Background] Requesting initial peer list from server");
                    if (wsClient && wsClient.getReadyState() === WebSocket.OPEN) {
                        wsClient.listPeers();
                        console.log("[Background] Initial peer list request sent successfully");
                    } else {
                        console.warn("[Background] WebSocket not ready for peer list request");
                    }
                }, 1000); // 1 second delay

                // Store init data for offscreen
                pendingOffscreenInitData = {
                    peerId: appState.peerId,
                    wsUrl: WEBSOCKET_URL
                };

                console.log("[Background] Stored pending init data:", pendingOffscreenInitData);

                // Send to offscreen if ready
                if (offscreenDocumentReady && !offscreenInitSent) {
                    console.log("[Background] Offscreen is ready, sending init data immediately");
                    safelySendOffscreenMessage({
                        type: "fromBackground",
                        payload: {
                            type: "init",
                            ...pendingOffscreenInitData
                        }
                    }, "init").then(result => {
                        if (result.success) {
                            console.log("[Background] Successfully sent init data immediately");
                            offscreenInitSent = true;
                            pendingOffscreenInitData = null;
                        } else {
                            console.warn("[Background] Failed to send init data immediately:", result.error);
                        }
                    });
                } else {
                    console.log("[Background] Offscreen not ready yet, init data will be sent when ready. Ready:", offscreenDocumentReady, "InitSent:", offscreenInitSent);

                    // If offscreen is ready but we haven't sent init yet, try again with the new pending data
                    if (offscreenDocumentReady && !offscreenInitSent) {
                        console.log("[Background] Offscreen was ready, retrying init data send");
                        setTimeout(() => {
                            if (pendingOffscreenInitData && !offscreenInitSent) {
                                safelySendOffscreenMessage({
                                    type: "fromBackground",
                                    payload: {
                                        type: "init",
                                        ...pendingOffscreenInitData
                                    }
                                }, "init").then(result => {
                                    if (result.success) {
                                        console.log("[Background] Successfully sent init data on retry");
                                        offscreenInitSent = true;
                                        pendingOffscreenInitData = null;
                                    } else {
                                        console.warn("[Background] Failed to send init data on retry:", result.error);
                                    }
                                });
                            }
                        }, 100); // Small delay to ensure state is consistent
                    }
                }
            });

            wsClient.onClose((event) => {
                console.log("[Background] WebSocket onClose event triggered, event:", event);
                appState.wsConnected = false;

                // Broadcast disconnection status
                console.log("[Background] Broadcasting wsConnected=false to popups");
                broadcastToPopupPorts({ type: "wsStatus", connected: false });

                // Also broadcast updated state
                broadcastToPopupPorts({
                    type: "initialState",
                    ...appState
                });
            });

            wsClient.onError((error) => {
                console.error("[Background] WebSocket onError event triggered, error:", error);
                appState.wsConnected = false;

                // Broadcast error and disconnection status
                broadcastToPopupPorts({
                    type: "wsError",
                    error: error.toString()
                });
                broadcastToPopupPorts({ type: "wsStatus", connected: false });

                // Also broadcast updated state
                broadcastToPopupPorts({
                    type: "initialState",
                    ...appState
                });
            });

            // Set up the message handler
            wsClient.onMessage((message: ServerMsg) => {
                console.log("[Background] WebSocket message received:", message);
                broadcastToPopupPorts({ type: "wsMessage", message });

                // Helper function to handle relay messages
                const handleRelayMessage = (msg: ServerMsg, messageType: string) => {
                    console.log(`[Background] Received ${messageType} message from server:`, msg);
                    if (msg.from) {
                        safelySendOffscreenMessage({
                            type: "fromBackground",
                            payload: {
                                type: "relayMessage",
                                fromPeerId: msg.from,
                                data: msg.data || {}
                            }
                        }, "relay message").then(result => {
                            if (!result.success) {
                                console.warn("[Background] Failed to relay message to offscreen:", result.error);
                            }
                        });
                    } else {
                        console.warn(`[Background] ${messageType} message missing 'from' field:`, msg);
                    }
                };

                // Helper function to handle peer list messages
                const handlePeerListMessage = (msg: ServerMsg, messageType: string) => {
                    const peerList = msg.peers || [];
                    peers = peerList;
                    appState.connectedPeers = peerList;
                    console.log(`[Background] Updated peer list from server (${messageType}):`, peerList);

                    // Broadcast peer list update
                    broadcastToPopupPorts({ type: "peerList", peers: peerList });

                    // Also broadcast updated state
                    broadcastToPopupPorts({
                        type: "initialState",
                        ...appState
                    });
                };

                // Handle specific message types with proper null checks
                switch (message.type) {
                    case "peers": // Handle lowercase "peers" messages from server
                    case "PeerList": // Handle uppercase "PeerList" messages for compatibility
                        handlePeerListMessage(message, message.type);
                        break;

                    case "relay": // Handle lowercase "relay" messages from server
                    case "Relay": // Handle uppercase "Relay" messages for compatibility
                        handleRelayMessage(message, message.type);
                        break;

                    default:
                        console.log("[Background] Unhandled WebSocket message type:", message.type);
                        break;
                }
            });

            // Now connect after event handlers are set up
            console.log("[Background] Event handlers configured, attempting to connect to WebSocket:", WEBSOCKET_URL);
            wsClient.connect();
            console.log("[Background] WebSocket connect() method completed");

            // Enhanced manual check for connection state after connection attempt
            setTimeout(() => {
                if (wsClient) {
                    const readyState = wsClient.getReadyState();
                    console.log("[Background] WebSocket ready state after connection attempt:", readyState);
                    if (readyState === WebSocket.OPEN) {
                        console.log("[Background] WebSocket is open, updating state and triggering registration");
                        if (!appState.wsConnected) {
                            appState.wsConnected = true;

                            // Broadcast the connection status
                            broadcastToPopupPorts({ type: "wsStatus", connected: true });
                            broadcastToPopupPorts({
                                type: "initialState",
                                ...appState
                            });

                            // Also trigger registration and peer list request
                            console.log("[Background] Manually triggering registration and peer list request");
                            try {
                                wsClient.register(appState.peerId);
                                console.log("[Background] Manual registration sent to server");

                                // Request peer list after a delay
                                setTimeout(() => {
                                    if (wsClient && wsClient.getReadyState() === WebSocket.OPEN) {
                                        wsClient.listPeers();
                                        console.log("[Background] Manual peer list request sent successfully");
                                    }
                                }, 1000);
                            } catch (regError) {
                                console.error("[Background] Error during manual registration:", regError);
                            }
                        }
                    } else if (readyState === WebSocket.CONNECTING) {
                        console.log("[Background] WebSocket still connecting...");
                    } else {
                        console.error("[Background] WebSocket connection failed. Ready state:", readyState);
                    }
                }
            }, 2000); // Check after 2 seconds

        } catch (error) {
            console.error("[Background] Failed to initialize WebSocket:", error);
            appState.wsConnected = false;

            broadcastToPopupPorts({
                type: "wsError",
                error: error instanceof Error ? error.message : "Unknown error"
            });
            broadcastToPopupPorts({ type: "wsStatus", connected: false });

            // Also broadcast updated state
            broadcastToPopupPorts({
                type: "initialState",
                ...appState
            });
        }
    };

    // Start WebSocket connection
    initializeWebSocket();

    console.log("[Background] Background script initialized");
});