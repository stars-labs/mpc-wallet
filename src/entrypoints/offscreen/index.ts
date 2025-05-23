import { WebRTCManager } from './webrtc'; // Adjust path as necessary
import type { SessionInfo, MeshStatus, DkgState, WebRTCAppMessage } from '../../types/appstate'; // Fixed import
import { ServerMsg, ClientMsg, WebSocketMessagePayload, WebRTCSignal } from '../../types/websocket';
console.log("Offscreen script loaded.");

let webRTCManager: WebRTCManager | null = null;
let localPeerId: string | null = null;
// Track WebRTC connections
let webrtcConnections: Record<string, boolean> = {};

// Removed wsRelayCallback as WebRTCManager will use a direct callback for sending payloads

// Function to send messages to the background script
function sendToBackground(message: { type: string; payload: unknown }) {
    console.log("Offscreen: Sending message to background:", message);
    chrome.runtime.sendMessage(message, (response) => {
        if (chrome.runtime.lastError) {
            console.error("Offscreen: Error sending message to background or receiving ack:", chrome.runtime.lastError.message, "Original message:", message);
        } else {
            console.log("Offscreen: Message to background acknowledged:", response, "Original message:", message);
        }
    });
}

// Listen for messages from the background script
chrome.runtime.onMessage.addListener((message: { type?: string; payload?: any }, sender, sendResponse) => {
    console.log("Offscreen: Message received from background:", message);

    let msgType: string | undefined;
    let actualPayload: any = {};

    // Ensure message and message.payload are defined before accessing properties
    if (message && message.payload && typeof message.payload.type === 'string') {
        // Message format: { payload: { type: "...", ...data } }
        msgType = message.payload.type;
        const { type, ...rest } = message.payload;
        actualPayload = rest;
        console.log(`Offscreen: Processing wrapped message. Type: ${msgType}, Payload:`, actualPayload);
    } else if (message && typeof message.type === 'string') {
        // Message format: { type: "...", ...data }
        msgType = message.type;
        const { type, ...rest } = message;
        actualPayload = rest;
        console.log(`Offscreen: Processing top-level type message. Type: ${msgType}, Payload:`, actualPayload);
    } else {
        console.warn("Offscreen: Received message with unknown structure or missing type:", message);
        sendResponse({ success: false, error: "Malformed or untyped message" });
        return false;
    }

    const payload = actualPayload;

    switch (msgType) {
        case "createOffscreen":
            console.log("Offscreen: Received 'createOffscreen' command. Document is already active.", payload);
            sendResponse({ success: true, message: "Offscreen document is already active." });
            break;
        case "init":
            console.log("Offscreen: Received 'init' command", payload);

            if (!payload.peerId) {
                console.error("Offscreen: Init message missing peerId:", payload);
                sendResponse({ success: false, error: "Missing peerId in init message" });
                break;
            }

            localPeerId = payload.peerId;

            if (localPeerId) {
                // Define how the offscreen WebRTCManager will send WebSocket payloads out
                // (via the background script)
                const sendPayloadToBackgroundForRelay = (toPeerId: string, payloadData: WebSocketMessagePayload) => {
                    console.log(`Offscreen: Sending WebRTC signal to ${toPeerId} via background:`, payloadData);

                    // Add debugging to see what type of data we're sending
                    if (payloadData && typeof payloadData === 'object') {
                        console.log(`Offscreen: Payload type check - websocket_msg_type: ${payloadData.websocket_msg_type}`);
                        if (payloadData.websocket_msg_type === 'WebRTCSignal') {
                            console.log(`Offscreen: This is a WebRTC signal, should be relayed to WebSocket`);
                        }
                    }

                    sendToBackground({
                        type: "fromOffscreen",
                        payload: {
                            type: "relayViaWs",
                            to: toPeerId,
                            data: payloadData, // This is the full WebSocketMessagePayload
                        }
                    });
                };

                console.log(`Offscreen: Creating WebRTCManager for peer ID: ${localPeerId}`);
                webRTCManager = new WebRTCManager(localPeerId, sendPayloadToBackgroundForRelay);

                webRTCManager.onLog = (logMessage) => {
                    console.log(`[Offscreen WebRTC] ${logMessage}`);
                    // Instead of sending all log messages, parse and send specific status updates

                    // Check for data channel status changes
                    if (logMessage.includes("Data channel") && logMessage.includes("opened")) {
                        const peerMatch = logMessage.match(/with ([\w-]+)/);
                        const channelMatch = logMessage.match(/'([^']+)'/);
                        if (peerMatch && channelMatch) {
                            sendToBackground({
                                type: "fromOffscreen",
                                payload: {
                                    type: "dataChannelStatusUpdate",
                                    peerId: peerMatch[1],
                                    channelName: channelMatch[1],
                                    state: "open"
                                }
                            });
                        }
                    }

                    // Check for connection status changes
                    if (logMessage.includes("data channel to") && logMessage.includes("is now open")) {
                        const peerMatch = logMessage.match(/to ([\w-]+)/);
                        if (peerMatch) {
                            sendToBackground({
                                type: "fromOffscreen",
                                payload: {
                                    type: "webrtcStatusUpdate",
                                    peerId: peerMatch[1],
                                    status: "connected"
                                }
                            });
                        }
                    }

                    // Only send important operational messages, not routine status
                    if (logMessage.includes("Error") ||
                        logMessage.includes("Failed") ||
                        logMessage.includes("Warning")) {
                        console.warn(`[Offscreen WebRTC] Important: ${logMessage}`);
                    }
                };
                webRTCManager.onSessionUpdate = (sessionInfo, invites) => {
                    console.log("Offscreen: Session update:", { sessionInfo, invites });
                    sendToBackground({ type: "fromOffscreen", payload: { type: "sessionUpdate", sessionInfo, invites } });
                };
                webRTCManager.onMeshStatusUpdate = (status) => {
                    console.log("Offscreen: Mesh status update:", status);
                    sendToBackground({ type: "fromOffscreen", payload: { type: "meshStatusUpdate", status } });
                };
                webRTCManager.onWebRTCAppMessage = (fromPeerId: string, appMessage: WebRTCAppMessage) => {
                    console.log("Offscreen: WebRTC app message:", { fromPeerId, appMessage });
                    sendToBackground({ type: "fromOffscreen", payload: { type: "webrtcMessage", fromPeerId, message: appMessage } });
                };
                webRTCManager.onDkgStateUpdate = (state) => {
                    console.log("Offscreen: DKG state update:", state);
                    sendToBackground({ type: "fromOffscreen", payload: { type: "dkgStateUpdate", state } });
                };

                webRTCManager.onWebRTCConnectionUpdate = (peerId: string, connected: boolean) => {
                    console.log("Offscreen: WebRTC connection update:", peerId, connected);

                    // Update local tracking
                    webrtcConnections[peerId] = connected;

                    sendToBackground({
                        type: "fromOffscreen",
                        payload: {
                            type: "webrtcConnectionUpdate",
                            peerId,
                            connected
                        }
                    });
                };

                console.log(`Offscreen: WebRTC Manager successfully initialized for peer ID: ${localPeerId}.`);
                sendResponse({ success: true, message: "Offscreen initialized with WebRTCManager." });
            } else {
                console.error("Offscreen: LocalPeerId is falsy after assignment:", localPeerId);
                sendResponse({ success: false, error: "LocalPeerId assignment failed." });
            }
            break;

        case "relayViaWs":
            console.log("Offscreen: Received 'relayViaWs' (WebSocket payload) from background", payload);
            if (webRTCManager && payload.data) {
                // The payload should contain either 'fromPeerId' or we need to extract it from the data
                let fromPeerId = payload.to;
                if (fromPeerId) {
                    console.log(`Offscreen: Calling webRTCManager.handleWebSocketMessagePayload with fromPeerId: ${fromPeerId}, data:`, payload.data);
                    // The payload.data is expected to be WebSocketMessagePayload
                    webRTCManager.handleWebSocketMessagePayload(fromPeerId, payload.data as WebSocketMessagePayload);
                    console.log("Offscreen: Relayed message to WebRTCManager for peer:", fromPeerId);
                    sendResponse({ success: true, message: "Message relayed to WebRTCManager." });
                } else {
                    const debugInfo = {
                        webRTCManagerReady: !!webRTCManager,
                        hasData: !!payload.data,
                        localPeerId,
                        payload,
                        missingFromPeerId: "fromPeerId not found in payload or payload.data.from"
                    };
                    console.warn("Offscreen: Cannot handle relayViaWs - missing fromPeerId.", debugInfo);
                    sendResponse({ success: false, error: "Missing fromPeerId in relayViaWs payload.", debugInfo });
                }
            } else {
                const debugInfo = {
                    webRTCManagerReady: !!webRTCManager,
                    hasData: !!payload.data,
                    localPeerId,
                    payload
                };
                console.warn("Offscreen: Cannot handle relayViaWs - WebRTCManager not ready or missing data.", debugInfo);
                sendResponse({ success: false, error: "WebRTCManager not ready or missing data in relayViaWs payload.", debugInfo });
            }
            break;

        case "sessionAccepted":
            console.log("Offscreen: Received 'sessionAccepted' command", payload);
            if (webRTCManager && payload.sessionInfo && payload.currentPeerId) {
                console.log(`Offscreen: Setting up WebRTC for accepted session: ${payload.sessionInfo.session_id}`);
                console.log(`Offscreen: Current peer: ${payload.currentPeerId}, Participants:`, payload.sessionInfo.participants);

                // Update the WebRTCManager with the session info
                webRTCManager.sessionInfo = payload.sessionInfo;

                // Initiate WebRTC connections to peers with lexicographically larger IDs
                const currentPeerId: string = payload.currentPeerId;
                const participants: string[] = payload.sessionInfo.participants || [];
                const peersToConnect: string[] = participants.filter((peerId: string) =>
                    peerId !== currentPeerId && peerId > currentPeerId
                );

                console.log(`Offscreen: Peers to initiate offers to (ID > ${currentPeerId}):`, peersToConnect);

                if (peersToConnect.length > 0) {
                    peersToConnect.forEach((peerId: string) => {
                        console.log(`Offscreen: Initiating WebRTC connection to ${peerId}`);
                        webRTCManager!.initiatePeerConnection(peerId);
                    });
                } else {
                    console.log(`Offscreen: No peers to initiate offers to based on ID ordering. Waiting for incoming offers.`);
                }

                sendResponse({ success: true, message: "Session accepted and WebRTC setup initiated." });
            } else {
                const debugInfo = {
                    webRTCManagerReady: !!webRTCManager,
                    hasSessionInfo: !!payload.sessionInfo,
                    hasCurrentPeerId: !!payload.currentPeerId,
                    localPeerId,
                    payload
                };
                console.warn("Offscreen: Cannot handle sessionAccepted - missing required data.", debugInfo);
                sendResponse({ success: false, error: "WebRTCManager not ready or missing sessionInfo/currentPeerId in sessionAccepted payload.", debugInfo });
            }
            break;

        case "sessionAllAccepted":
            console.log("Offscreen: Received 'sessionAllAccepted' command - all participants have accepted!", payload);
            if (webRTCManager && payload.sessionInfo) {
                // Update session info and trigger mesh readiness check
                webRTCManager.updateSessionInfo(payload.sessionInfo);
                console.log("Offscreen: Updated session info and triggered mesh readiness check");
                sendResponse({ success: true, message: "Session all accepted processed - mesh readiness triggered." });
            } else {
                console.warn("Offscreen: Cannot handle sessionAllAccepted - WebRTCManager not ready or missing sessionInfo");
                sendResponse({ success: false, error: "WebRTCManager not ready or missing sessionInfo" });
            }
            break;

        case "sessionResponseUpdate":
            console.log("Offscreen: Received 'sessionResponseUpdate' command", payload);
            if (webRTCManager && payload.sessionInfo) {
                // Update session info for tracking acceptance progress
                webRTCManager.updateSessionInfo(payload.sessionInfo);
                console.log("Offscreen: Updated session info with latest acceptance status");
                sendResponse({ success: true, message: "Session response update processed." });
            } else {
                console.warn("Offscreen: Cannot handle sessionResponseUpdate - WebRTCManager not ready or missing sessionInfo");
                sendResponse({ success: false, error: "WebRTCManager not ready or missing sessionInfo" });
            }
            break;

        case "acceptSession":
            console.log("Offscreen: Received 'acceptSession' command", payload);
            // This message type should be handled by background script only
            console.warn("Offscreen: acceptSession should be handled by background script, not offscreen. Ignoring.");
            sendResponse({ success: true, message: "acceptSession ignored - should be handled by background script." });
            break;

        case "getState":
            console.log(`Offscreen: Received '${msgType}' command`, payload);
            if (webRTCManager && localPeerId) {
                const state = {
                    initialized: true,
                    localPeerId: localPeerId,
                    webrtcConnections: webrtcConnections, // Include tracked connections
                    sessionInfo: webRTCManager.sessionInfo,
                    invites: webRTCManager.invites,
                    dkgState: webRTCManager.dkgState,
                    meshStatus: webRTCManager.meshStatus,
                    dataChannelStatus: webRTCManager.getDataChannelStatus(),
                    connectedPeers: webRTCManager.getConnectedPeers(),
                    peerConnectionStatus: webRTCManager.getPeerConnectionStatus()
                };
                console.log("Offscreen: Sending combined state to background:", state);
                sendResponse({ success: true, data: state });
            } else {
                console.log("Offscreen: WebRTCManager not ready, sending uninitialized state.");
                sendResponse({ success: true, data: { initialized: false, localPeerId: localPeerId, webrtcConnections: {} } });
            }
            break;

        case "sendDirectMessage":
            console.log("Offscreen: Received 'sendDirectMessage' command", payload);
            if (webRTCManager && payload.toPeerId && payload.message) {
                const success = webRTCManager.sendDirectMessage(payload.toPeerId, payload.message);
                if (!success) {
                    console.warn(`Offscreen: Failed to send direct message to ${payload.toPeerId}`);
                }
                sendResponse({ success, message: success ? "Message sent" : "Failed to send message" });
            } else {
                const debugInfo = {
                    webRTCManagerReady: !!webRTCManager,
                    hasToPeerId: !!payload.toPeerId,
                    hasMessage: !!payload.message,
                    localPeerId,
                    payload
                };
                console.warn("Offscreen: Cannot send direct message - missing required data.", debugInfo);
                sendResponse({ success: false, error: "WebRTCManager not ready or missing toPeerId/message in payload.", debugInfo });
            }
            break;

        case "getWebRTCStatus":
            console.log("Offscreen: Received 'getWebRTCStatus' command", payload);
            if (webRTCManager) {
                const status = {
                    dataChannelStatus: webRTCManager.getDataChannelStatus(),
                    connectedPeers: webRTCManager.getConnectedPeers(),
                    peerConnectionStatus: webRTCManager.getPeerConnectionStatus(),
                    sessionInfo: webRTCManager.sessionInfo,
                    meshStatus: webRTCManager.meshStatus
                };
                console.log("Offscreen: Sending WebRTC status:", status);
                sendResponse({ success: true, data: status });
            } else {
                console.log("Offscreen: WebRTCManager not ready for status request.");
                sendResponse({ success: true, data: { initialized: false } });
            }
            break;

        case "getDkgStatus":
            console.log("Offscreen: Received 'getDkgStatus' command", payload);
            if (webRTCManager) {
                const dkgStatus = webRTCManager.getDkgStatus();
                console.log("Offscreen: Sending DKG status:", dkgStatus);
                sendResponse({ success: true, data: dkgStatus });
            } else {
                console.log("Offscreen: WebRTCManager not ready for DKG status request.");
                sendResponse({ success: true, data: { initialized: false } });
            }
            break;

        case "getGroupPublicKey":
            console.log("Offscreen: Received 'getGroupPublicKey' command", payload);
            if (webRTCManager) {
                const groupPublicKey = webRTCManager.getGroupPublicKey();
                console.log("Offscreen: Sending group public key:", groupPublicKey);
                sendResponse({ success: true, data: { groupPublicKey } });
            } else {
                console.log("Offscreen: WebRTCManager not ready for group public key request.");
                sendResponse({ success: false, error: "WebRTCManager not initialized" });
            }
            break;

        case "getSolanaAddress":
            console.log("Offscreen: Received 'getSolanaAddress' command", payload);
            if (webRTCManager) {
                const solanaAddress = webRTCManager.getSolanaAddress();
                console.log("Offscreen: Sending Solana address:", solanaAddress);
                sendResponse({ success: true, data: { solanaAddress } });
            } else {
                console.log("Offscreen: WebRTCManager not ready for Solana address request.");
                sendResponse({ success: false, error: "WebRTCManager not initialized" });
            }
            break;

        default:
            console.warn("Offscreen: Received unhandled message type from background:", msgType, payload);
            sendResponse({ success: false, error: `Unknown message type: ${msgType}` });
            break;
    }
    // Return true if sendResponse will be called asynchronously.
    // For most of these, sendResponse is called synchronously.
    return false;
});

// Signal to the background script that the offscreen document is ready
console.log("Offscreen: All listeners set up. Sending 'offscreenReady' to background.");

// Add a small delay to ensure background script is ready to receive messages

chrome.runtime.sendMessage({ type: "offscreenReady" }, (response) => {
    if (chrome.runtime.lastError) {
        console.error("Offscreen: Error sending 'offscreenReady' or receiving ack from background:", chrome.runtime.lastError.message);

        // Retry sending the ready signal if it failed
        setTimeout(() => {
            console.log("Offscreen: Retrying 'offscreenReady' signal...");
            chrome.runtime.sendMessage({ type: "offscreenReady" }, (retryResponse) => {
                if (chrome.runtime.lastError) {
                    console.error("Offscreen: Retry also failed:", chrome.runtime.lastError.message);

                    // Try one more time with longer delay
                    setTimeout(() => {
                        console.log("Offscreen: Final retry 'offscreenReady' signal...");
                        chrome.runtime.sendMessage({ type: "offscreenReady" }, (finalResponse) => {
                            if (chrome.runtime.lastError) {
                                console.error("Offscreen: Final retry failed:", chrome.runtime.lastError.message);
                            } else {
                                console.log("Offscreen: 'offscreenReady' final retry successful:", finalResponse);
                            }
                        });
                    }, 2000);
                } else {
                    console.log("Offscreen: 'offscreenReady' retry successful:", retryResponse);
                }
            });
        }, 1000);
    } else {
        console.log("Offscreen: 'offscreenReady' signal sent and acknowledged by background:", response);

        // Check if we received a successful response and expect init soon
        if (response && response.success) {
            // Set a timeout to check if init was received
            setTimeout(() => {
                if (!webRTCManager || !localPeerId) {
                    console.warn("Offscreen: Init data not received within expected time. WebRTCManager:", !!webRTCManager, "localPeerId:", localPeerId);
                    console.warn("Offscreen: This may indicate the background script failed to send init data.");

                    // Request init data manually
                    chrome.runtime.sendMessage({ type: "requestInit" }, (initResponse) => {
                        if (chrome.runtime.lastError) {
                            console.error("Offscreen: Error requesting init data:", chrome.runtime.lastError.message);
                        } else {
                            console.log("Offscreen: Init data request response:", initResponse);
                        }
                    });
                }
            }, 3000);
        }
    }
});


console.log("Offscreen document setup complete and active.");
