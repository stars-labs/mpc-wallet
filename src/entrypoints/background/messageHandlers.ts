// ===================================================================
// MESSAGE HANDLERS MODULE
// ===================================================================
//
// This module contains specialized message handlers for different
// types of background script communications including:
// - Popup message handling
// - Offscreen message routing
// - WebSocket relay operations
// - Address and blockchain requests
// ===================================================================

import type {
    PopupToBackgroundMessage,
    OffscreenToBackgroundMessage
} from "../../types/messages";
import { MESSAGE_TYPES, isRpcMessage, isAccountManagement, isNetworkManagement, isUIRequest } from "../../types/messages";
import { StateManager } from "./stateManager";
import { OffscreenManager } from "./offscreenManager";
import { WebSocketManager } from "./webSocketManager";
import { SessionManager } from "./sessionManager";
import { RpcHandler, UIRequestHandler } from "./rpcHandler";
import AccountService from "../../services/accountService";
import { DkgState } from "../../types/dkg";

/**
 * Handles messages from popup interface
 */
export class PopupMessageHandler {
    private stateManager: StateManager;
    private offscreenManager: OffscreenManager;
    private webSocketManager: WebSocketManager;
    private sessionManager: SessionManager;
    private rpcHandler: RpcHandler;
    private uiRequestHandler: UIRequestHandler;

    constructor(
        stateManager: StateManager,
        offscreenManager: OffscreenManager,
        webSocketManager: WebSocketManager,
        sessionManager: SessionManager,
        rpcHandler: RpcHandler,
        uiRequestHandler: UIRequestHandler
    ) {
        this.stateManager = stateManager;
        this.offscreenManager = offscreenManager;
        this.webSocketManager = webSocketManager;
        this.sessionManager = sessionManager;
        this.rpcHandler = rpcHandler;
        this.uiRequestHandler = uiRequestHandler;
    }

    /**
     * Handle messages from popup with enhanced pattern-based categorization
     */
    async handlePopupMessage(
        message: PopupToBackgroundMessage,
        sendResponse: (response: any) => void
    ): Promise<void> {
        const startTime = Date.now();
        const messageType = message.type;

        // Enhanced pattern-based categorization
        const { category, categoryInfo } = this.categorizeMessage(message);

        console.log("┌─────────────────────────────────────────────────────────────────");
        console.log(`│ ${categoryInfo.color}[PopupMessageHandler] ${categoryInfo.icon} Processing: ${messageType}\x1b[0m`);
        console.log(`│ Category: ${categoryInfo.icon} ${categoryInfo.name}`);

        // Keep messageCategory for backward compatibility
        let messageCategory = categoryInfo.name;

        // Enhanced logging for RPC messages
        if (isRpcMessage(message)) {
            messageCategory = 'RPC';
            const rpcMethod = (message as any).payload?.method || 'unknown';
            const rpcParams = (message as any).payload?.params;
            const rpcId = (message as any).payload?.id;

            console.log(`│ RPC Method: ${rpcMethod}`);
            console.log(`│ RPC ID: ${rpcId}`);
            console.log(`│ RPC Params:`, rpcParams);

            // Log specific RPC methods for better tracking
            if (rpcMethod.includes('eth_')) {
                console.log(`│ 🔗 Ethereum RPC: ${rpcMethod}`);
            } else if (rpcMethod.includes('sol_') || rpcMethod.includes('solana_')) {
                console.log(`│ 🟣 Solana RPC: ${rpcMethod}`);
            } else if (rpcMethod.includes('sign')) {
                console.log(`│ ✍️ Signing RPC: ${rpcMethod}`);
            } else if (rpcMethod.includes('account') || rpcMethod.includes('address')) {
                console.log(`│ 👤 Account RPC: ${rpcMethod}`);
            } else {
                console.log(`│ 🔧 Generic RPC: ${rpcMethod}`);
            }
        } else if (isAccountManagement(message)) {
            messageCategory = 'Account Management';
        } else if (isNetworkManagement(message)) {
            messageCategory = 'Network Management';
        } else if (isUIRequest(message)) {
            messageCategory = 'UI Request';
        }

        console.log(`│ Data:`, message);
        console.log("└─────────────────────────────────────────────────────────────────");

        try {
            switch (message.type) {
                case MESSAGE_TYPES.GET_STATE:
                    console.log("📊 [PopupMessageHandler] GET_STATE: Returning current application state");
                    const state = this.stateManager.getState();
                    console.log("📊 [PopupMessageHandler] State keys:", Object.keys(state));
                    sendResponse(state);
                    break;

                case MESSAGE_TYPES.GET_WEBRTC_STATE:
                    console.log("📡 [PopupMessageHandler] GET_WEBRTC_STATE: Returning WebRTC connections");
                    const webrtcConnections = this.stateManager.getWebRTCConnections();
                    console.log("📡 [PopupMessageHandler] WebRTC connections:", webrtcConnections);
                    sendResponse({ webrtcConnections });
                    break;

                case MESSAGE_TYPES.LIST_DEVICES:
                    console.log("📋 [PopupMessageHandler] LIST_DEVICES: Requesting peer discovery");
                    await this.handleListDevicesRequest(sendResponse);
                    break;

                case MESSAGE_TYPES.RELAY:
                    console.log("🔄 [PopupMessageHandler] RELAY: Forwarding message via WebSocket");
                    await this.handleRelayRequest(message, sendResponse);
                    break;

                case MESSAGE_TYPES.CREATE_OFFSCREEN:
                    console.log("📄 [PopupMessageHandler] CREATE_OFFSCREEN: Creating offscreen document");
                    await this.handleCreateOffscreenRequest(sendResponse);
                    break;

                case MESSAGE_TYPES.GET_OFFSCREEN_STATUS:
                    console.log("📄 [PopupMessageHandler] GET_OFFSCREEN_STATUS: Checking offscreen status");
                    await this.handleGetOffscreenStatusRequest(sendResponse);
                    break;

                case MESSAGE_TYPES.FROM_OFFSCREEN:
                    console.log("📤 [PopupMessageHandler] FROM_OFFSCREEN: Processing offscreen message");
                    await this.handleFromOffscreenMessage(message, sendResponse);
                    break;

                case "requestInit":
                    console.log("🔧 [PopupMessageHandler] REQUEST_INIT: Handling initialization request");
                    await this.handleRequestInitMessage(sendResponse);
                    break;
                    
                case "approveMessageSignature":
                    console.log("✍️ [PopupMessageHandler] APPROVE_MESSAGE_SIGNATURE: Handling signature approval");
                    await this.handleApproveMessageSignature(message, sendResponse);
                    break;

                // Session restore removed - sessions are ephemeral for security
                // case "requestSessionRestore": removed

                case MESSAGE_TYPES.PROPOSE_SESSION:
                    console.log("🔐 [PopupMessageHandler] PROPOSE_SESSION: Creating new MPC session");
                    await this.handleProposeSessionRequest(message, sendResponse);
                    break;

                case MESSAGE_TYPES.ACCEPT_SESSION:
                    console.log("🔐 [PopupMessageHandler] ACCEPT_SESSION: Accepting MPC session invite");
                    await this.handleAcceptSessionRequest(message, sendResponse);
                    break;

                case MESSAGE_TYPES.SEND_DIRECT_MESSAGE:
                    console.log("💬 [PopupMessageHandler] SEND_DIRECT_MESSAGE: Sending direct peer message");
                    await this.handleSendDirectMessageRequest(message, sendResponse);
                    break;

                case MESSAGE_TYPES.GET_WEBRTC_STATUS:
                    console.log("📡 [PopupMessageHandler] GET_WEBRTC_STATUS: Getting WebRTC status");
                    await this.handleGetWebRTCStatusRequest(sendResponse);
                    break;

                case "setBlockchain":
                    console.log("🔗 [PopupMessageHandler] SET_BLOCKCHAIN: Setting blockchain preference");
                    this.handleSetBlockchainRequest(message, sendResponse);
                    break;

                case "getEthereumAddress":
                    console.log("🏠 [PopupMessageHandler] GET_ETHEREUM_ADDRESS: Requesting Ethereum address");
                    await this.handleGetEthereumAddressRequest(sendResponse);
                    break;

                case "getSolanaAddress":
                    console.log("🏠 [PopupMessageHandler] GET_SOLANA_ADDRESS: Requesting Solana address");
                    await this.handleGetSolanaAddressRequest(sendResponse);
                    break;

                case MESSAGE_TYPES.REQUEST_SIGNING:
                    console.log("✍️ [PopupMessageHandler] REQUEST_SIGNING: Initiating MPC signing");
                    await this.handleRequestSigningMessage(message, sendResponse);
                    break;

                default:
                    if (isRpcMessage(message)) {
                        console.log("🔗 [PopupMessageHandler] RPC_MESSAGE: Processing JSON-RPC request");
                        await this.handleRpcMessage(message, sendResponse);
                    } else if (isAccountManagement(message)) {
                        console.log("👤 [PopupMessageHandler] ACCOUNT_MANAGEMENT: Not implemented");
                        sendResponse({ success: false, error: "Account management not implemented" });
                    } else if (isNetworkManagement(message)) {
                        console.log("🌐 [PopupMessageHandler] NETWORK_MANAGEMENT: Not implemented");
                        sendResponse({ success: false, error: "Network management not implemented" });
                    } else if (isUIRequest(message)) {
                        console.log("🖼️ [PopupMessageHandler] UI_REQUEST: Processing UI request");
                        await this.handleUIRequestMessage(message, sendResponse);
                    } else {
                        console.warn("❓ [PopupMessageHandler] UNKNOWN_MESSAGE_TYPE:", message.type);
                        sendResponse({ success: false, error: `Unknown message type: ${message.type}` });
                    }
                    break;
            }
        } catch (error) {
            const duration = Date.now() - startTime;
            const errorDetails = (error as Error).message;

            if (messageCategory === 'RPC') {
                const rpcMethod = (message as any).payload?.method || 'unknown';
                const rpcId = (message as any).payload?.id;
                console.error(`❌ [PopupMessageHandler] RPC ERROR: ${rpcMethod} (ID: ${rpcId}) failed after ${duration}ms`);
                console.error(`❌ RPC Error Details:`, errorDetails);
                sendResponse({
                    success: false,
                    error: errorDetails,
                    rpcMethod,
                    rpcId,
                    duration
                });
            } else {
                console.error(`❌ [PopupMessageHandler] Error in ${messageType} (${messageCategory}) after ${duration}ms:`, error);
                sendResponse({ success: false, error: errorDetails });
            }
        } finally {
            const duration = Date.now() - startTime;

            if (messageCategory === 'RPC') {
                const rpcMethod = (message as any).payload?.method || 'unknown';
                const rpcId = (message as any).payload?.id;
                console.log(`⏱️ [PopupMessageHandler] 🔗 RPC ${rpcMethod} (ID: ${rpcId}) completed in ${duration}ms`);
            } else {
                console.log(`⏱️ [PopupMessageHandler] ${messageType} (${messageCategory}) completed in ${duration}ms`);
            }
        }
    }

    /**
     * Pattern-based message categorization using simple matching
     */
    private categorizeMessage(message: PopupToBackgroundMessage): { category: string; categoryInfo: any } {
        const messageType = message.type;

        // Pattern matching for message categories
        if (messageType.includes('getState') || messageType.includes('setState') ||
            messageType === 'GET_STATE' || messageType === 'GET_WEBRTC_STATE') {
            return {
                category: 'state_management',
                categoryInfo: {
                    name: 'State Management',
                    icon: '📊',
                    color: '\x1b[36m' // cyan
                }
            };
        }

        if (messageType.includes('session') || messageType.includes('Session') ||
            messageType === 'CREATE_SESSION' || messageType === 'JOIN_SESSION' ||
            messageType === 'LEAVE_SESSION' || messageType === 'PROPOSE_SESSION' ||
            messageType === 'ACCEPT_SESSION') {
            return {
                category: 'session_management',
                categoryInfo: {
                    name: 'Session Management',
                    icon: '🔐',
                    color: '\x1b[35m' // magenta
                }
            };
        }

        if (messageType.includes('webrtc') || messageType.includes('WebRTC') ||
            messageType.includes('WEBRTC') || messageType === 'GET_WEBRTC_STATUS') {
            return {
                category: 'webrtc_control',
                categoryInfo: {
                    name: 'WebRTC Control',
                    icon: '📡',
                    color: '\x1b[34m' // blue
                }
            };
        }

        if (messageType.includes('offscreen') || messageType.includes('Offscreen') ||
            messageType.includes('OFFSCREEN') || messageType === 'CREATE_OFFSCREEN' ||
            messageType === 'GET_OFFSCREEN_STATUS' || messageType === 'FROM_OFFSCREEN' ||
            messageType === 'offscreenReady') {
            return {
                category: 'offscreen_control',
                categoryInfo: {
                    name: 'Offscreen Control',
                    icon: '📄',
                    color: '\x1b[33m' // yellow
                }
            };
        }

        if (messageType.includes('address') || messageType.includes('Address') ||
            messageType.includes('ADDRESS') || messageType === 'getEthereumAddress' ||
            messageType === 'getSolanaAddress') {
            return {
                category: 'address_management',
                categoryInfo: {
                    name: 'Address Management',
                    icon: '🏠',
                    color: '\x1b[32m' // green
                }
            };
        }

        if (messageType === 'setBlockchain' || messageType.includes('network') ||
            messageType.includes('Network')) {
            return {
                category: 'network_management',
                categoryInfo: {
                    name: 'Network Management',
                    icon: '🌐',
                    color: '\x1b[31m' // red
                }
            };
        }

        if (messageType.includes('rpc') || messageType.includes('RPC') ||
            messageType.startsWith('eth_')) {
            return {
                category: 'rpc_request',
                categoryInfo: {
                    name: 'RPC Request',
                    icon: '⚡',
                    color: '\x1b[93m' // bright yellow
                }
            };
        }

        if (messageType === 'RELAY') {
            return {
                category: 'relay',
                categoryInfo: {
                    name: 'Message Relay',
                    icon: '🔄',
                    color: '\x1b[94m' // bright blue
                }
            };
        }

        if (messageType === 'LIST_DEVICES' || messageType === 'requestInit') {
            return {
                category: 'ui_request',
                categoryInfo: {
                    name: 'UI Request',
                    icon: '🖼️',
                    color: '\x1b[96m' // bright cyan
                }
            };
        }

        if (messageType.includes('sign') || messageType.includes('Sign') ||
            messageType.includes('SIGN') || messageType === 'REQUEST_SIGNING' ||
            messageType === 'ACCEPT_SIGNING') {
            return {
                category: 'signing',
                categoryInfo: {
                    name: 'Signing Operations',
                    icon: '✍️',
                    color: '\x1b[95m' // bright magenta
                }
            };
        }

        // Default unknown category
        return {
            category: 'unknown',
            categoryInfo: {
                name: 'Unknown',
                icon: '❓',
                color: '\x1b[90m' // gray
            }
        };
    }

    private async handleListDevicesRequest(sendResponse: (response: any) => void): Promise<void> {
        console.log("[PopupMessageHandler] LIST_DEVICES request received. WebSocket state:", this.webSocketManager.isReady());

        const result = await this.webSocketManager.listDevices();
        if (result.success) {
            console.log("[PopupMessageHandler] Peer list request sent successfully");
            sendResponse({ success: true });
        } else {
            console.warn("[PopupMessageHandler] WebSocket not connected, cannot list devices");
            sendResponse({ success: false, error: result.error });
        }
    }

    private async handleRelayRequest(message: any, sendResponse: (response: any) => void): Promise<void> {
        if ('to' in message && 'data' in message) {
            const result = await this.webSocketManager.relayMessage(message.to as string, message.data);
            sendResponse(result);
        } else {
            sendResponse({ success: false, error: "Invalid relay message format" });
        }
    }

    private async handleCreateOffscreenRequest(sendResponse: (response: any) => void): Promise<void> {
        const createResult = await this.offscreenManager.createOffscreenDocument();
        sendResponse(createResult);
    }

    private async handleGetOffscreenStatusRequest(sendResponse: (response: any) => void): Promise<void> {
        const status = await this.offscreenManager.getOffscreenStatus();
        sendResponse(status);
    }

    private async handleFromOffscreenMessage(message: any, sendResponse: (response: any) => void): Promise<void> {
        if ('payload' in message) {
            this.stateManager.handleOffscreenStateUpdate(message.payload as OffscreenToBackgroundMessage);
            sendResponse({ success: true });
        } else {
            sendResponse({ success: false, error: "FromOffscreen message missing payload" });
        }
    }

    private async handleRequestInitMessage(sendResponse: (response: any) => void): Promise<void> {
        const result = await this.offscreenManager.handleInitRequest();
        sendResponse(result);
    }
    
    private async handleApproveMessageSignature(message: any, sendResponse: (response: any) => void): Promise<void> {
        if (!message.requestId || typeof message.approved !== 'boolean') {
            sendResponse({ success: false, error: "Invalid approval message" });
            return;
        }
        
        console.log(`[PopupMessageHandler] Signature approval for ${message.requestId}: ${message.approved}`);
        
        if (!message.approved) {
            // User rejected the signature
            // Find the pending signature in RPC handler and reject it
            if (this.rpcHandler.handleSignatureError) {
                this.rpcHandler.handleSignatureError(message.requestId, "User rejected signature request");
            }
            sendResponse({ success: true });
            return;
        }
        
        // User approved - the signature will be processed by the offscreen document
        // which is already handling the MPC signing flow
        sendResponse({ success: true });
    }

    // Session restore removed - sessions are ephemeral for security

    private async handleProposeSessionRequest(message: any, sendResponse: (response: any) => void): Promise<void> {
        if ('session_id' in message && 'total' in message && 'threshold' in message && 'participants' in message) {
            console.log("[PopupMessageHandler] Proposing session:", message.session_id);
            
            const blockchain = message.blockchain || "solana";

            const result = await this.sessionManager.proposeSession(
                message.session_id,
                message.total,
                message.threshold,
                message.participants,
                blockchain
            );

            sendResponse(result);
        } else {
            sendResponse({ success: false, error: "Invalid session proposal" });
        }
    }

    private async handleAcceptSessionRequest(message: any, sendResponse: (response: any) => void): Promise<void> {
        if ('session_id' in message && 'accepted' in message) {
            console.log("[PopupMessageHandler] Session acceptance:", message.session_id, message.accepted);
            
            // Log current state for debugging
            const currentInvites = this.stateManager.getInvites();
            const currentSessionInfo = this.stateManager.getSessionInfo();
            console.log("[PopupMessageHandler] Current invites:", currentInvites);
            console.log("[PopupMessageHandler] Current sessionInfo:", currentSessionInfo);

            if (message.accepted) {
                const blockchain = message.blockchain || "solana";
                const result = await this.sessionManager.acceptSession(message.session_id, blockchain);
                sendResponse(result);
            } else {
                // Handle session decline
                const invites = this.stateManager.getInvites();
                const sessionIndex = invites.findIndex(inv => inv.session_id === message.session_id);

                if (sessionIndex !== -1) {
                    invites.splice(sessionIndex, 1);
                    this.stateManager.updateInvites(invites);
                    sendResponse({ success: true });
                } else {
                    sendResponse({ success: false, error: "Session not found in invites" });
                }
            }
        } else {
            sendResponse({ success: false, error: "Invalid session acceptance" });
        }
    }

    private async handleSendDirectMessageRequest(message: any, sendResponse: (response: any) => void): Promise<void> {
        console.log("[PopupMessageHandler] Received sendDirectMessage request:", message);

        if ('todeviceId' in message && 'message' in message &&
            typeof message.todeviceId === 'string' && typeof message.message === 'string') {

            const result = await this.offscreenManager.sendToOffscreen({
                type: "sendDirectMessage",
                todeviceId: message.todeviceId,
                message: message.message
            }, "sendDirectMessage");

            if (result.success) {
                sendResponse({ success: true, message: "Direct message sent to offscreen" });
            } else {
                sendResponse({ success: false, error: `Failed to send to offscreen: ${result.error}` });
            }
        } else {
            sendResponse({ success: false, error: "Missing or invalid todeviceId or message" });
        }
    }

    private async handleGetWebRTCStatusRequest(sendResponse: (response: any) => void): Promise<void> {
        console.log("[PopupMessageHandler] Received getWebRTCStatus request");

        const result = await this.offscreenManager.sendToOffscreen({
            type: "getWebRTCStatus"
        }, "getWebRTCStatus");

        if (result.success) {
            sendResponse({ success: true, message: "WebRTC status request sent to offscreen" });
        } else {
            sendResponse({ success: false, error: `Failed to get WebRTC status: ${result.error}` });
        }
    }

    private handleSetBlockchainRequest(message: any, sendResponse: (response: any) => void): void {
        if ('blockchain' in message) {
            console.log("[PopupMessageHandler] Setting blockchain selection:", message.blockchain);
            this.stateManager.setBlockchain(message.blockchain);
            sendResponse({ success: true, blockchain: this.stateManager.getBlockchain() });
        } else {
            sendResponse({ success: false, error: "Missing blockchain parameter" });
        }
    }

    private async handleGetEthereumAddressRequest(sendResponse: (response: any) => void): Promise<void> {
        try {
            const ethResult = await this.offscreenManager.sendToOffscreen({
                type: "getEthereumAddress"
            }, "getEthereumAddress");
            
            // Store address in chrome.storage.local for content script access
            if (ethResult.success && ethResult.data?.ethereumAddress) {
                chrome.storage.local.set({ 
                    'mpc_ethereum_address': ethResult.data.ethereumAddress 
                }, () => {
                    console.log("[PopupMessageHandler] Stored Ethereum address in chrome.storage.local");
                });
                sendResponse(ethResult);
            } else {
                // If no address from offscreen, check if we have a stored address
                chrome.storage.local.get(['mpc_ethereum_address'], (result) => {
                    if (result && result.mpc_ethereum_address) {
                        console.log("[PopupMessageHandler] Using stored Ethereum address:", result.mpc_ethereum_address);
                        sendResponse({
                            success: true,
                            data: { ethereumAddress: result.mpc_ethereum_address }
                        });
                    } else {
                        console.log("[PopupMessageHandler] No Ethereum address available (DKG not complete)");
                        sendResponse({
                            success: false,
                            error: "No Ethereum address available. Please complete DKG setup first."
                        });
                    }
                });
            }
        } catch (error) {
            console.error("[PopupMessageHandler] Error getting Ethereum address:", error);
            sendResponse({ success: false, error: `Error getting Ethereum address: ${(error as Error).message}` });
        }
    }

    private async handleGetSolanaAddressRequest(sendResponse: (response: any) => void): Promise<void> {
        try {
            const solResult = await this.offscreenManager.sendToOffscreen({
                type: "getSolanaAddress"
            }, "getSolanaAddress");
            
            // Store address in chrome.storage.local for content script access
            if (solResult.success && solResult.data?.solanaAddress) {
                chrome.storage.local.set({ 
                    'mpc_solana_address': solResult.data.solanaAddress 
                }, () => {
                    console.log("[PopupMessageHandler] Stored Solana address in chrome.storage.local");
                });
            }
            
            sendResponse(solResult);
        } catch (error) {
            console.error("[PopupMessageHandler] Error getting Solana address:", error);
            sendResponse({ success: false, error: `Error getting Solana address: ${(error as Error).message}` });
        }
    }

    private async handleRpcMessage(message: any, sendResponse: (response: any) => void): Promise<void> {
        try {
            // Set origin if provided (from content script)
            if (message.origin) {
                this.rpcHandler.setOrigin(message.origin);
            }
            
            const result = await this.rpcHandler.handleRpcRequest(message.payload);
            sendResponse({ success: true, result });
        } catch (error) {
            console.error("[PopupMessageHandler] RPC request failed:", error);
            sendResponse({ success: false, error: (error as Error).message });
        }
    }

    private async handleUIRequestMessage(message: any, sendResponse: (response: any) => void): Promise<void> {
        const result = await this.uiRequestHandler.handleUIRequest(message.payload);
        sendResponse(result);
    }

    private async handleRequestSigningMessage(message: any, sendResponse: (response: any) => void): Promise<void> {
        console.log("[PopupMessageHandler] Received requestSigning:", message);

        if ('signingId' in message && 'transactionData' in message && 'requiredSigners' in message) {
            // Forward signing request to offscreen document
            const result = await this.offscreenManager.sendToOffscreen({
                type: "requestSigning",
                signingId: message.signingId,
                transactionData: message.transactionData,
                requiredSigners: message.requiredSigners
            }, "requestSigning");

            if (result.success) {
                sendResponse({ success: true, message: "Signing request sent to offscreen" });
            } else {
                sendResponse({ success: false, error: `Failed to send signing request: ${result.error}` });
            }
        } else {
            sendResponse({ success: false, error: "Invalid signing request format" });
        }
    }
}

/**
 * Handles messages from offscreen document
 */
export class OffscreenMessageHandler {
    private stateManager: StateManager;
    private webSocketManager: WebSocketManager;

    constructor(
        stateManager: StateManager,
        webSocketManager: WebSocketManager
    ) {
        this.stateManager = stateManager;
        this.webSocketManager = webSocketManager;
    }

    /**
     * Handle messages from offscreen document
     */
    async handleOffscreenMessage(payload: OffscreenToBackgroundMessage): Promise<void> {
        console.log("[OffscreenMessageHandler] Handling offscreen message:", payload);

        switch (payload.type) {
            case "relayViaWs":
                await this.handleRelayViaWebSocket(payload);
                break;

            case "log":
                this.handleLogMessage(payload);
                break;

            case "signingComplete":
                this.handleSigningComplete(payload);
                break;

            case "signingError":
                this.handleSigningError(payload);
                break;

            case "dkg_complete":
                this.handleDkgComplete(payload);
                break;

            case "dkg_state_update":
                this.handleDkgStateUpdate(payload);
                break;

            default:
                // Forward to state manager for state updates
                this.stateManager.handleOffscreenStateUpdate(payload);
                break;
        }
    }

    private async handleRelayViaWebSocket(payload: any): Promise<void> {
        // Handle nested payload structure - the actual data is in payload.payload
        const relayData = payload.payload || payload;

        // Enhanced debugging for WebSocket relay issues
        const hasTo = 'to' in relayData;
        const hasData = 'data' in relayData;
        const wsReady = this.webSocketManager.isReady();
        const wsState = this.webSocketManager.getConnectionStatus();

        console.log("[OffscreenMessageHandler] WebSocket relay check:", {
            hasTo,
            hasData,
            wsReady,
            wsState,
            originalPayloadKeys: Object.keys(payload),
            relayDataKeys: Object.keys(relayData),
            relayData: relayData
        });

        if (hasTo && hasData && wsReady) {
            try {
                console.log("[OffscreenMessageHandler] Attempting to relay WebSocket message:", {
                    to: relayData.to,
                    dataType: relayData.data?.websocket_msg_type,
                    data: relayData.data
                });
                await this.webSocketManager.relayMessage(relayData.to as string, relayData.data);
                console.log("[OffscreenMessageHandler] WebSocket relay successful");
            } catch (error) {
                console.error("[OffscreenMessageHandler] Error relaying via WebSocket:", error);
            }
        } else {
            const issues = [];
            if (!hasTo) issues.push("missing 'to' property");
            if (!hasData) issues.push("missing 'data' property");
            if (!wsReady) issues.push(`WebSocket not ready (state: ${wsState.readyState})`);

            console.warn("[OffscreenMessageHandler] Cannot relay message:", issues.join(", "));
            console.warn("[OffscreenMessageHandler] Full payload structure:", JSON.stringify(payload, null, 2));
        }
    }

    private handleLogMessage(payload: any): void {
        if ('payload' in payload && payload.payload && payload.payload.message) {
            const source = payload.payload.source || 'offscreen';
            console.log(`📄 [OffscreenMessageHandler] LOG from ${source}: ${payload.payload.message}`);
        } else {
            console.log("[OffscreenMessageHandler] LOG:", payload);
        }
    }

    private handleSigningComplete(payload: any): void {
        console.log("[OffscreenMessageHandler] Signing complete:", payload);
        if (payload.signingId && payload.signature) {
            // Forward to popup/content scripts if needed
            chrome.runtime.sendMessage({
                type: MESSAGE_TYPES.SIGNING_COMPLETE,
                signingId: payload.signingId,
                signature: payload.signature
            });
        }
    }

    private handleSigningError(payload: any): void {
        console.error("[OffscreenMessageHandler] Signing error:", payload);
        if (payload.signingId && payload.error) {
            // Forward to popup/content scripts if needed
            chrome.runtime.sendMessage({
                type: MESSAGE_TYPES.SIGNING_ERROR,
                signingId: payload.signingId,
                error: payload.error
            });
        }
    }

    private async handleDkgComplete(payload: any): void {
        console.log("[OffscreenMessageHandler] DKG complete:", payload);
        
        // Update DKG state to Complete
        this.stateManager.updateStateProperty('dkgState', DkgState.Complete);
        
        if (payload.payload && payload.payload.keyShareData) {
            const keyShareData = payload.payload.keyShareData;
            const sessionId = keyShareData.sessionId;
            
            // Get the appropriate address based on blockchain
            const address = keyShareData.curve === 'secp256k1' 
                ? keyShareData.ethereumAddress 
                : keyShareData.solanaAddress;
            
            // Store addresses in chrome.storage.local for immediate access
            if (keyShareData.ethereumAddress) {
                chrome.storage.local.set({ 
                    'mpc_ethereum_address': keyShareData.ethereumAddress 
                }, () => {
                    console.log("[OffscreenMessageHandler] Stored Ethereum address in chrome.storage.local:", keyShareData.ethereumAddress);
                });
            }
            
            if (keyShareData.solanaAddress) {
                chrome.storage.local.set({ 
                    'mpc_solana_address': keyShareData.solanaAddress 
                }, () => {
                    console.log("[OffscreenMessageHandler] Stored Solana address in chrome.storage.local:", keyShareData.solanaAddress);
                });
            }
            
            if (address && sessionId) {
                try {
                    // Complete account creation
                    const accountService = AccountService.getInstance();
                    const newAccount = await accountService.completeAccountCreation(
                        sessionId,
                        address,
                        keyShareData
                    );
                    
                    if (newAccount) {
                        console.log("[OffscreenMessageHandler] Account created for session:", sessionId);
                        
                        // Notify popup to refresh accounts
                        this.stateManager.broadcastToPopupPorts({
                            type: 'accountsUpdated',
                            blockchain: newAccount.blockchain,
                            accounts: accountService.getAccountsByBlockchain(newAccount.blockchain)
                        });
                    } else {
                        console.warn("[OffscreenMessageHandler] Account creation returned null, but DKG is still complete");
                    }
                } catch (error) {
                    console.error("[OffscreenMessageHandler] Error during account creation:", error);
                    // Even if account creation fails, DKG is still complete
                    // The user can still use the wallet for signing
                }
            }
        }
        
        // Ensure DKG state remains Complete regardless of account creation outcome
        this.stateManager.updateStateProperty('dkgState', DkgState.Complete);
    }

    private handleDkgStateUpdate(payload: any): void {
        console.log("[OffscreenMessageHandler] DKG state update:", payload);
        
        if (payload.payload && typeof payload.payload.state === 'number') {
            const newState = payload.payload.state;
            // Only update state if it's not going backwards from Complete
            const currentState = this.stateManager.getState().dkgState;
            
            if (currentState === DkgState.Complete && newState === DkgState.Idle) {
                console.log("[OffscreenMessageHandler] Ignoring attempt to reset DKG state from Complete to Idle");
                return;
            }
            
            this.stateManager.updateStateProperty('dkgState', newState);
        }
    }
}
