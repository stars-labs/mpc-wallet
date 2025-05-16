/**
 * Diagnostic utilities for debugging WebRTC and session issues
 */

/**
 * Check WebRTC connection between participants
 * @param currentPeerId 
 * @param participants 
 * @param sessionId 
 */
export async function diagnoseWebRTCConnection(
    currentPeerId: string,
    participants: string[],
    sessionId: string
): Promise<string[]> {
    const diagnostics: string[] = [];

    // Check if the current peer is part of the session
    const isParticipant = participants.includes(currentPeerId);
    diagnostics.push(`Current peer ${currentPeerId} ${isParticipant ? 'is' : 'is NOT'} in participant list`);

    // Check other participants
    diagnostics.push(`Session ${sessionId} has ${participants.length} participants`);
    participants.forEach(peerId => {
        if (peerId !== currentPeerId) {
            diagnostics.push(`- Should connect to: ${peerId}`);
        }
    });

    // Log WebRTC configuration
    try {
        // Check for STUN server availability
        const rtcConfig = {
            iceServers: [
                { urls: 'stun:stun.l.google.com:19302' }
            ]
        };

        const pc = new RTCPeerConnection(rtcConfig);
        diagnostics.push('RTCPeerConnection created successfully');

        // Listen for ICE gathering state changes
        pc.onicegatheringstatechange = () => {
            diagnostics.push(`ICE gathering state: ${pc.iceGatheringState}`);
        };

        // Clean up
        setTimeout(() => {
            pc.close();
        }, 5000);
    } catch (error) {
        diagnostics.push(`Error creating RTCPeerConnection: ${error.message}`);
    }

    return diagnostics;
}

/**
 * Fix common session issues by cleaning up and resetting state
 */
export async function resetSessionState(): Promise<void> {
    try {
        // Send message to background script to reset WebRTC state
        chrome.runtime.sendMessage({
            type: "resetWebRTC"
        });

        console.log("Sent reset request to background script");
    } catch (error) {
        console.error("Failed to reset WebRTC state:", error);
    }
}

/**
 * Log diagnostic information about the current session and WebRTC state
 */
export async function logDiagnosticInfo(
    sessionInfo: any,
    peersList: string[],
    currentPeerId: string
): Promise<void> {
    console.group("MPC Wallet Diagnostic Info");
    console.log("Current Peer ID:", currentPeerId);
    console.log("Connected Peers:", peersList);
    console.log("Session Info:", sessionInfo);

    if (sessionInfo) {
        const diagnostics = await diagnoseWebRTCConnection(
            currentPeerId,
            sessionInfo.participants,
            sessionInfo.session_id
        );

        console.log("WebRTC Diagnostics:", diagnostics);
    }

    console.groupEnd();
}
