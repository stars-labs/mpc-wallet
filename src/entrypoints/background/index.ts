// filepath: /home/freeman.xiong/Documents/github/hecoinfo/mpc-wallet/src/entrypoints/background/index.ts
// ===================================================================
// MAIN BACKGROUND SCRIPT COORDINATOR
// ===================================================================
//
// This is the main background script that coordinates all modular
// components for the MPC wallet extension. It imports and initializes
// all specialized managers and handlers:
//
// - SessionManager: Handles MPC session lifecycle
// - RpcHandler: Processes JSON-RPC and UI requests  
// - OffscreenManager: Manages Chrome Extension offscreen documents
// - WebSocketManager: Handles signaling server connections
// - StateManager: Manages central application state
// - Message Handlers: Process inter-component communications
// ===================================================================

import { defineBackground } from '#imports';
import { MESSAGE_PREFIX, MessageType } from '../../constants';
import AccountService from '../../services/accountService';
import NetworkService from '../../services/networkService';
import WalletClientService from '../../services/walletClient';
import { toHex } from 'viem';
import WalletController from "../../services/walletController";
import { WebSocketClient } from "./websocket";

// Import modular components
import { SessionManager } from './sessionManager';
import { RpcHandler, UIRequestHandler } from './rpcHandler';
import { OffscreenManager } from './offscreenManager';
import { WebSocketManager } from './webSocketManager';
import { StateManager } from './stateManager';
import { PopupMessageHandler, OffscreenMessageHandler } from './messageHandlers';

// Import types
import { AppState, INITIAL_APP_STATE } from "../../types/appstate";
import { SessionProposal, SessionResponse, SessionInfo } from "../../types/session";
import { MeshStatusType, MeshStatus } from "../../types/mesh";
import { DkgState } from "../../types/dkg";
import {
    type JsonRpcRequest,
    type PopupToBackgroundMessage,
    type BackgroundToOffscreenMessage,
    type BackgroundToPopupMessage,
    type OffscreenToBackgroundMessage,
    type BackgroundToOffscreenWrapper,
    type InitialStateMessage,
    validateMessage,
    validateSessionProposal,
    validateSessionAcceptance,
    isRpcMessage,
    isAccountManagement,
    isNetworkManagement,
    isUIRequest,
    MESSAGE_TYPES,
    // Legacy aliases for backward compatibility
    type BackgroundMessage,
    type OffscreenMessage,
    type PopupMessage,
} from "../../types/messages";
import { ServerMsg, ClientMsg, WebSocketMessagePayload, WebRTCSignal } from '../../types/websocket';

// ===================================================================
// SERVICE INITIALIZATION AND GLOBAL STATE
// ===================================================================

// Initialize services
const accountService = AccountService.getInstance();
const networkService = NetworkService.getInstance();
const walletClientService = WalletClientService.getInstance();

// Initialize managers and handlers
let stateManager: StateManager;
let sessionManager: SessionManager;
let rpcHandler: RpcHandler;
let uiRequestHandler: UIRequestHandler;
let offscreenManager: OffscreenManager;
let webSocketManager: WebSocketManager;
let popupMessageHandler: PopupMessageHandler;
let offscreenMessageHandler: OffscreenMessageHandler;

// Global state variables for legacy compatibility
let wsClient: WebSocketClient | null = null;
let devices: string[] = [];

// ===================================================================
// COMPONENT INITIALIZATION
// ===================================================================

/**
 * Initialize all modular components
 */
function initializeComponents(): void {
    console.log("�� [Background] Initializing modular components...");

    // Initialize state manager with initial state
    stateManager = new StateManager(INITIAL_APP_STATE);

    // Initialize RPC and UI request handlers (no parameters needed)
    rpcHandler = new RpcHandler();
    uiRequestHandler = new UIRequestHandler();

    // Initialize offscreen manager (use shared state reference)
    offscreenManager = new OffscreenManager(stateManager.getStateReference());

    // Initialize session manager (use shared state reference)
    sessionManager = new SessionManager(
        stateManager.getStateReference(), // Use reference, not copy
        wsClient,
        (message) => stateManager.broadcastToPopupPorts(message),
        (message, description) => offscreenManager.sendToOffscreen(message, description)
    );

    // Initialize WebSocket manager (use shared state reference)
    webSocketManager = new WebSocketManager(
        stateManager.getStateReference(), // Use reference, not copy
        sessionManager,
        (message) => stateManager.broadcastToPopupPorts(message),
        (message, description) => offscreenManager.sendToOffscreen(message, description),
        stateManager  // Add StateManager for persistence
    );

    // Initialize message handlers with all dependencies
    popupMessageHandler = new PopupMessageHandler(
        stateManager,
        offscreenManager,
        webSocketManager,
        sessionManager,
        rpcHandler,
        uiRequestHandler
    );

    offscreenMessageHandler = new OffscreenMessageHandler(
        stateManager,
        webSocketManager
    );

    console.log("✅ [Background] All components initialized successfully");
}

// ===================================================================
// POPUP PORT MANAGEMENT
// ===================================================================

/**
 * Set up popup port connections
 */
function setupPopupConnections(): void {
    chrome.runtime.onConnect.addListener((port) => {
        if (port.name === "popup") {
            console.log("🔌 [Background] Popup connected");
            stateManager.addPopupPort(port);
        }
    });
}

// ===================================================================
// MESSAGE HANDLING
// ===================================================================

/**
 * Handle incoming messages from popup and content scripts
 */
function setupMessageHandlers(): void {
    chrome.runtime.onMessage.addListener((message: unknown, sender, sendResponse) => {
        // Enhanced logging for message routing with RPC detection
        const senderType = sender.tab ? 'content-script' : (sender.url?.includes('popup') ? 'popup' : (sender.url?.includes('offscreen') ? 'offscreen' : 'unknown'));
        const tabInfo = sender.tab ? `tab-${sender.tab.id}` : 'no-tab';

        console.log("┌─────────────────────────────────────────────────────────────────");
        console.log(`│ [Background Router] 📨 Message Received`);
        console.log(`│ Type: ${(message as any)?.type || 'unknown'}`);
        console.log(`│ From: ${senderType} (${tabInfo})`);
        console.log(`│ URL: ${sender.url || 'unknown'}`);
        console.log(`│ Message:`, message);
        console.log("└─────────────────────────────────────────────────────────────────");

        // Validate basic message structure
        if (!validateMessage(message)) {
            console.warn("❌ [Background] Invalid message structure:", message);
            sendResponse({ success: false, error: "Invalid message structure" });
            return true;
        }

        // Handle async operations
        (async () => {
            const startTime = Date.now();
            const messageType = (message as any).type;

            // Detect if this is an RPC message for special logging
            const isRpc = isRpcMessage(message as PopupToBackgroundMessage);
            const rpcMethod = isRpc ? (message as any).payload?.method : null;
            const rpcId = isRpc ? (message as any).payload?.id : null;

            try {
                if (isRpc) {
                    console.log(`🔄 [Background Router] Processing RPC ${rpcMethod} (ID: ${rpcId})...`);
                } else {
                    console.log(`🔄 [Background Router] Processing ${messageType} message...`);
                }

                // Route messages to appropriate handlers
                if (message.type === "fromOffscreen") {
                    console.log("📤 [Background] Routing to OffscreenMessageHandler");
                    if ('payload' in message) {
                        await offscreenMessageHandler.handleOffscreenMessage(message.payload as OffscreenToBackgroundMessage);
                        console.log("✅ [Background] OffscreenMessage handled successfully");
                        sendResponse({ success: true });
                    } else {
                        console.warn("❌ [Background] FromOffscreen message missing payload");
                        sendResponse({ success: false, error: "FromOffscreen message missing payload" });
                    }
                    return;
                }

                // Handle offscreen ready signal
                if (message.type === MESSAGE_TYPES.OFFSCREEN_READY) {
                    console.log("🎯 [Background] Handling OFFSCREEN_READY signal");
                    offscreenManager.handleOffscreenReady();

                    console.log("✅ [Background] OffscreenReady handled successfully");
                    sendResponse({ success: true });
                    return;
                }

                // Handle init requests
                if (message.type === "requestInit") {
                    console.log("🔧 [Background] Handling requestInit from offscreen");
                    const result = await offscreenManager.handleInitRequest();
                    console.log("✅ [Background] Init request completed:", result);
                    sendResponse(result);
                    return;
                }

                // Route to popup message handler for most messages
                console.log("📋 [Background] Routing to PopupMessageHandler");
                await popupMessageHandler.handlePopupMessage(message, sendResponse);

            } catch (error) {
                const duration = Date.now() - startTime;
                if (isRpc) {
                    console.error(`❌ [Background Router] RPC ${rpcMethod} (ID: ${rpcId}) failed after ${duration}ms:`, error);
                } else {
                    console.error(`❌ [Background Router] Error handling ${messageType} message after ${duration}ms:`, error);
                }
                sendResponse({ success: false, error: (error as Error).message });
            } finally {
                const duration = Date.now() - startTime;
                if (isRpc) {
                    console.log(`⏱️ [Background Router] 🔗 RPC ${rpcMethod} (ID: ${rpcId}) completed in ${duration}ms`);
                } else {
                    console.log(`⏱️ [Background Router] ${messageType} message processing completed in ${duration}ms`);
                }
            }
        })();

        return true;
    });
}

// ===================================================================
// SESSION RESTORATION
// ===================================================================

/**
 * Handle session restoration on offscreen ready
 */
// ===================================================================
// INITIALIZATION AND CLEANUP
// ===================================================================

/**
 * Initialize WebSocket connection
 */
async function initializeWebSocket(): Promise<void> {
    try {
        const WEBSOCKET_URL = "wss://auto-life.tech";

        // Generate device ID
        const deviceId = "mpc-2"; // TODO: Generate unique device ID
        stateManager.updateState({ deviceId });

        // Initialize WebSocket manager and connect
        await webSocketManager.initialize(WEBSOCKET_URL, deviceId);

        // Store WebSocket client reference for legacy compatibility
        wsClient = webSocketManager.getClient();

        console.log("🌐 [Background] WebSocket initialization complete");
    } catch (error) {
        console.error("❌ [Background] Failed to initialize WebSocket:", error);
        stateManager.updateWebSocketStatus(false, (error as Error).message);
    }
}

/**
 * Main background script entry point
 */
export default defineBackground(() => {
    console.log("🚀 [Background] Background script starting...");

    // Initialize all components
    initializeComponents();

    // Set up popup connections
    setupPopupConnections();

    // Set up message handlers
    setupMessageHandlers();

    // Initialize offscreen document on startup
    offscreenManager.createOffscreenDocument().then((result: any) => {
        console.log("🖥️ [Background] Initial offscreen document setup:", result);
    });

    // Initialize WebSocket connection
    initializeWebSocket();

    console.log("🎉 [Background] Background script initialized successfully");
});
