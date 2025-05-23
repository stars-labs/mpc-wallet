import { SessionInfo, DkgState, MeshStatus, WebRTCAppMessage, MeshStatusType } from "../../types/appstate";
import { WebSocketMessagePayload, WebRTCSignal } from '../../types/websocket';

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

  // FROST DKG integration
  private frostDkg: any | null = null;
  private participantIndex: number | null = null;
  private receivedRound1Packages: Set<string> = new Set();
  private receivedRound2Packages: Set<string> = new Set();

  // Callbacks
  public onLog: (message: string) => void = console.log;
  public onSessionUpdate: (sessionInfo: SessionInfo | null, invites: SessionInfo[]) => void = () => { };
  public onMeshStatusUpdate: (status: MeshStatus) => void = () => { };
  public onWebRTCAppMessage: (fromPeerId: string, message: WebRTCAppMessage) => void = () => { };
  public onDkgStateUpdate: (state: DkgState) => void = () => { };
  public onWebRTCConnectionUpdate: (peerId: string, connected: boolean) => void = () => { };

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
      // Only trigger DKG in browser environment (skip in tests/Node)
      if (typeof window !== 'undefined') {
        this.checkAndTriggerDkg().catch(error => {
          this._log(`Error triggering DKG: ${error}`);
        });
      } else {
        this._log('Skipping DKG trigger in non-browser environment');
      }
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
        this._log(`WebRTCSignal data: ${JSON.stringify(msg)}`);

        // Accept WebRTC signals from any peer - no session requirement
        this._log(`Processing WebRTC signal from ${fromPeerId} (no session check)`);

        // Handle different message structures
        let signalData = null;
        if ((msg as any).data) {
          // Standard structure: { data: { type: "Offer/Answer/Candidate", data: {...} } }
          signalData = (msg as any).data;
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
            // When receiving an offer, the offerer should have created the data channel
            // We'll receive it via ondatachannel event
            await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp: actualSignal.data.sdp }));
            this._log(`Set remote offer from ${fromPeerId}. Creating answer.`);

            const answer = await pc.createAnswer();
            await pc.setLocalDescription(answer);

            // Create WebSocketMessage that matches Rust enum structure exactly
            const wsMsgPayload = {
              websocket_msg_type: 'WebRTCSignal',
              Answer: { sdp: answer.sdp! }  // Direct at root level, no nesting
            };

            if (this.sendPayloadToBackgroundForRelay) {
              this.sendPayloadToBackgroundForRelay(fromPeerId, wsMsgPayload as any);
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
            // Fix: Handle empty string sdpMid but valid sdpMLineIndex
            const sdpMid = actualSignal.data.sdpMid && actualSignal.data.sdpMid.trim() !== ""
              ? actualSignal.data.sdpMid
              : null;

            // Keep original sdpMLineIndex value (0 is valid!)
            const sdpMLineIndex = actualSignal.data.sdpMLineIndex;

            const candidate = new RTCIceCandidate({
              candidate: actualSignal.data.candidate,
              sdpMid: sdpMid,
              sdpMLineIndex: sdpMLineIndex,
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

    // Report all connections as disconnected
    this.peerConnections.forEach((pc, peerId) => {
      this.onWebRTCConnectionUpdate(peerId, false);
      pc.close();
    });

    this.peerConnections.clear();
    this.dataChannels.clear();
    this.pendingIceCandidates.clear();
    this._updateSession(null);
    this.invites = [];
    this.onSessionUpdate(this.sessionInfo, this.invites);
    this._updateMeshStatus({ type: MeshStatusType.Incomplete });
    this._updateDkgState(DkgState.Idle);

    // Reset FROST DKG state
    this._resetDkgState();
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

  // --- Session Management ---
  public async startSession(sessionInfo: SessionInfo): Promise<void> {
    this._log(`Starting session: ${sessionInfo.session_id} with participants: ${sessionInfo.participants.join(', ')}`);

    // Ensure the proposer is marked as accepted
    if (!sessionInfo.accepted_peers.includes(this.localPeerId)) {
      sessionInfo.accepted_peers.push(this.localPeerId);
    }

    this._updateSession(sessionInfo);
    await this.initiateWebRTCConnectionsForAllSessionParticipants();
  }

  public async acceptSession(sessionInfo: SessionInfo): Promise<void> {
    this._log(`Accepting session: ${sessionInfo.session_id}`);

    // Remove from invites
    this.invites = this.invites.filter(invite => invite.session_id !== sessionInfo.session_id);

    // Ensure this peer is in the accepted peers list
    if (!sessionInfo.accepted_peers.includes(this.localPeerId)) {
      sessionInfo.accepted_peers.push(this.localPeerId);
    }

    this._updateSession(sessionInfo);

    // Trigger mesh status check after session acceptance
    this._log("Session accepted, checking mesh readiness conditions");
    this._checkMeshStatus();
  }

  // Add method to handle session updates from background
  public updateSessionInfo(updatedSessionInfo: SessionInfo): void {
    this._log(`Updating session info for: ${updatedSessionInfo.session_id}`);
    this._log(`Accepted peers updated: [${updatedSessionInfo.accepted_peers.join(', ')}]`);

    this.sessionInfo = updatedSessionInfo;

    // Check if mesh conditions are now met
    this._checkMeshStatus();
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

    // Check if all participants have accepted the session
    const allParticipantsAccepted = this.sessionInfo.participants.every(peerId =>
      this.sessionInfo!.accepted_peers.includes(peerId)
    );

    // Check if we have all required connections for the session
    const hasAllRequiredConnections = openDataChannelsToPeers.length === expectedPeers.length;

    this._log(`Mesh status check - Data channels: ${openDataChannelsToPeers.length}/${expectedPeers.length}, All accepted: ${allParticipantsAccepted}`);
    this._log(`Accepted peers: [${this.sessionInfo.accepted_peers.join(', ')}], All participants: [${this.sessionInfo.participants.join(', ')}]`);

    if (hasAllRequiredConnections && allParticipantsAccepted) {
      // All conditions met for mesh readiness
      if (this.meshStatus.type === MeshStatusType.Incomplete) {
        this._log("All conditions met for mesh readiness. Sending MeshReady signal to all peers.");
        this._sendMeshReadyToAllPeers();

        // Mark ourselves as ready
        const readyPeersSet = new Set<string>([this.localPeerId]);
        this._updateMeshStatus({
          type: MeshStatusType.PartiallyReady,
          ready_peers: readyPeersSet,
          total_peers: this.sessionInfo.participants.length
        });
      }
    } else {
      // Conditions not met
      if (this.meshStatus.type !== MeshStatusType.Incomplete) {
        this._log("Mesh readiness conditions not met. Resetting to Incomplete.");
        this._log(`Missing: ${!hasAllRequiredConnections ? 'data channels' : ''} ${!allParticipantsAccepted ? 'session acceptances' : ''}`);
        this._updateMeshStatus({ type: MeshStatusType.Incomplete });
      }
    }
  }

  private _sendMeshReadyToAllPeers(): void {
    if (!this.sessionInfo) return;

    const meshReadyMsg: WebRTCAppMessage = {
      webrtc_msg_type: 'MeshReady',
      session_id: this.sessionInfo.session_id,
      peer_id: this.localPeerId
    };

    this.sessionInfo.participants.forEach(peerId => {
      if (peerId !== this.localPeerId) {
        this.sendWebRTCAppMessage(peerId, meshReadyMsg);
        this._log(`Sent MeshReady signal to ${peerId}`);
      }
    });
  }

  // --- DKG Implementation ---
  public async checkAndTriggerDkg(): Promise<void> {
    this._log(`Checking DKG trigger conditions:`);
    this._log(`  - Session: ${!!this.sessionInfo} (${this.sessionInfo?.session_id})`);
    this._log(`  - Mesh Status: ${MeshStatusType[this.meshStatus.type]}`);
    this._log(`  - DKG State: ${DkgState[this.dkgState]}`);

    if (this.sessionInfo && this.meshStatus.type === MeshStatusType.Ready && this.dkgState === DkgState.Idle) {
      this._log("✅ All conditions met: Session active, Mesh ready, DKG idle. Triggering DKG Round 1.");
      await this._initializeDkg();
    } else {
      this._log(`❌ DKG trigger conditions not met:`);
      if (!this.sessionInfo) this._log(`   - Missing session info`);
      if (this.meshStatus.type !== MeshStatusType.Ready) this._log(`   - Mesh not ready: ${MeshStatusType[this.meshStatus.type]}`);
      if (this.dkgState !== DkgState.Idle) this._log(`   - DKG not idle: ${DkgState[this.dkgState]}`);
    }
  }

  private async _initializeDkg(): Promise<void> {
    if (!this.sessionInfo) {
      this._log("Cannot initialize DKG: no session info");
      return;
    }

    try {
      // Skip WASM DKG initialization in test environment
      if (typeof process !== 'undefined' && process.env.NODE_ENV === 'test') {
        this._log('Skipping FROST DKG initialization in test environment');
        return;
      }

      // Calculate participant index (1-based)
      this.participantIndex = this.sessionInfo.participants.indexOf(this.localPeerId) + 1;

      if (this.participantIndex <= 0) {
        throw new Error("Local peer not found in participants list");
      }

      this._log(`Initializing DKG with participant index: ${this.participantIndex}, total: ${this.sessionInfo.total}, threshold: ${this.sessionInfo.threshold}`);

      // Initialize the DKG with our parameters
      this.frostDkg.init_dkg(
        this.participantIndex,
        this.sessionInfo.total,
        this.sessionInfo.threshold
      );

      // Generate and broadcast Round 1 package
      await this._generateAndBroadcastRound1();

    } catch (error) {
      this._log(`Error initializing DKG: ${error}`);
      this._updateDkgState(DkgState.Failed);
    }
  }

  private _generateAndBroadcastRound1(): void {
    if (!this.frostDkg || !this.sessionInfo) {
      this._log("Cannot generate Round 1: DKG not initialized");
      return;
    }

    try {
      this._updateDkgState(DkgState.Round1InProgress);

      // Generate our Round 1 package
      const round1Package = this.frostDkg.generate_round1();
      this._log(`Generated Round 1 package: ${round1Package.substring(0, 100)}...`);

      // Mark ourselves as having provided Round 1 package
      this.receivedRound1Packages.add(this.localPeerId);

      // Broadcast to all other participants
      this.sessionInfo.participants.forEach(peerId => {
        if (peerId !== this.localPeerId) {
          const message: WebRTCAppMessage = {
            webrtc_msg_type: 'DkgRound1Package',
            package: {
              from: this.localPeerId,
              sender_index: this.participantIndex!,
              data: round1Package
            }
          };
          this.sendWebRTCAppMessage(peerId, message);
          this._log(`Sent Round 1 package to ${peerId}`);
        }
      });

      // Check if we can proceed to Round 2 (if we've received all packages)
      this._checkRound1Completion();

    } catch (error) {
      this._log(`Error generating Round 1 package: ${error}`);
      this._updateDkgState(DkgState.Failed);
    }
  }

  private _handleDkgRound1Package(fromPeerId: string, packageData: any): void {
    if (!this.frostDkg || !this.sessionInfo) {
      this._log(`Ignoring Round 1 package from ${fromPeerId}: DKG not initialized`);
      return;
    }

    if (this.dkgState !== DkgState.Round1InProgress) {
      this._log(`Ignoring Round 1 package from ${fromPeerId}: not in Round 1 state (current: ${DkgState[this.dkgState]})`);
      return;
    }

    try {
      this._log(`Processing Round 1 package from ${fromPeerId}, sender_index: ${packageData.sender_index}`);

      // Add the package to our DKG instance
      this.frostDkg.add_round1_package(packageData.sender_index, packageData.data);

      // Mark this peer as having provided their Round 1 package
      this.receivedRound1Packages.add(fromPeerId);

      this._log(`Received Round 1 packages from: [${Array.from(this.receivedRound1Packages).join(', ')}]`);

      // Check if we can proceed to Round 2
      this._checkRound1Completion();

    } catch (error) {
      this._log(`Error processing Round 1 package from ${fromPeerId}: ${error}`);
      this._updateDkgState(DkgState.Failed);
    }
  }

  private _checkRound1Completion(): void {
    if (!this.sessionInfo || !this.frostDkg) return;

    // Check if we've received Round 1 packages from all participants
    if (this.receivedRound1Packages.size === this.sessionInfo.participants.length) {
      this._log("All Round 1 packages received! Proceeding to Round 2.");

      if (this.frostDkg.can_start_round2()) {
        this._updateDkgState(DkgState.Round1Complete);
        this._generateAndBroadcastRound2();
      } else {
        this._log("Error: Cannot start Round 2 despite having all packages");
        this._updateDkgState(DkgState.Failed);
      }
    } else {
      this._log(`Waiting for Round 1 packages. Received: ${this.receivedRound1Packages.size}/${this.sessionInfo.participants.length}`);
    }
  }

  private _generateAndBroadcastRound2(): void {
    if (!this.frostDkg || !this.sessionInfo) {
      this._log("Cannot generate Round 2: DKG not initialized");
      return;
    }

    try {
      this._updateDkgState(DkgState.Round2InProgress);

      // Generate Round 2 packages
      const round2PackagesHex = this.frostDkg.generate_round2();
      this._log(`Generated Round 2 packages: ${round2PackagesHex.substring(0, 100)}...`);

      // Mark ourselves as having provided Round 2 packages
      this.receivedRound2Packages.add(this.localPeerId);

      // Broadcast to all other participants
      this.sessionInfo.participants.forEach(peerId => {
        if (peerId !== this.localPeerId) {
          const message: WebRTCAppMessage = {
            webrtc_msg_type: 'DkgRound2Package',
            package: {
              from: this.localPeerId,
              sender_index: this.participantIndex!,
              data: round2PackagesHex
            }
          };
          this.sendWebRTCAppMessage(peerId, message);
          this._log(`Sent Round 2 packages to ${peerId}`);
        }
      });

      // Check if we can finalize (if we've received all Round 2 packages)
      this._checkRound2Completion();

    } catch (error) {
      this._log(`Error generating Round 2 packages: ${error}`);
      this._updateDkgState(DkgState.Failed);
    }
  }

  private _handleDkgRound2Package(fromPeerId: string, packageData: any): void {
    if (!this.frostDkg || !this.sessionInfo) {
      this._log(`Ignoring Round 2 package from ${fromPeerId}: DKG not initialized`);
      return;
    }

    if (this.dkgState !== DkgState.Round2InProgress) {
      this._log(`Ignoring Round 2 package from ${fromPeerId}: not in Round 2 state (current: ${DkgState[this.dkgState]})`);
      return;
    }

    try {
      this._log(`Processing Round 2 package from ${fromPeerId}, sender_index: ${packageData.sender_index}`);

      // Add the package to our DKG instance
      this.frostDkg.add_round2_package(packageData.sender_index, packageData.data);

      // Mark this peer as having provided their Round 2 packages
      this.receivedRound2Packages.add(fromPeerId);

      this._log(`Received Round 2 packages from: [${Array.from(this.receivedRound2Packages).join(', ')}]`);

      // Check if we can finalize
      this._checkRound2Completion();

    } catch (error) {
      this._log(`Error processing Round 2 package from ${fromPeerId}: ${error}`);
      this._updateDkgState(DkgState.Failed);
    }
  }

  private _checkRound2Completion(): void {
    if (!this.sessionInfo || !this.frostDkg) return;

    // Check if we've received Round 2 packages from all participants
    if (this.receivedRound2Packages.size === this.sessionInfo.participants.length) {
      this._log("All Round 2 packages received! Finalizing DKG.");

      if (this.frostDkg.can_finalize()) {
        this._updateDkgState(DkgState.Round2Complete);
        this._finalizeDkg();
      } else {
        this._log("Error: Cannot finalize DKG despite having all packages");
        this._updateDkgState(DkgState.Failed);
      }
    } else {
      this._log(`Waiting for Round 2 packages. Received: ${this.receivedRound2Packages.size}/${this.sessionInfo.participants.length}`);
    }
  }

  private _finalizeDkg(): void {
    if (!this.frostDkg) {
      this._log("Cannot finalize DKG: DKG not initialized");
      return;
    }

    try {
      this._updateDkgState(DkgState.Finalizing);

      // Finalize the DKG protocol
      const groupPublicKey = this.frostDkg.finalize_dkg();
      this._log(`DKG finalized successfully! Group public key: ${groupPublicKey}`);

      // Get the derived address
      const solAddress = this.frostDkg.get_sol_address();
      this._log(`Generated Solana address: ${solAddress}`);

      this._updateDkgState(DkgState.Complete);

    } catch (error) {
      this._log(`Error finalizing DKG: ${error}`);
      this._updateDkgState(DkgState.Failed);
    }
  }

  private _resetDkgState(): void {
    this._log("Resetting DKG state");

    if (this.frostDkg) {
      this.frostDkg.free();
      this.frostDkg = null;
    }

    this.participantIndex = null;
    this.receivedRound1Packages.clear();
    this.receivedRound2Packages.clear();
  }

  // --- Status and Information Methods ---
  public getDataChannelStatus(): Record<string, string> {
    const status: Record<string, string> = {};
    this.dataChannels.forEach((dc, peerId) => {
      status[peerId] = dc.readyState;
    });
    return status;
  }

  public getConnectedPeers(): string[] {
    const connectedPeers: string[] = [];
    this.dataChannels.forEach((dc, peerId) => {
      if (dc.readyState === 'open') {
        connectedPeers.push(peerId);
      }
    });
    return connectedPeers;
  }

  public getPeerConnectionStatus(): Record<string, string> {
    const status: Record<string, string> = {};
    this.peerConnections.forEach((pc, peerId) => {
      status[peerId] = pc.connectionState;
    });
    return status;
  }

  public sendDirectMessage(toPeerId: string, message: string): boolean {
    const directMessage: WebRTCAppMessage = {
      webrtc_msg_type: 'DirectMessage',
      from: this.localPeerId,
      message: message,
      timestamp: Date.now() // Add required timestamp property
    };

    const dc = this.dataChannels.get(toPeerId);
    if (dc && dc.readyState === 'open') {
      try {
        dc.send(JSON.stringify(directMessage));
        this._log(`Sent direct message to ${toPeerId}: ${message}`);
        return true;
      } catch (error) {
        this._log(`Error sending direct message to ${toPeerId}: ${error}`);
        return false;
      }
    } else {
      this._log(`Cannot send direct message to ${toPeerId}: data channel not open or doesn't exist. State: ${dc?.readyState || 'none'}`);
      return false;
    }
  }

  // --- WebRTC Signaling and Connection ---
  private async _getOrCreatePeerConnection(peerId: string): Promise<RTCPeerConnection | null> {
    if (this.peerConnections.has(peerId)) {
      const existingPc = this.peerConnections.get(peerId)!;
      // Check if the existing connection is still usable
      if (existingPc.connectionState === 'closed' || existingPc.connectionState === 'failed') {
        this._log(`Existing connection to ${peerId} is ${existingPc.connectionState}, creating new one`);
        this.peerConnections.delete(peerId);
        this.dataChannels.delete(peerId);
      } else {
        this._log(`Reusing existing peer connection for ${peerId} (state: ${existingPc.connectionState})`);
        return existingPc;
      }
    }

    // No session check - allow connections to any peer
    this._log(`Creating WebRTC connection object for ${peerId}`);
    const pc = new RTCPeerConnection({ iceServers: ICE_SERVERS });
    this.peerConnections.set(peerId, pc);
    this._log(`Stored WebRTC connection object for ${peerId}`);

    pc.onicecandidate = (event) => {
      if (event.candidate) {
        // Create WebSocketMessage that matches Rust enum structure exactly
        const wsMsgPayload = {
          websocket_msg_type: 'WebRTCSignal',
          Candidate: {  // Direct at root level, no nesting
            candidate: event.candidate.candidate,
            sdpMid: event.candidate.sdpMid,
            sdpMLineIndex: event.candidate.sdpMLineIndex,
          }
        };

        // Use the callback to send via background
        if (this.sendPayloadToBackgroundForRelay) {
          this.sendPayloadToBackgroundForRelay(peerId, wsMsgPayload as any);
          this._log(`Sent ICE candidate to ${peerId} via background`);
        }
      }
    };

    pc.oniceconnectionstatechange = () => {
      this._log(`ICE connection state for ${peerId}: ${pc.iceConnectionState}`);
      if (pc.iceConnectionState === 'failed' || pc.iceConnectionState === 'disconnected' || pc.iceConnectionState === 'closed') {
        // Handle reconnection or cleanup
        this._log(`ICE connection for ${peerId} is ${pc.iceConnectionState}. Consider cleanup/reconnect.`);

        // Only report as disconnected and clean up if truly failed/closed, not just disconnected
        if (pc.iceConnectionState === 'failed' || pc.iceConnectionState === 'closed') {
          this.onWebRTCConnectionUpdate(peerId, false);
          this.peerConnections.delete(peerId);
          this.dataChannels.delete(peerId);
          this.pendingIceCandidates.delete(peerId);
        }
      }
    };

    pc.onconnectionstatechange = () => {
      this._log(`Connection state changed for ${peerId}: ${pc.connectionState}`);

      // Report connection status based on connection state
      const connected = pc.connectionState === 'connected';
      this.onWebRTCConnectionUpdate(peerId, connected);

      // Only clean up on truly failed/closed states, not on disconnected
      if (pc.connectionState === 'failed' || pc.connectionState === 'closed') {
        this._log(`Connection to ${peerId} ${pc.connectionState}. Cleaning up.`);
        this.peerConnections.delete(peerId);
        this.dataChannels.delete(peerId);
        this.pendingIceCandidates.delete(peerId);
      }
    };

    pc.ondatachannel = (event) => {
      this._log(`Received data channel from ${peerId}: ${event.channel.label}`);
      const dc = event.channel;

      // Only accept data channels with the expected "frost-dkg" label
      if (dc.label !== "frost-dkg") {
        this._log(`Rejecting data channel from ${peerId} with unexpected label '${dc.label}' - only accepting 'frost-dkg' channels`);
        dc.close();
        return;
      }

      // Check if we already have a data channel for this peer
      const existingDc = this.dataChannels.get(peerId);
      if (existingDc && existingDc.readyState === 'open') {
        // Use politeness pattern: accept channel based on peer ID comparison
        if (this.localPeerId < peerId) {
          // We have the smaller ID, we should be the offerer, reject incoming channel
          this._log(`Rejecting incoming frost-dkg data channel from ${peerId} - we should be the offerer (${this.localPeerId} < ${peerId})`);
          dc.close();
          return;
        } else {
          // They have the smaller ID, they should be the offerer, accept their channel
          this._log(`Accepting incoming frost-dkg data channel from ${peerId} - they should be the offerer (${peerId} < ${this.localPeerId})`);
          // Close our existing channel since we should accept theirs
          this._log(`Closing our existing frost-dkg data channel to accept theirs`);
          existingDc.onclose = null; // Prevent disconnect events
          existingDc.close();
        }
      }

      // If existing channel is closed/closing or doesn't exist, or we're accepting based on politeness, set up the new one
      if (!existingDc || existingDc.readyState === 'closed' || this.localPeerId > peerId) {
        if (existingDc && existingDc.readyState !== 'closed') {
          this._log(`Closing existing frost-dkg data channel for ${peerId} (state: ${existingDc.readyState}) before setting up new one`);
          // Don't set up onclose handler for the old channel to avoid triggering disconnect events
          existingDc.onclose = null;
          existingDc.close();
        }

        // Set up the received data channel
        this.dataChannels.set(peerId, dc);
        this._setupDataChannelHandlers(dc, peerId);
        this._log(`Set up received frost-dkg data channel for ${peerId} (politeness: ${this.localPeerId} > ${peerId})`);
      } else {
        this._log(`Not setting up incoming frost-dkg data channel from ${peerId} due to politeness pattern (${this.localPeerId} < ${peerId})`);
        dc.close();
      }
    };

    // Apply pending ICE candidates if any
    const pending = this.pendingIceCandidates.get(peerId);
    if (pending) {
      this._log(`Applying ${pending.length} pending ICE candidates for ${peerId}`);
      pending.forEach(candidate => pc.addIceCandidate(candidate).catch(e => this._log(`Error adding pending ICE candidate for ${peerId}: ${e}`)));
      this.pendingIceCandidates.delete(peerId);
    }

    this._log(`Created peer connection for ${peerId}`);
    return pc;
  }

  // Add missing method to initiate a peer connection with offer
  public async initiatePeerConnection(peerId: string): Promise<void> {
    this._log(`Initiating peer connection to ${peerId}`);

    try {
      const pc = await this._getOrCreatePeerConnection(peerId);
      if (!pc) {
        this._log(`Failed to create peer connection for ${peerId}`);
        return;
      }

      // Check if we already have a working data channel for this peer
      const existingDc = this.dataChannels.get(peerId);
      if (existingDc && existingDc.readyState === 'open') {
        this._log(`Already have open data channel to ${peerId}, not creating new offer`);
        return;
      }

      // Check if we're already in the process of connecting (have a connecting data channel)
      if (existingDc && existingDc.readyState === 'connecting') {
        this._log(`Data channel to ${peerId} is already ${existingDc.readyState}, waiting for it to open`);
        return;
      }

      // Apply politeness pattern: only create data channel if we have the smaller ID
      if (this.localPeerId >= peerId) {
        this._log(`Not creating data channel to ${peerId} due to politeness pattern (${this.localPeerId} >= ${peerId})`);
        return;
      }

      // Only create data channel and offer if we don't already have a working connection AND we should be the offerer
      if (!existingDc || existingDc.readyState === 'closed') {
        // Use standardized "frost-dkg" label to match Rust CLI nodes
        this._log(`Creating data channel for ${peerId} as offerer with label 'frost-dkg' (politeness: ${this.localPeerId} < ${peerId})`);
        const dc = pc.createDataChannel("frost-dkg", {
          ordered: true
        });
        this.dataChannels.set(peerId, dc);
        this._setupDataChannelHandlers(dc, peerId);

        // Create and send offer
        const offer = await pc.createOffer();
        await pc.setLocalDescription(offer);

        // Create WebSocketMessage that matches Rust enum structure exactly
        const wsMsgPayload = {
          websocket_msg_type: 'WebRTCSignal',
          Offer: { sdp: offer.sdp! }  // Direct at root level, no nesting
        };

        // Use the callback to send via background
        if (this.sendPayloadToBackgroundForRelay) {
          this.sendPayloadToBackgroundForRelay(peerId, wsMsgPayload as any);
          this._log(`Sent Offer to ${peerId} via background`);
        } else {
          this._log(`Cannot send Offer to ${peerId}: no relay callback available`);
        }
      }
    } catch (error) {
      this._log(`Error initiating peer connection to ${peerId}: ${error}`);
    }
  }

  private async initiateWebRTCConnectionsForAllSessionParticipants(): Promise<void> {
    if (!this.sessionInfo) {
      this._log("No active session to initiate WebRTC connections for.");
      return;
    }
    this._log("Initiating WebRTC connections for all session participants...");

    for (const peerId of this.sessionInfo.participants) {
      if (peerId === this.localPeerId) continue;

      // Check if we already have a working connection to this peer
      const existingDc = this.dataChannels.get(peerId);
      if (existingDc && existingDc.readyState === 'open') {
        this._log(`Already have open data channel to ${peerId}, skipping connection initiation`);
        continue;
      }

      const existingPc = this.peerConnections.get(peerId);
      if (existingPc && (existingPc.connectionState === 'connected' || existingPc.connectionState === 'connecting')) {
        this._log(`Already have ${existingPc.connectionState} peer connection to ${peerId}, skipping initiation`);
        continue;
      }

      // Enhanced politeness: always create peer connection, but only initiate if we have smaller ID
      await this._getOrCreatePeerConnection(peerId);

      if (this.localPeerId < peerId) {
        this._log(`Will initiate offer to ${peerId} (politeness: ${this.localPeerId} < ${peerId}).`);
        await this.initiatePeerConnection(peerId);
      } else {
        this._log(`Will wait for offer from ${peerId} (politeness: ${this.localPeerId} >= ${peerId}).`);
        // Peer connection is already created above, just wait for their offer and data channel
      }
    }
  }

  private _setupDataChannelHandlers(dc: RTCDataChannel, peerId: string): void {
    dc.onopen = () => {
      this._log(`Data channel '${dc.label}' opened with ${peerId}`);

      // Only process if this is still the current data channel for this peer
      const currentDc = this.dataChannels.get(peerId);
      if (currentDc === dc) {
        this._handleLocalChannelOpen(peerId);
        this.onWebRTCConnectionUpdate(peerId, true);
      } else {
        this._log(`Data channel '${dc.label}' opened with ${peerId} but it's no longer the current channel, ignoring`);
      }
    };

    dc.onclose = () => {
      this._log(`Data channel '${dc.label}' closed with ${peerId}`);

      // Only process disconnect if this is still the current data channel for this peer
      const currentDc = this.dataChannels.get(peerId);
      if (currentDc === dc) {
        this.onWebRTCConnectionUpdate(peerId, false);
      } else {
        this._log(`Data channel '${dc.label}' closed with ${peerId} but it's no longer the current channel, ignoring disconnect event`);
      }
    };

    dc.onerror = (event) => {
      this._log(`Data channel '${dc.label}' error with ${peerId}: ${event}`);

      // Only process error if this is still the current data channel for this peer
      const currentDc = this.dataChannels.get(peerId);
      if (currentDc === dc) {
        this.onWebRTCConnectionUpdate(peerId, false);
      } else {
        this._log(`Data channel '${dc.label}' error with ${peerId} but it's no longer the current channel, ignoring error event`);
      }
    };

    dc.onmessage = (event) => {
      this._log(`Received data from ${peerId} on channel '${dc.label}': ${event.data}`);

      // Process messages from any data channel, even if not current
      try {
        const message: WebRTCAppMessage = JSON.parse(event.data);
        this._handleIncomingWebRTCAppMessage(peerId, message);
      } catch (error) {
        this._log(`Error parsing message from ${peerId}: ${error}`);
      }
    };
  }

  private _handleLocalChannelOpen(peerId: string): void {
    // This is called when OUR data channel to peerId opens.
    // We need to check if this completes our part of the mesh.
    this._log(`Local data channel to ${peerId} is now open.`);
    this._checkMeshStatus();
  }

  // Update the _handleIncomingWebRTCAppMessage to handle DirectMessage
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

      case 'DirectMessage':
        // Handle direct messages between peers
        if ('message' in message && 'from' in message) {
          this._log(`Received direct message from ${fromPeerId}: ${(message as any).message}`);
          // Forward to the onWebRTCAppMessage callback for external handling
          this.onWebRTCAppMessage(fromPeerId, message);
        } else {
          this._log(`DirectMessage from ${fromPeerId} missing required fields.`);
        }
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
          this._handleDkgRound1Package(fromPeerId, (message as any).package);
        } else {
          this._log(`DkgRound1Package from ${fromPeerId} missing package field.`);
        }
        break;

      case 'DkgRound2Package':
        if ('package' in message) {
          this._log(`Received DkgRound2Package from ${fromPeerId}`);
          this._handleDkgRound2Package(fromPeerId, (message as any).package);
        } else {
          this._log(`DkgRound2Package from ${fromPeerId} missing package field.`);
        }
        break;

      default:
        this._log(`Unknown WebRTCAppMessage type from ${fromPeerId}: ${(message as any).webrtc_msg_type}. Full message: ${JSON.stringify(message)}`);
        break;
    }
  }

  // Add the missing _processPeerMeshReady method
  private _processPeerMeshReady(fromPeerId: string): void {
    this._log(`Processing MeshReady signal from ${fromPeerId}`);

    if (!this.sessionInfo) {
      this._log(`Cannot process MeshReady from ${fromPeerId}: no active session`);
      return;
    }

    // Initialize ready_peers set if needed, ensuring we include the local peer and the sender
    let readyPeers: Set<string>;

    if (this.meshStatus.type === MeshStatusType.PartiallyReady && this.meshStatus.ready_peers) {
      readyPeers = new Set(this.meshStatus.ready_peers);
    } else {
      // Initialize with local peer
      readyPeers = new Set([this.localPeerId]);
    }

    // Add the peer that sent the MeshReady signal
    readyPeers.add(fromPeerId);

    this._log(`Peer ${fromPeerId} is now mesh ready. Ready peers: [${Array.from(readyPeers).join(', ')}]`);

    // Check if all participants are now ready
    const allParticipantsReady = this.sessionInfo.participants.every(peerId =>
      readyPeers.has(peerId)
    );

    if (allParticipantsReady) {
      this._log("All participants are mesh ready! Transitioning to fully Ready state.");
      this._updateMeshStatus({
        type: MeshStatusType.Ready
      });
    } else {
      this._log(`Not all participants ready yet. Ready: ${readyPeers.size}/${this.sessionInfo.participants.length}`);
      this._updateMeshStatus({
        type: MeshStatusType.PartiallyReady,
        ready_peers: readyPeers,
        total_peers: this.sessionInfo.participants.length
      });
    }
  }

  // --- DKG Status Methods ---
  public getDkgStatus(): any {
    return {
      state: this.dkgState,
      stateName: DkgState[this.dkgState],
      participantIndex: this.participantIndex,
      sessionInfo: this.sessionInfo ? {
        session_id: this.sessionInfo.session_id,
        total: this.sessionInfo.total,
        threshold: this.sessionInfo.threshold,
        participants: this.sessionInfo.participants
      } : null,
      receivedRound1Packages: Array.from(this.receivedRound1Packages),
      receivedRound2Packages: Array.from(this.receivedRound2Packages),
      frostDkgInitialized: !!this.frostDkg
    };
  }

  public getGroupPublicKey(): string | null {
    if (!this.frostDkg || this.dkgState !== DkgState.Complete) {
      return null;
    }

    try {
      return this.frostDkg.get_group_public_key();
    } catch (error) {
      this._log(`Error getting group public key: ${error}`);
      return null;
    }
  }

  public getSolanaAddress(): string | null {
    if (!this.frostDkg || this.dkgState !== DkgState.Complete) {
      return null;
    }

    try {
      return this.frostDkg.get_sol_address();
    } catch (error) {
      this._log(`Error getting Solana address: ${error}`);
      return null;
    }
  }
}