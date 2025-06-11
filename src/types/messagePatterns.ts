/**
 * Message Pattern Matching System
 * 
 * This module provides type-safe pattern matching for message handling
 * across the MPC wallet extension. It uses ts-pattern to create
 * exhaustive and maintainable message routing.
 */

import { match, P } from 'ts-pattern';
import type { PopupToBackgroundMessage } from './messages';

// Message category patterns for enhanced logging and routing
export const MessageCategories = {
    STATE_MANAGEMENT: 'state_management',
    SESSION_MANAGEMENT: 'session_management',
    WEBRTC_CONTROL: 'webrtc_control',
    OFFSCREEN_CONTROL: 'offscreen_control',
    ADDRESS_MANAGEMENT: 'address_management',
    NETWORK_MANAGEMENT: 'network_management',
    RPC_REQUEST: 'rpc_request',
    UI_REQUEST: 'ui_request',
    RELAY: 'relay',
    UNKNOWN: 'unknown'
} as const;

export type MessageCategory = typeof MessageCategories[keyof typeof MessageCategories];

// Pattern for detecting message categories
export const categorizeMessage = (message: PopupToBackgroundMessage): MessageCategory => {
    return match(message)
        .with({ type: P.string.includes('getState') }, () => MessageCategories.STATE_MANAGEMENT)
        .with({ type: P.string.includes('setState') }, () => MessageCategories.STATE_MANAGEMENT)
        .with({ type: P.string.includes('GET_STATE') }, () => MessageCategories.STATE_MANAGEMENT)
        .with({ type: P.string.includes('GET_WEBRTC_STATE') }, () => MessageCategories.STATE_MANAGEMENT)

        .with({ type: P.string.includes('session') }, () => MessageCategories.SESSION_MANAGEMENT)
        .with({ type: P.string.includes('Session') }, () => MessageCategories.SESSION_MANAGEMENT)
        .with({ type: P.string.includes('CREATE_SESSION') }, () => MessageCategories.SESSION_MANAGEMENT)
        .with({ type: P.string.includes('JOIN_SESSION') }, () => MessageCategories.SESSION_MANAGEMENT)
        .with({ type: P.string.includes('LEAVE_SESSION') }, () => MessageCategories.SESSION_MANAGEMENT)

        .with({ type: P.string.includes('webrtc') }, () => MessageCategories.WEBRTC_CONTROL)
        .with({ type: P.string.includes('WebRTC') }, () => MessageCategories.WEBRTC_CONTROL)
        .with({ type: P.string.includes('WEBRTC') }, () => MessageCategories.WEBRTC_CONTROL)

        .with({ type: P.string.includes('offscreen') }, () => MessageCategories.OFFSCREEN_CONTROL)
        .with({ type: P.string.includes('Offscreen') }, () => MessageCategories.OFFSCREEN_CONTROL)
        .with({ type: P.string.includes('OFFSCREEN') }, () => MessageCategories.OFFSCREEN_CONTROL)

        .with({ type: P.string.includes('address') }, () => MessageCategories.ADDRESS_MANAGEMENT)
        .with({ type: P.string.includes('Address') }, () => MessageCategories.ADDRESS_MANAGEMENT)
        .with({ type: P.string.includes('ADDRESS') }, () => MessageCategories.ADDRESS_MANAGEMENT)
        .with({ type: P.string.includes('GET_DKG_ADDRESS') }, () => MessageCategories.ADDRESS_MANAGEMENT)

        .with({ type: P.string.includes('network') }, () => MessageCategories.NETWORK_MANAGEMENT)
        .with({ type: P.string.includes('Network') }, () => MessageCategories.NETWORK_MANAGEMENT)
        .with({ type: P.string.includes('setBlockchain') }, () => MessageCategories.NETWORK_MANAGEMENT)

        .with({ type: P.string.includes('rpc') }, () => MessageCategories.RPC_REQUEST)
        .with({ type: P.string.includes('RPC') }, () => MessageCategories.RPC_REQUEST)
        .with({ type: P.string.includes('eth_') }, () => MessageCategories.RPC_REQUEST)

        .with({ type: P.string.includes('RELAY') }, () => MessageCategories.RELAY)
        .with({ type: P.string.includes('relay') }, () => MessageCategories.RELAY)

        .with({ type: P.string.includes('LIST_DEVICES') }, () => MessageCategories.UI_REQUEST)

        .otherwise(() => MessageCategories.UNKNOWN);
};

// Pattern for getting category display info
export interface CategoryInfo {
    name: string;
    icon: string;
    color: string;
}

export const getCategoryInfo = (category: MessageCategory): CategoryInfo => {
    return match(category)
        .with(MessageCategories.STATE_MANAGEMENT, () => ({
            name: 'State Management',
            icon: '📊',
            color: '\x1b[36m' // cyan
        }))
        .with(MessageCategories.SESSION_MANAGEMENT, () => ({
            name: 'Session Management',
            icon: '🔐',
            color: '\x1b[35m' // magenta
        }))
        .with(MessageCategories.WEBRTC_CONTROL, () => ({
            name: 'WebRTC Control',
            icon: '📡',
            color: '\x1b[34m' // blue
        }))
        .with(MessageCategories.OFFSCREEN_CONTROL, () => ({
            name: 'Offscreen Control',
            icon: '📄',
            color: '\x1b[33m' // yellow
        }))
        .with(MessageCategories.ADDRESS_MANAGEMENT, () => ({
            name: 'Address Management',
            icon: '🏠',
            color: '\x1b[32m' // green
        }))
        .with(MessageCategories.NETWORK_MANAGEMENT, () => ({
            name: 'Network Management',
            icon: '🌐',
            color: '\x1b[31m' // red
        }))
        .with(MessageCategories.RPC_REQUEST, () => ({
            name: 'RPC Request',
            icon: '⚡',
            color: '\x1b[93m' // bright yellow
        }))
        .with(MessageCategories.UI_REQUEST, () => ({
            name: 'UI Request',
            icon: '🖼️',
            color: '\x1b[96m' // bright cyan
        }))
        .with(MessageCategories.RELAY, () => ({
            name: 'Message Relay',
            icon: '🔄',
            color: '\x1b[94m' // bright blue
        }))
        .with(MessageCategories.UNKNOWN, () => ({
            name: 'Unknown',
            icon: '❓',
            color: '\x1b[90m' // gray
        }))
        .exhaustive();
};

// Pattern for determining if a message requires async handling
export const requiresAsyncHandling = (message: PopupToBackgroundMessage): boolean => {
    return match(message)
        .with({ type: P.string.includes('LIST_DEVICES') }, () => true)
        .with({ type: P.string.includes('RELAY') }, () => true)
        .with({ type: P.string.includes('CREATE_SESSION') }, () => true)
        .with({ type: P.string.includes('JOIN_SESSION') }, () => true)
        .with({ type: P.string.includes('LEAVE_SESSION') }, () => true)
        .with({ type: P.string.includes('rpc') }, () => true)
        .with({ type: P.string.includes('RPC') }, () => true)
        .with({ type: P.string.includes('eth_') }, () => true)
        .with({ type: P.string.includes('GET_DKG_ADDRESS') }, () => true)
        .otherwise(() => false);
};

// Pattern for validating message structure
export const validateMessageStructure = (message: any): message is PopupToBackgroundMessage => {
    return match(message)
        .with({ type: P.string }, () => true)
        .otherwise(() => false);
};
