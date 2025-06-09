import { SessionInfo, DkgState, MeshStatus, WebRTCAppMessage, MeshStatusType, SigningState } from "../../types/appstate";
import { WebSocketMessagePayload, WebRTCSignal } from '../../types/websocket';

export { DkgState, MeshStatusType, SigningState }; // Export DkgState and MeshStatusType

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

  // Mesh ready tracking to prevent duplicate signals
  private ownMeshReadySent: boolean = false;

  // FROST DKG integration
  private frostDkg: any | null = null;
  private participantIndex: number | null = null;
  private receivedRound1Packages: Set<string> = new Set();
  private receivedRound2Packages: Set<string> = new Set();
  private groupPublicKey: string | null = null;
  private solanaAddress: string | null = null;
  private walletAddress: string | null = null; // Generic address property for current blockchain
  private currentBlockchain: "ethereum" | "solana" = "solana"; // Store current blockchain selection

  // Package buffering for handling packages that arrive before DKG initialization
  private bufferedRound1Packages: Array<{ fromPeerId: string; packageData: any }> = [];
  private bufferedRound2Packages: Array<{ fromPeerId: string; packageData: any }> = [];

  // FROST Signing integration
  public signingState: SigningState = SigningState.Idle;
  private currentMessage: Uint8Array | null = null;
  private transactionData: any = null;
  private receivedCommitments: Set<string> = new Set();
  private receivedShares: Set<string> = new Set();
  private selectedSigners: string[] = [];
  private signingCommitments: Map<string, any> = new Map();
  private signingShares: Map<string, any> = new Map();
  private aggregatedSignature: any = null;

  // Callbacks
  public onLog: (message: string) => void = console.log;
  public onSessionUpdate: (sessionInfo: SessionInfo | null, invites: SessionInfo[]) => void = () => { };
  public onMeshStatusUpdate: (status: MeshStatus) => void = () => { };
  public onWebRTCAppMessage: (fromPeerId: string, message: WebRTCAppMessage) => void = () => { };
  public onDkgStateUpdate: (state: DkgState) => void = () => { };
  public onSigningStateUpdate: (state: SigningState) => void = () => { };
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
        // Use the stored blockchain parameter from the current session
        this._log(`Using blockchain parameter for mesh-triggered DKG: ${this.currentBlockchain}`);
        this.checkAndTriggerDkg(this.currentBlockchain).catch(error => {
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

  private _updateSigningState(newState: SigningState) {
    this.signingState = newState;
    this.onSigningStateUpdate(this.signingState);
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

    // Reset mesh ready flag to allow mesh_ready signals for new sessions
    this.ownMeshReadySent = false;
    this._log("Reset ownMeshReadySent flag for session reset");

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

    // Reset mesh ready flag for new session
    this.ownMeshReadySent = false;
    this._log("Reset ownMeshReadySent flag for new session (startSession)");

    // Ensure the proposer is marked as accepted
    if (!sessionInfo.accepted_peers.includes(this.localPeerId)) {
      sessionInfo.accepted_peers.push(this.localPeerId);
    }

    this._updateSession(sessionInfo);
    await this.initiateWebRTCConnectionsForAllSessionParticipants();
  }

  public async acceptSession(sessionInfo: SessionInfo): Promise<void> {
    this._log(`Accepting session: ${sessionInfo.session_id}`);

    // Reset mesh ready flag for new session
    this.ownMeshReadySent = false;
    this._log("Reset ownMeshReadySent flag for new session (acceptSession)");

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
    console.log("📢 updateSessionInfo CALLED with session:", updatedSessionInfo.session_id);
    console.log("📢 Accepted peers:", updatedSessionInfo.accepted_peers);

    this._log(`📢 Updating session info for: ${updatedSessionInfo.session_id}`);
    this._log(`Accepted peers updated: [${updatedSessionInfo.accepted_peers.join(', ')}]`);

    this.sessionInfo = updatedSessionInfo;

    console.log("📢 About to call _checkMeshStatus immediately");
    // Check if mesh conditions are now met immediately
    this._checkMeshStatus();

    console.log("📢 Scheduling delayed mesh check in 500ms");
    // Also schedule a delayed check in case data channels are still opening
    // This handles the race condition where sessionAllAccepted arrives before data channels are ready
    setTimeout(() => {
      console.log("⏰ DELAYED MESH CHECK - Checking mesh status after 500ms delay");
      this._log(`⏰ Delayed mesh status check after session update (handling potential race condition)`);
      this._checkMeshStatus();
    }, 500); // 500ms delay to allow data channels to open
  }

  public setBlockchain(blockchain: "ethereum" | "solana"): void {
    this._log(`Setting blockchain selection to: ${blockchain}`);
    this.currentBlockchain = blockchain;
    this._log(`Blockchain updated successfully - current selection: ${this.currentBlockchain}`);
  }

  // --- Mesh Management ---
  private _checkMeshStatus(): void {
    console.log("🔍 _checkMeshStatus CALLED - Starting mesh status check");

    if (!this.sessionInfo) {
      console.log("❌ _checkMeshStatus: No session info, resetting to Incomplete");
      if (this.meshStatus.type !== MeshStatusType.Incomplete) {
        this._updateMeshStatus({ type: MeshStatusType.Incomplete });
      }
      return;
    }

    console.log("✅ _checkMeshStatus: Session info available, proceeding with check");

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

    // Enhanced logging for debugging
    this._log(`=== MESH STATUS CHECK ===`);
    this._log(`Session ID: ${this.sessionInfo.session_id}`);
    this._log(`Local Peer ID: ${this.localPeerId}`);
    this._log(`Expected peers: [${expectedPeers.join(', ')}]`);
    this._log(`Open data channels to: [${openDataChannelsToPeers.join(', ')}]`);
    this._log(`Data channels status: ${openDataChannelsToPeers.length}/${expectedPeers.length}`);
    this._log(`All participants accepted: ${allParticipantsAccepted}`);
    this._log(`Accepted peers: [${this.sessionInfo.accepted_peers.join(', ')}]`);
    this._log(`All participants: [${this.sessionInfo.participants.join(', ')}]`);
    this._log(`Has all required connections: ${hasAllRequiredConnections}`);
    this._log(`Own mesh ready sent: ${this.ownMeshReadySent}`);
    this._log(`Current mesh status: ${this.meshStatus.type}`);

    // Detailed data channel status logging
    expectedPeers.forEach(peerId => {
      const dc = this.dataChannels.get(peerId);
      if (dc) {
        this._log(`Data channel to ${peerId}: readyState=${dc.readyState}, label=${dc.label}`);
      } else {
        this._log(`Data channel to ${peerId}: NOT FOUND`);
      }
    });

    if (hasAllRequiredConnections && allParticipantsAccepted) {
      console.log("🚀 MESH CONDITIONS MET! Both data channels and session acceptance complete");
      // All conditions met for mesh readiness
      if (!this.ownMeshReadySent) {
        console.log("🚀 SENDING MESH_READY NOW! All conditions met and mesh_ready not sent yet");
        this._log("🚀 ALL CONDITIONS MET FOR MESH READINESS! Sending MeshReady signal to all peers.");
        this._sendMeshReadyToAllPeers();

        // Set the flag to prevent duplicate signals
        this.ownMeshReadySent = true;
        this._log("✅ Set ownMeshReadySent flag to prevent duplicate mesh_ready signals");

        // Only update mesh status to PartiallyReady if we're still in Incomplete state
        // If we're already in a higher state due to receiving other peers' mesh_ready, don't downgrade
        if (this.meshStatus.type === MeshStatusType.Incomplete) {
          const readyPeersSet = new Set<string>([this.localPeerId]);
          this._updateMeshStatus({
            type: MeshStatusType.PartiallyReady,
            ready_peers: readyPeersSet,
            total_peers: this.sessionInfo.participants.length
          });
        }
      } else {
        console.log("⚠️ Mesh conditions met but mesh_ready already sent");
        this._log("⚠️ Mesh ready conditions met but mesh_ready already sent - no action needed");
      }
    } else {
      console.log("❌ MESH CONDITIONS NOT MET");
      console.log(`   Data channels ready: ${hasAllRequiredConnections} (${openDataChannelsToPeers.length}/${expectedPeers.length})`);
      console.log(`   Session accepted: ${allParticipantsAccepted}`);
      // Conditions not met
      this._log("❌ Mesh readiness conditions NOT MET");
      if (!hasAllRequiredConnections) {
        this._log(`   - Missing data channels: ${expectedPeers.length - openDataChannelsToPeers.length} of ${expectedPeers.length}`);
      }
      if (!allParticipantsAccepted) {
        this._log(`   - Missing session acceptances: ${this.sessionInfo.participants.length - this.sessionInfo.accepted_peers.length} of ${this.sessionInfo.participants.length}`);
      }

      if (this.meshStatus.type !== MeshStatusType.Incomplete) {
        this._log("Resetting mesh status to Incomplete.");
        this._updateMeshStatus({ type: MeshStatusType.Incomplete });
      }
    }
    this._log(`=== END MESH STATUS CHECK ===`);
  }

  private _sendMeshReadyToAllPeers(): void {
    console.log("📡 _sendMeshReadyToAllPeers CALLED - Starting to send mesh ready signals");

    if (!this.sessionInfo) {
      console.log("❌ Cannot send MeshReady: no session info");
      this._log("❌ Cannot send MeshReady: no session info");
      return;
    }

    this._log(`📡 SENDING MESH_READY SIGNALS to all peers`);
    this._log(`Session ID: ${this.sessionInfo.session_id}`);
    this._log(`Local Peer ID: ${this.localPeerId}`);
    this._log(`Target peers: [${this.sessionInfo.participants.filter(p => p !== this.localPeerId).join(', ')}]`);

    const meshReadyMsg: WebRTCAppMessage = {
      webrtc_msg_type: 'MeshReady',
      session_id: this.sessionInfo.session_id,
      peer_id: this.localPeerId
    };

    let sentCount = 0;
    this.sessionInfo.participants.forEach(peerId => {
      if (peerId !== this.localPeerId) {
        this.sendWebRTCAppMessage(peerId, meshReadyMsg);
        sentCount++;
        this._log(`✅ Sent MeshReady signal to ${peerId}`);
      }
    });

    this._log(`📡 MESH_READY SIGNALS SENT: ${sentCount} signals sent to peers`);
    this._log(`Message content: ${JSON.stringify(meshReadyMsg)}`);
  }

  // --- DKG Implementation ---
  public async checkAndTriggerDkg(blockchain: "ethereum" | "solana" = "solana"): Promise<void> {
    // Store the blockchain parameter for future use (like mesh-triggered DKG)
    this.currentBlockchain = blockchain;

    this._log(`Checking DKG trigger conditions:`);
    this._log(`  - Session: ${!!this.sessionInfo} (${this.sessionInfo?.session_id})`);
    this._log(`  - Mesh Status: ${MeshStatusType[this.meshStatus.type]}`);
    this._log(`  - DKG State: ${DkgState[this.dkgState]}`);
    this._log(`  - Blockchain: ${blockchain}`);

    if (this.sessionInfo && this.meshStatus.type === MeshStatusType.Ready && this.dkgState === DkgState.Idle) {
      this._log("✅ All conditions met: Session active, Mesh ready, DKG idle. Triggering DKG Round 1.");
      await this._initializeDkg(blockchain);
    } else {
      this._log(`❌ DKG trigger conditions not met:`);
      if (!this.sessionInfo) this._log(`   - Missing session info`);
      if (this.meshStatus.type !== MeshStatusType.Ready) this._log(`   - Mesh not ready: ${MeshStatusType[this.meshStatus.type]}`);
      if (this.dkgState !== DkgState.Idle) this._log(`   - DKG not idle: ${DkgState[this.dkgState]}`);
    }
  }

  private async _initializeDkg(blockchain: "ethereum" | "solana" = "solana"): Promise<void> {
    if (!this.sessionInfo) {
      this._log("Cannot initialize DKG: no session info");
      return;
    }

    try {
      // Calculate participant index (1-based)
      this.participantIndex = this.sessionInfo.participants.indexOf(this.localPeerId) + 1;

      if (this.participantIndex <= 0) {
        throw new Error("Local peer not found in participants list");
      }

      this._log(`🔧 DEBUG: Local peer: ${this.localPeerId}, participants: [${this.sessionInfo.participants.join(', ')}]`);
      this._log(`🔧 DEBUG: Calculated participant index: ${this.participantIndex}`);
      this._log(`Initializing DKG with participant index: ${this.participantIndex}, total: ${this.sessionInfo.total}, threshold: ${this.sessionInfo.threshold}`);

      // Initialize FROST DKG instance if not already done
      if (!this.frostDkg) {
        // In test environment, use the WASM module passed from test setup
        if (typeof process !== 'undefined' && process.env.NODE_ENV === 'test') {
          // The test should have set this.frostDkg already with a real WASM instance
          if (!this.frostDkg) {
            throw new Error("Test environment requires frostDkg to be initialized before calling _initializeDkg");
          }
        } else {
          // In browser environment, dynamically import and initialize WASM
          const wasmModule = await import('../../../pkg/mpc_wallet.js');
          await wasmModule.default(); // Initialize WASM

          // Use the appropriate DKG class based on blockchain selection
          if (blockchain === "ethereum") {
            this.frostDkg = new wasmModule.FrostDkgSecp256k1();
            this._log('FROST DKG WASM module initialized successfully for Ethereum (Secp256k1)');
          } else {
            this.frostDkg = new wasmModule.FrostDkg(); // FrostDkg is Ed25519 (for Solana)
            this._log('FROST DKG WASM module initialized successfully for Solana (Ed25519)');
          }
        }
      }

      // Initialize the DKG with our parameters
      this.frostDkg.init_dkg(
        this.participantIndex,
        this.sessionInfo.total,
        this.sessionInfo.threshold
      );

      // Generate and broadcast Round 1 package first (this sets the state to Round1InProgress)
      this._generateAndBroadcastRound1();

      // Now replay any buffered packages that arrived before DKG was initialized
      this._replayBufferedPackages();

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

      this._log(`🔧 DEBUG: About to call generate_round1() with participant index: ${this.participantIndex}`);
      // Generate our Round 1 package
      const round1PackageHex = this.frostDkg.generate_round1();
      this._log(`🔧 DEBUG: generate_round1() completed, package length: ${round1PackageHex.length}`);
      this._log(`Generated Round 1 package hex: ${round1PackageHex.substring(0, 100)}...`);

      // Decode hex to JSON to get the proper FROST package structure
      const round1PackageJson = this._hexToJson(round1PackageHex);
      this._log(`Decoded Round 1 package: ${JSON.stringify(round1PackageJson).substring(0, 200)}...`);

      // Mark ourselves as having provided Round 1 package
      this.receivedRound1Packages.add(this.localPeerId);

      // Broadcast to all other participants
      this.sessionInfo.participants.forEach(peerId => {
        if (peerId !== this.localPeerId) {
          const message: WebRTCAppMessage = {
            webrtc_msg_type: 'DkgRound1Package',
            package: round1PackageJson // Send the parsed JSON structure, not wrapped
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

  private _hexToJson(hexString: string): any {
    try {
      // Remove "0x" prefix if present
      const cleanHex = hexString.startsWith('0x') ? hexString.slice(2) : hexString;

      // Convert hex to bytes
      const bytes = new Uint8Array(cleanHex.length / 2);
      for (let i = 0; i < cleanHex.length; i += 2) {
        bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
      }

      // Convert bytes to string and parse as JSON
      const jsonString = new TextDecoder().decode(bytes);
      return JSON.parse(jsonString);
    } catch (error) {
      this._log(`Error decoding hex to JSON: ${error}`);
      throw error;
    }
  }

  private _handleDkgRound1Package(fromPeerId: string, packageData: any): void {
    // If DKG is not initialized yet, buffer the package for later processing
    if (!this.frostDkg || !this.sessionInfo) {
      this._log(`🔴 DKG not initialized yet, buffering Round 1 package from ${fromPeerId}`);
      this._log(`🔴 Current state: frostDkg=${!!this.frostDkg}, sessionInfo=${!!this.sessionInfo}, dkgState=${DkgState[this.dkgState]}`);
      this.bufferedRound1Packages.push({ fromPeerId, packageData });
      this._log(`🔴 Total buffered Round 1 packages: ${this.bufferedRound1Packages.length}`);
      return;
    }

    if (this.dkgState !== DkgState.Round1InProgress) {
      this._log(`🟡 Ignoring Round 1 package from ${fromPeerId}: not in Round 1 state (current: ${DkgState[this.dkgState]})`);
      return;
    }

    try {
      let senderIndex: number;
      let packageHex: string;

      // Handle different package formats:
      // 1. Old format from CLI nodes: { sender_index: number, data: string }
      // 2. New format from CLI nodes: { header: {...}, commitment: [...], proof_of_knowledge: string }
      if (packageData.sender_index !== undefined && packageData.data !== undefined) {
        // Old format with explicit sender_index and hex data
        senderIndex = packageData.sender_index;
        packageHex = packageData.data;
        this._log(`Processing Round 1 package from ${fromPeerId} (old format), sender_index: ${senderIndex}`);
      } else if (packageData.header && packageData.commitment && packageData.proof_of_knowledge) {
        // New format - direct FROST package structure, need to determine sender index
        // and serialize to hex
        const peerIndex = this.sessionInfo.participants.indexOf(fromPeerId);
        if (peerIndex === -1) {
          throw new Error(`Peer ${fromPeerId} not found in session participants`);
        }
        senderIndex = peerIndex + 1; // FROST uses 1-based indexing

        // Convert the FROST package back to hex for WASM
        const packageJson = JSON.stringify(packageData);
        packageHex = Array.from(new TextEncoder().encode(packageJson))
          .map(b => b.toString(16).padStart(2, '0'))
          .join('');

        this._log(`Processing Round 1 package from ${fromPeerId} (new format), calculated sender_index: ${senderIndex}`);
      } else {
        throw new Error(`Invalid package format from ${fromPeerId}: ${JSON.stringify(packageData)}`);
      }

      // **CRITICAL FIX**: Actually add the package to the DKG instance
      this._log(`Adding Round 1 package from ${fromPeerId} (sender_index: ${senderIndex}) to DKG instance`);
      this.frostDkg.add_round1_package(senderIndex, packageHex);

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
    this._log("🔍 _checkRound1Completion: Method called");
    if (!this.sessionInfo || !this.frostDkg) {
      this._log("🔍 _checkRound1Completion: Early return - missing sessionInfo or frostDkg");
      return;
    }

    this._log(`🔍 _checkRound1Completion: Checking completion. Received: ${this.receivedRound1Packages.size}/${this.sessionInfo.participants.length}`);
    this._log(`🔍 _checkRound1Completion: Session participants: [${this.sessionInfo.participants.join(', ')}]`);
    this._log(`🔍 _checkRound1Completion: Received packages from: [${Array.from(this.receivedRound1Packages).join(', ')}]`);
    this._log(`🔍 _checkRound1Completion: Current DKG state: ${DkgState[this.dkgState]}`);

    // Check if we've received Round 1 packages from all participants
    if (this.receivedRound1Packages.size === this.sessionInfo.participants.length) {
      this._log("All Round 1 packages received! Proceeding to Round 2.");

      this._log("🔍 _checkRound1Completion: Checking if can_start_round2()");
      this._log(`🔧 DEBUG: About to call can_start_round2() - current participant index: ${this.participantIndex}`);
      const canStartRound2 = this.frostDkg.can_start_round2();
      this._log(`🔧 DEBUG: can_start_round2() returned: ${canStartRound2}`);
      if (canStartRound2) {
        this._log("🔍 _checkRound1Completion: can_start_round2() returned true, updating state and calling _generateAndBroadcastRound2");
        this._updateDkgState(DkgState.Round1Complete);
        this._generateAndBroadcastRound2();
      } else {
        this._log("Error: Cannot start Round 2 despite having all packages");
        this._updateDkgState(DkgState.Failed);
      }
    } else {
      this._log(`Waiting for Round 1 packages. Received: ${this.receivedRound1Packages.size}/${this.sessionInfo.participants.length}`);
      this._log(`🔍 _checkRound1Completion: Missing packages from: [${this.sessionInfo.participants.filter(p => !this.receivedRound1Packages.has(p)).join(', ')}]`);
    }
  }

  private _generateAndBroadcastRound2(): void {
    this._log("🔄 _generateAndBroadcastRound2: Method called");

    if (!this.frostDkg || !this.sessionInfo) {
      this._log("Cannot generate Round 2: DKG not initialized");
      return;
    }

    this._log("🔄 _generateAndBroadcastRound2: Starting Round 2 generation");

    try {
      this._log("🔄 _generateAndBroadcastRound2: Updating state to Round2InProgress");
      this._updateDkgState(DkgState.Round2InProgress);

      // Generate Round 2 packages map
      this._log("🔄 _generateAndBroadcastRound2: Calling WASM generate_round2()");
      const round2PackagesHex = this.frostDkg.generate_round2();
      this._log(`Generated Round 2 packages: ${round2PackagesHex.substring(0, 100)}...`);

      // Decode hex to JSON to get the map of packages for each participant
      const round2PackagesMap = this._hexToJson(round2PackagesHex);
      this._log(`Decoded Round 2 packages map with ${Object.keys(round2PackagesMap).length} entries`);
      this._log(`Round 2 package keys: ${Object.keys(round2PackagesMap).join(', ')}`);

      // Mark ourselves as having provided Round 2 packages
      this.receivedRound2Packages.add(this.localPeerId);

      // Send individual packages to each recipient (similar to Round 1 approach)
      this.sessionInfo.participants.forEach(peerId => {
        if (peerId !== this.localPeerId) {
          const recipientIndex = this.sessionInfo!.participants.indexOf(peerId) + 1; // FROST uses 1-based indexing
          this._log(`Looking for Round 2 package for ${peerId} (participant index ${recipientIndex})`);

          // Find the package for this recipient in the map
          let recipientPackage = null;
          let recipientKey = null;

          // Try each key in the map to find the one for this recipient
          for (const [hexKey, packageData] of Object.entries(round2PackagesMap)) {
            this._log(`Checking key ${hexKey} for recipient ${peerId}`);

            // Try simple approach first - check if any keys match the pattern we expect
            try {
              // Convert hex string to bytes array (browser-compatible alternative to Buffer)
              const keyBytes = new Uint8Array(hexKey.length / 2);
              for (let i = 0; i < hexKey.length; i += 2) {
                keyBytes[i / 2] = parseInt(hexKey.substr(i, 2), 16);
              }
              this._log(`Key ${hexKey} decoded to ${keyBytes.length} bytes: ${Array.from(keyBytes).slice(0, 8).join(',')}`);

              // Handle different key encodings based on blockchain/curve
              let participantIndex: number;

              if (this.currentBlockchain === "ethereum") {
                // For Secp256k1 (Ethereum), the participant index is encoded as a u16 in the last 2 bytes (big-endian)
                if (keyBytes.length === 32) {
                  // Extract the last 2 bytes and interpret as big-endian u16
                  participantIndex = (keyBytes[30] << 8) | keyBytes[31];
                  this._log(`Extracted participant index ${participantIndex} from Secp256k1 key ${hexKey} (last 2 bytes, big-endian)`);
                } else {
                  this._log(`Unexpected Secp256k1 key length: ${keyBytes.length} bytes`);
                  continue;
                }
              } else {
                // For Ed25519 (Solana), try little-endian u16 at start
                if (keyBytes.length >= 2) {
                  participantIndex = keyBytes[0] | (keyBytes[1] << 8);
                  this._log(`Extracted participant index ${participantIndex} from Ed25519 key ${hexKey} (little-endian u16)`);
                } else {
                  this._log(`Unexpected Ed25519 key length: ${keyBytes.length} bytes`);
                  continue;
                }
              }

              if (participantIndex === recipientIndex) {
                recipientPackage = packageData;
                recipientKey = hexKey;
                this._log(`Found matching package for ${peerId}: key ${hexKey}, participant ${participantIndex}`);
                break;
              }
            } catch (error) {
              this._log(`Error parsing key ${hexKey}: ${error}`);
            }
          }

          if (recipientPackage) {
            const message: WebRTCAppMessage = {
              webrtc_msg_type: 'DkgRound2Package',
              package: recipientPackage // Send the direct FROST package structure
            };
            this.sendWebRTCAppMessage(peerId, message);
            this._log(`✅ Sent Round 2 package to ${peerId} (index ${recipientIndex}, key ${recipientKey})`);
          } else {
            this._log(`❌ Error: No Round 2 package found for ${peerId} (index ${recipientIndex})`);
            this._log(`Available keys in map: ${Object.keys(round2PackagesMap).join(', ')}`);
            // Don't fail immediately, continue trying other recipients
          }
        }
      });

      // Check if we can finalize (if we've received all Round 2 packages)
      this._checkRound2Completion();

    } catch (error) {
      this._log(`❌ Error generating Round 2 packages: ${error}`);
      this._log(`Error type: ${typeof error}`);
      if (error && typeof error === 'object') {
        this._log(`Error properties: ${Object.getOwnPropertyNames(error).join(', ')}`);
        if ('message' in error) {
          this._log(`WASM error message: ${(error as any).message}`);
        }
        if ('stack' in error) {
          this._log(`Error stack: ${(error as any).stack}`);
        }
      }
      this._updateDkgState(DkgState.Failed);
    }
  }

  private _handleDkgRound2Package(fromPeerId: string, packageData: any): void {
    // If DKG is not initialized yet, buffer the package for later processing
    if (!this.frostDkg || !this.sessionInfo) {
      this._log(`DKG not initialized yet, buffering Round 2 package from ${fromPeerId}`);
      this.bufferedRound2Packages.push({ fromPeerId, packageData });
      return;
    }

    if (this.dkgState !== DkgState.Round2InProgress) {
      this._log(`Ignoring Round 2 package from ${fromPeerId}: not in Round 2 state (current: ${DkgState[this.dkgState]})`);
      return;
    }

    try {
      let senderIndex: number;
      let packageHex: string;

      // Handle both old and new package formats
      if (packageData.sender_index !== undefined && packageData.data !== undefined) {
        // Old format with explicit sender_index and hex data
        senderIndex = packageData.sender_index;
        packageHex = packageData.data;
        this._log(`Processing Round 2 package from ${fromPeerId} (old format), sender_index: ${senderIndex}`);
      } else if (packageData.header && packageData.signing_share) {
        // New format - direct FROST package structure, need to determine sender index
        // and serialize to hex
        const peerIndex = this.sessionInfo!.participants.indexOf(fromPeerId);
        if (peerIndex === -1) {
          throw new Error(`Peer ${fromPeerId} not found in session participants`);
        }
        senderIndex = peerIndex + 1; // FROST uses 1-based indexing

        // Convert the FROST package back to hex for WASM
        const packageJson = JSON.stringify(packageData);
        packageHex = Array.from(new TextEncoder().encode(packageJson))
          .map(b => b.toString(16).padStart(2, '0'))
          .join('');

        this._log(`Processing Round 2 package from ${fromPeerId} (new format), calculated sender_index: ${senderIndex}`);
      } else {
        throw new Error(`Invalid package format from ${fromPeerId}: ${JSON.stringify(packageData)}`);
      }

      // Add the package to our DKG instance
      this.frostDkg.add_round2_package(senderIndex, packageHex);

      // Mark this peer as having provided their Round 2 packages
      this.receivedRound2Packages.add(fromPeerId);

      this._log(`Received Round 2 packages from: [${Array.from(this.receivedRound2Packages).join(', ')}]`);

      // Check if we can finalize
      this._checkRound2Completion();

    } catch (error) {
      this._log(`Error processing Round 2 package from ${fromPeerId}: ${(error as Error).message}`);
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

    // Prevent multiple finalization attempts, but allow retry if previous attempt failed
    if (this.dkgState === DkgState.Complete) {
      this._log("DKG already completed, skipping finalization");
      return;
    }

    if (this.dkgState === DkgState.Finalizing) {
      this._log("DKG finalization already in progress");
      // Don't return here - allow it to proceed and potentially fail
    }

    try {
      this._updateDkgState(DkgState.Finalizing);
      this._log("Starting DKG finalization...");

      // Finalize the DKG protocol
      const groupPublicKey = this.frostDkg.finalize_dkg();
      this._log(`DKG finalized successfully! Group public key: ${groupPublicKey}`);

      // Store the group public key as a property
      this.groupPublicKey = groupPublicKey;

      // Update state to complete first
      this._updateDkgState(DkgState.Complete);

      // Now try to get the derived address based on blockchain type
      try {
        let walletAddress: string;
        if (this.currentBlockchain === "ethereum") {
          walletAddress = this.frostDkg.get_eth_address();
          this._log(`Generated Ethereum address: ${walletAddress}`);
        } else {
          walletAddress = this.frostDkg.get_sol_address();
          this._log(`Generated Solana address: ${walletAddress}`);
          // Also store in legacy property for backward compatibility
          this.solanaAddress = walletAddress;
        }

        // Store the address in a generic property 
        this.walletAddress = walletAddress;

      } catch (addressError) {
        this._log(`Warning: DKG completed but failed to generate ${this.currentBlockchain} address: ${(addressError as any).message || addressError}`);
        // Don't fail the entire DKG for address generation issues
      }

    } catch (error) {
      this._log(`Error finalizing DKG: ${(error as any).message || error}`);
      this._log(`Error details: ${JSON.stringify(error)}`);
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
    this.groupPublicKey = null;
    this.solanaAddress = null;
    this.currentBlockchain = "solana"; // Reset to default blockchain

    // Clear package buffers when resetting DKG state
    this.bufferedRound1Packages = [];
    this.bufferedRound2Packages = [];
  }

  private _replayBufferedPackages(): void {
    this._log(`🔄 Replaying buffered packages: ${this.bufferedRound1Packages.length} Round 1, ${this.bufferedRound2Packages.length} Round 2`);
    this._log(`🔄 Current DKG state before replay: ${DkgState[this.dkgState]}`);
    this._log(`🔄 Current receivedRound1Packages before replay: [${Array.from(this.receivedRound1Packages).join(', ')}]`);

    // Replay buffered Round 1 packages
    const round1Buffer = [...this.bufferedRound1Packages]; // Copy array to avoid modification during iteration
    this.bufferedRound1Packages = []; // Clear buffer

    for (const { fromPeerId, packageData } of round1Buffer) {
      this._log(`🔄 Replaying buffered Round 1 package from ${fromPeerId}`);
      this._handleDkgRound1Package(fromPeerId, packageData);
    }

    this._log(`🔄 After Round 1 replay - receivedRound1Packages: [${Array.from(this.receivedRound1Packages).join(', ')}]`);

    // Replay buffered Round 2 packages
    const round2Buffer = [...this.bufferedRound2Packages]; // Copy array to avoid modification during iteration
    this.bufferedRound2Packages = []; // Clear buffer

    for (const { fromPeerId, packageData } of round2Buffer) {
      this._log(`🔄 Replaying buffered Round 2 package from ${fromPeerId}`);
      this._handleDkgRound2Package(fromPeerId, packageData);
    }

    this._log(`🔄 Replay completed. Final receivedRound1Packages: [${Array.from(this.receivedRound1Packages).join(', ')}]`);
  }

  // --- FROST Signing Methods ---

  public initiateSigning(txData: any, blockchain: "ethereum" | "solana" = "solana"): boolean {
    this._log(`Initiating FROST signing for ${blockchain} transaction`);

    // Check if DKG is complete
    if (this.dkgState !== DkgState.Complete) {
      this._log(`Cannot start signing: DKG not complete (current state: ${DkgState[this.dkgState]})`);
      return false;
    }

    // Check if mesh is ready
    if (this.meshStatus.type !== MeshStatusType.Ready) {
      this._log(`Cannot start signing: Mesh not ready (current status: ${this.meshStatus.type})`);
      return false;
    }

    // Reset signing state
    this._resetSigningState();

    // Store transaction data and blockchain
    this.transactionData = txData;
    this.currentBlockchain = blockchain;

    // Update state
    this._updateSigningState(SigningState.TransactionComposition);

    // Broadcast SignTx message to all peers
    const signTxMessage = {
      webrtc_msg_type: 'SignTx' as const,
      tx_data: txData,
      blockchain: blockchain
    };

    this._broadcastToAllPeers(signTxMessage);
    this._log(`Broadcasted SignTx message to all peers`);

    return true;
  }

  private _handleSignTx(fromPeerId: string, txData: any): void {
    this._log(`Handling SignTx from ${fromPeerId}`);

    // Validate DKG state
    if (this.dkgState !== DkgState.Complete) {
      this._log(`Ignoring SignTx: DKG not complete`);
      return;
    }

    // Store transaction data
    this.transactionData = txData;
    this._updateSigningState(SigningState.SigningCommitment);

    // Generate signing commitment
    this._generateSigningCommitment();
  }

  private _generateSigningCommitment(): void {
    this._log(`Generating signing commitment`);

    try {
      // Get participant info
      if (!this.groupPublicKey || this.participantIndex === null) {
        this._log(`Missing group public key or participant index for signing commitment`);
        this._updateSigningState(SigningState.Failed);
        return;
      }

      // Import the appropriate WASM module
      let frostCommit;
      if (this.currentBlockchain === "ethereum") {
        frostCommit = (window as any).eth_sign?.frost_round1?.commit;
      } else {
        frostCommit = (window as any).sol_sign?.frost_round1?.commit;
      }

      if (!frostCommit) {
        this._log(`FROST commit function not available for ${this.currentBlockchain}`);
        this._updateSigningState(SigningState.Failed);
        return;
      }

      // Generate commitment
      const commitment = frostCommit(this.participantIndex);

      // Store our commitment
      this.signingCommitments.set(this.localPeerId, commitment);

      // Broadcast commitment to all peers
      const commitmentMessage = {
        webrtc_msg_type: 'SignCommitment' as const,
        commitment: commitment,
        participant_index: this.participantIndex
      };

      this._broadcastToAllPeers(commitmentMessage);
      this._log(`Broadcasted signing commitment to all peers`);

    } catch (error) {
      this._log(`Error generating signing commitment: ${(error as any).message || error}`);
      this._updateSigningState(SigningState.Failed);
    }
  }

  private _handleSignCommitment(fromPeerId: string, commitment: any): void {
    this._log(`Handling SignCommitment from ${fromPeerId}`);

    // Store the commitment
    this.signingCommitments.set(fromPeerId, commitment);
    this.receivedCommitments.add(fromPeerId);

    this._log(`Received ${this.receivedCommitments.size} commitments so far`);

    // Check if we have all commitments (including our own)
    const expectedPeers = this._getExpectedPeers();
    if (this.receivedCommitments.size >= expectedPeers.length) {
      this._log(`All commitments received, proceeding to signer selection`);
      this._performSignerSelection();
    }
  }

  private _performSignerSelection(): void {
    this._log(`Performing signer selection`);

    // For now, use all available participants as signers
    // In a production system, this could be more sophisticated
    this.selectedSigners = this._getExpectedPeers();

    this._updateSigningState(SigningState.SignerSelection);

    // Broadcast signer selection
    const signerSelectionMessage = {
      webrtc_msg_type: 'SignerSelection' as const,
      signers: this.selectedSigners
    };

    this._broadcastToAllPeers(signerSelectionMessage);
    this._log(`Broadcasted signer selection: [${this.selectedSigners.join(', ')}]`);

    // Proceed to signature generation
    this._generateSignature();
  }

  private _handleSignerSelection(fromPeerId: string, signers: string[]): void {
    this._log(`Handling SignerSelection from ${fromPeerId}: [${signers.join(', ')}]`);

    // Update selected signers (should be consistent across all peers)
    this.selectedSigners = signers;
    this._updateSigningState(SigningState.SignerSelection);

    // If we're in the signer list, generate our signature
    if (signers.includes(this.localPeerId)) {
      this._generateSignature();
    }
  }

  private _generateSignature(): void {
    this._log(`Generating signature`);

    try {
      if (!this.transactionData || this.participantIndex === null) {
        this._log(`Missing transaction data or participant index for signature generation`);
        this._updateSigningState(SigningState.Failed);
        return;
      }

      this._updateSigningState(SigningState.SignatureGeneration);

      // Import the appropriate WASM module
      let frostSign;
      if (this.currentBlockchain === "ethereum") {
        frostSign = (window as any).eth_sign?.frost_round2?.sign;
      } else {
        frostSign = (window as any).sol_sign?.frost_round2?.sign;
      }

      if (!frostSign) {
        this._log(`FROST sign function not available for ${this.currentBlockchain}`);
        this._updateSigningState(SigningState.Failed);
        return;
      }

      // Generate signature share
      const messageBytes = typeof this.transactionData === 'string'
        ? new TextEncoder().encode(this.transactionData)
        : this.transactionData;

      const signatureShare = frostSign(messageBytes, this.participantIndex);

      // Store our signature share
      this.signingShares.set(this.localPeerId, signatureShare);

      // Broadcast signature share to all peers
      const signShareMessage = {
        webrtc_msg_type: 'SignShare' as const,
        share: signatureShare,
        participant_index: this.participantIndex
      };

      this._broadcastToAllPeers(signShareMessage);
      this._log(`Broadcasted signature share to all peers`);

    } catch (error) {
      this._log(`Error generating signature: ${(error as any).message || error}`);
      this._updateSigningState(SigningState.Failed);
    }
  }

  private _handleSignShare(fromPeerId: string, share: any): void {
    this._log(`Handling SignShare from ${fromPeerId}`);

    // Store the signature share
    this.signingShares.set(fromPeerId, share);
    this.receivedShares.add(fromPeerId);

    this._log(`Received ${this.receivedShares.size} signature shares so far`);

    // Check if we have enough shares to aggregate
    if (this.receivedShares.size >= this.selectedSigners.length) {
      this._log(`All signature shares received, proceeding to aggregation`);
      this._aggregateSignatures();
    }
  }

  private _aggregateSignatures(): void {
    this._log(`Aggregating signatures`);

    try {
      this._updateSigningState(SigningState.SignatureAggregation);

      // Import the appropriate WASM module
      let frostAggregate;
      if (this.currentBlockchain === "ethereum") {
        frostAggregate = (window as any).eth_sign?.frost?.aggregate;
      } else {
        frostAggregate = (window as any).sol_sign?.frost?.aggregate;
      }

      if (!frostAggregate) {
        this._log(`FROST aggregate function not available for ${this.currentBlockchain}`);
        this._updateSigningState(SigningState.Failed);
        return;
      }

      // Collect all signature shares
      const shares = Array.from(this.signingShares.values());

      // Aggregate signatures
      const aggregatedSignature = frostAggregate(shares);
      this.aggregatedSignature = aggregatedSignature;

      // Broadcast aggregated signature
      const signAggregatedMessage = {
        webrtc_msg_type: 'SignAggregated' as const,
        signature: aggregatedSignature
      };

      this._broadcastToAllPeers(signAggregatedMessage);
      this._log(`Broadcasted aggregated signature to all peers`);

      this._updateSigningState(SigningState.SignatureVerification);
      this._verifyAggregatedSignature();

    } catch (error) {
      this._log(`Error aggregating signatures: ${(error as any).message || error}`);
      this._updateSigningState(SigningState.Failed);
    }
  }

  private _handleSignAggregated(fromPeerId: string, signature: any): void {
    this._log(`Handling SignAggregated from ${fromPeerId}`);

    // Store the aggregated signature
    this.aggregatedSignature = signature;
    this._updateSigningState(SigningState.SignatureVerification);

    // Verify the signature
    this._verifyAggregatedSignature();
  }

  private _verifyAggregatedSignature(): void {
    this._log(`Verifying aggregated signature`);

    try {
      if (!this.aggregatedSignature || !this.groupPublicKey || !this.transactionData) {
        this._log(`Missing signature, group public key, or transaction data for verification`);
        this._updateSigningState(SigningState.Failed);
        return;
      }

      // Import the appropriate WASM module for verification
      let frostVerify;
      if (this.currentBlockchain === "ethereum") {
        frostVerify = (window as any).eth_sign?.frost?.verify;
      } else {
        frostVerify = (window as any).sol_sign?.frost?.verify;
      }

      if (!frostVerify) {
        this._log(`FROST verify function not available for ${this.currentBlockchain}`);
        this._updateSigningState(SigningState.Failed);
        return;
      }

      // Verify signature
      const messageBytes = typeof this.transactionData === 'string'
        ? new TextEncoder().encode(this.transactionData)
        : this.transactionData;

      const isValid = frostVerify(this.aggregatedSignature, messageBytes, this.groupPublicKey);

      if (isValid) {
        this._log(`✅ Signature verification successful!`);
        this._updateSigningState(SigningState.Complete);
      } else {
        this._log(`❌ Signature verification failed!`);
        this._updateSigningState(SigningState.Failed);
      }

    } catch (error) {
      this._log(`Error verifying signature: ${(error as any).message || error}`);
      this._updateSigningState(SigningState.Failed);
    }
  }

  private _resetSigningState(): void {
    this._log("Resetting signing state");

    this.currentMessage = null;
    this.transactionData = null;
    this.receivedCommitments.clear();
    this.receivedShares.clear();
    this.selectedSigners = [];
    this.signingCommitments.clear();
    this.signingShares.clear();
    this.aggregatedSignature = null;
    this._updateSigningState(SigningState.Idle);
  }

  private _getExpectedPeers(): string[] {
    if (!this.sessionInfo) {
      return [];
    }
    return [...this.sessionInfo.participants];
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
      webrtc_msg_type: 'SimpleMessage',
      text: message
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
    console.log(`🔗 _handleLocalChannelOpen CALLED for peer: ${peerId}`);
    // This is called when OUR data channel to peerId opens.
    // We need to check if this completes our part of the mesh.
    this._log(`🔗 Local data channel to ${peerId} is now OPEN!`);

    // Log current data channel status
    if (this.sessionInfo) {
      const expectedPeers = this.sessionInfo.participants.filter(p => p !== this.localPeerId);
      const openDataChannelsToPeers = expectedPeers.filter(p => {
        const dc = this.dataChannels.get(p);
        return dc && dc.readyState === 'open';
      });
      console.log(`Data channel status: ${openDataChannelsToPeers.length}/${expectedPeers.length} channels open`);
      this._log(`Data channel status after ${peerId} opened: ${openDataChannelsToPeers.length}/${expectedPeers.length} channels open`);
    }

    console.log(`🔗 About to call _checkMeshStatus from _handleLocalChannelOpen`);
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

      case 'SimpleMessage':
        // Handle simple text messages between peers
        if ('text' in message) {
          this._log(`Received simple message from ${fromPeerId}: ${(message as any).text}`);
          // Forward to the onWebRTCAppMessage callback for external handling
          this.onWebRTCAppMessage(fromPeerId, message);
        } else {
          this._log(`SimpleMessage from ${fromPeerId} missing required fields.`);
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

      case 'SignTx':
        if ('tx_data' in message) {
          this._log(`Received SignTx from ${fromPeerId}`);
          this._handleSignTx(fromPeerId, (message as any).tx_data);
        } else {
          this._log(`SignTx from ${fromPeerId} missing tx_data field.`);
        }
        break;

      case 'SignCommitment':
        if ('commitment' in message) {
          this._log(`Received SignCommitment from ${fromPeerId}`);
          this._handleSignCommitment(fromPeerId, (message as any).commitment);
        } else {
          this._log(`SignCommitment from ${fromPeerId} missing commitment field.`);
        }
        break;

      case 'SignShare':
        if ('share' in message) {
          this._log(`Received SignShare from ${fromPeerId}`);
          this._handleSignShare(fromPeerId, (message as any).share);
        } else {
          this._log(`SignShare from ${fromPeerId} missing share field.`);
        }
        break;

      case 'SignAggregated':
        if ('signature' in message) {
          this._log(`Received SignAggregated from ${fromPeerId}`);
          this._handleSignAggregated(fromPeerId, (message as any).signature);
        } else {
          this._log(`SignAggregated from ${fromPeerId} missing signature field.`);
        }
        break;

      case 'SignerSelection':
        if ('signers' in message) {
          this._log(`Received SignerSelection from ${fromPeerId}`);
          this._handleSignerSelection(fromPeerId, (message as any).signers);
        } else {
          this._log(`SignerSelection from ${fromPeerId} missing signers field.`);
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
      blockchain: this.currentBlockchain,
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
    // If we have a stored group public key, return it
    if (this.groupPublicKey) {
      return this.groupPublicKey;
    }

    // Otherwise, try to get it from the WASM instance
    if (!this.frostDkg || this.dkgState !== DkgState.Complete) {
      return null;
    }

    try {
      const gpk = this.frostDkg.get_group_public_key();
      // Store it for future use
      if (gpk) {
        this.groupPublicKey = gpk;
      }
      return gpk;
    } catch (error) {
      this._log(`Error getting group public key: ${error}`);
      return null;
    }
  }

  public getSolanaAddress(): string | null {
    if (!this.frostDkg) {
      this._log(`Cannot get Solana address: FROST DKG not initialized`);
      return null;
    }

    if (this.dkgState !== DkgState.Complete) {
      this._log(`Cannot get Solana address: DKG not complete (current state: ${DkgState[this.dkgState]})`);
      return null;
    }

    try {
      this._log(`Getting Solana address from completed DKG...`);
      const address = this.frostDkg.get_sol_address();
      this._log(`Successfully generated Solana address: ${address}`);
      return address;
    } catch (error) {
      this._log(`Error getting Solana address from FROST DKG: ${error}`);
      // Check if the error is related to DKG not being properly completed
      if (error instanceof Error && error.toString().includes('DKG not completed yet')) {
        this._log(`DKG completion status inconsistent - forcing state check`);
        this._updateDkgState(DkgState.Failed);
      }
      return null;
    }
  }

  private _broadcastToAllPeers(message: WebRTCAppMessage): void {
    if (!this.sessionInfo) {
      this._log("Cannot broadcast: no active session");
      return;
    }

    this.sessionInfo.participants.forEach(peerId => {
      if (peerId !== this.localPeerId) {
        this.sendWebRTCAppMessage(peerId, message);
        this._log(`Sent ${message.webrtc_msg_type} message to ${peerId}`);
      }
    });
  }
}