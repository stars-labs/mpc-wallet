// ===================================================================
// STATE MANAGEMENT MODULE
// ===================================================================
//
// This module manages the central application state and provides
// utilities for state synchronization across different components.
// It handles:
// - Application state management
// - Popup port communication
// - State broadcasting and updates
// - Cross-component state consistency
// ===================================================================

import { AppState, INITIAL_APP_STATE } from "../../types/appstate";
import { MeshStatusType } from "../../types/mesh";
import { DkgState } from "../../types/dkg";
import type {
    BackgroundToPopupMessage,
    InitialStateMessage,
    OffscreenToBackgroundMessage
} from "../../types/messages";

/**
 * Manages central application state and popup communication
 */
export class StateManager {
    private appState: AppState;
    private popupPorts = new Set<chrome.runtime.Port>();
    private static readonly STATE_STORAGE_KEY = 'mpc_wallet_background_state';
    private isStateLoaded = false;
    private pendingPopupPorts: chrome.runtime.Port[] = [];

    constructor(initialState?: Partial<AppState>) {
        this.appState = {
            ...INITIAL_APP_STATE,
            ...initialState
        };
        // Load persisted state asynchronously
        this.loadPersistedState();
    }

    /**
     * Load persisted state asynchronously from Chrome storage
     */
    private async loadPersistedState(): Promise<void> {
        try {
            const result = await chrome.storage.local.get(StateManager.STATE_STORAGE_KEY);
            if (result[StateManager.STATE_STORAGE_KEY]) {
                const persistedState = result[StateManager.STATE_STORAGE_KEY];
                console.log("[StateManager] Loading persisted state:", persistedState);
                
                // Merge persisted state with current state, preserving important runtime values
                this.appState = {
                    ...this.appState,
                    ...persistedState,
                    // Reset transient connection states that shouldn't persist
                    wsConnected: false,
                    meshStatus: { type: MeshStatusType.Incomplete },
                    webrtcConnections: {},
                };
                console.log("[StateManager] State restored from persistence");
            } else {
                console.log("[StateManager] No persisted state found, using initial state");
            }
        } catch (error) {
            console.warn("[StateManager] Failed to load persisted state:", error);
        } finally {
            // Mark state as loaded and process any pending popup connections
            this.isStateLoaded = true;
            this.processPendingPopupPorts();
        }
    }

    /**
     * Process any popup ports that connected before state was loaded
     */
    private processPendingPopupPorts(): void {
        console.log(`[StateManager] Processing ${this.pendingPopupPorts.length} pending popup ports`);
        this.pendingPopupPorts.forEach(port => {
            this.addPopupPortInternal(port);
        });
        this.pendingPopupPorts = [];
    }

    /**
     * Persist state to Chrome storage
     */
    private async persistState(): Promise<void> {
        try {
            // Only persist important state, not transient connection states
            const stateToPersist = {
                deviceId: this.appState.deviceId,
                chain: this.appState.chain,
                curve: this.appState.curve,
                dkgAddress: this.appState.dkgAddress,
                dkgState: this.appState.dkgState,
                sessionInfo: this.appState.sessionInfo,
                threshold: this.appState.threshold,
                totalParticipants: this.appState.totalParticipants,
                // Don't persist: wsConnected, meshStatus, webrtcConnections, connecteddevices
            };

            await chrome.storage.local.set({
                [StateManager.STATE_STORAGE_KEY]: stateToPersist
            });
            console.log("[StateManager] State persisted to storage");
        } catch (error) {
            console.warn("[StateManager] Failed to persist state:", error);
        }
    }

    /**
     * Get current application state
     */
    getState(): AppState {
        return { ...this.appState };
    }

    /**
     * Update application state
     */
    updateState(updates: Partial<AppState>): void {
        console.log("[StateManager] Updating state:", updates);
        this.appState = {
            ...this.appState,
            ...updates
        };
        // Persist state changes
        this.persistState();
    }

    /**
     * Update specific state properties with deep merge support
     */
    updateStateProperty<K extends keyof AppState>(key: K, value: AppState[K]): void {
        console.log(`[StateManager] Updating state property ${String(key)}:`, value);
        this.appState[key] = value;
        // Persist state changes
        this.persistState();
    }

    /**
     * Update WebSocket connection status and persist it
     */
    updateWebSocketStatus(connected: boolean, error?: string): void {
        console.log(`[StateManager] Updating WebSocket status: connected=${connected}, error=${error || 'none'}`);
        this.appState.wsConnected = connected;
        if (error) {
            this.appState.wsError = error;
        } else {
            this.appState.wsError = "";
        }
        
        // Broadcast status update
        this.broadcastToPopupPorts({ type: "wsStatus", connected });
        
        // Persist the WebSocket status update
        this.persistState();
    }

    /**
     * Update connected devices list and persist it
     */
    updateConnectedDevices(devices: string[]): void {
        console.log(`[StateManager] Updating connected devices:`, devices);
        // Exclude current device from connected devices list
        this.appState.connecteddevices = devices.filter(deviceId => deviceId !== this.appState.deviceId);
        
        console.log(`[StateManager] Updated connected devices:`, this.appState.connecteddevices);

        // Broadcast device list update
        this.broadcastToPopupPorts({
            type: "deviceList",
            devices: this.appState.connecteddevices
        });

        // Persist the devices update
        this.persistState();
    }

    /**
     * Add a popup port connection
     */
    addPopupPort(port: chrome.runtime.Port): void {
        console.log("[StateManager] Adding popup port, state loaded:", this.isStateLoaded);
        
        if (!this.isStateLoaded) {
            // State not loaded yet, queue the port for later
            console.log("[StateManager] State not loaded yet, queuing popup port");
            this.pendingPopupPorts.push(port);
            return;
        }

        this.addPopupPortInternal(port);
    }

    /**
     * Internal method to add popup port once state is loaded
     */
    private addPopupPortInternal(port: chrome.runtime.Port): void {
        console.log("[StateManager] Adding popup port (internal)");
        this.popupPorts.add(port);

        // Send current state to newly connected popup
        const initialStateMessage: InitialStateMessage = {
            type: "initialState",
            ...this.appState
        };
        console.log("[StateManager] Sending current state to popup:", {
            deviceId: this.appState.deviceId,
            wsConnected: this.appState.wsConnected,
            connecteddevices: this.appState.connecteddevices?.length || 0,
            sessionInfo: !!this.appState.sessionInfo,
            dkgState: this.appState.dkgState,
            dkgAddress: this.appState.dkgAddress
        });
        port.postMessage(initialStateMessage);

        port.onDisconnect.addListener(() => {
            console.log("[StateManager] Popup disconnected");
            this.popupPorts.delete(port);
        });
    }

    /**
     * Broadcast message to all connected popup ports
     */
    broadcastToPopupPorts(message: BackgroundToPopupMessage): void {
        console.log("[StateManager] Broadcasting to", this.popupPorts.size, "popup ports:", message);
        this.popupPorts.forEach(port => {
            try {
                port.postMessage(message);
                console.log("[StateManager] Successfully sent message to popup port");
            } catch (error) {
                console.error("[StateManager] Error sending message to popup port:", error);
                this.popupPorts.delete(port);
            }
        });
    }

    /**
     * Broadcast current state to all popup ports
     */
    broadcastCurrentState(): void {
        const stateMessage: InitialStateMessage = {
            type: "initialState",
            ...this.appState
        };
        this.broadcastToPopupPorts(stateMessage as any);
    }

    /**
     * Handle state updates from offscreen document
     */
    handleOffscreenStateUpdate(payload: OffscreenToBackgroundMessage): void {
        console.log("[StateManager] Handling offscreen state update:", payload);

        switch (payload.type) {
            case "webrtcConnectionUpdate":
                if ('deviceId' in payload && 'connected' in payload) {
                    console.log("[StateManager] Received WebRTC connection update:", {
                        deviceId: payload.deviceId,
                        connected: payload.connected
                    });

                    // Update appState with WebRTC connection info
                    this.appState.webrtcConnections[payload.deviceId] = payload.connected;
                    console.log("[StateManager] Updated appState.webrtcConnections:", this.appState.webrtcConnections);

                    // Send WebRTC connection update directly to popup
                    const webrtcMessage = {
                        type: "webrtcConnectionUpdate",
                        deviceId: payload.deviceId,
                        connected: payload.connected
                    };

                    console.log("[StateManager] Sending WebRTC connection update to popup:", webrtcMessage);
                    this.broadcastToPopupPorts(webrtcMessage as any);
                } else {
                    console.warn("[StateManager] Invalid WebRTC connection update payload:", payload);
                }
                break;

            case "meshStatusUpdate":
                console.log("[StateManager] Received mesh status update from offscreen:", payload);
                this.appState.meshStatus = payload.status || { type: MeshStatusType.Incomplete };

                // Broadcast mesh status update directly to popup
                this.broadcastToPopupPorts({
                    type: "meshStatusUpdate",
                    status: this.appState.meshStatus
                } as any);
                break;

            case "dkgStateUpdate":
                console.log("[StateManager] Received DKG state update from offscreen:", payload);
                this.appState.dkgState = payload.state || DkgState.Idle;

                // Auto-fetch DKG address when DKG completes (business logic moved from popup)
                if (this.appState.dkgState === DkgState.Complete && this.appState.sessionInfo) {
                    console.log("[StateManager] DKG completed, auto-fetching DKG address");
                    this.fetchAndUpdateDkgAddress();
                }

                // Broadcast DKG state update directly to popup
                this.broadcastToPopupPorts({
                    type: "dkgStateUpdate",
                    state: this.appState.dkgState
                } as any);
                break;

            case "sessionUpdate":
                if ('sessionInfo' in payload && 'invites' in payload) {
                    console.log("[StateManager] Received session update from offscreen:", payload);

                    this.appState.sessionInfo = payload.sessionInfo || null;
                    this.appState.invites = payload.invites || [];

                    // Broadcast session update to popup
                    this.broadcastToPopupPorts({
                        type: "sessionUpdate",
                        sessionInfo: this.appState.sessionInfo,
                        invites: this.appState.invites
                    } as any);
                }
                break;

            case "webrtcStatusUpdate":
                if ('deviceId' in payload && 'status' in payload) {
                    console.log(`[StateManager] WebRTC status update for ${payload.deviceId}: ${payload.status}`);
                    // Forward to popup if needed
                    this.broadcastToPopupPorts({
                        type: "webrtcStatusUpdate",
                        deviceId: payload.deviceId,
                        status: payload.status
                    } as any);
                }
                break;

            case "peerConnectionStatusUpdate":
                if ('deviceId' in payload && 'connectionState' in payload) {
                    console.log(`[StateManager] Peer connection status update for ${payload.deviceId}: ${payload.connectionState}`);
                }
                break;

            case "dataChannelStatusUpdate":
                if ('deviceId' in payload && 'channelName' in payload && 'state' in payload) {
                    console.log(`[StateManager] Data channel ${payload.channelName} for ${payload.deviceId}: ${payload.state}`);
                }
                break;

            default:
                console.log("[StateManager] Forwarding unknown message to popup:", payload);
                this.broadcastToPopupPorts({
                    type: "fromOffscreen",
                    payload
                } as any);
                break;
        }
    }

    /**
     * Update session information
     */
    updateSessionInfo(sessionInfo: typeof this.appState.sessionInfo): void {
        this.appState.sessionInfo = sessionInfo;

        this.broadcastToPopupPorts({
            type: "sessionUpdate",
            sessionInfo: this.appState.sessionInfo,
            invites: this.appState.invites
        } as any);
    }

    /**
     * Update session invites
     */
    updateInvites(invites: typeof this.appState.invites): void {
        this.appState.invites = invites;

        this.broadcastToPopupPorts({
            type: "sessionUpdate",
            sessionInfo: this.appState.sessionInfo,
            invites: this.appState.invites
        } as any);
    }

    /**
     * Clear session-related state
     */
    clearSessionState(): void {
        this.appState.sessionInfo = null;
        this.appState.invites = [];
        this.appState.meshStatus = { type: MeshStatusType.Incomplete };
        this.appState.dkgState = DkgState.Idle;
        this.appState.webrtcConnections = {};

        console.log("[StateManager] Cleared session state");
        this.broadcastCurrentState();
    }

    /**
     * Get popup ports count for debugging
     */
    getPopupPortsCount(): number {
        return this.popupPorts.size;
    }

    /**
     * Set device ID
     */
    setDeviceId(deviceId: string): void {
        this.appState.deviceId = deviceId;
        console.log("[StateManager] Set device ID:", deviceId);
    }

    /**
     * Set blockchain selection (maintains blockchain field for backward compatibility)
     */
    setBlockchain(blockchain: "ethereum" | "solana"): void {
        // Convert blockchain to curve for new state model
        this.appState.curve = blockchain === "ethereum" ? "secp256k1" : "ed25519";
        console.log("[StateManager] Set blockchain:", blockchain, "-> curve:", this.appState.curve);
    }

    /**
     * Set curve selection
     */
    setCurve(curve: "ed25519" | "secp256k1"): void {
        this.appState.curve = curve;
        console.log("[StateManager] Set curve:", curve);
    }

    /**
     * Get specific state properties
     */
    getDeviceId(): string { return this.appState.deviceId; }
    getSessionInfo() { return this.appState.sessionInfo; }
    getInvites() { return this.appState.invites; }
    getConnectedDevices(): string[] { return this.appState.connecteddevices; }
    getWebRTCConnections() { return this.appState.webrtcConnections; }
    isWebSocketConnected(): boolean { return this.appState.wsConnected; }
    getMeshStatus() { return this.appState.meshStatus; }
    getDkgState() { return this.appState.dkgState; }
    getCurve() { return this.appState.curve; }
    getBlockchain() {
        // Convert curve back to blockchain for backward compatibility
        return this.appState.curve === "secp256k1" ? "ethereum" : "solana";
    }

    /**
     * Auto-fetch DKG address when DKG completes (moved from popup)
     * This handles the business logic that was previously in the popup reactive statement
     */
    private async fetchAndUpdateDkgAddress(): Promise<void> {
        try {
            const blockchain = this.getBlockchain();
            const command = blockchain === "ethereum" ? "getEthereumAddress" : "getSolanaAddress";
            
            console.log("[StateManager] Auto-fetching DKG address for blockchain:", blockchain);
            
            // Send message to offscreen document to get DKG address
            const response = await chrome.runtime.sendMessage({
                type: command,
                payload: {},
            });

            if (response && response.success) {
                const addressKey = blockchain === "ethereum" ? "ethereumAddress" : "solanaAddress";
                const dkgAddress = response.data[addressKey] || "";

                if (dkgAddress) {
                    console.log("[StateManager] Successfully fetched DKG address:", dkgAddress);
                    
                    // Update app state
                    this.appState.dkgAddress = dkgAddress;
                    this.appState.dkgError = "";
                    
                    // Broadcast DKG address update to popup
                    this.broadcastToPopupPorts({
                        type: "dkgAddressUpdate",
                        address: dkgAddress,
                        blockchain: blockchain
                    } as any);
                } else {
                    const error = `No DKG ${blockchain} address available. Please complete DKG first.`;
                    console.warn("[StateManager]", error);
                    this.appState.dkgError = error;
                    this.appState.dkgAddress = "";
                }
            } else {
                const error = response?.error || `Failed to get DKG ${blockchain} address`;
                console.error("[StateManager] DKG address fetch failed:", error);
                this.appState.dkgError = error;
                this.appState.dkgAddress = "";
            }
        } catch (error: any) {
            const errorMessage = `Error fetching DKG address: ${error.message || error}`;
            console.error("[StateManager]", errorMessage);
            this.appState.dkgError = errorMessage;
            this.appState.dkgAddress = "";
        }

        // Broadcast current state to ensure popup is updated
        this.broadcastCurrentState();
    }
}
