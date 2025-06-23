// ===================================================================
// SESSION MANAGEMENT MODULE
// ===================================================================
//
// This module handles MPC session lifecycle management including:
// - Session persistence across extension restarts
// - Session proposal handling
// - Session response processing
// - Session state validation
// ===================================================================

import { AppState } from "../../types/appstate";
import { SessionInfo, SessionProposal, SessionResponse } from "../../types/session";
import { MeshStatus } from "../../types/mesh";
import { DkgState } from "../../types/dkg";
import { WebSocketClient } from "./websocket";
import { validateSessionProposal, validateSessionAcceptance } from "../../types/messages";
import type { BackgroundToPopupMessage, OffscreenMessage } from "../../types/messages";

// Session persistence removed - sessions are ephemeral for security

/**
 * Handles MPC session lifecycle and coordination
 */
export class SessionManager {
    private appState: AppState;
    private wsClient: WebSocketClient | null;
    private broadcastToPopup: (message: BackgroundToPopupMessage) => void;
    private sendToOffscreen: (message: OffscreenMessage, description: string) => Promise<{ success: boolean; error?: string }>;
    private stateManager: any; // StateManager reference

    constructor(
        appState: AppState,
        wsClient: WebSocketClient | null,
        broadcastToPopup: (message: BackgroundToPopupMessage) => void,
        sendToOffscreen: (message: OffscreenMessage, description: string) => Promise<{ success: boolean; error?: string }>,
        stateManager?: any
    ) {
        this.appState = appState;
        this.wsClient = wsClient;
        this.broadcastToPopup = broadcastToPopup;
        this.sendToOffscreen = sendToOffscreen;
        this.stateManager = stateManager;
    }

    /**
     * Update the WebSocket client reference (used when WebSocket reconnects)
     */
    updateWebSocketClient(wsClient: WebSocketClient | null): void {
        this.wsClient = wsClient;
        console.log("[SessionManager] WebSocket client reference updated");
    }

    /**
     * Validate WebSocket session proposal data
     */
    private validateWebSocketSessionProposal(proposalData: any): boolean {
        return proposalData &&
            typeof proposalData.session_id === 'string' &&
            typeof proposalData.total === 'number' &&
            typeof proposalData.threshold === 'number' &&
            Array.isArray(proposalData.participants) &&
            proposalData.websocket_msg_type === 'SessionProposal';
    }

    /**
     * Handle incoming session proposals
     */
    async handleSessionProposal(fromPeerId: string, proposalData: any) {
        console.log("[SessionManager] Processing session proposal from:", fromPeerId);
        console.log("[SessionManager] Proposal data:", {
            session_id: proposalData?.session_id,
            total: proposalData?.total,
            threshold: proposalData?.threshold,
            participants: proposalData?.participants,
            websocket_msg_type: proposalData?.websocket_msg_type
        });

        if (!this.validateWebSocketSessionProposal(proposalData)) {
            console.error("[SessionManager] Invalid session proposal data:", proposalData);
            console.error("[SessionManager] Expected: session_id (string), total (number), threshold (number), participants (array), websocket_msg_type='SessionProposal'");
            return;
        }

        // Sort participants to ensure consistent indexing across all peers
        const sortedParticipants = [...(proposalData.participants || [])].sort();
        
        const sessionInfo: SessionInfo = {
            session_id: proposalData.session_id,
            proposer_id: fromPeerId,
            participants: sortedParticipants,
            threshold: proposalData.threshold,
            total: proposalData.total,
            accepted_devices: [fromPeerId], // Proposer automatically accepts
            status: "proposed"
        };

        console.log("[SessionManager] Session proposal validated:", sessionInfo);

        // Get current device ID from StateManager if available
        const currentDeviceId = this.stateManager ? this.stateManager.getState().deviceId : this.appState.deviceId;
        
        console.log("[SessionManager] Checking inclusion - currentDeviceId:", currentDeviceId, "participants:", sessionInfo.participants);
        
        // Check if this peer is included in the session
        if (sessionInfo.participants.includes(currentDeviceId)) {
            console.log("[SessionManager] This peer is included in session proposal");

            // Get current invites from StateManager
            const currentState = this.stateManager ? this.stateManager.getState() : this.appState;
            const invites = [...currentState.invites];
            
            // Check for existing invite
            const existingInviteIndex = invites.findIndex(inv =>
                inv.session_id === sessionInfo.session_id
            );

            if (existingInviteIndex !== -1) {
                console.log("[SessionManager] Updating existing session invite");
                invites[existingInviteIndex] = sessionInfo;
            } else {
                console.log("[SessionManager] Adding new session invite");
                invites.push(sessionInfo);
            }
            
            // Update local state
            this.appState.invites = invites;
            
            // Update StateManager with new invites
            if (this.stateManager) {
                this.stateManager.updateStateProperty('invites', invites);
                // Also update the local appState to sync
                this.appState = this.stateManager.getState();
            }

            // If this peer is the proposer, automatically accept and set up WebRTC
            if (fromPeerId === currentDeviceId) {
                console.log("[SessionManager] This peer is the proposer, auto-accepting and setting up WebRTC");

                const acceptedSessionInfo = { ...sessionInfo, status: "accepted" as const };
                const updatedInvites = invites.filter(inv => inv.session_id !== sessionInfo.session_id);
                
                // Update local state
                this.appState.sessionInfo = acceptedSessionInfo;
                this.appState.invites = updatedInvites;
                
                // Update StateManager with session changes
                if (this.stateManager) {
                    this.stateManager.updateState({
                        sessionInfo: acceptedSessionInfo,
                        invites: updatedInvites
                    });
                    // Sync local state
                    this.appState = this.stateManager.getState();
                }

                // No persistence - sessions are ephemeral

                // Forward to offscreen for WebRTC setup
                this.sendToOffscreen({
                    type: "sessionAccepted",
                    sessionInfo: this.appState.sessionInfo,
                    currentdeviceId: this.appState.deviceId,
                    blockchain: this.appState.blockchain || "solana"
                }, "proposerWebRTCSetup");
            }

            // Broadcast session update to popup
            this.broadcastToPopup({
                type: "sessionUpdate",
                sessionInfo: this.appState.sessionInfo,
                invites: this.appState.invites
            } as any);

            console.log("[SessionManager] Session proposal processed and broadcasted to popup");
        } else {
            console.log("[SessionManager] This peer is not included in session proposal, ignoring");
        }
    }

    /**
     * Handle session response from participants
     */
    handleSessionResponse(fromPeerId: string, responseData: any) {
        console.log("[SessionManager] Processing session response from:", fromPeerId);

        // Validate WebSocket session response data
        if (!responseData || 
            typeof responseData.session_id !== 'string' || 
            typeof responseData.accepted !== 'boolean' ||
            responseData.websocket_msg_type !== 'SessionResponse') {
            console.error("[SessionManager] Invalid session response data:", responseData);
            console.error("[SessionManager] Expected: session_id (string), accepted (boolean), websocket_msg_type='SessionResponse'");
            return;
        }

        const { session_id, accepted } = responseData;

        // Find the session
        const session = this.appState.sessionInfo?.session_id === session_id
            ? this.appState.sessionInfo
            : this.appState.invites.find(inv => inv.session_id === session_id);

        if (session) {
            console.log(`[SessionManager] Found session ${session_id}, updating acceptance status`);

            if (accepted) {
                // Add to accepted devices if not already present
                if (!session.accepted_devices.includes(fromPeerId)) {
                    session.accepted_devices.push(fromPeerId);
                    console.log(`[SessionManager] Added ${fromPeerId} to accepted devices`);
                }

                // Check if all participants have accepted
                const allAccepted = session.participants.every(participantId =>
                    session.accepted_devices.includes(participantId)
                );

                // Update the sessionInfo in appState if this is the active session
                if (this.appState.sessionInfo && this.appState.sessionInfo.session_id === session_id) {
                    this.appState.sessionInfo = { ...session };
                    if (this.stateManager) {
                        this.stateManager.updateStateProperty('sessionInfo', this.appState.sessionInfo);
                    }
                }
                
                if (allAccepted) {
                    console.log("[SessionManager] All participants have accepted the session! Notifying offscreen for mesh readiness.");

                    // Send updated session info to offscreen to trigger mesh readiness check
                    // Use the updated session object, not the potentially stale appState.sessionInfo
                    const sessionAllAcceptedMessage: OffscreenMessage = {
                        type: "sessionAllAccepted",
                        sessionInfo: session,
                        currentdeviceId: this.stateManager ? this.stateManager.getState().deviceId : this.appState.deviceId,
                        blockchain: this.appState.blockchain || "solana" // Use stored blockchain or default to solana
                    };

                    this.sendToOffscreen(sessionAllAcceptedMessage, "sessionAllAccepted");
                } else {
                    console.log(`[SessionManager] Not all participants accepted yet.`);

                    // Still send update to offscreen for tracking
                    const sessionResponseUpdateMessage: OffscreenMessage = {
                        type: "sessionResponseUpdate",
                        sessionInfo: session,
                        currentdeviceId: this.stateManager ? this.stateManager.getState().deviceId : this.appState.deviceId
                    };

                    this.sendToOffscreen(sessionResponseUpdateMessage, "sessionResponseUpdate");
                }
            }

            // Broadcast session update to popup
            this.broadcastToPopup({
                type: "sessionUpdate",
                sessionInfo: this.appState.sessionInfo,
                invites: this.appState.invites
            } as any);

            console.log("[SessionManager] Session response processed and broadcasted");
        } else {
            console.warn("[SessionManager] Received session response for unknown session:", session_id);
        }
    }

    /**
     * Accept a session invitation
     */
    async acceptSession(sessionId: string, blockchain: "ethereum" | "solana" = "solana"): Promise<{ success: boolean; error?: string }> {
        console.log(`[SessionManager] Accepting session: ${sessionId} with blockchain: ${blockchain}`);

        // Get current state from StateManager
        const currentState = this.stateManager ? this.stateManager.getState() : this.appState;
        const invites = [...currentState.invites];
        
        const sessionIndex = invites.findIndex(inv => inv.session_id === sessionId);

        if (sessionIndex === -1) {
            console.error(`[SessionManager] Session ${sessionId} not found in invites:`, invites);
            return { success: false, error: "Session not found in invites" };
        }

        const session = invites[sessionIndex];
        console.log(`[SessionManager] Found session to accept:`, session);

        // Store blockchain selection
        this.appState.blockchain = blockchain;

        // Get current device ID
        const currentDeviceId = this.stateManager ? this.stateManager.getState().deviceId : this.appState.deviceId;
        
        // Move session to active and update status, adding current device to accepted_devices
        const newSessionInfo = { 
            ...session, 
            status: "accepted" as const,
            accepted_devices: [...new Set([...session.accepted_devices, currentDeviceId])] // Add current device and dedupe
        };
        invites.splice(sessionIndex, 1);
        
        // Update local state
        this.appState.sessionInfo = newSessionInfo;
        this.appState.invites = invites;
        
        // Update StateManager with session changes
        if (this.stateManager) {
            this.stateManager.updateState({
                sessionInfo: newSessionInfo,
                invites: invites,
                blockchain: blockchain
            });
            // Sync local state with StateManager
            this.appState = this.stateManager.getState();
        }

        // No persistence - sessions are ephemeral

        // Send acceptance message to other participants
        const acceptanceData = {
            websocket_msg_type: "SessionResponse",
            session_id: sessionId,
            accepted: true
        };

        if (this.wsClient?.getReadyState() === WebSocket.OPEN) {
            // Send to all other participants
            const currentDeviceId = this.stateManager ? this.stateManager.getState().deviceId : this.appState.deviceId;
            const otherParticipants = newSessionInfo.participants.filter(p => p !== currentDeviceId);

            try {
                await Promise.all(otherParticipants.map(async (peerId) => {
                    try {
                        await this.wsClient!.relayMessage(peerId, acceptanceData);
                        console.log(`[SessionManager] Session acceptance sent to ${peerId}`);
                    } catch (error) {
                        console.error(`[SessionManager] Failed to send acceptance to ${peerId}:`, error);
                    }
                }));

                console.log("[SessionManager] All session acceptances sent");

                // Forward session info to offscreen for WebRTC setup
                console.log("[SessionManager] Forwarding session info to offscreen for WebRTC setup with blockchain:", blockchain);

                await this.sendToOffscreen({
                    type: "sessionAccepted",
                    sessionInfo: newSessionInfo,
                    currentdeviceId: this.stateManager ? this.stateManager.getState().deviceId : this.appState.deviceId,
                    blockchain: blockchain
                }, "sessionAccepted");
                
                // Broadcast session update to popup to ensure UI updates
                this.broadcastToPopup({
                    type: "sessionUpdate",
                    sessionInfo: newSessionInfo,
                    invites: invites
                } as any);

                return { success: true };
            } catch (error) {
                return { success: false, error: (error as Error).message };
            }
        } else {
            return { success: false, error: "WebSocket not connected" };
        }
    }

    /**
     * Propose a new session
     */
    async proposeSession(
        sessionId: string,
        totalParticipants: number,
        threshold: number,
        participants: string[],
        blockchain: "ethereum" | "solana" = "solana"
    ): Promise<{ success: boolean; error?: string }> {
        console.log(`[SessionManager] Proposing session: ${sessionId} with blockchain: ${blockchain}`);

        if (!this.wsClient || this.wsClient.getReadyState() !== WebSocket.OPEN) {
            return { success: false, error: "WebSocket not connected" };
        }

        const currentDeviceId = this.stateManager ? this.stateManager.getState().deviceId : this.appState.deviceId;
        
        // Store blockchain selection
        this.appState.blockchain = blockchain;
        if (this.stateManager) {
            this.stateManager.updateState({ blockchain });
        }
        
        // Sort participants to ensure consistent indexing across all peers
        const sortedParticipants = [...participants].sort();
        
        const proposalData = {
            websocket_msg_type: "SessionProposal",
            session_id: sessionId,
            proposer_id: currentDeviceId,
            participants: sortedParticipants,
            threshold: threshold,
            total: totalParticipants
        };

        try {
            // Send proposal to all other participants
            const otherParticipants = participants.filter(p => p !== currentDeviceId);

            await Promise.all(otherParticipants.map(async (peerId) => {
                try {
                    await this.wsClient!.relayMessage(peerId, proposalData);
                    console.log(`[SessionManager] Session proposal sent to ${peerId}`);
                } catch (error) {
                    console.error(`[SessionManager] Failed to send proposal to ${peerId}:`, error);
                }
            }));

            // Handle our own proposal (auto-accept for proposer)
            await this.handleSessionProposal(currentDeviceId, proposalData);

            return { success: true };
        } catch (error) {
            return { success: false, error: (error as Error).message };
        }
    }
}
