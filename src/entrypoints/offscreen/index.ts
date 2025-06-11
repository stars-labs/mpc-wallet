// ===================================================================
// OFFSCREEN ENTRY POINT - MODULAR ARCHITECTURE
// ===================================================================
//
// This is the main entry point for the offscreen document that handles
// WebRTC operations for the MPC wallet. It has been refactored to use
// a modular architecture for better maintainability and readability.
//
// Architecture Overview:
// - WasmInitializer: Handles FROST DKG WASM module initialization
// - MessageRouter: Routes messages between background and offscreen
// - WebRTCManager: Main coordinator for all WebRTC operations
//
// This modular approach makes it easier for junior developers to:
// 1. Understand individual components
// 2. Debug specific functionality
// 3. Add new features without touching the entire codebase
// 4. Test components in isolation
// ===================================================================

// Import modular components
import { WasmInitializer } from './wasmInitializer';
import { MessageRouter } from './messageRouter';
import { SimpleTest } from './testExport';

// Import types
import type { SessionInfo } from '../../types/session';
import type { MeshStatus } from '../../types/mesh';
import type { DkgState } from '../../types/dkg';
import type { WebRTCAppMessage } from '../../types/webrtc';
import { ServerMsg, ClientMsg, WebSocketMessagePayload, WebRTCSignal } from '../../types/websocket';

console.log("🚀 Offscreen script loaded with modular architecture");

// ===================================================================
// GLOBAL STATE AND INITIALIZATION
// ===================================================================

// Module instances
let wasmInitializer: WasmInitializer | null = null;
let messageRouter: MessageRouter | null = null;
let webRTCManager: any | null = null;

// Local state
let localdeviceId: string | null = null;
let webrtcConnections: Record<string, boolean> = {};

// ===================================================================
// INITIALIZATION FUNCTIONS
// ===================================================================

/**
 * Initialize all modules in the correct order
 */
async function initializeModules(): Promise<void> {
    try {
        console.log("🔧 [Init] Starting modular initialization...");

        // Step 1: Initialize WASM modules
        wasmInitializer = WasmInitializer.getInstance();
        const wasmSuccess = await wasmInitializer.initialize();

        if (!wasmSuccess) {
            throw new Error("Failed to initialize WASM modules");
        }

        console.log("✅ [Init] WASM modules initialized successfully");

        // Step 2: Initialize message router
        messageRouter = new MessageRouter();
        messageRouter.registerHandler('initWebRTC', handleInitWebRTC);
        messageRouter.registerHandler('webrtc_signal', handleWebRTCSignal);
        messageRouter.registerHandler('create_session', handleCreateSession);
        messageRouter.registerHandler('join_session', handleJoinSession);
        messageRouter.registerHandler('start_dkg', handleStartDkg);
        messageRouter.registerHandler('request_signing', handleRequestSigning);
        messageRouter.registerHandler('accept_signing', handleAcceptSigning);
        messageRouter.registerHandler('set_blockchain', handleSetBlockchain);
        messageRouter.registerHandler('get_addresses', handleGetAddresses);

        console.log("✅ [Init] Message router initialized successfully");

        console.log("🎉 [Init] All modules initialized successfully!");
    } catch (error) {
        console.error("❌ [Init] Failed to initialize modules:", error);
        throw error;
    }
}

/**
 * Initialize WebRTC Manager when requested
 */
async function initializeWebRTCManager(deviceId: string): Promise<void> {
    try {
        console.log(`🔧 [WebRTC] Initializing WebRTCManager for device: ${deviceId}`);

        if (!wasmInitializer?.isInitialized()) {
            throw new Error("WASM modules not initialized");
        }

        localdeviceId = deviceId;

        // Create WebRTC manager with callback for relaying messages
        webRTCManager = new SimpleTest();

        // Set up WebRTC manager callbacks
        if (webRTCManager) {
            // Configure all the callback handlers
            webRTCManager.onLog = (logMessage: string) => {
                console.log(`🔗 [WebRTC] ${logMessage}`);
                sendToBackground({
                    type: 'log',
                    payload: { message: logMessage, source: 'offscreen' }
                });
            };

            webRTCManager.onSessionUpdate = (sessionInfo: SessionInfo | null, invites: SessionInfo[]) => {
                console.log("🔗 [WebRTC] Session update:", sessionInfo, invites);
                sendToBackground({
                    type: 'session_update',
                    payload: { sessionInfo, invites }
                });
            };

            webRTCManager.onMeshStatusUpdate = (status: MeshStatus) => {
                console.log("🔗 [WebRTC] Mesh status update:", status);
                sendToBackground({
                    type: 'mesh_status_update',
                    payload: { status }
                });
            };

            webRTCManager.onDkgStateUpdate = (state: DkgState) => {
                console.log("🔗 [WebRTC] DKG state update:", state);
                sendToBackground({
                    type: 'dkg_state_update',
                    payload: { state }
                });
            };

            webRTCManager.onSigningStateUpdate = (state: any, info: any) => {
                console.log("🔗 [WebRTC] Signing state update:", state, info);
                sendToBackground({
                    type: 'signing_state_update',
                    payload: { state, info }
                });
            };

            webRTCManager.onWebRTCConnectionUpdate = (peerId: string, connected: boolean) => {
                console.log(`🔗 [WebRTC] Connection update: ${peerId} -> ${connected}`);
                webrtcConnections[peerId] = connected;
                sendToBackground({
                    type: 'webrtc_connection_update',
                    payload: { peerId, connected }
                });
            };

            console.log("✅ [WebRTC] WebRTCManager initialized successfully");
        }

    } catch (error) {
        console.error("❌ [WebRTC] Failed to initialize WebRTCManager:", error);
        throw error;
    }
}

// ===================================================================
// MESSAGE HANDLING FUNCTIONS
// ===================================================================

/**
 * Handle WebRTC initialization request
 */
async function handleInitWebRTC(messageType: string, payload: any): Promise<any> {
    try {
        console.log("🔧 [Handler] Handling initWebRTC request:", payload);

        if (!payload.localdeviceId) {
            throw new Error("localdeviceId is required for WebRTC initialization");
        }

        await initializeWebRTCManager(payload.localdeviceId);

        return {
            success: true,
            message: "WebRTC initialized successfully",
            data: { deviceId: payload.localdeviceId }
        };
    } catch (error) {
        console.error("❌ [Handler] Error initializing WebRTC:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

/**
 * Handle WebRTC signaling messages
 */
async function handleWebRTCSignal(messageType: string, payload: any): Promise<any> {
    try {
        if (!webRTCManager) {
            throw new Error("WebRTC manager not initialized");
        }

        console.log("🔧 [Handler] Handling WebRTC signal:", payload);
        await webRTCManager.handleWebRTCSignal(payload.fromPeerId, payload.signal);

        return { success: true };
    } catch (error) {
        console.error("❌ [Handler] Error handling WebRTC signal:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

/**
 * Handle session creation request
 */
async function handleCreateSession(messageType: string, payload: any): Promise<any> {
    try {
        if (!webRTCManager) {
            throw new Error("WebRTC manager not initialized");
        }

        console.log("🔧 [Handler] Creating session:", payload);
        await webRTCManager.createSession(payload.sessionId, payload.threshold);

        return { success: true };
    } catch (error) {
        console.error("❌ [Handler] Error creating session:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

/**
 * Handle join session request
 */
async function handleJoinSession(messageType: string, payload: any): Promise<any> {
    try {
        if (!webRTCManager) {
            throw new Error("WebRTC manager not initialized");
        }

        console.log("🔧 [Handler] Joining session:", payload);
        await webRTCManager.joinSession(payload.sessionId);

        return { success: true };
    } catch (error) {
        console.error("❌ [Handler] Error joining session:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

/**
 * Handle DKG start request
 */
async function handleStartDkg(messageType: string, payload: any): Promise<any> {
    try {
        if (!webRTCManager) {
            throw new Error("WebRTC manager not initialized");
        }

        console.log("🔧 [Handler] Starting DKG");
        await webRTCManager.startDkg();

        return { success: true };
    } catch (error) {
        console.error("❌ [Handler] Error starting DKG:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

/**
 * Handle signing request
 */
async function handleRequestSigning(messageType: string, payload: any): Promise<any> {
    try {
        if (!webRTCManager) {
            throw new Error("WebRTC manager not initialized");
        }

        console.log("🔧 [Handler] Requesting signing:", payload);
        await webRTCManager.requestSigning(payload.transactionData);

        return { success: true };
    } catch (error) {
        console.error("❌ [Handler] Error requesting signing:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

/**
 * Handle accept signing request
 */
async function handleAcceptSigning(messageType: string, payload: any): Promise<any> {
    try {
        if (!webRTCManager) {
            throw new Error("WebRTC manager not initialized");
        }

        console.log("🔧 [Handler] Accepting signing:", payload);
        await webRTCManager.acceptSigning(payload.signingId);

        return { success: true };
    } catch (error) {
        console.error("❌ [Handler] Error accepting signing:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

/**
 * Handle blockchain setting
 */
async function handleSetBlockchain(messageType: string, payload: any): Promise<any> {
    try {
        if (!webRTCManager) {
            throw new Error("WebRTC manager not initialized");
        }

        console.log("🔧 [Handler] Setting blockchain:", payload);
        webRTCManager.setBlockchain(payload.blockchain);

        return { success: true };
    } catch (error) {
        console.error("❌ [Handler] Error setting blockchain:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

/**
 * Handle get addresses request
 */
async function handleGetAddresses(messageType: string, payload: any): Promise<any> {
    try {
        if (!webRTCManager) {
            throw new Error("WebRTC manager not initialized");
        }

        console.log("🔧 [Handler] Getting addresses");
        const addresses = webRTCManager.getAddresses();

        return {
            success: true,
            data: addresses
        };
    } catch (error) {
        console.error("❌ [Handler] Error getting addresses:", error);
        return {
            success: false,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

// ===================================================================
// COMMUNICATION FUNCTIONS
// ===================================================================

/**
 * Send messages to the background script
 */
function sendToBackground(message: { type: string; payload: unknown }): void {
    console.log("📤 [Comm] Sending message to background:", message);
    chrome.runtime.sendMessage(message, (response) => {
        if (chrome.runtime.lastError) {
            console.error("❌ [Comm] Error sending message to background:", chrome.runtime.lastError.message);
        } else {
            console.log("✅ [Comm] Message acknowledged:", response);
        }
    });
}

/**
 * Listen for messages from the background script
 */
chrome.runtime.onMessage.addListener((message: { type?: string; payload?: any }, sender, sendResponse) => {
    console.log("📥 [Comm] Message received from background:", message);

    // Use the message router to process all incoming messages
    if (messageRouter) {
        messageRouter.processMessage(message, (response) => {
            console.log("✅ [Comm] Message processed successfully:", response);
            sendResponse(response);
        });
    } else {
        console.error("❌ [Comm] Message router not initialized");
        sendResponse({
            success: false,
            error: "Message router not initialized"
        });
    }

    // Return true to indicate we will send a response asynchronously
    return true;
});

// ===================================================================
// STARTUP SEQUENCE
// ===================================================================

/**
 * Main startup function
 */
async function startup(): Promise<void> {
    try {
        console.log("🚀 [Startup] Beginning offscreen initialization...");

        // Initialize all modules
        await initializeModules();

        // Send ready signal to background
        sendToBackground({
            type: 'offscreenReady',
            payload: {
                timestamp: Date.now(),
                wasmInitialized: wasmInitializer?.isInitialized() || false,
                messageRouterReady: !!messageRouter
            }
        });

        console.log("🎉 [Startup] Offscreen script fully initialized and ready!");
    } catch (error) {
        console.error("💥 [Startup] Failed to initialize offscreen script:", error);

        // Send error signal to background
        sendToBackground({
            type: 'offscreen_error',
            payload: {
                error: error instanceof Error ? error.message : String(error),
                timestamp: Date.now()
            }
        });
    }
}

// Start the initialization process
startup();
