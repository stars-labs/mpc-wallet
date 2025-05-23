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
                    // Only send actual log messages, not WebRTC signaling data
                    if (!logMessage.includes('WebRTCSignal') && !logMessage.includes('Sending SDP') && !logMessage.includes('Sending ICE candidate')) {
                        sendToBackground({ type: "fromOffscreen", payload: { type: "log", message: `[Offscreen WebRTC] ${logMessage}` } });
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
                    meshStatus: webRTCManager.meshStatus
                };
                console.log("Offscreen: Sending combined state to background:", state);
                sendResponse({ success: true, data: state });
            } else {
                console.log("Offscreen: WebRTCManager not ready, sending uninitialized state.");
                sendResponse({ success: true, data: { initialized: false, localPeerId: localPeerId, webrtcConnections: {} } });
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
