// ===================================================================
// MESSAGE ROUTER MODULE (REFACTORED WITH LOGGER)
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

import { logger } from '../../utils/logger';

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
    private readonly logContext = { component: 'MessageRouter' };

    /**
     * Send a message to the background script
     */
    sendToBackground(message: BackgroundMessage): Promise<MessageResponse> {
        return new Promise((resolve) => {
            logger.debug('Sending to background', { ...this.logContext, method: 'sendToBackground' }, message);

            chrome.runtime.sendMessage(message, (response) => {
                if (chrome.runtime.lastError) {
                    logger.error('Error sending to background', 
                        { ...this.logContext, method: 'sendToBackground', error: chrome.runtime.lastError.message }, 
                        message
                    );
                    resolve({
                        success: false,
                        error: chrome.runtime.lastError.message
                    });
                } else {
                    logger.debug('Background acknowledged', { ...this.logContext, method: 'sendToBackground' }, response);
                    resolve(response || { success: true });
                }
            });
        });
    }

    /**
     * Parse incoming message and normalize its format
     */
    parseMessage(message: any): ParsedMessage | null {
        logger.debug('Parsing message', { ...this.logContext, method: 'parseMessage' }, message);

        let msgType: string | undefined;
        let actualPayload: any = {};

        // Handle wrapped message format: { payload: { type: "...", ...data } }
        if (message && message.payload && typeof message.payload.type === 'string') {
            msgType = message.payload.type;
            const { type, ...rest } = message.payload;
            actualPayload = rest;
            logger.debug('Wrapped message format detected', 
                { ...this.logContext, method: 'parseMessage', msgType }, 
                actualPayload
            );
        }
        // Handle top-level message format: { type: "...", ...data }
        else if (message && typeof message.type === 'string') {
            msgType = message.type;
            const { type, ...rest } = message;
            actualPayload = rest;
            logger.debug('Top-level message format detected', 
                { ...this.logContext, method: 'parseMessage', msgType }, 
                actualPayload
            );
        }
        // Unknown format
        else {
            logger.warn('Unknown message structure', { ...this.logContext, method: 'parseMessage' }, message);
            return null;
        }

        const result = {
            type: msgType,
            payload: actualPayload
        };

        logger.debug('Message parsed successfully', { ...this.logContext, method: 'parseMessage' }, result);
        return result;
    }

    /**
     * Register a handler for a specific message type
     */
    registerHandler(messageType: string, handler: MessageHandler): void {
        logger.debug('Registering handler', 
            { ...this.logContext, method: 'registerHandler', messageType }
        );
        this.messageHandlers.set(messageType, handler);
    }

    /**
     * Handle an incoming message from the background script
     */
    async handleMessage(message: any): Promise<MessageResponse> {
        const parsed = this.parseMessage(message);
        if (!parsed) {
            return {
                success: false,
                error: "Invalid message format"
            };
        }

        const handler = this.messageHandlers.get(parsed.type);
        if (!handler) {
            logger.warn('No handler registered for message type', 
                { ...this.logContext, method: 'handleMessage', messageType: parsed.type }
            );
            return {
                success: false,
                error: `No handler for message type: ${parsed.type}`
            };
        }

        try {
            return await handler(parsed.type, parsed.payload);
        } catch (error) {
            logger.error('Handler error', 
                { ...this.logContext, method: 'handleMessage', messageType: parsed.type }, 
                error
            );
            return {
                success: false,
                error: error instanceof Error ? error.message : "Handler error"
            };
        }
    }
}

// Create singleton instance
export const messageRouter = new MessageRouter();