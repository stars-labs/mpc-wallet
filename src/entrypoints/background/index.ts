import { defineBackground } from '#imports';
import { MESSAGE_PREFIX, MessageType } from '../../constants';
import AccountService from '../../services/accountService';
import NetworkService from '../../services/networkService';
import WalletClientService from '../../services/walletClient';
import { toHex } from 'viem';
import WalletController from "../../services/walletController";
import { WebSocketClient, type ClientMsg } from "./websocket";
import type { ServerMsg } from "./types";
import { WebRTCManager, type WebSocketMessagePayload, type SessionProposal, type SessionInfo } from "./webrtc";
import { storage } from "#imports";

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

let popupPorts = new Set<chrome.runtime.Port>();

export default defineBackground({
    main() {
        // Start WebSocket and WebRTC initialization (SINGLE CALL HERE)
        initializeConnection();

        // Listener for messages from popups (SINGLE LISTENER)
        chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
            console.log("[Background] Message from popup:", message); // Log sender info if available from 'sender'

            // Handle RPC Requests
            if (
                message &&
                typeof message === 'object' &&
                message.type === `${MESSAGE_PREFIX}${MessageType.REQUEST}`
            ) {
                const request = message.payload;
                handleRpcRequest(request)
                    .then(result => {
                        sendResponse({
                            id: request.id,
                            jsonrpc: request.jsonrpc,
                            result
                        });
                    })
                    .catch(error => {
                        sendResponse({
                            id: request.id,
                            jsonrpc: request.jsonrpc,
                            error: {
                                code: -32603,
                                message: error.message || 'Internal error'
                            }
                        });
                    });
                return true;
            }

            // Handle Account Management
            if (message.type === MessageType.ACCOUNT_MANAGEMENT) {
                const { action, payload } = message;
                handleAccountManagement(action, payload)
                    .then(response => sendResponse(response));
                return true;
            }

            // Handle Network Management
            if (message.type === MessageType.NETWORK_MANAGEMENT) {
                const { action, payload } = message;
                handleNetworkManagement(action, payload)
                    .then(response => sendResponse(response));
                return true;
            }

            // Handle UI Requests
            if (message.type === MessageType.UI_REQUEST) {
                handleUIRequest(message.payload)
                    .then(sendResponse)
                    .catch(error => sendResponse({ success: false, error: error.message }));
                return true;
            }

            // Handle MPC-specific messages
            switch (message.type) {
                case "getState":
                    const statePayload = {
                        peerId,
                        connectedPeers, // This is the crucial part
                        wsConnected: wsClient?.getReadyState() === WebSocket.OPEN,
                        sessionInfo: null, // No direct webrtcManager usage
                        invites: [], // No direct webrtcManager usage
                        meshStatus: { type: 0 }, // MeshStatusType.Incomplete
                        dkgState: 0 // DkgState.Idle
                    };
                    console.log("[Background] Handling getState. Current connectedPeers:", connectedPeers, "Sending payload:", statePayload);
                    sendResponse(statePayload);
                    return true; // Keep open for async response

                case "listPeers":
                    wsClient?.listPeers(); // This will trigger an async "peers" message from server
                    sendResponse({ success: true, message: "listPeers command sent" }); // Respond immediately
                    return true; // Indicate async nature if wsClient.listPeers() itself was async, but it's likely fire-and-forget to server

                case "relay":
                    wsClient?.relayMessage(message.to, message.data);
                    sendResponse({ success: true });
                    return true;

                case "proposeSession":
                    chrome.runtime.sendMessage({
                        type: "toOffscreen",
                        payload: {
                            type: "proposeSession",
                            sessionId: message.sessionId,
                            total: message.total,
                            threshold: message.threshold,
                            participants: message.participants
                        }
                    });
                    sendResponse({ success: true });
                    return true;

                case "acceptSession":
                    chrome.runtime.sendMessage({
                        type: "toOffscreen",
                        payload: {
                            type: "acceptSession",
                            sessionId: message.sessionId
                        }
                    });
                    sendResponse({ success: true });
                    return true;
            }
            // If no specific handler matched and it's not an async response,
            // you might want to send a default response or nothing.
            // sendResponse({ success: false, error: "Unknown message type or not handled." }); 
            // Be careful with returning true if not all paths sendResponse.
        });

        // Handle connections from popups
        if (typeof chrome !== 'undefined' && chrome.runtime && chrome.runtime.onConnect) {
            chrome.runtime.onConnect.addListener(function (port) {
                if (port.name === "popup") {
                    popupPorts.add(port);
                    console.log("[Background] Popup port connected:", port.name, "Total ports:", popupPorts.size);

                    // Immediately send current state to the newly connected popup
                    try {
                        const currentState = {
                            type: "initialState",
                            peerId,
                            connectedPeers: [...connectedPeers], // Send a copy of current connectedPeers
                            wsConnected: wsClient?.getReadyState() === WebSocket.OPEN,
                            sessionInfo: null, // No direct webrtcManager usage
                            invites: [], // No direct webrtcManager usage
                            meshStatus: { type: 0 },
                            dkgState: 0,
                        };
                        console.log("[Background] Sending 'initialState' to new popup port. connectedPeers:", currentState.connectedPeers);
                        port.postMessage(currentState);
                    } catch (e) {
                        console.error("[Background] Error sending initial state to popup port:", e);
                    }

                    port.onDisconnect.addListener(function () {
                        popupPorts.delete(port);
                        console.log("[Background] Popup port disconnected. Total ports:", popupPorts.size);
                    });
                } else {
                    console.log("[Background] Port connected (not popup):", port.name);
                    port.onDisconnect.addListener(function () {
                        console.log("[Background] Port disconnected (not popup):", port.name);
                    });
                }
            });
        }
    },
    persistent: true,
});

// Initialize websocket and webrtc state
let wsClient: WebSocketClient | null = null;
let webrtcManager: WebRTCManager | null = null; // Add this declaration
let peerId: string = "";
let connectedPeers: string[] = [];

// Add this at the global scope to keep track of active session proposals
let sessionProposals: {
    [sessionId: string]: {
        proposer: string;
        data: any;
    }
} = {};

// Notify all open popups of state changes
function notifyPopups(message: any) {
    if (popupPorts.size > 0) {
        console.log(`[Background] Notifying ${popupPorts.size} popup ports with message:`, message);
        popupPorts.forEach(port => {
            try {
                port.postMessage(message);
            } catch (e) {
                console.error("[Background] Error posting message to a popup port (port might be closed or invalid):", e);
                // It's generally better to let the onDisconnect handler remove the port
            }
        });
    } else {
        // This log indicates that no popup is currently connected via a port to receive this specific push.
        // The popup will rely on its polling (`getState`) or the `initialState` message upon its next connection.
        console.warn("[Background] notifyPopups: No active popup ports. Message not pushed via port:", message);
    }
}

// Initialize the WebSocket connection and WebRTC manager
async function initializeConnection() {
    try {
        const WEBSOCKET_URL = "wss://auto-life.tech";
        // Ensure peerId is loaded before wsClient initialization if wsClient uses it immediately
        const storedPeerId = await storage.getItem<string>("local:peer_id");
        peerId = storedPeerId || `mpc-2`; // Ensure unique peerId
        if (!storedPeerId) {
            await storage.setItem("local:peer_id", peerId);
        }
        console.log("[Background] Using Peer ID:", peerId);

        // Try to create offscreen document at initialization
        await ensureOffscreenDocument();

        // Initialize WebSocket client
        wsClient = new WebSocketClient(WEBSOCKET_URL);

        wsClient.onOpen(() => {
            console.log("[Background] WebSocket connected. Registering peer:", peerId);
            wsClient?.register(peerId);
            notifyPopups({ type: "wsStatus", connected: true });
        });

        wsClient.onMessage((message: ServerMsg) => {
            console.log("[Background] Message from server:", message);

            // General notification for all server messages (e.g., for logging in popup)
            notifyPopups({ type: "wsMessage", message });

            if (message.type === "peers") {
                const newPeers = message.peers || [];
                // Update global connectedPeers only if it has changed
                if (JSON.stringify(connectedPeers) !== JSON.stringify(newPeers)) {
                    console.log("[Background] Peers list changed. Old:", connectedPeers, "New:", newPeers);
                    connectedPeers = [...newPeers]; // Update the global list reactively
                }
                // Always send peerList update to ensure popups get the latest, even if it's the same.
                // Or, only send if changed, but initialState on connect should cover late popups.
                notifyPopups({ type: "peerList", peers: [...connectedPeers] });

            } else if (message.type === "relay") {
                if (message.from) {
                    handleRelayMessage(message.from, message.data);
                } else {
                    console.error("[Background] Relay message received without 'from' field:", message);
                }
            }
        });

        wsClient.onError((event: Event) => {
            console.error("[Background] WebSocket error:", event);
            notifyPopups({ type: "wsError", error: "Connection error" });
        });

        wsClient.onClose((event: CloseEvent) => {
            console.log("[Background] WebSocket disconnected:", event.reason);
            notifyPopups({ type: "wsStatus", connected: false, reason: event.reason });

            // Try to reconnect after a delay
            setTimeout(() => {
                console.log("[Background] Attempting to reconnect...");
                wsClient?.connect();
            }, 5000); // Add timeout value - 5000ms
        }); // <-- Fixed: Properly close the onClose handler

        // Initialize WebRTC Manager - moved outside the onClose handler
        webrtcManager = new WebRTCManager(wsClient, peerId);

        // Set up WebRTC event handlers
        webrtcManager.onLog = (message) => {
            console.log(`[WebRTC] ${message}`);
        };

        webrtcManager.onSessionUpdate = (sessionInfo, invites) => {
            notifyPopups({ type: "sessionUpdate", sessionInfo, invites });
        };

        webrtcManager.onMeshStatusUpdate = (status) => {
            notifyPopups({ type: "meshStatusUpdate", status });
        };

        webrtcManager.onWebRTCAppMessage = (fromPeerId, message) => {
            console.log(`[WebRTC] App message from ${fromPeerId}:`, message);
            notifyPopups({ type: "webrtcMessage", fromPeerId, message });
        };

        webrtcManager.onDkgStateUpdate = (state) => {
            notifyPopups({ type: "dkgStateUpdate", state });
        };

        wsClient.connect();

    } catch (error) {
        console.error("[Background] Initialization error:", error);
    }
}

// Handle relay messages - this is key for processing session proposals
function handleRelayMessage(fromPeerId: string, data: any) {
    try {
        // Log the raw data for debugging
        console.log(`[Background] Received relay message from ${fromPeerId}:`, JSON.stringify(data));

        // Forward to offscreen document first - let it handle WebRTC processing
        chrome.runtime.sendMessage({
            type: "fromBackground",
            payload: {
                type: "relayMessage",
                fromPeerId,
                data
            }
        }).catch(err => {
            console.error("[Background] Error forwarding relay to offscreen:", err);
            // Fallback to local processing if offscreen fails
            processRelayMessageLocally(fromPeerId, data);
        });
    } catch (error) {
        console.error(`[Background] Error handling relay message from ${fromPeerId}:`, error);
    }
}

// Fallback processing for relay messages if offscreen document isn't available
function processRelayMessageLocally(fromPeerId: string, data: any) {
    // Check for websocket_msg_type which is present in both formats
    if (data && data.websocket_msg_type) {
        console.log(`[Background] Fallback processing ${data.websocket_msg_type} from ${fromPeerId}`);

        // Handle different message formats:
        // Format 1: { websocket_msg_type: "X", data: { ... } }
        // Format 2: { websocket_msg_type: "X", session_id: "...", ... } (direct properties)

        switch (data.websocket_msg_type) {
            case "SessionProposal":
                if (webrtcManager) {
                    let proposalData: any;

                    // Handle Format 1: Nested in data property
                    if (data.data && data.data.session_id) {
                        proposalData = {
                            session_id: data.data.session_id,
                            total: data.data.total,
                            threshold: data.data.threshold,
                            participants: data.data.participants
                        };
                    }
                    // Handle Format 2: Direct properties
                    else if (data.session_id) {
                        proposalData = {
                            session_id: data.session_id,
                            total: data.total,
                            threshold: data.threshold,
                            participants: data.participants
                        };
                    }

                    console.log("[Background] Parsed session proposal:", proposalData);

                    if (proposalData && proposalData.session_id && Array.isArray(proposalData.participants)) {
                        // Log session details for debugging
                        console.log(`[Background] Session "${proposalData.session_id}" proposed with ${proposalData.total} participants, threshold ${proposalData.threshold}`);
                        console.log(`[Background] Participants: ${JSON.stringify(proposalData.participants)}`);

                        // Check if current peer is in participants list
                        const isParticipant = proposalData.participants.includes(peerId);
                        console.log(`[Background] Is local peer (${peerId}) a participant? ${isParticipant}`);

                        webrtcManager.handleSessionProposal(fromPeerId, proposalData);

                        // Store session proposal in case WebRTC signals arrive before session is fully established
                        sessionProposals[proposalData.session_id] = {
                            proposer: fromPeerId,
                            data: proposalData
                        };

                    } else {
                        console.error("[Background] Invalid SessionProposal content:", data);
                    }
                }
                break;

            case "SessionResponse":
                if (webrtcManager) {
                    // ...existing code...
                }
                break;

            case "WebRTCSignal":
                if (webrtcManager) {
                    console.log("[Background] Processing WebRTC signal:", data);
                    let signalDataForManager: any = null;
                    let sessionId = null;

                    // Try to identify the session ID from the signal if possible
                    if (data.session_id) {
                        sessionId = data.session_id;
                    } else if (data.data && data.data.session_id) {
                        sessionId = data.data.session_id;
                    }

                    if (sessionId) {
                        console.log(`[Background] WebRTC signal for session: ${sessionId}`);
                    }

                    // Get WebRTC signal content
                    // Handle Format 1: Nested in data property
                    if (data.data) {
                        const signalContent = data.data;
                        if (signalContent.Offer) {
                            signalDataForManager = { type: 'Offer', data: signalContent.Offer };
                        } else if (signalContent.Answer) {
                            signalDataForManager = { type: 'Answer', data: signalContent.Answer };
                        } else if (signalContent.Candidate) {
                            signalDataForManager = { type: 'Candidate', data: signalContent.Candidate };
                        }
                    }
                    // Handle Format 2: Direct properties
                    else if (data.Offer) {
                        signalDataForManager = { type: 'Offer', data: data.Offer };
                    } else if (data.Answer) {
                        signalDataForManager = { type: 'Answer', data: data.Answer };
                    } else if (data.Candidate) {
                        signalDataForManager = { type: 'Candidate', data: data.Candidate };
                    }

                    if (signalDataForManager) {
                        console.log("[Background] Formatted WebRTC signal for manager:", signalDataForManager);

                        // Log the current WebRTC state
                        const activeSession = webrtcManager.sessionInfo ?
                            webrtcManager.sessionInfo.session_id : "none";
                        console.log(`[Background] Current active session: ${activeSession}`);

                        // Attempt to process the WebRTC signal
                        webrtcManager.handleWebRTCSignal(fromPeerId, signalDataForManager);

                        // Log any errors or rejection reasons
                        if (webrtcManager.lastError) {
                            console.warn(`[Background] WebRTCManager error: ${webrtcManager.lastError}`);
                        }
                    } else {
                        console.error("[Background] Unrecognized WebRTC signal content:", data);
                    }
                }
                break;
            default:
                console.log(`[Background] Unknown websocket_msg_type: ${data.websocket_msg_type}`);
                break;
        }
    } else {
        console.log(`[Background] Unknown or malformed relay message format from ${fromPeerId}:`, data);
    }
}

// Utility to ensure offscreen document exists (call only in background)
async function ensureOffscreenDocument() {
    if (!chrome.offscreen) {
        console.warn("[Background] chrome.offscreen API not available");
        return false;
    }

    try {
        const has = await chrome.offscreen.hasDocument();
        if (!has) {
            console.log("[Background] Creating offscreen document");
            await chrome.offscreen.createDocument({
                url: chrome.runtime.getURL('offscreen.html'),
                reasons: [chrome.offscreen.Reason.DOM_SCRAPING],
                justification: 'Need WebRTC APIs in DOM context for MPC',
            });
            // Send initial configuration to offscreen
            await chrome.runtime.sendMessage({
                type: "fromBackground",
                payload: {
                    type: "init",
                    peerId,
                    wsUrl: "wss://auto-life.tech" // Use same URL as background
                }
            });
            return true;
        }
        console.log("[Background] Offscreen document already exists");
        return true;
    } catch (e) {
        console.error("[Background] Error with offscreen document:", e);
        return false;
    }
}

// Listen for popup requests to create offscreen document
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    // ...existing code...
    if (message && message.type === "ensureOffscreen") {
        ensureOffscreenDocument().then(() => sendResponse({ success: true })).catch(e => sendResponse({ success: false, error: e?.message || e }));
        return true;
    }
    // ...existing code...
});

// Forward messages between popup/background and offscreen document
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    // ...existing code...
    if (message && message.type === "toOffscreen") {
        chrome.runtime.sendMessage({ ...message, type: "fromBackground" });
        sendResponse({ success: true });
        return true;
    }
    // ...existing code...
});

// Listen for messages from offscreen and forward to popup(s)
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    // ...existing code...
    if (message && message.type === "fromOffscreen") {
        notifyPopups({ type: "fromOffscreen", payload: message.payload });
        sendResponse({ success: true });
        return true;
    }
    // ...existing code...
});

chrome.runtime.onInstalled.addListener(async () => {
    await ensureOffscreenDocument();
});