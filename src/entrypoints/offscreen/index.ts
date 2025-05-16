import { WebRTCManager } from '../background/webrtc'; // Adjust path as needed
import type { WebSocketClient } from '../background/websocket'; // Adjust path as needed
// Note: WebSocketClient might not be directly needed here if background handles WebSocket

console.log("Options page script loaded.");

// Check if running in SSR/pre-render mode (Vite specific)
if (typeof import.meta !== 'undefined' && import.meta.env && import.meta.env.SSR) {
    console.log("Options page: Running in SSR/pre-render mode. Skipping client-side specific initialization.");
} else {
    // Not in SSR mode, proceed with client-side checks and initialization
    if (typeof window !== 'undefined' && typeof chrome !== 'undefined' && chrome.runtime && typeof chrome.runtime.connect === 'function') {
        const statusDiv = document.getElementById('status');
        const logOutputPre = document.getElementById('log-output');

        function log(message: string) {
            if (logOutputPre) {
                logOutputPre.textContent += `${new Date().toISOString()}: ${message}\n`;
            }
        }

        if (statusDiv) statusDiv.textContent = "Status: Connecting to background script...";

        // Establish a long-lived connection with the background script
        const backgroundPort = chrome.runtime.connect({ name: "options-webrtc" });
        log("Attempting to connect to background script via port 'options-webrtc'.");

        // Placeholder for WebRTCManager instance
        let webRTCManager: WebRTCManager | null = null;
        let localPeerId: string | null = null; // Will be provided by background

        backgroundPort.onMessage.addListener((message) => {
            log(`Message from background: ${JSON.stringify(message)}`);

            if (message.type === "init") {
                localPeerId = message.peerId;
                // The WebSocketClient instance is managed by the background script.
                // WebRTCManager in the options page will need a way to send signals
                // back to the background, which then relays them via WebSocket.
                // So, we pass a 'sendSignalCallback' to WebRTCManager.
                if (localPeerId) {
                    webRTCManager = new WebRTCManager(localPeerId, {
                        sendSignal: (to, signal) => {
                            backgroundPort.postMessage({ type: "relayRtcSignal", to, signal });
                        },
                        onDataChannelMessage: (from, data) => {
                            log(`Data from ${from}: ${JSON.stringify(data)}`);
                            // Handle incoming data channel messages if needed, or relay to background
                            backgroundPort.postMessage({ type: "rtcDataReceived", from, data });
                        },
                        onConnectionStateChange: (peerId, state) => {
                            log(`RTC connection with ${peerId} state: ${state}`);
                            backgroundPort.postMessage({ type: "rtcConnectionState", peerId, state });
                        }
                    });
                    // For now, let's assume WebRTCManager is adapted or we create a simplified one here.
                    // The key is that it needs to communicate back via `backgroundPort.postMessage`.
                    log(`WebRTC Manager initialized for peer ID: ${localPeerId}. Waiting for session info.`);
                    if (statusDiv) statusDiv.textContent = `Status: Initialized for Peer ID ${localPeerId}. Waiting for session.`;

                    // If there's an active session info provided during init, pass it.
                    if (message.sessionInfo) {
                        // webRTCManager?.setCurrentSession(message.sessionInfo);
                        log(`Session info received: ${JSON.stringify(message.sessionInfo)}`);
                    }

                } else {
                    log("Error: No peerId provided by background for WebRTCManager initialization.");
                    if (statusDiv) statusDiv.textContent = "Status: Error - No Peer ID from background.";
                }
            } else if (message.type === "handleRtcSignal") {
                if (webRTCManager) {
                    webRTCManager.handleWebSocketMessage(message.from, message.signal);
                } else {
                    log("WebRTCManager not initialized, cannot handle signal.");
                }
                log("Received RTC signal from background to handle (logic to be implemented).");
                // This is where you'd call your WebRTCManager's method to process the incoming signal
                // e.g., webRTCManager.handleSignal(message.from, message.signalData);
            } else if (message.type === "updateSession") {
                if (webRTCManager && message.sessionInfo) {
                    webRTCManager.setCurrentSession(message.sessionInfo);
                    log(`Session updated: ${JSON.stringify(message.sessionInfo)}`);
                }
                log(`Session update received: ${JSON.stringify(message.sessionInfo)} (logic to be implemented).`);
            }
            // Add more message handlers as needed
        });

        backgroundPort.onDisconnect.addListener(() => {
            log("Disconnected from background script.");
            if (statusDiv) statusDiv.textContent = "Status: Disconnected from background. Please reload.";
            // Handle cleanup or reconnection attempts if necessary
        });

        // Inform background that options page is ready (optional)
        backgroundPort.postMessage({ type: "optionsPageReady" });
        log("Sent 'optionsPageReady' to background.");

        // Placeholder for actual WebRTCManager integration.
        // You'll need to adapt your WebRTCManager or create one here that uses
        // `backgroundPort.postMessage` to send signals out and receives signals
        // via `backgroundPort.onMessage`.

        // Example of how WebRTCManager might be structured or adapted:
        class OptionsWebRTCManager {
            constructor(private localPeerId: string, private port: chrome.runtime.Port) {
                log(`[OptionsWebRTCManager] Initialized for ${localPeerId}`);
            }

            handleIncomingSignal(fromPeerId: string, signal: any) {
                log(`[OptionsWebRTCManager] Received signal from ${fromPeerId}: ${JSON.stringify(signal)}. Processing...`);
                // Actual WebRTC logic to process offer/answer/candidate
            }

            sendSignal(toPeerId: string, signal: any) {
                log(`[OptionsWebRTCManager] Sending signal to ${toPeerId} via background: ${JSON.stringify(signal)}`);
                this.port.postMessage({ type: 'webrtcSignalToRelay', payload: { to: toPeerId, signal } });
            }

            // ... other WebRTC methods (createOffer, createAnswer, addIceCandidate, etc.)
        }

        // This is a simplified example. Your actual WebRTCManager will be more complex.
        // The key is that all external communication (sending signals) goes through the `backgroundPort`.
        // And all incoming signals arrive via `backgroundPort.onMessage`.
    } else {
        // Log or handle the case where the script is not running in a browser environment
        // or chrome.runtime.connect is not available.
        console.log("Options page: Not running in a compatible browser environment or chrome.runtime.connect not available. Skipping client-side initialization.");
        if (typeof document !== 'undefined') {
            const statusDiv = document.getElementById('status');
            if (statusDiv) statusDiv.textContent = "Status: Extension APIs not available in this context.";
        }
    }
}

export default {};
