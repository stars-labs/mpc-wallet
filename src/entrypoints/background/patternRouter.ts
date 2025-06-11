/**
 * Pattern-Based Message Router
 * 
 * This module provides a type-safe, exhaustive message routing system
 * using pattern matching instead of traditional switch statements.
 */

import { match, P } from 'ts-pattern';
import type { PopupToBackgroundMessage } from '../../types/messages';
import {
    categorizeMessage,
    getCategoryInfo,
    requiresAsyncHandling
} from '../../types/messagePatterns';

export interface MessageHandlerDependencies {
    stateManager: any; // Replace with actual StateManager type
    // Add other dependencies as needed
}

export interface MessageHandlerResult {
    success: boolean;
    data?: any;
    error?: string;
    category: string;
    processingTime: number;
}

export class PatternBasedMessageRouter {
    constructor(private dependencies: MessageHandlerDependencies) { }

    /**
     * Routes and handles messages using pattern matching
     */
    async routeMessage(
        message: PopupToBackgroundMessage,
        sendResponse: (response: any) => void
    ): Promise<MessageHandlerResult> {
        const startTime = Date.now();

        // Basic validation - ensure message has a type
        if (!message || typeof message.type !== 'string') {
            const error = 'Invalid message structure';
            console.error(`🚨 [MessageRouter] ${error}:`, message);
            sendResponse({ error });
            return {
                success: false,
                error,
                category: 'validation',
                processingTime: Date.now() - startTime
            };
        }

        // Categorize and log message
        const category = categorizeMessage(message);
        const categoryInfo = getCategoryInfo(category);
        this.logMessage(message, categoryInfo);

        try {
            // Route message based on pattern matching
            const result = await match(message)
                // State Management Messages
                .with({ type: 'GET_STATE' }, () => this.handleGetState(sendResponse))
                .with({ type: 'GET_WEBRTC_STATE' }, () => this.handleGetWebRTCState(sendResponse))

                // Session Management Messages
                .with({ type: 'CREATE_SESSION' }, (msg) => this.handleCreateSession(msg, sendResponse))
                .with({ type: 'JOIN_SESSION' }, (msg) => this.handleJoinSession(msg, sendResponse))
                .with({ type: 'LEAVE_SESSION' }, (msg) => this.handleLeaveSession(msg, sendResponse))

                // WebRTC Control Messages
                .with({ type: P.string.startsWith('WEBRTC_') }, (msg) => this.handleWebRTCControl(msg, sendResponse))

                // Offscreen Control Messages
                .with({ type: P.string.startsWith('OFFSCREEN_') }, (msg) => this.handleOffscreenControl(msg, sendResponse))

                // Address Management Messages
                .with({ type: 'GET_DKG_ADDRESS' }, (msg) => this.handleGetDKGAddress(msg, sendResponse))
                .with({ type: P.string.includes('ADDRESS') }, (msg) => this.handleAddressManagement(msg, sendResponse))

                // Network Management Messages
                .with({ type: 'setBlockchain' }, (msg) => this.handleSetBlockchain(msg, sendResponse))
                .with({ type: P.string.includes('NETWORK') }, (msg) => this.handleNetworkManagement(msg, sendResponse))

                // RPC Request Messages
                .with({ type: P.string.startsWith('eth_') }, (msg) => this.handleRPCRequest(msg, sendResponse))
                .with({ type: P.string.includes('rpc') }, (msg) => this.handleRPCRequest(msg, sendResponse))

                // Relay Messages
                .with({ type: 'RELAY' }, (msg) => this.handleRelay(msg, sendResponse))

                // UI Request Messages
                .with({ type: 'LIST_DEVICES' }, () => this.handleListDevices(sendResponse))

                // Catch-all for unknown messages
                .otherwise((msg) => this.handleUnknownMessage(msg, sendResponse));

            return {
                success: true,
                data: result,
                category: categoryInfo.name,
                processingTime: Date.now() - startTime
            };

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            console.error(`🚨 [MessageRouter] Error handling ${message.type}:`, error);

            sendResponse({
                error: errorMessage,
                type: 'HANDLER_ERROR',
                originalMessageType: message.type
            });

            return {
                success: false,
                error: errorMessage,
                category: categoryInfo.name,
                processingTime: Date.now() - startTime
            };
        }
    }

    /**
     * Enhanced logging with pattern-based categorization
     */
    private logMessage(message: PopupToBackgroundMessage, categoryInfo: { name: string; icon: string; color: string }) {
        console.log("┌─────────────────────────────────────────────────────────────────");
        console.log(`│ ${categoryInfo.color}[MessageRouter] ${categoryInfo.icon} Processing: ${message.type}\x1b[0m`);
        console.log(`│ Category: ${categoryInfo.icon} ${categoryInfo.name}`);
        console.log(`│ Async Required: ${requiresAsyncHandling(message) ? '✓' : '✗'}`);
        console.log(`│ Data:`, message);
        console.log("└─────────────────────────────────────────────────────────────────");
    }

    // State Management Handlers
    private async handleGetState(sendResponse: (response: any) => void) {
        console.log("📊 [MessageRouter] GET_STATE: Returning current application state");
        const state = this.dependencies.stateManager.getState();
        console.log("📊 [MessageRouter] State keys:", Object.keys(state));
        sendResponse(state);
        return { type: 'STATE_RESPONSE', state };
    }

    private async handleGetWebRTCState(sendResponse: (response: any) => void) {
        console.log("📡 [MessageRouter] GET_WEBRTC_STATE: Returning WebRTC connections");
        const webrtcConnections = this.dependencies.stateManager.getWebRTCConnections();
        console.log("📡 [MessageRouter] WebRTC connections:", webrtcConnections);
        sendResponse({ webrtcConnections });
        return { type: 'WEBRTC_STATE_RESPONSE', webrtcConnections };
    }

    // Session Management Handlers
    private async handleCreateSession(message: any, sendResponse: (response: any) => void) {
        console.log("🔐 [MessageRouter] CREATE_SESSION: Creating new session");
        // TODO: Implement session creation logic
        sendResponse({ success: true, sessionId: 'new-session-id' });
        return { type: 'SESSION_CREATED' };
    }

    private async handleJoinSession(message: any, sendResponse: (response: any) => void) {
        console.log("🔐 [MessageRouter] JOIN_SESSION: Joining existing session");
        // TODO: Implement session joining logic
        sendResponse({ success: true });
        return { type: 'SESSION_JOINED' };
    }

    private async handleLeaveSession(message: any, sendResponse: (response: any) => void) {
        console.log("🔐 [MessageRouter] LEAVE_SESSION: Leaving current session");
        // TODO: Implement session leaving logic
        sendResponse({ success: true });
        return { type: 'SESSION_LEFT' };
    }

    // WebRTC Control Handlers
    private async handleWebRTCControl(message: any, sendResponse: (response: any) => void) {
        console.log("📡 [MessageRouter] WEBRTC_CONTROL: Handling WebRTC control message");
        // TODO: Implement WebRTC control logic
        sendResponse({ success: true });
        return { type: 'WEBRTC_CONTROL_RESPONSE' };
    }

    // Offscreen Control Handlers
    private async handleOffscreenControl(message: any, sendResponse: (response: any) => void) {
        console.log("📄 [MessageRouter] OFFSCREEN_CONTROL: Handling offscreen control message");
        // TODO: Implement offscreen control logic
        sendResponse({ success: true });
        return { type: 'OFFSCREEN_CONTROL_RESPONSE' };
    }

    // Address Management Handlers
    private async handleGetDKGAddress(message: any, sendResponse: (response: any) => void) {
        console.log("🏠 [MessageRouter] GET_DKG_ADDRESS: Retrieving DKG address");
        // TODO: Implement DKG address retrieval logic
        sendResponse({ address: 'mock-dkg-address' });
        return { type: 'DKG_ADDRESS_RESPONSE' };
    }

    private async handleAddressManagement(message: any, sendResponse: (response: any) => void) {
        console.log("🏠 [MessageRouter] ADDRESS_MANAGEMENT: Handling address management");
        // TODO: Implement address management logic
        sendResponse({ success: true });
        return { type: 'ADDRESS_MANAGEMENT_RESPONSE' };
    }

    // Network Management Handlers
    private async handleSetBlockchain(message: any, sendResponse: (response: any) => void) {
        console.log("🌐 [MessageRouter] SET_BLOCKCHAIN: Setting blockchain");
        // TODO: Implement blockchain setting logic
        sendResponse({ success: true });
        return { type: 'BLOCKCHAIN_SET' };
    }

    private async handleNetworkManagement(message: any, sendResponse: (response: any) => void) {
        console.log("🌐 [MessageRouter] NETWORK_MANAGEMENT: Handling network management");
        // TODO: Implement network management logic
        sendResponse({ success: true });
        return { type: 'NETWORK_MANAGEMENT_RESPONSE' };
    }

    // RPC Request Handlers
    private async handleRPCRequest(message: any, sendResponse: (response: any) => void) {
        console.log("⚡ [MessageRouter] RPC_REQUEST: Handling RPC request");
        // TODO: Implement RPC request logic
        sendResponse({ result: 'mock-rpc-result' });
        return { type: 'RPC_RESPONSE' };
    }

    // Relay Handlers
    private async handleRelay(message: any, sendResponse: (response: any) => void) {
        console.log("🔄 [MessageRouter] RELAY: Forwarding message via WebSocket");
        // TODO: Implement relay logic
        sendResponse({ success: true });
        return { type: 'RELAY_RESPONSE' };
    }

    // UI Request Handlers
    private async handleListDevices(sendResponse: (response: any) => void) {
        console.log("📋 [MessageRouter] LIST_DEVICES: Requesting peer discovery");
        // TODO: Implement device listing logic
        sendResponse({ devices: [] });
        return { type: 'DEVICES_LIST_RESPONSE' };
    }

    // Unknown Message Handler
    private async handleUnknownMessage(message: any, sendResponse: (response: any) => void) {
        console.warn(`❓ [MessageRouter] UNKNOWN_MESSAGE: ${message.type}`);
        sendResponse({
            error: `Unknown message type: ${message.type}`,
            type: 'UNKNOWN_MESSAGE_ERROR'
        });
        return { type: 'UNKNOWN_MESSAGE_RESPONSE', originalType: message.type };
    }
}
