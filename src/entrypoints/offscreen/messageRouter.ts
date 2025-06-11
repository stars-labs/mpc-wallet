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
            console.log("📤 [MessageRouter] Sending to background:", message);

            chrome.runtime.sendMessage(message, (response) => {
                if (chrome.runtime.lastError) {
                    console.error("❌ [MessageRouter] Error sending to background:", chrome.runtime.lastError.message);
                    console.error("❌ [MessageRouter] Original message:", message);
                    resolve({
                        success: false,
                        error: chrome.runtime.lastError.message
                    });
                } else {
                    console.log("✅ [MessageRouter] Background acknowledged:", response);
                    resolve(response || { success: true });
                }
            });
        });
    }

    /**
     * Parse incoming message and normalize its format
     */
    parseMessage(message: any): ParsedMessage | null {
        console.log("🔍 [MessageRouter] Parsing message:", message);

        let msgType: string | undefined;
        let actualPayload: any = {};

        // Handle wrapped message format: { payload: { type: "...", ...data } }
        if (message && message.payload && typeof message.payload.type === 'string') {
            msgType = message.payload.type;
            const { type, ...rest } = message.payload;
            actualPayload = rest;
            console.log(`🔍 [MessageRouter] Wrapped message - Type: ${msgType}, Payload:`, actualPayload);
        }
        // Handle top-level message format: { type: "...", ...data }
        else if (message && typeof message.type === 'string') {
            msgType = message.type;
            const { type, ...rest } = message;
            actualPayload = rest;
            console.log(`🔍 [MessageRouter] Top-level message - Type: ${msgType}, Payload:`, actualPayload);
        }
        // Unknown message format
        else {
            console.warn("⚠️ [MessageRouter] Unknown message structure:", message);
            return null;
        }

        return {
            type: msgType || 'unknown',
            payload: actualPayload
        };
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
        const parsedMessage = this.parseMessage(message);

        if (!parsedMessage) {
            console.warn("❌ [MessageRouter] Could not parse message");
            sendResponse({
                success: false,
                error: "Malformed or untyped message"
            });
            return;
        }

        const { type, payload } = parsedMessage;
        const handler = this.messageHandlers.get(type);

        if (!handler) {
            console.warn(`❌ [MessageRouter] No handler registered for message type: ${type}`);
            sendResponse({
                success: false,
                error: `No handler for message type: ${type}`
            });
            return;
        }

        try {
            console.log(`🎯 [MessageRouter] Processing ${type} with registered handler`);
            const response = await handler(type, payload);
            sendResponse(response);
        } catch (error) {
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
            console.log("📨 [MessageRouter] Message received from background:", message);
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
