import { ClientMsg, ServerMsg, SessionInfo, DkgState, MeshStatus, WebRTCAppMessage, MeshStatusType, WebSocketMessagePayload, WebRTCSignal, CandidateInfo, SDPInfo } from "../../types/appstate";
// --- WebRTCManager Class ---
const ICE_SERVERS = [{ urls: 'stun:stun.l.google.com:19302' }]; // Example STUN server

export class WebRTCManager {
  private localPeerId: string;
  private peerConnections: Map<string, RTCPeerConnection> = new Map();
  private dataChannels: Map<string, RTCDataChannel> = new Map();

  public sessionInfo: SessionInfo | null = null;
  public invites: SessionInfo[] = []; // Store incoming session proposals/invites
  public dkgState: DkgState = DkgState.Idle;
  public meshStatus: MeshStatus = { type: MeshStatusType.Incomplete };
  private pendingIceCandidates: Map<string, RTCIceCandidateInit[]> = new Map();

  // Callbacks
  public onLog: (message: string) => void = console.log;
  public onSessionUpdate: (sessionInfo: SessionInfo | null, invites: SessionInfo[]) => void = () => { };
  public onMeshStatusUpdate: (status: MeshStatus) => void = () => { };
  public onWebRTCAppMessage: (fromPeerId: string, message: WebRTCAppMessage) => void = () => { };
  public onDkgStateUpdate: (state: DkgState) => void = () => { };

  // Add the missing callback property and constructor parameter
  private sendPayloadToBackgroundForRelay: ((toPeerId: string, payload: WebSocketMessagePayload) => void) | null = null;

  constructor(localPeerId: string, sendPayloadCallback?: (toPeerId: string, payload: WebSocketMessagePayload) => void) {

    if (typeof localPeerId !== 'string') {
      // Use console.warn for this initial setup phase, as _log depends on localPeerId which is being initialized.
      // JSON.stringify might fail for complex objects or circular refs, but good for simple ones.
      let localPeerIdStringRepresentation = '';
      try {
        localPeerIdStringRepresentation = JSON.stringify(localPeerId);
      } catch (e) {
        localPeerIdStringRepresentation = '[Unserializable Object]';
      }
      console.warn(`[WebRTCManager] Constructor: localPeerId expected a string but received type ${typeof localPeerId}. Value: ${localPeerIdStringRepresentation}`);

      if (localPeerId && typeof (localPeerId as any).id === 'string') {
        this.localPeerId = (localPeerId as any).id;
        console.warn(`[WebRTCManager] Constructor: Using 'id' property from localPeerId object: ${this.localPeerId}`);
      } else {
        this.localPeerId = String(localPeerId); // Fallback, may result in "[object Object]"
        console.warn(`[WebRTCManager] Constructor: Fallback: Converted localPeerId to string: ${this.localPeerId}. This may not be the intended ID. Please check instantiation site.`);
      }
    } else {
      this.localPeerId = localPeerId;
    }

    this.sendPayloadToBackgroundForRelay = sendPayloadCallback || null;
  }

  private _log(message: string) {
    this.onLog(`[WebRTCManager-${this.localPeerId}] ${message}`);
  }

  private _updateSession(newSessionInfo: SessionInfo | null) {
    this.sessionInfo = newSessionInfo;
    this.onSessionUpdate(this.sessionInfo, this.invites);
  }

  private _updateMeshStatus(newStatus: MeshStatus) {
    this.meshStatus = newStatus;
    this.onMeshStatusUpdate(this.meshStatus);
    if (newStatus.type === MeshStatusType.Ready) {
      this._log("Mesh is Ready! Checking DKG trigger conditions.");
      this.checkAndTriggerDkg();
    }
  }

  private _updateDkgState(newState: DkgState) {
    this.dkgState = newState;
    this.onDkgStateUpdate(this.dkgState);
  }

  public handleWebSocketMessagePayload(fromPeerId: string, msg: WebSocketMessagePayload): void {
    this._log(`Received WebSocketMessage from ${fromPeerId}: ${msg.websocket_msg_type}`);
    this._log(`Full message payload: ${JSON.stringify(msg)}`);

    switch (msg.websocket_msg_type) {
      case 'WebRTCSignal':
        this._log(`WebRTCSignal data: ${JSON.stringify(msg.data)}`);

        // Accept WebRTC signals from any peer - no session requirement
        this._log(`Processing WebRTC signal from ${fromPeerId} (no session check)`);

        // Handle different message structures
        let signalData = null;
        if (msg.data) {
          // Standard structure: { data: { type: "Offer/Answer/Candidate", data: {...} } }
          signalData = msg.data;
        } else if ((msg as any).Offer) {
          // Server structure: { Offer: {...}, websocket_msg_type: "WebRTCSignal" }
          signalData = { type: 'Offer', data: (msg as any).Offer };
        } else if ((msg as any).Answer) {
          // Server structure: { Answer: {...}, websocket_msg_type: "WebRTCSignal" }
          signalData = { type: 'Answer', data: (msg as any).Answer };
        } else if ((msg as any).Candidate) {
          // Server structure: { Candidate: {...}, websocket_msg_type: "WebRTCSignal" }
          signalData = { type: 'Candidate', data: (msg as any).Candidate };
        }

        if (signalData) {
          this._log(`Extracted WebRTC signal: ${JSON.stringify(signalData)}`);
          this.handleWebRTCSignal(fromPeerId, signalData as WebRTCSignal);
        } else {
          this._log(`WebRTCSignal from ${fromPeerId} missing data - full msg: ${JSON.stringify(msg)}`);
        }
        break;

      default:
        // Handle unknown message types with proper logging
        this._log(`Unknown WebSocketMessage type from ${fromPeerId}: ${(msg as any).websocket_msg_type}. Full payload: ${JSON.stringify(msg)}`);
        break;
    }
  }

  public async handleWebRTCSignal(fromPeerId: string, signal: any): Promise<void> {
    try {
      this._log(`handleWebRTCSignal called with: ${JSON.stringify(signal)}`);

      // Normalize signal structure
      let actualSignal = signal;
      if (signal && signal.type && signal.data) {
        actualSignal = signal;
      } else if (signal && (signal.sdp || signal.candidate)) {
        if (signal.sdp) {
          actualSignal = {
            type: signal.type || (signal.sdp.includes('a=sendrecv') ? 'Offer' : 'Answer'),
            data: { sdp: signal.sdp }
          };
        } else if (signal.candidate) {
          actualSignal = {
            type: 'Candidate',
            data: {
              candidate: signal.candidate,
              sdpMid: signal.sdpMid,
              sdpMLineIndex: signal.sdpMLineIndex
            }
          };
        }
      } else {
        this._log(`Invalid WebRTCSignal structure from ${fromPeerId}: ${JSON.stringify(signal)}`);
        return;
      }

      this._log(`Processing WebRTCSignal ${actualSignal.type} from ${fromPeerId}`);

      const pc = await this._getOrCreatePeerConnection(fromPeerId);
      if (!pc) {
        this._log(`No peer connection for ${fromPeerId} to handle signal.`);
        return;
      }

      // Comprehensive pattern matching for signal types
      switch (actualSignal.type) {
        case 'Offer':
          if (actualSignal.data && actualSignal.data.sdp) {
            await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp: actualSignal.data.sdp }));
            this._log(`Set remote offer from ${fromPeerId}. Creating answer.`);

            const answer = await pc.createAnswer();
            await pc.setLocalDescription(answer);

            const answerSignal: WebRTCSignal = { type: 'Answer', data: { sdp: answer.sdp! } };
            const wsMsgPayload: WebSocketMessagePayload = { websocket_msg_type: 'WebRTCSignal', data: answerSignal };

            if (this.sendPayloadToBackgroundForRelay) {
              this.sendPayloadToBackgroundForRelay(fromPeerId, wsMsgPayload);
              this._log(`Sent Answer to ${fromPeerId} via background`);
            } else {
              this._log(`Cannot send Answer to ${fromPeerId}: no relay callback available`);
            }
          } else {
            this._log(`Offer from ${fromPeerId} missing SDP data. Data: ${JSON.stringify(actualSignal.data)}`);
          }
          break;

        case 'Answer':
          if (actualSignal.data && actualSignal.data.sdp) {
            await pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp: actualSignal.data.sdp }));
            this._log(`Set remote answer from ${fromPeerId}. Connection should be established soon.`);
          } else {
            this._log(`Answer from ${fromPeerId} missing SDP data. Data: ${JSON.stringify(actualSignal.data)}`);
          }
          break;

        case 'Candidate':
          if (actualSignal.data && actualSignal.data.candidate) {
            const candidate = new RTCIceCandidate({
              candidate: actualSignal.data.candidate,
              sdpMid: actualSignal.data.sdpMid || null,
              sdpMLineIndex: actualSignal.data.sdpMLineIndex || null,
            });

            if (pc.remoteDescription) {
              await pc.addIceCandidate(candidate);
              this._log(`Added ICE candidate from ${fromPeerId}`);
            } else {
              this._log(`Queued ICE candidate from ${fromPeerId} (remote description not set)`);
              const pending = this.pendingIceCandidates.get(fromPeerId) || [];
              pending.push(candidate);
              this.pendingIceCandidates.set(fromPeerId, pending);
            }
          } else {
            this._log(`Candidate from ${fromPeerId} missing candidate data. Data: ${JSON.stringify(actualSignal.data)}`);
          }
          break;

        // Handle potential additional signal types
        case 'offer':
        case 'answer':
          this._log(`Received lowercase signal type '${actualSignal.type}' from ${fromPeerId}, converting to title case`);
          // Recursively handle with proper casing
          const normalizedSignal = {
            ...actualSignal,
            type: actualSignal.type.charAt(0).toUpperCase() + actualSignal.type.slice(1)
          };
          await this.handleWebRTCSignal(fromPeerId, normalizedSignal);
          break;

        default:
          this._log(`Unknown signal type '${actualSignal.type}' from ${fromPeerId}. Full signal: ${JSON.stringify(actualSignal)}`);
          break;
      }
    } catch (error) {
      this._log(`Error handling WebRTCSignal from ${fromPeerId}: ${error}. Signal: ${JSON.stringify(signal)}`);
    }
  }

  // --- Session Management ---
  public resetSession(): void {
    this._log("Resetting session state.");
    this.peerConnections.forEach(pc => pc.close());
    this.peerConnections.clear();
    this.dataChannels.clear();
    this.pendingIceCandidates.clear();
    this._updateSession(null);
    this.invites = []; // Clear invites as well
    this.onSessionUpdate(this.sessionInfo, this.invites);
    this._updateMeshStatus({ type: MeshStatusType.Incomplete });
    this._updateDkgState(DkgState.Idle);
  }

  public sendWebRTCAppMessage(toPeerId: string, message: WebRTCAppMessage): void {
    const dc = this.dataChannels.get(toPeerId);
    if (dc && dc.readyState === 'open') {
      dc.send(JSON.stringify(message));
      this._log(`Sent WebRTCAppMessage to ${toPeerId}: ${message.webrtc_msg_type}`);
    } else {
      this._log(`Cannot send WebRTCAppMessage to ${toPeerId}: data channel not open or doesn't exist.`);
    }
  }

  private _handleIncomingWebRTCAppMessage(fromPeerId: string, message: WebRTCAppMessage): void {
    // Internal handling of specific WebRTCAppMessages
    switch ((message as any).webrtc_msg_type) {
      case 'ChannelOpen':
        // Peer 'fromPeerId' is telling us their channel to us is open.
        // This confirms their side. Our side is confirmed by dc.onopen.
        this._log(`Peer ${fromPeerId} confirmed their data channel is open (sent ChannelOpen).`);
        // We might use this to confirm bi-directional readiness before MeshReady.
        // For now, our dc.onopen is the primary trigger for local readiness.
        this._checkMeshStatus(); // Re-check mesh status as peer confirmed their side.
        break;
      case 'MeshReady':
        if ('session_id' in message) {
          this._log(`Received MeshReady from ${fromPeerId} for session ${(message as any).session_id}.`);
          if (this.sessionInfo && this.sessionInfo.session_id === (message as any).session_id) {
            this._processPeerMeshReady(fromPeerId);
          } else {
            this._log(`MeshReady from ${fromPeerId} for unknown/stale session ${(message as any).session_id}.`);
          }
        } else {
          this._log(`MeshReady from ${fromPeerId} missing session_id field.`);
        }
        break;

      case 'DkgRound1Package':
        if ('package' in message) {
          this._log(`Received DkgRound1Package from ${fromPeerId}`);
          // TODO: Implement DKG Round 1 handling
          // this.handleDkgRound1Package(fromPeerId, message.package);
        } else {
          this._log(`DkgRound1Package from ${fromPeerId} missing package field.`);
        }
        break;

      case 'DkgRound2Package':
        if ('package' in message) {
          this._log(`Received DkgRound2Package from ${fromPeerId}`);
          // TODO: Implement DKG Round 2 handling
          // this.handleDkgRound2Package(fromPeerId, message.package);
        } else {
          this._log(`DkgRound2Package from ${fromPeerId} missing package field.`);
        }
        break;

      default:
        this._log(`Unknown WebRTCAppMessage type from ${fromPeerId}: ${(message as any).webrtc_msg_type}. Full message: ${JSON.stringify(message)}`);
        break;
    }
  }

  // --- Mesh Management ---
  private _checkMeshStatus(): void {
    if (!this.sessionInfo) {
      if (this.meshStatus.type !== MeshStatusType.Incomplete) {
        this._updateMeshStatus({ type: MeshStatusType.Incomplete });
      }
      return;
    }

    const expectedPeers = this.sessionInfo.participants.filter(p => p !== this.localPeerId);
    const openDataChannelsToPeers = expectedPeers.filter(p => {
      const dc = this.dataChannels.get(p);
      return dc && dc.readyState === 'open';
    });

    if (openDataChannelsToPeers.length === expectedPeers.length) {
      // All our outgoing/bidirectional channels are open. Now we need to know if others are ready.
      // If not already in PartiallyReady (meaning we sent our MeshReady) or Ready state, send our MeshReady.
      if (this.meshStatus.type === MeshStatusType.Incomplete) {
        this._log("All local data channels are open. Sending MeshReady signal.");
        const selfMeshReadyMsg: WebRTCAppMessage = {
          webrtc_msg_type: 'MeshReady',
          session_id: this.sessionInfo.session_id,
          peer_id: this.localPeerId
        };
        this.sessionInfo.participants.forEach(p => {
          if (p !== this.localPeerId) this.sendWebRTCAppMessage(p, selfMeshReadyMsg);
        });
        // We are now partially ready (our side is done, waiting for others)
        const readyPeersSet = new Set<string>([this.localPeerId]);
        this._updateMeshStatus({
          type: MeshStatusType.PartiallyReady,
          ready_peers: readyPeersSet,
          total_peers: this.sessionInfo.participants.length
        });
      }
    } else {
      // Not all local channels are open yet.
      if (this.meshStatus.type !== MeshStatusType.Incomplete) { // If we were ready/partially, but a channel dropped
        this._log("Not all local data channels are open. Mesh is Incomplete.");
        this._updateMeshStatus({ type: MeshStatusType.Incomplete });
      }
    }
  }

  private _processPeerMeshReady(peerId: string): void {
    if (!this.sessionInfo) return;

    if (this.meshStatus.type === MeshStatusType.Ready) {
      this._log(`Received MeshReady from ${peerId}, but mesh is already Ready.`);
      return;
    }

    let currentReadyPeers: Set<string>;
    if (this.meshStatus.type === MeshStatusType.PartiallyReady) {
      currentReadyPeers = this.meshStatus.ready_peers;
    } else { // Incomplete, this is the first MeshReady we process (likely our own if logic is tight) or from another peer
      currentReadyPeers = new Set();
      // If we are processing a MeshReady from another peer and we haven't sent ours,
      // it implies our local channels might not all be open yet, or _checkMeshStatus hasn't run.
      // This logic assumes _checkMeshStatus correctly transitions to PartiallyReady when local channels are open.
    }

    currentReadyPeers.add(peerId);
    this._log(`Peer ${peerId} is MeshReady. Total ready: ${currentReadyPeers.size}/${this.sessionInfo.participants.length}`);

    if (currentReadyPeers.size === this.sessionInfo.participants.length) {
      this._updateMeshStatus({ type: MeshStatusType.Ready });
      this._log("All peers are MeshReady! Mesh is fully established.");
    } else {
      this._updateMeshStatus({
        type: MeshStatusType.PartiallyReady,
        ready_peers: currentReadyPeers,
        total_peers: this.sessionInfo.participants.length
      });
    }
  }

  // --- DKG Placeholder ---
  public checkAndTriggerDkg(): void {
    if (this.sessionInfo && this.meshStatus.type === MeshStatusType.Ready && this.dkgState === DkgState.Idle) {
      this._log("Conditions met: Session active, Mesh ready, DKG idle. Triggering DKG Round 1.");
      this._updateDkgState(DkgState.Round1InProgress);
      // TODO: Implement actual DKG Round 1 logic (e.g., generating and sending packages)
      // For now, just log and move to next state for testing flow
      // this.sessionInfo.participants.forEach(p => {
      //     if (p !== this.localPeerId) {
      //         this.sendWebRTCAppMessage(p, { webrtc_msg_type: 'DkgRound1Package', package: { from: this.localPeerId, data: "round1_pkg" } });
      //     }
      // });
      // this.handleDkgRound1Package(this.localPeerId, { from: this.localPeerId, data: "round1_pkg" }); // Process self
    } else {
      this._log(`DKG trigger conditions not met. Session: ${!!this.sessionInfo}, Mesh: ${MeshStatusType[this.meshStatus.type]}, DKG: ${DkgState[this.dkgState]}`);
    }
  }

  // --- WebRTC Signaling and Connection ---
  private async _getOrCreatePeerConnection(peerId: string): Promise<RTCPeerConnection | null> {
    if (this.peerConnections.has(peerId)) {
      return this.peerConnections.get(peerId)!;
    }

    // No session check - allow connections to any peer
    this._log(`Creating new RTCPeerConnection for ${peerId} (no session requirement)`);
    const pc = new RTCPeerConnection({ iceServers: ICE_SERVERS });
    this.peerConnections.set(peerId, pc);

    pc.onicecandidate = (event) => {
      if (event.candidate) {
        const candidateInfo: CandidateInfo = {
          candidate: event.candidate.candidate,
          sdpMid: event.candidate.sdpMid,
          sdpMLineIndex: event.candidate.sdpMLineIndex,
        };
        const signal: WebRTCSignal = { type: 'Candidate', data: candidateInfo };
        const wsMsgPayload: WebSocketMessagePayload = { websocket_msg_type: 'WebRTCSignal', data: signal };

        // Use the callback to send via background
        if (this.sendPayloadToBackgroundForRelay) {
          this.sendPayloadToBackgroundForRelay(peerId, wsMsgPayload);
          this._log(`Sent ICE candidate to ${peerId} via background`);
        }
      }
    };

    pc.oniceconnectionstatechange = () => {
      this._log(`ICE connection state for ${peerId}: ${pc.iceConnectionState}`);
      if (pc.iceConnectionState === 'failed' || pc.iceConnectionState === 'disconnected' || pc.iceConnectionState === 'closed') {
        // Handle reconnection or cleanup
        this._log(`ICE connection for ${peerId} is ${pc.iceConnectionState}. Consider cleanup/reconnect.`);
      }
    };

    pc.onconnectionstatechange = () => {
      this._log(`Peer connection state for ${peerId}: ${pc.connectionState}`);
    };

    // For the peer that receives the offer, they set up ondatachannel
    pc.ondatachannel = (event) => {
      this._log(`Received data channel from ${peerId}`);
      const dc = event.channel;
      this._setupDataChannelHandlers(dc, peerId);
    };

    // Apply pending ICE candidates if any
    const pending = this.pendingIceCandidates.get(peerId);
    if (pending) {
      this._log(`Applying ${pending.length} pending ICE candidates for ${peerId}`);
      pending.forEach(candidate => pc.addIceCandidate(candidate).catch(e => this._log(`Error adding pending ICE candidate for ${peerId}: ${e}`)));
      this.pendingIceCandidates.delete(peerId);
    }

    return pc;
  }

  private async initiateWebRTCConnectionsForAllSessionParticipants(): Promise<void> {
    if (!this.sessionInfo) {
      this._log("No active session to initiate WebRTC connections for.");
      return;
    }
    this._log("Initiating WebRTC connections for all session participants...");
    for (const peerId of this.sessionInfo.participants) {
      if (peerId === this.localPeerId) continue;

      // Politeness: only the peer with the "smaller" ID initiates the offer.
      // This helps prevent glare (both peers sending offers simultaneously).
      if (this.localPeerId < peerId) {
        this._log(`Will initiate offer to ${peerId} (politeness).`);
        const pc = await this._getOrCreatePeerConnection(peerId);
        if (pc) {
          const dc = pc.createDataChannel(`dc-${this.localPeerId}-${peerId}`);
          this._setupDataChannelHandlers(dc, peerId);

          const offer = await pc.createOffer();
          await pc.setLocalDescription(offer);

          const offerSignal: WebRTCSignal = { type: 'Offer', data: { sdp: offer.sdp! } };
          const wsMsgPayload: WebSocketMessagePayload = { websocket_msg_type: 'WebRTCSignal', data: offerSignal };

          // Use the callback to send via background
          if (this.sendPayloadToBackgroundForRelay) {
            this.sendPayloadToBackgroundForRelay(peerId, wsMsgPayload);
            this._log(`Sent Offer to ${peerId} via background`);
          }
        }
      } else {
        this._log(`Will wait for offer from ${peerId} (politeness).`);
        await this._getOrCreatePeerConnection(peerId);
      }
    }
  }

  private _setupDataChannelHandlers(dc: RTCDataChannel, peerId: string): void {
    if (this.dataChannels.has(peerId)) {
      this._log(`Data channel for ${peerId} already exists. Label: ${dc.label}`);
      return;
    }
    this.dataChannels.set(peerId, dc);
    this._log(`Setting up data channel "${dc.label}" for peer ${peerId}`);

    dc.onopen = () => {
      this._log(`Data channel with ${peerId} opened.`);
      // Send ChannelOpen message to the peer
      const channelOpenMsg: WebRTCAppMessage = { webrtc_msg_type: 'ChannelOpen', peer_id: this.localPeerId };
      this.sendWebRTCAppMessage(peerId, channelOpenMsg);
      this._handleLocalChannelOpen(peerId); // Process our side of channel opening for mesh status
    };
    dc.onmessage = (event) => {
      try {
        const message: WebRTCAppMessage = JSON.parse(event.data as string);
        this._log(`Received WebRTCAppMessage from ${peerId}: ${(message as any).webrtc_msg_type}`);
        this.onWebRTCAppMessage(peerId, message); // Notify app/UI
        this._handleIncomingWebRTCAppMessage(peerId, message); // Internal handling
      } catch (e) {
        this._log(`Failed to parse WebRTCAppMessage from ${peerId}: ${e}`);
      }
    };
    dc.onclose = () => {
      this._log(`Data channel with ${peerId} closed.`);
      this.dataChannels.delete(peerId);
      // Update mesh status if a channel closes
      this._checkMeshStatus();
    };
    dc.onerror = (error) => {
      this._log(`Data channel error with ${peerId}: ${JSON.stringify(error)}`);
    };
  }

  private _handleLocalChannelOpen(peerId: string): void {
    // This is called when OUR data channel to peerId opens.
    // We need to check if this completes our part of the mesh.
    this._log(`Local data channel to ${peerId} is now open.`);
    this._checkMeshStatus();
  }
}

