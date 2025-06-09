import { defineBackground } from '#imports';
import { MESSAGE_PREFIX, MessageType, DEFAULT_ADDRESSES } from '../../constants';
import AccountService from '../../services/accountService';
import NetworkService from '../../services/networkService';
import WalletClientService from '../../services/walletClient';
import { toHex } from 'viem';
import WalletController from "../../services/walletController";
import { WebSocketClient } from "./websocket";
import {
    SessionProposal, SessionResponse, SessionInfo, AppState, MeshStatusType, DkgState, SigningState
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
    MESSAGE_TYPES,
} from "../../types/messages";
import { ServerMsg, ClientMsg, WebSocketMessagePayload, WebRTCSignal } from '../../types/websocket';

// Initialize services
const accountService = AccountService.getInstance();
const networkService = NetworkService.getInstance();
const walletClientService = WalletClientService.getInstance();

// Helper to get MPC address - will be useful for other wallet operations
async function getMPCAddress(): Promise<string | null> {
    if (!appState.blockchain || appState.dkgState !== DkgState.Complete) {
        return null;
    }
    
    try {
        // Determine which address type to get based on blockchain selection
        const addressType = appState.blockchain === "ethereum" ? "getEthereumAddress" : "getSolanaAddress";
        console.log(`[Background] Getting MPC ${appState.blockchain} address from offscreen using ${addressType}`);
        
        const offscreenResponse = await chrome.runtime.sendMessage({
            type: addressType
        });
        
        // Extract the correct address field based on blockchain
        const addressField = appState.blockchain === "ethereum" ? "ethereumAddress" : "solanaAddress";
        
        if (offscreenResponse && offscreenResponse.success && offscreenResponse.data[addressField]) {
            return offscreenResponse.data[addressField];
        }
    } catch (e) {
        console.error(`[Background] Error getting ${appState.blockchain} address:`, e);
    }
    
    return null;
}

// Get a default placeholder address based on current blockchain
function getPlaceholderAddress(): string {
    // Use extension ID to generate consistent placeholder addresses
    let seed;
    try {
        seed = chrome.runtime?.id;
    } catch (e) {}
    return appState.blockchain === "ethereum" 
        ? DEFAULT_ADDRESSES.ethereum(seed)
        : DEFAULT_ADDRESSES.solana(seed);
}

// 处理 RPC 请求
async function handleRpcRequest(request: JsonRpcRequest): Promise<unknown> {
    try {
        // 根据请求的方法执行相应的操作
        switch (request.method) {
            case 'eth_accounts':
            case 'eth_requestAccounts':
                // 返回当前选中的账户地址
                const currentAccount = accountService.getCurrentAccount();
                
                // Check if we have a regular account already
                if (currentAccount) {
                    return [currentAccount.address];
                }
                
                // If using MPC wallet with completed DKG, get address from there
                if (appState.dkgState === DkgState.Complete && appState.blockchain) {
                    // Get MPC address
                    const mpcAddress = await getMPCAddress() || getPlaceholderAddress();
                    console.log(`[Background] Using MPC ${appState.blockchain} address:`, mpcAddress);
                    
                    // Auto-create account in the AccountService so future calls work
                    console.log(`[Background] Auto-creating MPC account with address:`, mpcAddress);
                    
                    // Create a new MPC account
                    const mpcAccount = {
                        address: mpcAddress,
                        type: "mpc",
                        name: `MPC ${appState.blockchain.charAt(0).toUpperCase() + appState.blockchain.slice(1)} Wallet`
                    };
                    
                    try {
                        await accountService.addAccount(mpcAccount as any);
                        console.log(`[Background] Successfully created MPC account`);
                    } catch (e) {
                        console.log("[Background] Account may already exist:", e);
                        try {
                            // Try to set it as current account if it exists
                            await accountService.setCurrentAccount(mpcAddress);
                        } catch (e2) {
                            console.error("[Background] Failed to set MPC account as current:", e2);
                        }
                    }
                    
                    return [mpcAddress];
                }
                
                // No account available
                throw new Error('No account selected');

            case 'eth_chainId':
                // 返回当前网络的 chainId
                // First try to get from NetworkService
                const currentNetwork = networkService.getCurrentNetwork();
                if (currentNetwork) {
                    return toHex(currentNetwork.id);
                }
                
                // For MPC wallet on Ethereum, use mainnet by default
                if (appState.blockchain === "ethereum") {
                    console.log("[Background] Using default Ethereum mainnet chainId (1)");
                    return "0x1"; // Ethereum mainnet
                } else {
                    console.log("[Background] Using Solana, eth_chainId not applicable");
                    return "0x0"; // Not applicable for Solana
                }

            case 'net_version':
                // 返回当前网络的 chainId
                const network = networkService.getCurrentNetwork();
                if (network) {
                    return toHex(network.id);
                }
                
                // For MPC wallet on Ethereum, use mainnet by default
                if (appState.blockchain === "ethereum") {
                    console.log("[Background] Using default Ethereum mainnet net_version (1)");
                    return "0x1"; // Ethereum mainnet
                } else {
                    console.log("[Background] Using Solana, net_version not applicable");
                    return "0x0"; // Not applicable for Solana
                }

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
let appState: AppState = {
    peerId: "",
    connectedPeers: [],
    wsConnected: false,
    sessionInfo: null,
    invites: [],
    meshStatus: { type: MeshStatusType.Incomplete },
    dkgState: DkgState.Idle,
    signingState: SigningState.Idle,
    webrtcConnections: {},
    blockchain: undefined // Will be set when session is accepted
};

// Broadcast state to all connected popup ports
function broadcastToPopupPorts(message: PopupMessage) {
    console.log("[Background] Broadcasting to", popupPorts.size, "popup ports:", message);
    popupPorts.forEach(port => {
        try {
            port.postMessage(message);
            console.log("[Background] Successfully sent message to popup port");
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
            console.log("[Background] Sending initial state to popup");
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
        
        // Special handling for any message with a payload field first - highest priority
        if (message && typeof message === 'object' && 'payload' in message && 
            message.payload && typeof message.payload === 'object') {
            
            const payload = message.payload as any;
            
            // Check if it looks like a JSON-RPC request
            if (payload.id !== undefined && payload.jsonrpc === '2.0' && 
                typeof payload.method === 'string' && payload.method) {
                
                console.log(`[Background] Processing JSON-RPC request:`, payload.method);
                
                // Process any payload with method as a JSON-RPC request
                (async () => {
                    try {
                        const result = await handleRpcRequest(payload);
                        console.log(`[Background] JSON-RPC request completed:`, result);
                        sendResponse({ success: true, result });
                    } catch (error) {
                        console.error(`[Background] JSON-RPC request failed:`, error);
                        sendResponse({ 
                            success: false, 
                            error: error instanceof Error ? error.message : 'Unknown error handling request' 
                        });
                    }
                })();
                return true; // Keep sendResponse valid
            }
        }

        // Validate basic message structure
        if (!validateMessage(message)) {
            console.warn("[Background] Invalid message structure:", message);
            sendResponse({ success: false, error: "Invalid message structure" });
            return true;
        }

        // Add additional logging to debug message handling
        console.log("[Background] Processing message with type:", message.type, typeof message.type);
        console.log("[Background] Message type comparison:", {
            isForwardRequest: message.type === 'FORWARD_REQUEST',
            typeComparison: message.type === 'FORWARD_REQUEST' ? 'exact match' : 'no match',
            typeToString: String(message.type),
            typeOfType: typeof message.type
        });
        
        // Handle async operations
        (async () => {
            try {
                // Check for FORWARD_REQUEST first as a special case
                if (message.type === 'FORWARD_REQUEST' && 'payload' in message && message.payload) {
                    console.log(`[Background] Detected FORWARD_REQUEST at top level`);
                    try {
                        const result = await handleRpcRequest(message.payload as JsonRpcRequest);
                        console.log(`[Background] Forwarded request completed successfully:`, result);
                        sendResponse({ success: true, result });
                        return; // Exit early
                    } catch (error) {
                        console.error(`[Background] Error handling forwarded request:`, error);
                        sendResponse({ 
                            success: false, 
                            error: error instanceof Error ? error.message : 'Unknown error handling request' 
                        });
                        return; // Exit early
                    }
                }
                
                // Simple switch on message type with runtime validation
                switch (message.type) {
                    case MESSAGE_TYPES.GET_STATE:
                        // If WebSocket is connected but we have no peers, request peer list first
                        if (wsClient?.getReadyState() === WebSocket.OPEN && appState.connectedPeers.length === 0) {
                            console.log("[Background] getState called with empty peer list but WebSocket connected - requesting fresh peer list");
                            try {
                                wsClient.listPeers();
                                console.log("[Background] Fresh peer list request sent");
                            } catch (error) {
                                console.error("[Background] Error requesting fresh peer list:", error);
                            }
                        }
                        sendResponse(appState);
                        break;

                    case MESSAGE_TYPES.GET_WEBRTC_STATE:
                        console.log("[Background] GET_WEBRTC_STATE request, returning:", appState.webrtcConnections);
                        sendResponse({ webrtcConnections: appState.webrtcConnections });
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

                    case MESSAGE_TYPES.PROPOSE_SESSION:
                        if (validateSessionProposal(message)) {
                            console.log("[Background] Received session proposal request:", message);

                            // Create session proposal data
                            const proposalData = {
                                websocket_msg_type: "SessionProposal",
                                session_id: message.session_id,
                                total: message.total,
                                threshold: message.threshold,
                                participants: message.participants
                            };

                            // Send proposal to all participants via WebSocket
                            if (wsClient?.getReadyState() === WebSocket.OPEN) {
                                const participantsToNotify = message.participants.filter(p => p !== appState.peerId);

                                console.log("[Background] Sending session proposal to participants:", participantsToNotify);

                                Promise.all(participantsToNotify.map(async (peerId) => {
                                    try {
                                        await wsClient!.relayMessage(peerId, proposalData);
                                        console.log(`[Background] Session proposal sent to ${peerId}`);
                                    } catch (error) {
                                        console.error(`[Background] Failed to send proposal to ${peerId}:`, error);
                                    }
                                })).then(() => {
                                    console.log("[Background] All session proposals sent");
                                    sendResponse({ success: true });
                                }).catch((error) => {
                                    console.error("[Background] Error sending session proposals:", error);
                                    sendResponse({ success: false, error: error.message });
                                });
                            } else {
                                sendResponse({ success: false, error: "WebSocket not connected" });
                            }
                        } else {
                            sendResponse({ success: false, error: "Invalid session proposal" });
                        }
                        break;

                    case MESSAGE_TYPES.ACCEPT_SESSION:
                        if (validateSessionAcceptance(message)) {
                            console.log("[Background] Received session acceptance:", message);

                            // Extract blockchain parameter from message
                            const blockchain = message.blockchain || "solana"; // Default to solana if not specified
                            console.log("[Background] Session blockchain selection:", blockchain);

                            // Find the session in invites
                            const sessionIndex = appState.invites.findIndex(invite => invite.session_id === message.session_id);

                            if (sessionIndex >= 0) {
                                const session = appState.invites[sessionIndex];

                                if (message.accepted) {
                                    console.log("[Background] Accepting session:", session.session_id);

                                    // Move from invites to current session
                                    appState.sessionInfo = { ...session, status: "accepted" };
                                    appState.invites.splice(sessionIndex, 1);

                                    // Store blockchain selection for this session
                                    appState.blockchain = blockchain;

                                    // Ensure this peer is in the accepted peers list
                                    if (!appState.sessionInfo.accepted_peers.includes(appState.peerId)) {
                                        appState.sessionInfo.accepted_peers.push(appState.peerId);
                                    }

                                    console.log("[Background] Updated accepted peers:", appState.sessionInfo.accepted_peers);

                                    // Send acceptance message to other participants
                                    const acceptanceData = {
                                        websocket_msg_type: "SessionResponse",
                                        session_id: message.session_id,
                                        accepted: true
                                    };

                                    if (wsClient?.getReadyState() === WebSocket.OPEN) {
                                        // Send to all other participants
                                        const otherParticipants = session.participants.filter(p => p !== appState.peerId);

                                        Promise.all(otherParticipants.map(async (peerId) => {
                                            try {
                                                await wsClient!.relayMessage(peerId, acceptanceData);
                                                console.log(`[Background] Session acceptance sent to ${peerId}`);
                                            } catch (error) {
                                                console.error(`[Background] Failed to send acceptance to ${peerId}:`, error);
                                            }
                                        })).then(() => {
                                            console.log("[Background] All session acceptances sent");

                                            // Broadcast session update to popup
                                            broadcastToPopupPorts({
                                                type: "sessionUpdate",
                                                sessionInfo: appState.sessionInfo,
                                                invites: appState.invites
                                            } as any);

                                            // Forward session info to offscreen for WebRTC setup
                                            console.log("[Background] Forwarding session info to offscreen for WebRTC setup with blockchain:", blockchain);

                                            if (appState.sessionInfo) {
                                                safelySendOffscreenMessage({
                                                    type: "fromBackground",
                                                    payload: {
                                                        type: "sessionAccepted",
                                                        sessionInfo: appState.sessionInfo,
                                                        currentPeerId: appState.peerId,
                                                        blockchain: blockchain
                                                    }
                                                }, "sessionAccepted");
                                            } else {
                                                console.error("[Background] Cannot forward session info: sessionInfo is null");
                                            }

                                            sendResponse({ success: true });
                                        });
                                    } else {
                                        sendResponse({ success: false, error: "WebSocket not connected" });
                                    }
                                } else {
                                    console.log("[Background] Declining session:", session.session_id);

                                    // Remove from invites
                                    appState.invites.splice(sessionIndex, 1);

                                    // Send decline message
                                    const declineData = {
                                        websocket_msg_type: "SessionResponse",
                                        session_id: message.session_id,
                                        accepted: false,
                                        peer_id: appState.peerId
                                    };

                                    if (wsClient?.getReadyState() === WebSocket.OPEN) {
                                        const otherParticipants = session.participants.filter(p => p !== appState.peerId);

                                        Promise.all(otherParticipants.map(async (peerId) => {
                                            try {
                                                await wsClient!.relayMessage(peerId, declineData);
                                                console.log(`[Background] Session decline sent to ${peerId}`);
                                            } catch (error) {
                                                console.error(`[Background] Failed to send decline to ${peerId}:`, error);
                                            }
                                        }));
                                    }

                                    // Broadcast session update to popup
                                    broadcastToPopupPorts({
                                        type: "sessionUpdate",
                                        sessionInfo: appState.sessionInfo,
                                        invites: appState.invites
                                    } as any);

                                    sendResponse({ success: true });
                                }
                            } else {
                                sendResponse({ success: false, error: "Session not found in invites" });
                            }
                        } else {
                            sendResponse({ success: false, error: "Invalid session acceptance" });
                        }
                        break;

                    case MESSAGE_TYPES.SEND_DIRECT_MESSAGE:
                        console.log("[Background] Received sendDirectMessage request:", message);
                        if ('toPeerId' in message && 'message' in message &&
                            typeof message.toPeerId === 'string' && typeof message.message === 'string') {
                            // Forward to offscreen document
                            const result = await safelySendOffscreenMessage({
                                type: "fromBackground",
                                payload: {
                                    type: "sendDirectMessage",
                                    toPeerId: message.toPeerId,
                                    message: message.message
                                }
                            }, "sendDirectMessage");

                            if (result.success) {
                                sendResponse({ success: true, message: "Direct message sent to offscreen" });
                            } else {
                                sendResponse({ success: false, error: `Failed to send to offscreen: ${result.error}` });
                            }
                        } else {
                            sendResponse({ success: false, error: "Missing or invalid toPeerId or message" });
                        }
                        break;

                    case MESSAGE_TYPES.GET_WEBRTC_STATUS:
                        console.log("[Background] Received getWebRTCStatus request");
                        // Forward to offscreen document
                        const webrtcResult = await safelySendOffscreenMessage({
                            type: "fromBackground",
                            payload: {
                                type: "getWebRTCStatus"
                            }
                        }, "getWebRTCStatus");

                        if (webrtcResult.success) {
                            sendResponse({ success: true, message: "WebRTC status request sent to offscreen" });
                        } else {
                            sendResponse({ success: false, error: `Failed to get WebRTC status: ${webrtcResult.error}` });
                        }
                        break;

                    case "setBlockchain":
                        if ('blockchain' in message) {
                            console.log("[Background] Setting blockchain selection:", message.blockchain);
                            appState.blockchain = message.blockchain;
                            sendResponse({ success: true, blockchain: appState.blockchain });
                        } else {
                            console.warn("[Background] setBlockchain message missing blockchain field");
                            sendResponse({ success: false, error: "Missing blockchain field" });
                        }
                        break;
                        
                    case "getEthereumAddress":
                        console.log("[Background] Received getEthereumAddress request");
                        // First check if we have a saved real address from a previous DKG
                        try {
                            const savedResult = await chrome.storage.local.get(['mpc_ethereum_address']);
                            
                            if (savedResult && savedResult.mpc_ethereum_address && 
                                savedResult.mpc_ethereum_address !== DEFAULT_ADDRESSES.ethereum() &&
                                !savedResult.mpc_ethereum_address.startsWith('0x0000000')) {
                                
                                console.log("[Background] Found saved real Ethereum address:", savedResult.mpc_ethereum_address);
                                sendResponse({
                                    success: true,
                                    data: { ethereumAddress: savedResult.mpc_ethereum_address }
                                });
                                return true;
                            }
                        } catch (e) {
                            console.log("[Background] Error checking storage for saved address:", e);
                        }
                        
                        // If DKG is not complete and we didn't find a saved real address
                        if (appState.dkgState !== 6) { // DkgState.Complete is 6
                            // Use fallback for smoother experience
                            let seed;
                            try {
                                seed = chrome.runtime?.id;
                            } catch (e) {}
                            const fallbackAddress = DEFAULT_ADDRESSES.ethereum(seed);
                            console.log("[Background] DKG not completed, using fallback address:", fallbackAddress);
                            
                            // Don't save the fallback address to storage - this way we preserve any real address
                            // that might be saved there from a previous DKG completion
                            
                            sendResponse({
                                success: true,
                                data: { ethereumAddress: fallbackAddress }
                            });
                            return true;
                        }
                        
                        // Forward to offscreen document
                        const ethAddressResult = await safelySendOffscreenMessage({
                            type: "fromBackground",
                            payload: {
                                type: "getEthereumAddress"
                            }
                        }, "getEthereumAddress");
                        
                        if (ethAddressResult.success) {
                            // Get the address from the offscreen response
                            const offscreenResponse = await chrome.runtime.sendMessage({
                                type: "getEthereumAddress"
                            });
                            
                            const address = offscreenResponse && offscreenResponse.success && offscreenResponse.data.ethereumAddress
                                ? offscreenResponse.data.ethereumAddress
                                : DEFAULT_ADDRESSES.ethereum(); // Fallback
                                
                            // Also save the address to storage for content script quick access
                            try {
                                // Notify all content scripts to save this address
                                chrome.tabs.query({}, tabs => {
                                    tabs.forEach(tab => {
                                        if (tab.id) {
                                            chrome.tabs.sendMessage(tab.id, {
                                                type: "SAVE_ADDRESS",
                                                blockchain: "ethereum",
                                                address: address
                                            }).catch(() => {});
                                        }
                                    });
                                });
                            } catch (e) {
                                console.error("[Background] Error saving address to content scripts:", e);
                            }
                            
                            sendResponse({ 
                                success: true, 
                                data: { ethereumAddress: address } 
                            });
                        } else {
                            let seed;
                            try {
                                seed = chrome.runtime?.id;
                            } catch (e) {}
                            const fallbackAddress = DEFAULT_ADDRESSES.ethereum(seed);
                            console.log("[Background] Failed to get address from offscreen, using fallback:", fallbackAddress);
                            sendResponse({ 
                                success: true, 
                                data: { ethereumAddress: fallbackAddress }
                            });
                        }
                        break;
                        
                    case "getSolanaAddress":
                        console.log("[Background] Received getSolanaAddress request");
                        if (appState.dkgState !== 6) { // DkgState.Complete is 6
                            sendResponse({
                                success: false,
                                error: "DKG not completed yet. Complete DKG process first."
                            });
                            return true;
                        }
                        
                        // Forward to offscreen document
                        const solAddressResult = await safelySendOffscreenMessage({
                            type: "fromBackground",
                            payload: {
                                type: "getSolanaAddress"
                            }
                        }, "getSolanaAddress");
                        
                        if (solAddressResult.success) {
                            // Get the address from the offscreen response
                            const offscreenResponse = await chrome.runtime.sendMessage({
                                type: "getSolanaAddress"
                            });
                            
                            if (offscreenResponse && offscreenResponse.success) {
                                sendResponse({ success: true, data: offscreenResponse.data });
                            } else {
                                let seed;
                                try {
                                    seed = chrome.runtime?.id;
                                } catch (e) {}
                                const placeholderSolanaAddress = DEFAULT_ADDRESSES.solana(seed);
                                sendResponse({ success: true, data: { solanaAddress: placeholderSolanaAddress } });
                            }
                        } else {
                            sendResponse({ success: false, error: `Failed to get Solana address: ${solAddressResult.error}` });
                        }
                        break;
                        
                    case "saveAddress":
                        console.log("[Background] Received saveAddress request:", message);
                        if ('blockchain' in message && 'address' in message) {
                            const blockchain = message.blockchain as string;
                            const address = message.address as string;
                            
                            console.log(`[Background] Saving ${blockchain} address: ${address}`);
                            
                            // Save to storage for persistence
                            try {
                                await chrome.storage.local.set({
                                    [`mpc_${blockchain}_address`]: address
                                });
                                console.log(`[Background] Successfully saved ${blockchain} address to storage`);
                                
                                // Try to notify content scripts
                                try {
                                    chrome.tabs.query({}, tabs => {
                                        tabs.forEach(tab => {
                                            if (tab.id) {
                                                chrome.tabs.sendMessage(tab.id, {
                                                    type: "SAVE_ADDRESS",
                                                    blockchain,
                                                    address
                                                }).catch(() => {
                                                    // Ignore errors - content scripts may not be loaded yet
                                                });
                                            }
                                        });
                                    });
                                } catch (e) {
                                    console.log("[Background] Error notifying content scripts, not critical:", e);
                                }
                                
                                // Create account in AccountService
                                try {
                                    if (blockchain === "ethereum") {
                                        const mpcAccount = {
                                            address: address,
                                            type: "mpc",
                                            name: "MPC Ethereum Wallet"
                                        };
                                        await accountService.addAccount(mpcAccount as any);
                                    }
                                } catch (e) {
                                    console.log("[Background] Account may already exist:", e);
                                }
                                
                                sendResponse({ success: true });
                            } catch (e) {
                                console.error("[Background] Error saving address to storage:", e);
                                sendResponse({ success: false, error: `${e}` });
                            }
                        } else {
                            sendResponse({ success: false, error: "Missing blockchain or address" });
                        }
                        break;

                    case MESSAGE_TYPES.FORWARD_REQUEST:
                        console.log(`[Background] Handling forwarded request from content script:`, message);
                        if ('payload' in message && message.payload) {
                            try {
                                const result = await handleRpcRequest(message.payload as JsonRpcRequest);
                                console.log(`[Background] Forwarded request completed successfully:`, result);
                                sendResponse({ success: true, result });
                            } catch (error) {
                                console.error(`[Background] Error handling forwarded request:`, error);
                                sendResponse({ 
                                    success: false, 
                                    error: error instanceof Error ? error.message : 'Unknown error handling request' 
                                });
                            }
                        } else {
                            console.warn(`[Background] Invalid FORWARD_REQUEST, missing payload`); 
                            sendResponse({ success: false, error: 'Invalid request: missing payload' });
                        }
                        break;
                        
                    default:
                        // Handle FORWARD_REQUEST directly in the default case as a fallback
                        if (message.type === 'FORWARD_REQUEST' && 'payload' in message && message.payload) {
                            console.log(`[Background] Handling forwarded request from content script in default case:`, message);
                            try {
                                const result = await handleRpcRequest(message.payload as JsonRpcRequest);
                                console.log(`[Background] Forwarded request completed successfully:`, result);
                                sendResponse({ success: true, result });
                                return; // Exit early
                            } catch (error) {
                                console.error(`[Background] Error handling forwarded request:`, error);
                                sendResponse({ 
                                    success: false, 
                                    error: error instanceof Error ? error.message : 'Unknown error handling request' 
                                });
                                return; // Exit early
                            }
                        }
                        // Original default case handling
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
            case "webrtcStatusUpdate":
                // Handle WebRTC status updates from offscreen
                if ('peerId' in payload && 'status' in payload) {
                    console.log(`[Background] WebRTC status update for ${payload.peerId}: ${payload.status}`);
                    // Forward to popup if needed
                    broadcastToPopupPorts({
                        type: "webrtcStatusUpdate",
                        peerId: payload.peerId,
                        status: payload.status
                    } as any);
                } else {
                    console.warn("[Background] Invalid WebRTC status update payload:", payload);
                }
                break;
            case "peerConnectionStatusUpdate":
                // Handle peer connection status updates
                if ('peerId' in payload && 'connectionState' in payload) {
                    console.log(`[Background] Peer connection status update for ${payload.peerId}: ${payload.connectionState}`);
                } else {
                    console.warn("[Background] Invalid peer connection status update payload:", payload);
                }
                break;
            case "dataChannelStatusUpdate":
                // Handle data channel status updates
                if ('peerId' in payload && 'channelName' in payload && 'state' in payload) {
                    console.log(`[Background] Data channel ${payload.channelName} for ${payload.peerId}: ${payload.state}`);
                } else {
                    console.warn("[Background] Invalid data channel status update payload:", payload);
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
            case "webrtcConnectionUpdate":
                // Handle WebRTC connection updates

                if ('peerId' in payload && 'connected' in payload) {
                    console.log("[Background] Received WebRTC connection update:", {
                        peerId: payload.peerId,
                        connected: payload.connected
                    });

                    // Update appState with WebRTC connection info
                    appState.webrtcConnections[payload.peerId] = payload.connected;
                    console.log("[Background] Updated appState.webrtcConnections:", appState.webrtcConnections);

                    console.log("[Background] Current popup ports count:", popupPorts.size);

                    // Send WebRTC connection update directly to popup (not wrapped in fromOffscreen)
                    const webrtcMessage = {
                        type: "webrtcConnectionUpdate",
                        peerId: payload.peerId,
                        connected: payload.connected
                    };

                    console.log("[Background] Sending WebRTC connection update to popup:", webrtcMessage);
                    broadcastToPopupPorts(webrtcMessage as any);

                    console.log("[Background] WebRTC connection update sent to popup");
                } else {
                    console.warn("[Background] Invalid WebRTC connection update payload:", payload);
                }
                break;
            case "meshStatusUpdate":
                // Handle mesh status updates from offscreen
                console.log("[Background] Received mesh status update from offscreen:", payload);

                // Update local app state
                appState.meshStatus = payload.status || { type: MeshStatusType.Incomplete };

                // Broadcast mesh status update directly to popup
                broadcastToPopupPorts({
                    type: "meshStatusUpdate",
                    status: appState.meshStatus
                } as any);
                break;
            case "dkgStateUpdate":
                // Handle DKG state updates from offscreen
                console.log("[Background] Received DKG state update from offscreen:", payload);

                // Update local app state
                appState.dkgState = payload.state || DkgState.Idle;

                // Broadcast DKG state update directly to popup
                broadcastToPopupPorts({
                    type: "dkgStateUpdate",
                    state: appState.dkgState
                } as any);
                break;
            default:
                // Forward other messages to popup wrapped in fromOffscreen
                console.log("[Background] Forwarding unknown message to popup:", payload);
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
                const stateUpdate: InitialStateMessage = {
                    type: "initialState",
                    ...appState
                };
                console.log("[Background] Broadcasting full state update:", stateUpdate);
                broadcastToPopupPorts(stateUpdate as PopupMessage);

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
                const disconnectedState: InitialStateMessage = {
                    type: "initialState",
                    ...appState
                };
                broadcastToPopupPorts(disconnectedState as PopupMessage);
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
                const errorState: InitialStateMessage = {
                    type: "initialState",
                    ...appState
                };
                broadcastToPopupPorts(errorState as PopupMessage);
            });

            // Set up the message handler
            wsClient.onMessage((message: any) => {
                console.log("[Background] WebSocket message received:", message);

                // Cast to ServerMsg after receiving
                const serverMessage = message as ServerMsg;
                broadcastToPopupPorts({ type: "wsMessage", message: serverMessage });

                // Helper function to handle relay messages
                const handleRelayMessage = (msg: ServerMsg & { type: "relay" }, messageType: string) => {
                    console.log(`[Background] Received ${messageType} message from server:`, msg);
                    const data = msg.data as WebSocketMessagePayload;

                    switch (data.websocket_msg_type) {
                        case "WebRTCSignal":
                            console.log("[Background] WebRTC signal received:", data);
                            // Forward WebRTC signal to offscreen
                            const relayViaWs: OffscreenMessage = {
                                type: "relayViaWs",
                                to: msg.from,
                                data: data
                            };

                            safelySendOffscreenMessage({
                                type: "fromBackground",
                                payload: relayViaWs
                            }, "webrtc signal").then(result => {
                                if (!result.success) {
                                    console.warn("[Background] Failed to relay WebRTC signal to offscreen:", result.error);
                                }
                            });
                            break;

                        case "SessionProposal":
                            console.log("[Background] Session proposal received:", data);
                            // Handle session proposal
                            handleSessionProposal(msg.from, data);
                            break;

                        case "SessionResponse":
                            console.log("[Background] Session response received:", data);
                            handleSessionResponse(msg.from, data);
                            break;

                        default:
                            console.warn("[Background] Unknown relay message type:", (data as any).websocket_msg_type);
                            break;
                    }
                };

                // Helper function to handle peer list messages
                const handlePeerListMessage = (msg: ServerMsg & { type: "peers" }, messageType: string) => {
                    const peerList = msg.peers || [];
                    peers = peerList;
                    // Exclude current peer from connected peers list
                    appState.connectedPeers = peerList.filter((peerId: string) => peerId !== appState.peerId);
                    console.log(`[Background] Updated peer list from server (${messageType}):`, peerList);
                    console.log(`[Background] Connected peers (excluding self):`, appState.connectedPeers);

                    // Broadcast peer list update (excluding self)
                    broadcastToPopupPorts({ type: "peerList", peers: appState.connectedPeers });

                    // Also broadcast updated state
                    const peerListState: InitialStateMessage = {
                        type: "initialState",
                        ...appState
                    };
                    broadcastToPopupPorts(peerListState as PopupMessage);
                };

                // Handle specific message types with proper null checks
                switch (serverMessage.type) {
                    case "peers": // Handle lowercase "peers" messages from server
                        handlePeerListMessage(serverMessage as ServerMsg & { type: "peers" }, serverMessage.type);
                        break;

                    case "relay": // Handle lowercase "relay" messages from server
                        handleRelayMessage(serverMessage as ServerMsg & { type: "relay" }, serverMessage.type);
                        break;

                    case "error":
                        handleErrorMessage(serverMessage as ServerMsg & { type: "error" });
                        break;

                    default:
                        console.log("[Background] Unhandled WebSocket message type:", (serverMessage as any).type);
                        break;
                }
            });

            // Now connect after event handlers are set up
            console.log("[Background] Event handlers configured, attempting to connect to WebSocket:", WEBSOCKET_URL);
            wsClient.connect();
            console.log("[Background] WebSocket connect() method completed");

        } catch (error) {
            console.error("[Background] Failed to initialize WebSocket:", error);
            appState.wsConnected = false;

            broadcastToPopupPorts({
                type: "wsError",
                error: error instanceof Error ? error.message : "Unknown error"
            });
            broadcastToPopupPorts({ type: "wsStatus", connected: false });

            // Also broadcast updated state
            const initErrorState: InitialStateMessage = {
                type: "initialState",
                ...appState
            };
            broadcastToPopupPorts(initErrorState as PopupMessage);
        }
    };

    // Start WebSocket connection
    initializeWebSocket();

    console.log("[Background] Background script initialized");

    // Add session response handling function
    function handleSessionResponse(fromPeerId: string, responseData: any) {
        console.log("[Background] Handling session response from:", fromPeerId, "data:", responseData);

        const { session_id, accepted } = responseData;
        const peer_id = fromPeerId;

        // Validate peer_id
        if (!peer_id || typeof peer_id !== 'string') {
            console.warn("[Background] Invalid peer_id in session response:", peer_id);
            return;
        }

        // Find the target session
        let targetSession = appState.sessionInfo;
        if (!targetSession || targetSession.session_id !== session_id) {
            const inviteIndex = appState.invites.findIndex(invite => invite.session_id === session_id);
            if (inviteIndex >= 0) {
                targetSession = appState.invites[inviteIndex];
            }
        }

        if (targetSession && targetSession.session_id === session_id) {
            console.log("[Background] Session response for known session:", session_id);

            if (accepted) {
                // Add peer to accepted peers list if not already there
                if (peer_id && !targetSession.accepted_peers.includes(peer_id)) {
                    targetSession.accepted_peers.push(peer_id);
                    console.log("[Background] Added peer to accepted list:", peer_id);
                    console.log("[Background] Current accepted peers:", targetSession.accepted_peers);
                }
            } else {
                // Remove declining peer from accepted peers and participants
                targetSession.accepted_peers = targetSession.accepted_peers.filter(p => p !== peer_id);
                targetSession.participants = targetSession.participants.filter(p => p !== peer_id);
                console.log("[Background] Removed declining peer:", peer_id);
                console.log("[Background] Updated accepted peers:", targetSession.accepted_peers);
                console.log("[Background] Updated participants:", targetSession.participants);
            }

            // Update current session if this is it
            if (appState.sessionInfo && appState.sessionInfo.session_id === session_id) {
                appState.sessionInfo = { ...targetSession };

                // Check if all participants have now accepted
                const allAccepted = appState.sessionInfo.participants.every(participantId =>
                    appState.sessionInfo!.accepted_peers.includes(participantId)
                );

                console.log(`[Background] Session acceptance check - Participants: [${appState.sessionInfo.participants.join(', ')}], Accepted: [${appState.sessionInfo.accepted_peers.join(', ')}], All accepted: ${allAccepted}`);

                if (allAccepted) {
                    console.log("[Background] All participants have accepted the session! Notifying offscreen for mesh readiness.");

                    // Send updated session info to offscreen to trigger mesh readiness check
                    const sessionAllAcceptedMessage: OffscreenMessage = {
                        type: "sessionAllAccepted",
                        sessionInfo: appState.sessionInfo,
                        currentPeerId: appState.peerId,
                        blockchain: appState.blockchain || "solana" // Use stored blockchain or default to solana
                    };

                    safelySendOffscreenMessage({
                        type: "fromBackground",
                        payload: sessionAllAcceptedMessage
                    }, "sessionAllAccepted");
                } else {
                    console.log(`[Background] Not all participants accepted yet.`);

                    // Still send update to offscreen for tracking
                    const sessionResponseUpdateMessage: OffscreenMessage = {
                        type: "sessionResponseUpdate",
                        sessionInfo: appState.sessionInfo,
                        currentPeerId: appState.peerId
                    };

                    safelySendOffscreenMessage({
                        type: "fromBackground",
                        payload: sessionResponseUpdateMessage
                    }, "sessionResponseUpdate");
                }
            }

            // Broadcast session update to popup
            broadcastToPopupPorts({
                type: "sessionUpdate",
                sessionInfo: appState.sessionInfo,
                invites: appState.invites
            } as any);

            // Broadcast complete state update
            const completeState: InitialStateMessage = {
                type: "initialState",
                ...appState
            };
            broadcastToPopupPorts(completeState as PopupMessage);

            console.log("[Background] Session response processed and broadcasted");
        } else {
            console.warn("[Background] Received session response for unknown session:", session_id);
        }
    }

    // Add session proposal handling function
    function handleSessionProposal(fromPeerId: string, proposalData: any) {
        console.log("[Background] Handling session proposal from:", fromPeerId, "data:", proposalData);

        const sessionInfo: SessionInfo = {
            session_id: proposalData.session_id,
            total: proposalData.total,
            threshold: proposalData.threshold,
            participants: proposalData.participants || [],
            proposer_id: fromPeerId,
            accepted_peers: [fromPeerId], // Mark proposer as already accepted
            status: "proposed"
        };

        // Check if this peer is included in the participants
        const isParticipant = sessionInfo.participants.includes(appState.peerId);

        if (isParticipant) {
            console.log("[Background] This peer is included in session proposal, adding to invites");

            // Check if we already have this session in our invites
            const existingInviteIndex = appState.invites.findIndex(invite => invite.session_id === sessionInfo.session_id);

            if (existingInviteIndex >= 0) {
                console.log("[Background] Updating existing session invite");
                appState.invites[existingInviteIndex] = sessionInfo;
            } else {
                console.log("[Background] Adding new session invite");
                appState.invites.push(sessionInfo);
            }

            // If this peer is the proposer, automatically accept and set up WebRTC
            if (fromPeerId === appState.peerId) {
                console.log("[Background] This peer is the proposer, auto-accepting and setting up WebRTC");

                appState.sessionInfo = { ...sessionInfo, status: "accepted" };
                appState.invites = appState.invites.filter(inv => inv.session_id !== sessionInfo.session_id);

                // Forward to offscreen for WebRTC setup
                safelySendOffscreenMessage({
                    type: "fromBackground",
                    payload: {
                        type: "sessionAccepted",
                        sessionInfo: appState.sessionInfo,
                        currentPeerId: appState.peerId
                    }
                }, "proposerWebRTCSetup");
            }

            // Broadcast session update to popup
            broadcastToPopupPorts({
                type: "sessionUpdate",
                sessionInfo: appState.sessionInfo,
                invites: appState.invites
            } as any);

            // Broadcast complete state
            const proposalState: InitialStateMessage = {
                type: "initialState",
                ...appState
            };
            broadcastToPopupPorts(proposalState as PopupMessage);

            console.log("[Background] Session proposal processed and broadcasted to popup");
        } else {
            console.log("[Background] This peer is not included in session proposal, ignoring");
        }
    }

    // Add error message handler
    function handleErrorMessage(msg: ServerMsg & { type: "error" }) {
        console.error("[Background] Received error from server:", msg.error);
    }

    // ...existing code...
});