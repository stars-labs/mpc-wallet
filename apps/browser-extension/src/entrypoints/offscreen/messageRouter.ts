// ===================================================================
// MESSAGE ROUTER MODULE
// ===================================================================
//
// This module handles all message routing between the background script
// and the offscreen document. It provides a clean interface for sending
// and receiving messages with proper error handling and logging.
//
// Responsibilities:
// - Send messages to background script
// - Parse incoming messages from background
// - Route messages to appropriate handlers
// - Handle message format variations
// - Provide message validation and error handling
// ===================================================================

/**
 * Message structure for communication with background script
 */
export interface BackgroundMessage {
    type: string;
    payload: unknown;
}

/**
 * Response structure for message acknowledgments
 */
export interface MessageResponse {
    success: boolean;
    message?: string;
    error?: string;
    data?: any;
}

/**
 * Message handler function type
 */
export type MessageHandler = (messageType: string, payload: any) => Promise<MessageResponse> | MessageResponse;

/**
 * Parsed message structure with normalized format
 */
export interface ParsedMessage {
    type: string;
    payload: any;
}

/**
 * Message Router class for handling background-offscreen communication
 */
export class MessageRouter {
    private messageHandlers: Map<string, MessageHandler> = new Map();

    /**
     * Send a message to the background script
     */
    sendToBackground(message: BackgroundMessage): Promise<MessageResponse> {
        return new Promise((resolve) => {
//             console.log("📤 [MessageRouter] Sending to background:", message);

            chrome.runtime.sendMessage(message, (response) => {
                if (chrome.runtime.lastError) {
                    console.error("❌ [MessageRouter] Error sending to background:", chrome.runtime.lastError.message);
                    console.error("❌ [MessageRouter] Original message:", message);
                    resolve({
                        success: false,
                        error: chrome.runtime.lastError.message
                    });
                } else {
//                     console.log("✅ [MessageRouter] Background acknowledged:", response);
                    resolve(response || { success: true });
                }
            });
        });
    }

    /**
     * Parse incoming message and normalize its format
     */
    parseMessage(message: any): ParsedMessage | null {
//         console.log("🟡 [DEBUG] === MESSAGE PARSING ANALYSIS ===");
//         console.log("🟡 [DEBUG] Input message:", JSON.stringify(message, null, 2));
//         console.log("🟡 [DEBUG] message.payload:", message?.payload);
//         console.log("🟡 [DEBUG] message.payload.type:", message?.payload?.type);
//         console.log("🟡 [DEBUG] typeof message.payload.type:", typeof message?.payload?.type);
//         console.log("🟡 [DEBUG] message.type:", message?.type);
//         console.log("🟡 [DEBUG] typeof message.type:", typeof message?.type);

//         console.log("🔍 [MessageRouter] Parsing message:", message);

        let msgType: string | undefined;
        let actualPayload: any = {};

        // Handle wrapped message format: { payload: { type: "...", ...data } }
        if (message && message.payload && typeof message.payload.type === 'string') {
            msgType = message.payload.type;
            const { type, ...rest } = message.payload;
            actualPayload = rest;
//             console.log("🟡 [DEBUG] WRAPPED FORMAT - msgType:", msgType, "actualPayload:", actualPayload);
            console.log(`🔍 [MessageRouter] Wrapped message - Type: ${msgType}, Payload:`, actualPayload);
        }
        // Handle top-level message format: { type: "...", ...data }
        else if (message && typeof message.type === 'string') {
            msgType = message.type;
            const { type, ...rest } = message;
            actualPayload = rest;
//             console.log("🟡 [DEBUG] TOP-LEVEL FORMAT - msgType:", msgType, "actualPayload:", actualPayload);
            console.log(`🔍 [MessageRouter] Top-level message - Type: ${msgType}, Payload:`, actualPayload);
        }
        // Unknown message format
        else {
//             console.log("🟡 [DEBUG] UNKNOWN FORMAT");
            console.warn("⚠️ [MessageRouter] Unknown message structure:", message);
            return null;
        }

        const result = {
            type: msgType || 'unknown',
            payload: actualPayload
        };

//         console.log("🟡 [DEBUG] Final parsed result:", JSON.stringify(result, null, 2));
        return result;
    }

    /**
     * Register a handler for a specific message type
     */
    registerHandler(messageType: string, handler: MessageHandler): void {
        console.log(`📝 [MessageRouter] Registering handler for: ${messageType}`);
        this.messageHandlers.set(messageType, handler);
    }

    /**
     * Unregister a handler for a message type
     */
    unregisterHandler(messageType: string): void {
        console.log(`🗑️ [MessageRouter] Unregistering handler for: ${messageType}`);
        this.messageHandlers.delete(messageType);
    }

    /**
     * Process an incoming message by routing it to the appropriate handler
     */
    async processMessage(message: any, sendResponse: (response: MessageResponse) => void): Promise<void> {
//         console.log("🔵 [DEBUG] === MESSAGE PROCESSING ANALYSIS ===");
//         console.log("🔵 [DEBUG] Input to processMessage:", JSON.stringify(message, null, 2));
//         console.log("🔵 [DEBUG] Available handlers:", Array.from(this.messageHandlers.keys()));
//         console.log("🔵 [DEBUG] Handler count:", this.messageHandlers.size);

        const parsedMessage = this.parseMessage(message);

        if (!parsedMessage) {
//             console.log("🔵 [DEBUG] PARSE FAILED - parsedMessage is null");
            console.warn("❌ [MessageRouter] Could not parse message");
            sendResponse({
                success: false,
                error: "Malformed or untyped message"
            });
            return;
        }

//         console.log("🔵 [DEBUG] PARSE SUCCESS - parsedMessage:", JSON.stringify(parsedMessage, null, 2));

        const { type, payload } = parsedMessage;
//         console.log("🔵 [DEBUG] Looking for handler for type:", type);
//         console.log("🔵 [DEBUG] Handler exists?", this.messageHandlers.has(type));

        const handler = this.messageHandlers.get(type);

        if (!handler) {
//             console.log("🔵 [DEBUG] NO HANDLER FOUND");
//             console.log("🔵 [DEBUG] All available handlers:", Array.from(this.messageHandlers.keys()));
            console.warn(`❌ [MessageRouter] No handler registered for message type: ${type}`);
            sendResponse({
                success: false,
                error: `No handler for message type: ${type}`
            });
            return;
        }

//         console.log("🔵 [DEBUG] HANDLER FOUND - Calling handler...");
        try {
            console.log(`🎯 [MessageRouter] Processing ${type} with registered handler`);
            const response = await handler(type, payload);
//             console.log("🔵 [DEBUG] Handler response:", JSON.stringify(response, null, 2));
            sendResponse(response);
        } catch (error) {
//             console.log("🔵 [DEBUG] HANDLER ERROR:", error);
            console.error(`❌ [MessageRouter] Handler error for ${type}:`, error);
            sendResponse({
                success: false,
                error: error instanceof Error ? error.message : 'Unknown handler error'
            });
        }
    }

    /**
     * Set up Chrome runtime message listener
     */
    setupMessageListener(): void {
        console.log("🎧 [MessageRouter] Setting up Chrome runtime message listener");

        chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
//             console.log("📨 [MessageRouter] Message received from background:", message);
            this.processMessage(message, sendResponse);
            return true; // Indicates we will send a response asynchronously
        });
    }

    /**
     * Get all registered message types
     */
    getRegisteredTypes(): string[] {
        return Array.from(this.messageHandlers.keys());
    }

    /**
     * Clear all registered handlers
     */
    clearAllHandlers(): void {
        console.log("🧹 [MessageRouter] Clearing all message handlers");
        this.messageHandlers.clear();
    }
}

/**
 * Create and return a singleton message router instance
 */
let messageRouterInstance: MessageRouter | null = null;

export function getMessageRouter(): MessageRouter {
    if (!messageRouterInstance) {
        messageRouterInstance = new MessageRouter();
    }
    return messageRouterInstance;
}
