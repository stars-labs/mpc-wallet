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

/**
 * Manages session persistence across extension lifecycle
 */
export class SessionPersistenceManager {
    private static readonly STORAGE_KEY = 'mpc_wallet_session_state';
    private static readonly MAX_AGE_MS = 30 * 60 * 1000; // 30 minutes

    static async saveSessionState(sessionInfo: SessionInfo | null, dkgState: DkgState, meshStatus: MeshStatus) {
        try {
            const sessionState = {
                sessionInfo,
                dkgState,
                meshStatus,
                timestamp: Date.now()
            };

            await chrome.storage.local.set({ [this.STORAGE_KEY]: sessionState });
            console.log("[SessionManager] Session state saved to storage");
        } catch (error) {
            console.error("[SessionManager] Failed to save session state:", error);
        }
    }

    static async loadSessionState(): Promise<{ sessionInfo: SessionInfo | null; dkgState: DkgState; meshStatus: MeshStatus } | null> {
        try {
            const result = await chrome.storage.local.get(this.STORAGE_KEY);
            const sessionState = result[this.STORAGE_KEY];

            if (!sessionState) {
                console.log("[SessionManager] No session state found in storage");
                return null;
            }

            // Check if the session state is still valid (not expired)
            const age = Date.now() - sessionState.timestamp;
            if (age > this.MAX_AGE_MS) {
                console.log("[SessionManager] Session state expired, clearing it");
                await this.clearSessionState();
                return null;
            }

            console.log("[SessionManager] Loaded session state from storage");
            return {
                sessionInfo: sessionState.sessionInfo,
                dkgState: sessionState.dkgState,
                meshStatus: sessionState.meshStatus
            };
        } catch (error) {
            console.error("[SessionManager] Failed to load session state:", error);
            return null;
        }
    }

    static async clearSessionState() {
        try {
            await chrome.storage.local.remove(this.STORAGE_KEY);
            console.log("[SessionManager] Cleared session state from storage");
        } catch (error) {
            console.error("[SessionManager] Failed to clear session state:", error);
        }
    }
}

/**
 * Handles MPC session lifecycle and coordination
 */
export class SessionManager {
    private appState: AppState;
    private wsClient: WebSocketClient | null;
    private broadcastToPopup: (message: BackgroundToPopupMessage) => void;
    private sendToOffscreen: (message: OffscreenMessage, description: string) => Promise<{ success: boolean; error?: string }>;

    constructor(
        appState: AppState,
        wsClient: WebSocketClient | null,
        broadcastToPopup: (message: BackgroundToPopupMessage) => void,
        sendToOffscreen: (message: OffscreenMessage, description: string) => Promise<{ success: boolean; error?: string }>
    ) {
        this.appState = appState;
        this.wsClient = wsClient;
        this.broadcastToPopup = broadcastToPopup;
        this.sendToOffscreen = sendToOffscreen;
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
    handleSessionProposal(fromPeerId: string, proposalData: any) {
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

        const sessionInfo: SessionInfo = {
            session_id: proposalData.session_id,
            proposer_id: fromPeerId,
            participants: proposalData.participants || [],
            threshold: proposalData.threshold,
            total: proposalData.total,
            accepted_devices: [fromPeerId], // Proposer automatically accepts
            status: "proposed"
        };

        console.log("[SessionManager] Session proposal validated:", sessionInfo);

        // Check if this peer is included in the session
        if (sessionInfo.participants.includes(this.appState.deviceId)) {
            console.log("[SessionManager] This peer is included in session proposal");

            // Check for existing invite
            const existingInviteIndex = this.appState.invites.findIndex(inv =>
                inv.session_id === sessionInfo.session_id
            );

            if (existingInviteIndex !== -1) {
                console.log("[SessionManager] Updating existing session invite");
                this.appState.invites[existingInviteIndex] = sessionInfo;
            } else {
                console.log("[SessionManager] Adding new session invite");
                this.appState.invites.push(sessionInfo);
            }

            // If this peer is the proposer, automatically accept and set up WebRTC
            if (fromPeerId === this.appState.deviceId) {
                console.log("[SessionManager] This peer is the proposer, auto-accepting and setting up WebRTC");

                this.appState.sessionInfo = { ...sessionInfo, status: "accepted" };
                this.appState.invites = this.appState.invites.filter(inv => inv.session_id !== sessionInfo.session_id);

                // Forward to offscreen for WebRTC setup
                this.sendToOffscreen({
                    type: "sessionAccepted",
                    sessionInfo: this.appState.sessionInfo,
                    currentdeviceId: this.appState.deviceId
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

        if (!validateSessionAcceptance(responseData)) {
            console.error("[SessionManager] Invalid session response data:", responseData);
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

                if (allAccepted) {
                    console.log("[SessionManager] All participants have accepted the session! Notifying offscreen for mesh readiness.");

                    // Send updated session info to offscreen to trigger mesh readiness check
                    // Only send if sessionInfo is available
                    if (this.appState.sessionInfo) {
                        const sessionAllAcceptedMessage: OffscreenMessage = {
                            type: "sessionAllAccepted",
                            sessionInfo: this.appState.sessionInfo,
                            currentdeviceId: this.appState.deviceId,
                            blockchain: this.appState.blockchain || "solana" // Use stored blockchain or default to solana
                        };

                        this.sendToOffscreen(sessionAllAcceptedMessage, "sessionAllAccepted");
                    }
                } else {
                    console.log(`[SessionManager] Not all participants accepted yet.`);

                    // Still send update to offscreen for tracking
                    // Only send if sessionInfo is available
                    if (this.appState.sessionInfo) {
                        const sessionResponseUpdateMessage: OffscreenMessage = {
                            type: "sessionResponseUpdate",
                            sessionInfo: this.appState.sessionInfo,
                            currentdeviceId: this.appState.deviceId
                        };

                        this.sendToOffscreen(sessionResponseUpdateMessage, "sessionResponseUpdate");
                    }
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

        const sessionIndex = this.appState.invites.findIndex(inv => inv.session_id === sessionId);

        if (sessionIndex === -1) {
            return { success: false, error: "Session not found in invites" };
        }

        const session = this.appState.invites[sessionIndex];

        // Store blockchain selection
        this.appState.blockchain = blockchain;

        // Move session to active and update status
        this.appState.sessionInfo = { ...session, status: "accepted" };
        this.appState.invites.splice(sessionIndex, 1);

        // Persist session state
        try {
            await SessionPersistenceManager.saveSessionState(
                this.appState.sessionInfo,
                this.appState.dkgState,
                this.appState.meshStatus
            );
        } catch (error) {
            console.warn("[SessionManager] Failed to persist session state:", error);
        }

        // Send acceptance message to other participants
        const acceptanceData = {
            websocket_msg_type: "SessionResponse",
            session_id: sessionId,
            accepted: true
        };

        if (this.wsClient?.getReadyState() === WebSocket.OPEN) {
            // Send to all other participants
            const otherParticipants = session.participants.filter(p => p !== this.appState.deviceId);

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

                if (this.appState.sessionInfo) {
                    await this.sendToOffscreen({
                        type: "sessionAccepted",
                        sessionInfo: this.appState.sessionInfo,
                        currentdeviceId: this.appState.deviceId,
                        blockchain: blockchain
                    }, "sessionAccepted");
                } else {
                    console.error("[SessionManager] Cannot forward session info: sessionInfo is null");
                }

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
        participants: string[]
    ): Promise<{ success: boolean; error?: string }> {
        console.log(`[SessionManager] Proposing session: ${sessionId}`);

        if (!this.wsClient || this.wsClient.getReadyState() !== WebSocket.OPEN) {
            return { success: false, error: "WebSocket not connected" };
        }

        const proposalData = {
            websocket_msg_type: "SessionProposal",
            session_id: sessionId,
            proposer_id: this.appState.deviceId,
            participants: participants,
            threshold: threshold,
            total: totalParticipants
        };

        try {
            // Send proposal to all other participants
            const otherParticipants = participants.filter(p => p !== this.appState.deviceId);

            await Promise.all(otherParticipants.map(async (peerId) => {
                try {
                    await this.wsClient!.relayMessage(peerId, proposalData);
                    console.log(`[SessionManager] Session proposal sent to ${peerId}`);
                } catch (error) {
                    console.error(`[SessionManager] Failed to send proposal to ${peerId}:`, error);
                }
            }));

            // Handle our own proposal (auto-accept for proposer)
            this.handleSessionProposal(this.appState.deviceId, proposalData);

            return { success: true };
        } catch (error) {
            return { success: false, error: (error as Error).message };
        }
    }
}
