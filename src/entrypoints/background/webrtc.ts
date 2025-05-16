import type { WebSocketClient } from './websocket'; // Assuming WebSocketClient is exported
import { ClientMsg, ServerMsg, SessionInfo, DkgState, MeshStatus, WebRTCAppMessage, MeshStatusType, WebSocketMessagePayload, SessionProposal, SessionResponse, WebRTCSignal, CandidateInfo, SDPInfo } from "./types";
// --- Interfaces based on solnana-mpc-frost/src/protocal/signal.rs ---


// --- WebRTCManager Class ---
const ICE_SERVERS = [{ urls: 'stun:stun.l.google.com:19302' }]; // Example STUN server

export class WebRTCManager {
  private wsClient: WebSocketClient;
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


  constructor(wsClient: WebSocketClient, localPeerId: string) {
    this.wsClient = wsClient;
    this.localPeerId = localPeerId;
    this._setupWebSocketHandlers();
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

  private _setupWebSocketHandlers(): void {
    this.wsClient.onMessage((serverMsg: ServerMsg) => {
      if (serverMsg.type === 'relay') {
        const { from, data } = serverMsg;
        if (from) {
          try {
            // Data here is expected to be WebSocketMessagePayload
            const wsMsgPayload = data as WebSocketMessagePayload; // Assuming data is already parsed if wsClient does so, or parse here
            this._handleWebSocketMessagePayload(from, wsMsgPayload);
          } catch (error) {
            this._log(`Error parsing relayed WebSocketMessage from ${from}: ${error}`);
          }
        } else {
          this._log("Received relay message without 'from' field.");
        }
      }
      // Handle other ServerMsg types if necessary (e.g., 'peers', 'error')
    });
  }

  private _handleWebSocketMessagePayload(fromPeerId: string, msg: WebSocketMessagePayload): void {
    this._log(`Received WebSocketMessage from ${fromPeerId}: ${msg.websocket_msg_type}`);
    switch (msg.websocket_msg_type) {
      case 'SessionProposal':
        this.handleSessionProposal(fromPeerId, msg.data);
        break;
      case 'SessionResponse':
        this.handleSessionResponse(fromPeerId, msg.data);
        break;
      case 'WebRTCSignal':
        this.handleWebRTCSignal(fromPeerId, msg.data);
        break;
      default:
        this._log(`Unknown WebSocketMessage type from ${fromPeerId}`);
    }
  }

  // --- Session Management ---
  public proposeSession(sessionId: string, total: number, threshold: number, participants: string[]): void {
    if (this.sessionInfo) {
      this._log("Cannot propose a new session while another is active.");
      return;
    }
    const proposal: SessionProposal = { session_id: sessionId, total, threshold, participants };
    const wsMsgPayload: WebSocketMessagePayload = { websocket_msg_type: 'SessionProposal', data: proposal };

    const newSession: SessionInfo = {
      session_id: sessionId,
      proposer_id: this.localPeerId,
      total,
      threshold,
      participants,
      accepted_peers: [this.localPeerId] // Proposer auto-accepts
    };
    this._updateSession(newSession);
    this._updateMeshStatus({ type: MeshStatusType.Incomplete }); // Reset mesh status
    this._updateDkgState(DkgState.Idle); // Reset DKG state

    participants.forEach(peerId => {
      if (peerId !== this.localPeerId) {
        this.wsClient.relayMessage(peerId, wsMsgPayload);
        this._log(`Sent SessionProposal to ${peerId}`);
      }
    });
    this.initiateWebRTCConnectionsForAllSessionParticipants();
  }

  public acceptSession(sessionId: string): void {
    const inviteIndex = this.invites.findIndex(inv => inv.session_id === sessionId);
    if (inviteIndex === -1) {
      this._log(`Cannot accept session ${sessionId}: No such invite.`);
      return;
    }
    if (this.sessionInfo) {
      this._log(`Cannot accept session ${sessionId}: Already in session ${this.sessionInfo.session_id}.`);
      return;
    }

    const acceptedInvite = this.invites.splice(inviteIndex, 1)[0];

    // Update local session state
    const newSession: SessionInfo = {
      ...acceptedInvite,
      accepted_peers: [this.localPeerId, ...acceptedInvite.accepted_peers] // Add self to accepted peers
    };
    this._updateSession(newSession);
    this._updateMeshStatus({ type: MeshStatusType.Incomplete });
    this._updateDkgState(DkgState.Idle);

    this._log(`Accepted session ${sessionId}. Current accepted: ${newSession.accepted_peers.join(', ')}`);

    const response: SessionResponse = { session_id: sessionId, accepted: true };
    const wsMsgPayload: WebSocketMessagePayload = { websocket_msg_type: 'SessionResponse', data: response };

    // Notify all other participants (including proposer) of acceptance
    newSession.participants.forEach(peerId => {
      if (peerId !== this.localPeerId) {
        this.wsClient.relayMessage(peerId, wsMsgPayload);
        this._log(`Sent SessionResponse (accepted) to ${peerId} for session ${sessionId}`);
      }
    });
    this.onSessionUpdate(this.sessionInfo, this.invites); // Update UI
    this.initiateWebRTCConnectionsForAllSessionParticipants();
  }

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

  public handleSessionProposal(fromPeerId: string, proposal: SessionProposal): void {
    if (this.sessionInfo) {
      this._log(`Received session proposal from ${fromPeerId} for session ${proposal.session_id}, but already in session ${this.sessionInfo.session_id}. Ignoring.`);
      // Optionally, send a busy/reject message.
      return;
    }
    // Check if this peer is part of the proposed participants
    if (!proposal.participants.includes(this.localPeerId)) {
      this._log(`Received session proposal for session ${proposal.session_id} from ${fromPeerId}, but not a participant. Ignoring.`);
      return;
    }

    const invite: SessionInfo = {
      session_id: proposal.session_id,
      proposer_id: fromPeerId,
      total: proposal.total,
      threshold: proposal.threshold,
      participants: proposal.participants,
      accepted_peers: [] // Will be populated upon local acceptance and by SessionResponse messages
    };
    // Avoid duplicate invites
    if (!this.invites.find(inv => inv.session_id === invite.session_id)) {
      this.invites.push(invite);
      this._log(`Received SessionProposal from ${fromPeerId} for session ${invite.session_id}. Added to invites.`);
      this.onSessionUpdate(this.sessionInfo, this.invites); // Notify UI/App
    } else {
      this._log(`Duplicate SessionProposal from ${fromPeerId} for session ${invite.session_id}.`);
    }
  }

  public handleSessionResponse(fromPeerId: string, response: SessionResponse): void {
    this._log(`Received SessionResponse from ${fromPeerId} for session ${response.session_id}: accepted=${response.accepted}`);
    if (!this.sessionInfo || this.sessionInfo.session_id !== response.session_id) {
      // This might be a response to an invite we haven't accepted yet, or an old session.
      // If it's for a pending invite, we can record the early acceptance.
      const invite = this.invites.find(inv => inv.session_id === response.session_id);
      if (invite && response.accepted && !invite.accepted_peers.includes(fromPeerId)) {
        invite.accepted_peers.push(fromPeerId);
        this._log(`Recorded early acceptance from ${fromPeerId} for pending invite ${response.session_id}`);
        this.onSessionUpdate(this.sessionInfo, this.invites);
      } else {
        this._log(`SessionResponse for ${response.session_id} does not match current session or any pending invite. Ignoring or handling as stale.`);
      }
      return;
    }

    if (response.accepted) {
      if (!this.sessionInfo.accepted_peers.includes(fromPeerId)) {
        this.sessionInfo.accepted_peers.push(fromPeerId);
        this._log(`Peer ${fromPeerId} accepted session. Accepted peers: ${this.sessionInfo.accepted_peers.join(', ')}`);
        this._updateSession(this.sessionInfo); // Trigger update
      }
    } else {
      this._log(`Peer ${fromPeerId} rejected session ${response.session_id}. Tearing down session.`);
      // Handle session teardown: close peer connections, reset sessionInfo, etc.
      this.resetSession();
      return;
    }

    // Check if all participants have accepted
    if (this.sessionInfo.participants.every(p => this.sessionInfo!.accepted_peers.includes(p))) {
      this._log(`All participants have accepted session ${this.sessionInfo.session_id}.`);
      // Potentially trigger next step if not already handled by mesh readiness
      this.checkAndTriggerDkg();
    }
  }

  public async handleWebRTCSignal(fromPeerId: string, signal: any): Promise<void> {
    try {
      // Make sure we have a properly formatted signal
      if (!signal || !signal.type || !signal.data) {
        this._log(`Invalid WebRTCSignal from ${fromPeerId}: ${JSON.stringify(signal)}`);
        return;
      }

      this._log(`Handling WebRTCSignal ${signal.type} from ${fromPeerId}`);

      const pc = await this._getOrCreatePeerConnection(fromPeerId);
      if (!pc) {
        this._log(`No peer connection for ${fromPeerId} to handle signal.`);
        return;
      }

      switch (signal.type) {
        case 'Offer':
          if (signal.data.sdp) {
            await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp: signal.data.sdp }));
            this._log(`Set remote offer from ${fromPeerId}. Creating answer.`);

            const answer = await pc.createAnswer();
            await pc.setLocalDescription(answer);
            const answerSignal: WebRTCSignal = { type: 'Answer', data: { sdp: answer.sdp! } };
            const wsMsgPayload: WebSocketMessagePayload = { websocket_msg_type: 'WebRTCSignal', data: answerSignal };
            this.wsClient.relayMessage(fromPeerId, wsMsgPayload);
            this._log(`Sent Answer to ${fromPeerId}`);
          } else {
            this._log(`Offer from ${fromPeerId} missing SDP data`);
          }
          break;

        case 'Answer':
          if (signal.data.sdp) {
            await pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp: signal.data.sdp }));
            this._log(`Set remote answer from ${fromPeerId}. Connection should be established soon.`);
          } else {
            this._log(`Answer from ${fromPeerId} missing SDP data`);
          }
          break;

        case 'Candidate':
          if (signal.data.candidate) {
            const candidate = new RTCIceCandidate({
              candidate: signal.data.candidate,
              sdpMid: signal.data.sdpMid || null,
              sdpMLineIndex: signal.data.sdpMLineIndex || null,
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
            this._log(`Candidate from ${fromPeerId} missing candidate data`);
          }
          break;

        default:
          this._log(`Unknown signal type ${signal.type} from ${fromPeerId}`);
      }
    } catch (error) {
      this._log(`Error handling WebRTCSignal from ${fromPeerId}: ${error}`);
    }
  }

  // --- WebRTC Signaling and Connection ---
  private async _getOrCreatePeerConnection(peerId: string): Promise<RTCPeerConnection | null> {
    if (this.peerConnections.has(peerId)) {
      return this.peerConnections.get(peerId)!;
    }
    if (!this.sessionInfo || !this.sessionInfo.participants.includes(peerId)) {
      this._log(`Cannot create peer connection for ${peerId}: not a participant in current session.`);
      return null;
    }

    this._log(`Creating new RTCPeerConnection for ${peerId}`);
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
        this.wsClient.relayMessage(peerId, wsMsgPayload);
        this._log(`Sent ICE candidate to ${peerId}`);
      }
    };

    pc.oniceconnectionstatechange = () => {
      this._log(`ICE connection state for ${peerId}: ${pc.iceConnectionState}`);
      if (pc.iceConnectionState === 'failed' || pc.iceConnectionState === 'disconnected' || pc.iceConnectionState === 'closed') {
        // Handle reconnection or cleanup
        this._log(`ICE connection for ${peerId} is ${pc.iceConnectionState}. Consider cleanup/reconnect.`);
        // this.peerConnections.delete(peerId);
        // this.dataChannels.delete(peerId);
        // this._updateMeshStatus(...); // Re-evaluate mesh status
      }
    };

    pc.onconnectionstatechange = () => {
      this._log(`Peer connection state for ${peerId}: ${pc.connectionState}`);
      if (pc.connectionState === 'connected') {
        // This indicates general P2P connectivity, data channel opening is separate
      }
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
          // Create data channel before offer for two-way data channels
          this._log(`Creating data channel for ${peerId} before offer.`);
          const dc = pc.createDataChannel(`dc-${this.localPeerId}-${peerId}`);
          this._setupDataChannelHandlers(dc, peerId);

          const offer = await pc.createOffer();
          await pc.setLocalDescription(offer);
          const sdpInfo: SDPInfo = { sdp: offer.sdp! };
          const signal: WebRTCSignal = { type: 'Offer', data: sdpInfo };
          const wsMsgPayload: WebSocketMessagePayload = { websocket_msg_type: 'WebRTCSignal', data: signal };
          this.wsClient.relayMessage(peerId, wsMsgPayload);
          this._log(`Sent Offer to ${peerId}`);
        }
      } else {
        this._log(`Will wait for offer from ${peerId} (politeness).`);
        // Ensure PC exists to receive offer
        await this._getOrCreatePeerConnection(peerId);
      }
    }
  }

  private _setupDataChannelHandlers(dc: RTCDataChannel, peerId: string): void {
    if (this.dataChannels.has(peerId)) {
      this._log(`Data channel for ${peerId} already exists. Label: ${dc.label}`);
      // Potentially close old one or handle error
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
        this._log(`Received WebRTCAppMessage from ${peerId}: ${message.webrtc_msg_type}`);
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

  private _handleIncomingWebRTCAppMessage(fromPeerId: string, message: WebRTCAppMessage): void {
    // Internal handling of specific WebRTCAppMessages
    switch (message.webrtc_msg_type) {
      case 'ChannelOpen':
        // Peer 'fromPeerId' is telling us their channel to us is open.
        // This confirms their side. Our side is confirmed by dc.onopen.
        this._log(`Peer ${fromPeerId} confirmed their data channel is open (sent ChannelOpen).`);
        // We might use this to confirm bi-directional readiness before MeshReady.
        // For now, our dc.onopen is the primary trigger for local readiness.
        this._checkMeshStatus(); // Re-check mesh status as peer confirmed their side.
        break;
      case 'MeshReady':
        this._log(`Received MeshReady from ${fromPeerId} for session ${message.session_id}.`);
        if (this.sessionInfo && this.sessionInfo.session_id === message.session_id) {
          this._processPeerMeshReady(fromPeerId);
        } else {
          this._log(`MeshReady from ${fromPeerId} for unknown/stale session ${message.session_id}.`);
        }
        break;
      // Handle DKG messages if needed for state changes within WebRTCManager
      case 'DkgRound1Package':
        // this.handleDkgRound1Package(fromPeerId, message.package);
        break;
      case 'DkgRound2Package':
        // this.handleDkgRound2Package(fromPeerId, message.package);
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

  // TODO: Implement further DKG state transitions and message handling
  // public handleDkgRound1Package(fromPeerId: string, pkg: any) { ... }
  // public handleDkgRound2Package(fromPeerId: string, pkg: any) { ... }
}

