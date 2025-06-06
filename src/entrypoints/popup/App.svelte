<script lang="ts">
  import svelteLogo from "../../assets/svelte.svg";
  import {
    generate_priv_key,
    get_eth_address,
    get_sol_address,
    eth_sign,
    sol_sign,
  } from "../../../pkg/mpc_wallet.js";
  import { onMount, onDestroy } from "svelte";
  import { storage } from "#imports";
  import type { MeshStatus, SessionInfo } from "../../types/appstate";
  import { MeshStatusType, DkgState } from "../../types/appstate";

  // Private key and wallet operations
  let private_key: string = "";
  let address: string = "";
  let message: string = "Hello from MPC Wallet!";
  let signature: string = "";
  let error: string = "";
  let chain: "ethereum" | "solana" = "ethereum";

  // Connection state (synced from background)
  let currentPeerId: string = "";
  let peersList: string[] = [];
  let wsConnected: boolean = false;
  let wsError: string = "";

  // Session and WebRTC state
  let sessionInfo: SessionInfo | null = null;
  let invites: SessionInfo[] = [];
  let meshStatus: MeshStatus = { type: MeshStatusType.Incomplete };
  let dkgState = DkgState.Idle;
  let proposedSessionIdInput: string = "";

  // Session configuration
  let totalParticipants: number = 3;
  let threshold: number = 2;

  // Track WebRTC connections per peer
  let peerConnections: Record<string, boolean> = {}; // peer_id -> connected status
  let sessionAcceptanceStatus: Record<string, Record<string, boolean>> = {}; // session_id -> peer_id -> accepted

  // Keep connection to background script
  let port: chrome.runtime.Port;

  // Debug logging to console
  $: console.log("[UI Debug] State update:", {
    wsConnected,
    currentPeerId,
    peersList,
    sessionInfo,
    meshStatus,
    dkgState,
    peerConnections,
    sessionAcceptanceStatus,
  });

  // Reactive computation for WebRTC connection status
  $: webrtcConnected = sessionInfo && meshStatus?.type === MeshStatusType.Ready;
  $: webrtcConnecting =
    sessionInfo && meshStatus?.type === MeshStatusType.PartiallyReady;

  // Add reactive statement to force UI updates when peerConnections change
  $: {
    // This reactive block will trigger whenever peerConnections changes
    console.log("[UI] PeerConnections updated:", peerConnections);
    // Force a re-render by updating a dummy variable or just logging
    if (Object.keys(peerConnections).length > 0) {
      console.log(
        "[UI] Active WebRTC connections:",
        Object.entries(peerConnections).filter(([_, connected]) => connected),
      );
    }
  }

  // Common handler for messages from the background script
  function handleBackgroundMessage(message: any) {
    console.log("[UI] Background message received:", message);

    switch (message.type) {
      case "initialState":
        console.log("[UI] Processing initialState");
        currentPeerId = message.peerId || "";
        peersList = [...(message.connectedPeers || [])];
        wsConnected = message.wsConnected || false;
        sessionInfo = message.sessionInfo || null;
        if (JSON.stringify(invites) !== JSON.stringify(message.invites || [])) {
          invites = message.invites ? [...message.invites] : [];
        }
        meshStatus = message.meshStatus || { type: MeshStatusType.Incomplete };
        dkgState = message.dkgState || DkgState.Idle;

        // Initialize WebRTC connection state from background
        if (message.webrtcConnections) {
          console.log(
            "[UI] Initializing WebRTC connections from background:",
            message.webrtcConnections,
          );
          peerConnections = { ...message.webrtcConnections };
        } else {
          console.log("[UI] No WebRTC connections in initial state");
          peerConnections = {};
        }

        // Initialize session acceptance status from background
        if (message.sessionAcceptanceStatus) {
          console.log(
            "[UI] Initializing session acceptance status from background:",
            message.sessionAcceptanceStatus,
          );
          sessionAcceptanceStatus = { ...message.sessionAcceptanceStatus };
        } else {
          console.log("[UI] No session acceptance status in initial state");
          sessionAcceptanceStatus = {};
        }
        break;

      case "wsStatus":
        console.log("[UI] Processing wsStatus:", message);
        wsConnected = message.connected || false;
        if (!message.connected && message.reason) {
          wsError = `WebSocket disconnected: ${message.reason}`;
        } else if (message.connected) {
          wsError = "";
        }
        break;

      case "wsMessage":
        console.log("[UI] Processing wsMessage:", message);
        if (message.message) {
          console.log("[UI] Server message:", message.message);
          if (message.message.type === "peers") {
            peersList = [...(message.message.peers || [])];
          }
        }
        break;

      case "wsError":
        console.log("[UI] Processing wsError:", message);
        wsError = message.error;
        console.error("[UI] WebSocket error:", message.error);
        break;

      case "peerList":
        console.log("[UI] Processing peerList:", message);
        peersList = [...(message.peers || [])];
        break;

      case "sessionUpdate":
        console.log("[UI] Processing sessionUpdate:", message);
        sessionInfo = message.sessionInfo || null;
        invites = message.invites ? [...message.invites] : [];
        console.log("[UI] Session update:", { sessionInfo, invites });

        // Log accepted peers for debugging
        if (sessionInfo && sessionInfo.accepted_peers) {
          console.log(
            "[UI] Session accepted peers:",
            sessionInfo.accepted_peers,
          );
          // Filter out any null/undefined values that might have been added
          sessionInfo.accepted_peers = sessionInfo.accepted_peers.filter(
            (peer) => peer != null && peer !== undefined,
          );
        }
        break;

      case "meshStatusUpdate":
        console.log("[UI] Processing meshStatusUpdate:", message);
        meshStatus = message.status || { type: MeshStatusType.Incomplete };
        console.log("[UI] Mesh status update:", meshStatus);
        break;

      case "webrtcConnectionUpdate":
        console.log("[UI] Processing webrtcConnectionUpdate:", message);

        if (message.peerId && typeof message.connected === "boolean") {
          console.log(
            "[UI] Updating peer connection:",
            message.peerId,
            "->",
            message.connected,
          );

          // Force reactivity by creating a new object
          peerConnections = {
            ...peerConnections,
            [message.peerId]: message.connected,
          };

          console.log("[UI] Updated peerConnections:", peerConnections);

          // Trigger a reactive update explicitly
          peerConnections = peerConnections;
        } else {
          console.warn("[UI] Invalid webrtcConnectionUpdate message:", message);
        }
        break;

      case "dkgStateUpdate":
        console.log("[UI] Processing dkgStateUpdate:", message);
        dkgState = message.state || DkgState.Idle;
        console.log("[UI] DKG state update:", dkgState);
        break;

      case "fromOffscreen":
        console.log("[UI] Processing fromOffscreen wrapper:", message);
        // Handle wrapped messages from offscreen
        if (message.payload) {
          console.log(
            "[UI] Unwrapping and processing payload:",
            message.payload,
          );
          handleBackgroundMessage(message.payload);
        }
        break;

      case "webrtcStatusUpdate":
        console.log("[UI] Processing webrtcStatusUpdate:", message);
        if (message.peerId && message.status) {
          console.log(
            `[UI] WebRTC status for ${message.peerId}: ${message.status}`,
          );
          // Update UI state based on WebRTC status if needed
        }
        break;

      case "dataChannelStatusUpdate":
        console.log("[UI] Processing dataChannelStatusUpdate:", message);
        if (message.peerId && message.channelName && message.state) {
          console.log(
            `[UI] Data channel ${message.channelName} for ${message.peerId}: ${message.state}`,
          );
        }
        break;

      case "peerConnectionStatusUpdate":
        console.log("[UI] Processing peerConnectionStatusUpdate:", message);
        if (message.peerId && message.connectionState) {
          console.log(
            `[UI] Peer connection for ${message.peerId}: ${message.connectionState}`,
          );
        }
        break;

      default:
        console.log("[UI] Unhandled message type:", message.type, message);
    }
  }

  // Generate or load the private key for the selected chain
  async function ensurePrivateKey() {
    const curve = chain === "ethereum" ? "secp256k1" : "ed25519";
    const keyName = `local:private_key_${curve}` as `local:${string}`;
    const storedKey = await storage.getItem<string>(keyName);
    if (storedKey) {
      private_key = storedKey;
    } else {
      private_key = generate_priv_key(curve);
      await storage.setItem(keyName, private_key);
    }
    address = "";
    signature = "";
  }

  // Request offscreen document (must be triggered by user gesture)
  async function ensureOffscreenDocument() {
    try {
      chrome.runtime.sendMessage({ type: "createOffscreen" }, (response) => {
        if (chrome.runtime.lastError) {
          console.error(
            "[UI] Offscreen creation error:",
            chrome.runtime.lastError.message,
          );
        } else {
          console.log("[UI] Offscreen response:", response);
        }
      });
    } catch (e: any) {
      console.error("[UI] Failed to request offscreen:", e);
    }
  }

  onMount(async () => {
    console.log("[UI] Component mounting");
    port = chrome.runtime.connect({ name: "popup" });
    port.onMessage.addListener(handleBackgroundMessage);
    port.onDisconnect.addListener(() => {
      console.error("[UI] Port disconnected from background");
      wsConnected = false;
    });

    // Request initial state - let handleBackgroundMessage process it
    chrome.runtime.sendMessage({ type: "getState" }, (response) => {
      if (chrome.runtime.lastError) {
        console.error(
          "[UI] Error on initial getState:",
          chrome.runtime.lastError.message,
        );
        return;
      }
      if (response) {
        console.log("[UI] Initial state response:", response);
        // Process through the centralized handler to ensure reactivity
        handleBackgroundMessage({
          type: "initialState",
          ...response,
        });
      }
    });

    await ensurePrivateKey();
    await ensureOffscreenDocument();
  });

  onDestroy(() => {
    if (port) {
      port.disconnect();
    }
  });

  $: if (chain) {
    ensurePrivateKey();
  }

  async function fetchAddress() {
    error = "";
    signature = "";
    try {
      if (chain === "ethereum") {
        address = get_eth_address(private_key);
        if (address.startsWith("0x")) {
          address = address.slice(2);
        }
        if (address.length !== 40) {
          error = "Invalid Ethereum address length.";
        }
        if (!address) {
          error = "No address returned.";
        }
      } else if (chain === "solana") {
        address = get_sol_address(private_key);
        if (!address || address.startsWith("Error")) {
          error = "Failed to get Solana address.";
        }
      }
    } catch (e: any) {
      error = `Failed to fetch address: ${e.message || e}`;
    }
  }

  async function signDemoMessage() {
    error = "";
    signature = "";
    if (!private_key) {
      error = "Private key is not set.";
      return;
    }
    if (!address) {
      error = "Please fetch address first.";
      return;
    }
    try {
      if (chain === "ethereum") {
        signature = eth_sign(private_key, message);
        if (!signature) {
          error = "Signing failed. Check private key and message.";
        }
      } else if (chain === "solana") {
        signature = sol_sign(private_key, message);
        if (!signature || signature.startsWith("Error")) {
          error = "Solana signing failed.";
        }
      }
    } catch (e: any) {
      error = `Failed to sign message: ${e.message || e}`;
    }
  }

  function requestPeerList() {
    console.log("[UI] Requesting peer list");
    chrome.runtime.sendMessage({ type: "listPeers" }, (response) => {
      if (chrome.runtime.lastError) {
        console.error(
          "[UI] Error requesting peer list:",
          chrome.runtime.lastError.message,
        );
        return;
      }
      console.log("[UI] listPeers response:", response);
    });
  }

  function proposeSession() {
    const availablePeers = peersList.filter((p) => p !== currentPeerId);

    if (availablePeers.length < totalParticipants - 1) {
      error = `Need at least ${totalParticipants - 1} other peers for a ${totalParticipants}-participant session`;
      return;
    }

    if (threshold > totalParticipants) {
      error = "Threshold cannot be greater than total participants";
      return;
    }

    if (threshold < 1) {
      error = "Threshold must be at least 1";
      return;
    }

    const peersToInclude = availablePeers.slice(0, totalParticipants - 1);
    const allParticipants = [currentPeerId, ...peersToInclude];

    const sessionId =
      proposedSessionIdInput.trim() ||
      `wallet_${threshold}of${totalParticipants}_${Date.now()}`;

    chrome.runtime.sendMessage({
      type: "proposeSession",
      session_id: sessionId,
      total: totalParticipants,
      threshold: threshold,
      participants: allParticipants,
    });
    console.log(
      "[UI] Proposing session:",
      sessionId,
      `(${threshold}-of-${totalParticipants})`,
      "with participants:",
      allParticipants,
    );
  }

  function acceptInvite(sessionId: string) {
    chrome.runtime.sendMessage({
      type: "acceptSession",
      session_id: sessionId,
      accepted: true,
    });
    console.log("[UI] Accepting session invite:", sessionId);
  }

  // Add function to send direct message for testing
  function sendDirectMessage(toPeerId: string) {
    const testMessage = `Hello from ${currentPeerId} at ${new Date().toLocaleTimeString()}`;
    chrome.runtime.sendMessage(
      {
        type: "sendDirectMessage",
        toPeerId: toPeerId,
        message: testMessage,
      },
      (response) => {
        if (chrome.runtime.lastError) {
          console.error(
            "[UI] Error sending direct message:",
            chrome.runtime.lastError.message,
          );
          error = `Failed to send message: ${chrome.runtime.lastError.message}`;
        } else {
          console.log("[UI] Direct message response:", response);
          if (!response.success) {
            error = `Failed to send message: ${response.error}`;
          }
        }
      },
    );
    console.log(
      "[UI] Sending direct message to:",
      toPeerId,
      "Message:",
      testMessage,
    );
  }

  // Helper function to get WebRTC status for a peer
  function getWebRTCStatus(
    peerId: string,
  ): "connected" | "connecting" | "disconnected" {
    console.log(
      "[UI] Getting WebRTC status for peer:",
      peerId,
      "from peerConnections:",
      peerConnections,
    );

    // Check direct connection status first
    if (peerConnections[peerId] === true) {
      return "connected";
    } else if (
      sessionInfo &&
      sessionInfo.participants.includes(peerId) &&
      meshStatus?.type === MeshStatusType.PartiallyReady
    ) {
      return "connecting";
    } else {
      return "disconnected";
    }
  }

  // Helper function to get session acceptance status
  function getSessionAcceptanceStatus(
    sessionId: string,
    peerId: string,
  ): boolean | undefined {
    if (!sessionAcceptanceStatus[sessionId]) {
      return undefined;
    }
    return sessionAcceptanceStatus[sessionId][peerId];
  }
</script>

<main class="p-4 max-w-2xl mx-auto">
  <div class="text-center mb-6">
    <img src={svelteLogo} class="logo svelte mx-auto mb-2" alt="Svelte Logo" />
    <h1 class="text-3xl font-bold">MPC Wallet</h1>
  </div>

  <!-- Chain Selection -->
  <div class="mb-4 p-3 border rounded">
    <label for="chain-select" class="block font-bold mb-2">Blockchain:</label>
    <select
      id="chain-select"
      bind:value={chain}
      class="w-full border p-2 rounded"
    >
      <option value="ethereum">Ethereum (secp256k1)</option>
      <option value="solana">Solana (ed25519)</option>
    </select>
  </div>

  <!-- Wallet Operations -->
  <div class="mb-4 p-3 border rounded">
    <h2 class="text-xl font-semibold mb-3">Wallet Operations</h2>

    <button
      class="w-full bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mb-3"
      on:click={fetchAddress}
    >
      Generate Address
    </button>

    {#if address}
      <div class="mb-3">
        <span class="block font-bold mb-1">Address:</span>
        <code class="block bg-gray-100 p-2 rounded break-all">{address}</code>
      </div>
    {/if}

    <div class="mb-3">
      <label for="message-input" class="block font-bold mb-1"
        >Message to Sign:</label
      >
      <input
        id="message-input"
        type="text"
        bind:value={message}
        class="w-full border p-2 rounded"
        placeholder="Enter message to sign"
      />
    </div>

    <button
      class="w-full bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded mb-3"
      on:click={signDemoMessage}
      disabled={!private_key}
    >
      Sign Message
    </button>

    {#if signature}
      <div class="mb-3">
        <span class="block font-bold mb-1">Signature:</span>
        <code class="block bg-gray-100 p-2 rounded break-all text-sm">
          {signature}
        </code>
      </div>
    {/if}
  </div>

  <!-- Network Status -->
  <div class="mb-4 p-3 border rounded">
    <h2 class="text-xl font-semibold mb-3">Network Status</h2>

    <div class="grid grid-cols-1 gap-3 mb-3">
      <div>
        <span class="block font-bold mb-1">Peer ID:</span>
        <code class="block bg-gray-100 p-2 rounded text-sm">
          {currentPeerId || "Not connected"}
        </code>
      </div>
      <div>
        <span class="block font-bold mb-1">WebSocket:</span>
        <span
          class="inline-block px-2 py-1 rounded text-sm {wsConnected
            ? 'bg-green-100 text-green-800'
            : 'bg-red-100 text-red-800'}"
        >
          {wsConnected ? "Connected" : "Disconnected"}
        </span>
      </div>
    </div>
  </div>

  <!-- Connected Peers with Individual WebRTC Status -->
  <div class="mb-4 p-3 border rounded">
    <h2 class="text-xl font-semibold mb-3">
      Connected Peers ({peersList.length})
    </h2>

    {#if peersList && peersList.length > 0}
      <ul class="space-y-2">
        {#each peersList as peer}
          <li class="flex items-center justify-between p-3 bg-gray-50 rounded">
            <div class="flex items-center gap-3">
              <code class="text-sm font-mono">{peer}</code>
              {#if peer === currentPeerId}
                <span
                  class="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded"
                  >You</span
                >
              {/if}
            </div>

            {#if peer !== currentPeerId}
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-500">WebRTC:</span>
                {#if getWebRTCStatus(peer) === "connected"}
                  <span
                    class="text-xs bg-green-100 text-green-800 px-2 py-1 rounded"
                    >Connected</span
                  >
                  {#if sessionInfo && meshStatus?.type === MeshStatusType.Ready}
                    <button
                      class="text-xs bg-blue-500 hover:bg-blue-700 text-white px-2 py-1 rounded"
                      on:click={() => sendDirectMessage(peer)}
                    >
                      Test Message
                    </button>
                  {/if}
                {:else if getWebRTCStatus(peer) === "connecting"}
                  <span
                    class="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded"
                    >Connecting</span
                  >
                {:else}
                  <span
                    class="text-xs bg-red-100 text-red-800 px-2 py-1 rounded"
                    >Disconnected</span
                  >
                {/if}
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {:else}
      <p class="text-gray-500 text-center py-4">No peers connected</p>
    {/if}
  </div>

  <!-- MPC Session Management -->
  <div class="mb-4 p-3 border rounded">
    <h2 class="text-xl font-semibold mb-3">MPC Session</h2>

    {#if sessionInfo}
      <!-- Active Session -->
      <div class="bg-green-50 border border-green-200 rounded p-3 mb-3">
        <h3 class="font-bold text-green-800 mb-2">Active Session</h3>
        <div class="grid grid-cols-2 gap-2 text-sm">
          <div><strong>ID:</strong> {sessionInfo.session_id}</div>
          <div><strong>Proposer:</strong> {sessionInfo.proposer_id}</div>
          <div>
            <strong>Threshold:</strong>
            {sessionInfo.threshold} of
            {sessionInfo.total}
          </div>
          <div>
            <strong>Mesh:</strong>
            {MeshStatusType[meshStatus?.type] || "Unknown"}
          </div>
          <div>
            <strong>DKG:</strong>
            {DkgState[dkgState] || "Unknown"}
          </div>
        </div>
        <div class="mt-2">
          <span class="font-bold">Participants:</span>
          <div class="flex flex-wrap gap-1 mt-1">
            {#each sessionInfo.participants as participant}
              <span
                class="text-xs bg-gray-100 px-2 py-1 rounded {participant ===
                currentPeerId
                  ? 'bg-blue-100'
                  : ''}"
              >
                {participant}{participant === currentPeerId ? " (you)" : ""}
              </span>
            {/each}
          </div>
        </div>
        <div class="mt-2">
          <span class="font-bold">Accepted:</span>
          <div class="flex flex-wrap gap-1 mt-1">
            {#each sessionInfo.accepted_peers || [] as accepted}
              <span
                class="text-xs bg-green-100 text-green-800 px-2 py-1 rounded"
              >
                {accepted}
              </span>
            {/each}
          </div>
        </div>
      </div>
    {:else if invites && invites.length > 0}
      <!-- Pending Invitations -->
      <div class="space-y-3">
        {#each invites as invite}
          <div class="bg-yellow-50 border border-yellow-200 rounded p-3">
            <h3 class="font-bold text-yellow-800 mb-2">Session Invitation</h3>
            <div class="grid grid-cols-2 gap-2 text-sm mb-3">
              <div><strong>ID:</strong> {invite.session_id}</div>
              <div><strong>From:</strong> {invite.proposer_id}</div>
              <div>
                <strong>Type:</strong>
                {invite.threshold} of {invite.total}
              </div>
              <div>
                <strong>Participants:</strong>
                {invite.participants?.length || 0}
              </div>
            </div>
            <button
              class="w-full bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded"
              on:click={() => acceptInvite(invite.session_id)}
            >
              Accept Invitation
            </button>
          </div>
        {/each}
      </div>
    {:else}
      <!-- Create New Session -->
      <div class="space-y-3">
        <div>
          <label for="session-id-input" class="block font-bold mb-1"
            >Session ID (optional):</label
          >
          <input
            id="session-id-input"
            type="text"
            bind:value={proposedSessionIdInput}
            class="w-full border p-2 rounded"
            placeholder="Auto-generated if empty"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label for="total-participants" class="block font-bold mb-1"
              >Total Participants:</label
            >
            <input
              id="total-participants"
              type="number"
              bind:value={totalParticipants}
              min="2"
              max={peersList.length}
              class="w-full border p-2 rounded"
            />
          </div>
          <div>
            <label for="threshold-input" class="block font-bold mb-1"
              >Threshold:</label
            >
            <input
              id="threshold-input"
              type="number"
              bind:value={threshold}
              min="1"
              max={totalParticipants}
              class="w-full border p-2 rounded"
            />
          </div>
        </div>

        <button
          class="w-full bg-indigo-500 hover:bg-indigo-700 text-white font-bold py-2 px-4 rounded"
          on:click={proposeSession}
          disabled={!wsConnected ||
            peersList.filter((p) => p !== currentPeerId).length <
              totalParticipants - 1 ||
            threshold > totalParticipants ||
            threshold < 1}
        >
          Propose New Session ({threshold}-of-{totalParticipants})
        </button>

        {#if peersList.filter((p) => p !== currentPeerId).length < totalParticipants - 1}
          <p class="text-sm text-gray-500 text-center">
            Need at least {totalParticipants - 1} other peers for a {totalParticipants}-participant
            session
          </p>
        {:else if threshold > totalParticipants || threshold < 1}
          <p class="text-sm text-red-500 text-center">
            Invalid threshold: must be between 1 and {totalParticipants}
          </p>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Error Display -->
  {#if error}
    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded">
      <div class="flex justify-between items-center">
        <span class="text-red-600">{error}</span>
        <button
          class="text-sm bg-red-100 hover:bg-red-200 px-2 py-1 rounded"
          on:click={() => (error = "")}
        >
          ×
        </button>
      </div>
    </div>
  {/if}

  {#if wsError}
    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded">
      <div class="flex justify-between items-center">
        <span class="text-red-600">{wsError}</span>
        <button
          class="text-sm bg-red-100 hover:bg-red-200 px-2 py-1 rounded"
          on:click={() => (wsError = "")}
        >
          ×
        </button>
      </div>
    </div>
  {/if}
</main>

<style>
  :global(body) {
    width: 400px;
    height: 600px;
    overflow: auto;
  }

  .logo {
    height: 40px;
    width: 40px;
  }
</style>
