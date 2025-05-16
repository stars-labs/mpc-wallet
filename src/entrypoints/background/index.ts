import { defineBackground } from '#imports';
import { MESSAGE_PREFIX, MessageType } from '../../constants';
import AccountService from '../../services/accountService';
import NetworkService from '../../services/networkService';
import WalletClientService from '../../services/walletClient';
import { toHex } from 'viem';
import WalletController from "../../services/walletController";
import { WebSocketClient } from "./websocket";
import type {
    ServerMsg, ClientMsg
} from "./types";
import { WebRTCManager } from "./webrtc";
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

// Refined ensureOffscreenDocument function
async function ensureOffscreenDocument(): Promise<{ success: boolean; created?: boolean; message?: string; error?: string }> {
    if (!chrome.offscreen) {
        console.warn("[Background] chrome.offscreen API not available");
        return { success: false, error: "Offscreen API not available." };
    }

    try {
        if (await chrome.offscreen.hasDocument()) {
            console.log("[Background] Offscreen document already exists.");
            return { success: true, created: false, message: "Offscreen document already exists." };
        }

        console.log("[Background] Creating offscreen document as it does not exist.");
        await chrome.offscreen.createDocument({
            url: chrome.runtime.getURL('offscreen.html'),
            reasons: [chrome.offscreen.Reason.DOM_SCRAPING], // Consider RTC_CONNECTIONS if more appropriate
            justification: 'Manages WebRTC connections for MPC sessions.', // More specific justification
        });
        console.log("[Background] Offscreen document created successfully.");

        try {
            await new Promise(resolve => setTimeout(resolve, 200)); // Increased delay for offscreen to init
            console.log("[Background] Attempting to send init message to new offscreen document.");
            await chrome.runtime.sendMessage({
                type: "fromBackground",
                payload: {
                    type: "init",
                    peerId,
                    wsUrl: "wss://auto-life.tech"
                }
            });
            console.log("[Background] Sent init message to new offscreen document.");
            return { success: true, created: true, message: "Offscreen document created and init message sent." };
        } catch (initError: any) {
            console.error("[Background] CRITICAL: Error sending init message to new offscreen document. This means it's not listening or crashed early.", initError.message);
            return { success: true, created: true, message: "Offscreen document created, but failed to send init message.", error: `Init send failed: ${initError.message}` };
        }

    } catch (e: any) {
        if (e.message && e.message.includes("Only a single offscreen document may be created")) {
            console.warn("[Background] Attempted to create offscreen document, but it already exists (caught specific error):", e.message);
            return { success: true, created: false, message: "Offscreen document already exists (creation conflict)." };
        }
        console.error("[Background] Error with offscreen document operation:", e);
        return { success: false, error: e.message || "Unknown error with offscreen document." };
    }
}

export default defineBackground({
    main() {
        // Start WebSocket and WebRTC initialization (SINGLE CALL HERE)
        initializeConnection(); // This already calls ensureOffscreenDocument

        // Listener for messages from popups (SINGLE CONSOLIDATED LISTENER)
        chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
            console.log("[Background] Message received:", message, "from sender:", sender?.tab?.url || sender?.id);

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
                return true; // Indicate async response
            }

            // Handle Account Management
            if (message.type === MessageType.ACCOUNT_MANAGEMENT) {
                const { action, payload } = message;
                handleAccountManagement(action, payload)
                    .then(response => sendResponse(response));
                return true; // Indicate async response
            }

            // Handle Network Management
            if (message.type === MessageType.NETWORK_MANAGEMENT) {
                const { action, payload } = message;
                handleNetworkManagement(action, payload)
                    .then(response => sendResponse(response));
                return true; // Indicate async response
            }

            // Handle UI Requests
            if (message.type === MessageType.UI_REQUEST) {
                handleUIRequest(message.payload)
                    .then(sendResponse)
                    .catch(error => sendResponse({ success: false, error: error.message }));
                return true; // Indicate async response
            }

            // Handle MPC-specific and Offscreen-related messages
            switch (message.type) {
                case "getState":
                    const statePayload = {
                        peerId,
                        connectedPeers: [...connectedPeers],
                        wsConnected: wsClient?.getReadyState() === WebSocket.OPEN,
                        sessionInfo: webrtcManager?.sessionInfo || null,
                        invites: webrtcManager?.invites || [],
                        meshStatus: webrtcManager?.meshStatus || { type: 0 /* Incomplete */ },
                        dkgState: webrtcManager?.dkgState || 0 /* Idle */
                    };
                    console.log("[Background] Handling getState. Sending payload:", statePayload);
                    sendResponse(statePayload);
                    // No return true needed if sendResponse is synchronous here.
                    // However, if any part of creating statePayload was async, it would be.
                    break; // Explicitly break

                case "listPeers":
                    wsClient?.listPeers();
                    sendResponse({ success: true, message: "listPeers command sent" });
                    break;

                case "relay":
                    // Assuming wsClient.relayMessage is synchronous or handles its own errors.
                    // If it were async and we needed to wait, we'd return true.
                    wsClient?.relayMessage(message.to, message.data);
                    sendResponse({ success: true });
                    break;

                // Messages to be forwarded to the Offscreen Document
                case "proposeSession":
                case "acceptSession":
                    // These messages are intended for the offscreen document.
                    // We wrap them in a "fromBackground" message type for the offscreen document to process.
                    chrome.runtime.sendMessage({
                        type: "fromBackground",
                        payload: {
                            type: message.type, // Original type for offscreen's switch
                            ...message // Spread the rest of the message properties (sessionId, etc.)
                        }
                    }).catch(err => console.error(`[Background] Error sending ${message.type} to offscreen:`, err));
                    sendResponse({ success: true, message: `${message.type} forwarded to offscreen processing.` });
                    // Return true if chrome.runtime.sendMessage to offscreen is treated as async for response channel
                    return true;


                // Offscreen document management
                case "createOffscreen": // Message from popup to ensure offscreen exists
                    ensureOffscreenDocument()
                        .then(response => {
                            sendResponse(response);
                        })
                        .catch(err => { // Should be caught by ensureOffscreenDocument itself
                            console.error("[Background] Critical error from ensureOffscreenDocument:", err);
                            sendResponse({ success: false, error: err.message || "Unknown critical error ensuring offscreen document." });
                        });
                    return true; // Crucial for async sendResponse

                case "getOffscreenStatus": // Message from popup to check offscreen status
                    if (chrome.offscreen && typeof chrome.offscreen.hasDocument === 'function') {
                        chrome.offscreen.hasDocument()
                            .then(hasDoc => {
                                sendResponse({ hasDocument: hasDoc });
                            })
                            .catch(err => {
                                console.error("[Background] Error checking offscreen document status:", err);
                                sendResponse({ hasDocument: false, error: err.message });
                            });
                    } else {
                        console.error("[Background] chrome.offscreen.hasDocument API not available for getOffscreenStatus.");
                        sendResponse({ hasDocument: false, error: "Offscreen API not available." });
                    }
                    return true; // Crucial for asynchronous sendResponse

                // Messages from the Offscreen Document to be relayed or processed
                case "fromOffscreen":
                    console.log("[Background] Message from Offscreen Document:", message.payload);
                    // Example: Relay to popups or handle specific offscreen messages
                    if (message.payload && message.payload.type === "log") {
                        console.log("[Offscreen Log]", message.payload.message);
                    } else {
                        notifyPopups({ type: "fromOffscreen", payload: message.payload });
                    }
                    sendResponse({ success: true, message: "Received by background." }); // Acknowledge to offscreen
                    return true; // If any async processing or if sendResponse matters for channel.

                default:
                    console.warn("[Background] Unhandled message type:", message.type, message);
                    // sendResponse({ success: false, error: "Unknown message type." }); // Optional: respond for unhandled types
                    // If not sending a response, don't return true.
                    break;
            }
            // If a response hasn't been sent and it's not an async path, the channel will close.
            // Only return true if sendResponse will be called asynchronously.
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
        let storedPeerId = await storage.getItem<string>("local:peer_id");
        // Ensure peerId is in the desired format or generate a new one
        if (!storedPeerId || !storedPeerId.startsWith("mpc-")) {
            storedPeerId = `mpc-${Math.random().toString(36).substring(2, 7)}`;
            await storage.setItem("local:peer_id", storedPeerId);
            console.log("[Background] Generated new Peer ID:", storedPeerId);
        }
        peerId = storedPeerId;
        console.log("[Background] Using Peer ID:", peerId);

        const offscreenResult = await ensureOffscreenDocument();
        console.log("[Background] Initial ensureOffscreenDocument result:", offscreenResult);
        if (!offscreenResult.success || (offscreenResult.created && offscreenResult.error && offscreenResult.error.includes("Init send failed"))) {
            console.error("[Background] Offscreen document setup might have issues. WebRTC features may be impaired if init message failed.");
        }

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
async function handleRelayMessage(fromPeerId: string, data: any) {
    try {
        console.log(`[Background] Received relay message from ${fromPeerId}:`, JSON.stringify(data));

        let offscreenExists = false;
        if (chrome.offscreen && typeof chrome.offscreen.hasDocument === 'function') {
            offscreenExists = await chrome.offscreen.hasDocument();
        }

        if (offscreenExists) {
            console.log("[Background] Offscreen document confirmed to exist. Attempting to forward relay message.");
            try {
                await chrome.runtime.sendMessage({
                    type: "fromBackground",
                    payload: {
                        type: "relayMessage",
                        fromPeerId,
                        data
                    }
                });
                console.log("[Background] Successfully forwarded relay message to offscreen document.");
            } catch (err: any) {
                console.error("[Background] Error forwarding relay to offscreen (even after confirming existence):", err.message);
                console.warn("[Background] Falling back to local processing for relay message due to offscreen forward error.");
                processRelayMessageLocally(fromPeerId, data);
            }
        } else {
            console.warn("[Background] Offscreen document does not exist at time of relay. Falling back to local processing.");
            processRelayMessageLocally(fromPeerId, data);
        }
    } catch (error: any) {
        console.error(`[Background] Broader error in handleRelayMessage (e.g., from hasDocument) for message from ${fromPeerId}:`, error.message);
        console.warn("[Background] Falling back to local processing due to broader error in handleRelayMessage.");
        processRelayMessageLocally(fromPeerId, data);
    }
}

// Fallback processing for relay messages if offscreen document isn't available
async function processRelayMessageLocally(fromPeerId: string, data: any) {
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
                        try {
                            await webrtcManager.handleWebRTCSignal(fromPeerId, signalDataForManager);
                        } catch (error) {
                            console.warn(`[Background] WebRTCManager error during handleWebRTCSignal:`, error);
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
// async function ensureOffscreenDocument() { ... } // This function is now defined above defineBackground

// Listen for popup requests to create offscreen document
// REMOVED: This is now handled in the consolidated listener
// chrome.runtime.onMessage.addListener((message, sender, sendResponse) => { ... });

// Forward messages between popup/background and offscreen document
// REMOVED: This is now handled in the consolidated listener
// chrome.runtime.onMessage.addListener((message, sender, sendResponse) => { ... });

// Listen for messages from offscreen and forward to popup(s)
// REMOVED: This is now handled in the consolidated listener
// chrome.runtime.onMessage.addListener((message, sender, sendResponse) => { ... });

chrome.runtime.onInstalled.addListener(async (details) => {
    console.log("[Background] onInstalled event:", details.reason);
    const offscreenResult = await ensureOffscreenDocument();
    console.log("[Background] onInstalled ensureOffscreenDocument result:", offscreenResult);
    // Other onInstalled tasks...
});