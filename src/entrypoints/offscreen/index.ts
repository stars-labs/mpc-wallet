import { WebSocketClient } from "../background/websocket";
import { WebRTCManager } from "../background/webrtc";

console.log("[Offscreen] Offscreen document loaded.");

// Initialize state
let wsClient: WebSocketClient | null = null;
let webrtcManager: WebRTCManager | null = null;
let peerId: string = "";
let initialized = false;

// Session storage
let sessionProposals: {
    [sessionId: string]: {
        proposer: string;
        data: any;
    }
} = {};

// Initialize WebRTC with WebSocket
function initWebRTC(wsUrl: string, peerIdParam: string) {
    if (initialized) {
        console.log("[Offscreen] Already initialized, skipping");
        return;
    }

    try {
        peerId = peerIdParam;
        console.log(`[Offscreen] Initializing WebRTC for peer ${peerId}`);

        // Create WebSocket client
        wsClient = new WebSocketClient(wsUrl);
        wsClient.onOpen(() => {
            console.log(`[Offscreen] WebSocket connected, registering peer ${peerId}`);
            wsClient?.register(peerId);
            sendToBackground({ type: "wsStatus", connected: true });
        });

        wsClient.onError((event) => {
            console.error("[Offscreen] WebSocket error:", event);
            sendToBackground({ type: "wsError", error: "Connection error" });
        });

        wsClient.onClose((event) => {
            console.log("[Offscreen] WebSocket disconnected:", event.reason);
            sendToBackground({ type: "wsStatus", connected: false, reason: event.reason });

            // Try to reconnect after a delay
            setTimeout(() => {
                console.log("[Offscreen] Attempting to reconnect WebSocket...");
                wsClient?.connect();
            }, 5000);
        });

        // Create WebRTC Manager
        webrtcManager = new WebRTCManager(wsClient, peerId);

        // Set up WebRTC event handlers
        webrtcManager.onLog = (message) => {
            console.log(`[Offscreen:WebRTC] ${message}`);
        };

        webrtcManager.onSessionUpdate = (sessionInfo, invites) => {
            sendToBackground({
                type: "sessionUpdate",
                sessionInfo,
                invites
            });
        };

        webrtcManager.onMeshStatusUpdate = (status) => {
            sendToBackground({ type: "meshStatusUpdate", status });
        };

        webrtcManager.onWebRTCAppMessage = (fromPeerId, message) => {
            console.log(`[Offscreen] WebRTC app message from ${fromPeerId}:`, message);
            sendToBackground({ type: "webrtcMessage", fromPeerId, message });
        };

        webrtcManager.onDkgStateUpdate = (state) => {
            sendToBackground({ type: "dkgStateUpdate", state });
        };

        // Connect WebSocket
        wsClient.connect();
        initialized = true;

        // Report success to background
        sendToBackground({
            type: "initialized",
            success: true,
            peerId
        });
    } catch (error) {
        console.error("[Offscreen] Initialization error:", error);
        sendToBackground({
            type: "initialized",
            success: false,
            error: error?.toString()
        });
    }
}

// Helper to send messages back to background script
function sendToBackground(payload: any) {
    chrome.runtime.sendMessage({
        type: "fromOffscreen",
        payload
    }).catch(err => {
        console.error("[Offscreen] Error sending message to background:", err);
    });
}

// Process relay messages from background
function handleRelayMessage(fromPeerId: string, data: any) {
    if (!webrtcManager || !initialized) {
        console.error("[Offscreen] Cannot handle relay - not initialized");
        return;
    }

    try {
        // Check for websocket_msg_type which is present in both formats
        if (data && data.websocket_msg_type) {
            console.log(`[Offscreen] Processing ${data.websocket_msg_type} from ${fromPeerId}`);

            switch (data.websocket_msg_type) {
                case "SessionProposal":
                    let proposalData: any;

                    // Handle Format 1: Nested in data property
                    if (data.data && data.data.session_id) {
                        proposalData = {
                            session_id: data.data.session_id,
                            total: data.data.total,
                            threshold: data.data.threshold,
                            participants: data.data.participants
                        };
                    }
                    // Handle Format 2: Direct properties
                    else if (data.session_id) {
                        proposalData = {
                            session_id: data.session_id,
                            total: data.total,
                            threshold: data.threshold,
                            participants: data.participants
                        };
                    }

                    if (proposalData && proposalData.session_id && Array.isArray(proposalData.participants)) {
                        console.log(`[Offscreen] Session "${proposalData.session_id}" proposed with ${proposalData.total} participants`);

                        // Store the proposal
                        sessionProposals[proposalData.session_id] = {
                            proposer: fromPeerId,
                            data: proposalData
                        };

                        // Process with WebRTCManager
                        webrtcManager.handleSessionProposal(fromPeerId, proposalData);
                    }
                    break;

                case "WebRTCSignal":
                    console.log("[Offscreen] Processing WebRTC signal");
                    let signalDataForManager: any = null;

                    // Extract the signal data
                    if (data.data) {
                        const signalContent = data.data;
                        if (signalContent.Offer) {
                            signalDataForManager = { type: 'Offer', data: signalContent.Offer };
                        } else if (signalContent.Answer) {
                            signalDataForManager = { type: 'Answer', data: signalContent.Answer };
                        } else if (signalContent.Candidate) {
                            signalDataForManager = { type: 'Candidate', data: signalContent.Candidate };
                        }
                    } else if (data.Offer) {
                        signalDataForManager = { type: 'Offer', data: data.Offer };
                    } else if (data.Answer) {
                        signalDataForManager = { type: 'Answer', data: data.Answer };
                    } else if (data.Candidate) {
                        signalDataForManager = { type: 'Candidate', data: data.Candidate };
                    }

                    if (signalDataForManager) {
                        // Process with WebRTCManager
                        webrtcManager.handleWebRTCSignal(fromPeerId, signalDataForManager);
                    }
                    break;

                default:
                    console.log(`[Offscreen] Unknown websocket_msg_type: ${data.websocket_msg_type}`);
            }
        }
    } catch (error) {
        console.error(`[Offscreen] Error processing relay from ${fromPeerId}:`, error);
    }
}

// Handle direct user actions from UI/background
function handleAction(action: string, params: any) {
    if (!webrtcManager || !initialized) {
        console.error(`[Offscreen] Cannot handle action ${action} - not initialized`);
        return false;
    }

    try {
        switch (action) {
            case "proposeSession":
                webrtcManager.proposeSession(
                    params.sessionId,
                    params.total,
                    params.threshold,
                    params.participants
                );
                return true;

            case "acceptSession":
                webrtcManager.acceptSession(params.sessionId);
                return true;

            default:
                console.warn(`[Offscreen] Unknown action: ${action}`);
                return false;
        }
    } catch (error) {
        console.error(`[Offscreen] Error handling action ${action}:`, error);
        return false;
    }
}

// Listen for messages from background
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (!message || !message.type) return false;

    if (message.type === "fromBackground") {
        const payload = message.payload;

        if (!payload || !payload.type) {
            console.error("[Offscreen] Invalid payload from background");
            sendResponse({ success: false, error: "Invalid payload" });
            return false;
        }

        console.log(`[Offscreen] Received ${payload.type} from background`);

        switch (payload.type) {
            case "init":
                initWebRTC(payload.wsUrl, payload.peerId);
                sendResponse({ success: true });
                break;

            case "relayMessage":
                handleRelayMessage(payload.fromPeerId, payload.data);
                sendResponse({ success: true });
                break;

            case "proposeSession":
                const proposeResult = handleAction("proposeSession", payload);
                sendResponse({ success: proposeResult });
                break;

            case "acceptSession":
                const acceptResult = handleAction("acceptSession", payload);
                sendResponse({ success: acceptResult });
                break;

            default:
                console.warn(`[Offscreen] Unknown payload type: ${payload.type}`);
                sendResponse({ success: false, error: "Unknown payload type" });
        }

        return true;
    }
});

// Notify background that we're ready
console.log("[Offscreen] Ready to receive messages");
chrome.runtime.sendMessage({
    type: "fromOffscreen",
    payload: { type: "ready" }
}).catch(err => {
    console.error("[Offscreen] Error sending ready message:", err);
});
