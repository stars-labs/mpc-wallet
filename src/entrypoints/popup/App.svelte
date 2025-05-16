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
  import Settings from "@/components/Settings.svelte";
  import { storage } from "#imports";
  import type { ServerMsg } from "../background/websocket";
  import type {
    SessionInfo,
    WebRTCAppMessage,
    MeshStatus,
  } from "../background/webrtc";
  import { DkgState, MeshStatusType } from "../background/webrtc";
  import {
    logDiagnosticInfo,
    resetSessionState,
  } from "../../helpers/diagnostics";

  // Private key and wallet operations
  let private_key: string = "";
  let address: string = "";
  let message: string = "Hello from MPC Wallet!";
  let signature: string = "";
  let error: string = "";
  let isSettings: boolean = false;
  let chain: "ethereum" | "solana" = "ethereum";

  // Connection state (synced from background)
  let currentPeerId: string = "";
  let peersList: string[] = [];
  let wsConnected: boolean = false;
  let wsError: string = "";
  let serverMessages: ServerMsg[] = [];

  // Session and WebRTC state
  let sessionInfo: SessionInfo | null = null;
  let invites: SessionInfo[] = [];
  let meshStatus: MeshStatus = { type: MeshStatusType.Incomplete };
  let dkgState = DkgState.Idle;
  let proposedSessionIdInput: string = ""; // For the session ID input field

  // Keep connection to background script
  let port: chrome.runtime.Port;
  // Removing updateInterval variable since we're removing polling

  // Svelte reactive statement for debugging peersList changes
  $: console.log(
    "UI: peersList updated in Svelte context. Current value:",
    peersList,
  );

  // Common handler for messages from the background script (primarily via port)
  function handleBackgroundMessage(message: any) {
    console.log("UI: Message received from background (port):", message);

    switch (message.type) {
      case "initialState":
        console.log("UI: Processing 'initialState'", message);
        currentPeerId = message.peerId;
        if (
          JSON.stringify(peersList) !==
          JSON.stringify(message.connectedPeers || [])
        ) {
          peersList = [...(message.connectedPeers || [])];
        }
        wsConnected = message.wsConnected;
        sessionInfo = message.sessionInfo || null;
        if (JSON.stringify(invites) !== JSON.stringify(message.invites || [])) {
          invites = message.invites ? [...message.invites] : [];
        }
        meshStatus = message.meshStatus || { type: MeshStatusType.Incomplete };
        dkgState = message.dkgState || DkgState.Idle;
        // Optionally update serverMessages if included in initialState
        break;

      case "wsStatus":
        // ... (same as before)
        wsConnected = message.connected;
        if (!message.connected && message.reason) {
          wsError = `WebSocket disconnected: ${message.reason}`;
        } else if (message.connected) {
          wsError = "";
        }
        break;

      case "wsMessage":
        // ... (same as before, ensure peersList update is reactive)
        if (message.message) {
          serverMessages = [...serverMessages, message.message];
          if (message.message.type === "peers") {
            console.log(
              "UI: Direct peers update from server (wsMessage in wsMessage):",
              message.message.peers,
            );
            if (
              JSON.stringify(peersList) !==
              JSON.stringify(message.message.peers || [])
            ) {
              peersList = [...(message.message.peers || [])];
            }
          } else if (message.message.type === "relay" && message.message.data) {
            // --- BEGIN: Add relay message logging for websocket_msg_type ---
            if (message.message.data.websocket_msg_type) {
              serverMessages = [
                ...serverMessages,
                {
                  type: "info",
                  message: `Received relay: websocket_msg_type=${message.message.data.websocket_msg_type} from=${message.message.from || message.message.from}`,
                  relay: message.message,
                },
              ];
            }
            // --- END: relay message logging ---
            // ... relay handling
          }
        }
        break;

      case "wsError":
        // ... (same as before)
        wsError = message.error;
        serverMessages = [
          ...serverMessages,
          { type: "error", error: message.error },
        ];
        break;

      case "peerList":
        console.log("UI: Processing 'peerList'", message);
        if (Array.isArray(message.peers)) {
          if (JSON.stringify(peersList) !== JSON.stringify(message.peers)) {
            peersList = [...message.peers];
          }
        }
        break;

      // ... other cases: sessionUpdate, meshStatusUpdate, dkgStateUpdate, webrtcMessage, webrtcConnectionUpdate
      default:
        console.log(
          "UI: Received unhandled message type from background via port:",
          message.type,
          message,
        );
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
    if (!chrome?.offscreen) return;
    try {
      const hasOffscreen = await chrome.offscreen.hasDocument();
      if (!hasOffscreen) {
        await chrome.offscreen.createDocument({
          url: chrome.runtime.getURL("offscreen.html"),
          reasons: [chrome.offscreen.Reason.DOM_SCRAPING],
          justification: "Need WebRTC for MPC wallet communication",
        });
        serverMessages = [
          ...serverMessages,
          { type: "info", message: "Offscreen document created" } as any,
        ];
      }
    } catch (e: any) {
      serverMessages = [
        ...serverMessages,
        {
          type: "error",
          message: "Failed to create offscreen: " + (e?.message || e),
        } as any,
      ];
    }
  }

  onMount(async () => {
    port = chrome.runtime.connect({ name: "popup" });
    console.log("UI: Port connected to background script.");
    port.onMessage.addListener(handleBackgroundMessage);
    port.onDisconnect.addListener(() => {
      console.error(
        "UI: Port disconnected from background. UI state might be stale.",
      );
      wsConnected = false; // Example: reflect disconnection
      // Consider implementing a reconnect mechanism for the port or notifying the user.
    });

    // Initial request for full state via chrome.runtime.sendMessage
    // This serves as a primary way to get the state when the popup opens.
    // The background's onConnect will also send 'initialState' via port,
    // so one of them should provide the initial data.
    console.log(
      "UI: Requesting initial state with chrome.runtime.sendMessage({ type: 'getState' })",
    );
    chrome.runtime.sendMessage({ type: "getState" }, (response) => {
      if (chrome.runtime.lastError) {
        console.error(
          "UI: Error on initial getState:",
          chrome.runtime.lastError.message,
        );
        return;
      }
      if (response) {
        console.log("UI: Response from initial getState:", response);
        currentPeerId = response.peerId;
        // Prefer this over initialState if it arrives later and is more complete,
        // or ensure data merging logic is robust.
        if (
          JSON.stringify(peersList) !==
          JSON.stringify(response.connectedPeers || [])
        ) {
          peersList = [...(response.connectedPeers || [])];
        }
        wsConnected = response.wsConnected;
        sessionInfo = response.sessionInfo || null;
        if (
          JSON.stringify(invites) !== JSON.stringify(response.invites || [])
        ) {
          invites = response.invites ? [...response.invites] : [];
        }
        meshStatus = response.meshStatus || { type: MeshStatusType.Incomplete };
        dkgState = response.dkgState || DkgState.Idle;
        if (
          response.recentMessages &&
          JSON.stringify(serverMessages) !==
            JSON.stringify(response.recentMessages)
        ) {
          serverMessages = response.recentMessages
            ? [...response.recentMessages]
            : [];
        }
      } else {
        console.warn(
          "UI: No response for initial getState via chrome.runtime.sendMessage.",
        );
      }
    });

    await ensurePrivateKey();

    // Ensure offscreen document (user gesture required, so call from UI or here if popup is opened by user)
    await ensureOffscreenDocument();

    // Removing polling interval - not needed as state changes are pushed via port connection
  });

  onDestroy(() => {
    if (port) {
      console.log("UI: Disconnecting port on component destroy.");
      port.disconnect();
    }
    // Removing interval cleanup since we no longer have polling
  });

  $: if (chain) {
    // When chain changes, reload key/address/signature
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
        // Prefer eth_sign for Ethereum
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

  // Communication with background script
  function requestPeerList() {
    console.log("Requesting peer list from background");
    chrome.runtime.sendMessage({ type: "listPeers" }); // This will trigger "peerList" message
    chrome.runtime.sendMessage({ type: "getState" }, (response) => {
      // This will update general state
      if (response && response.connectedPeers) {
        console.log(
          "Got peers from getState in requestPeerList:",
          response.connectedPeers,
        );
        if (
          JSON.stringify(peersList) !== JSON.stringify(response.connectedPeers)
        ) {
          peersList = [...response.connectedPeers]; // Reactive update
        }
      }
    });
  }

  function proposeSession() {
    const peersToInclude = peersList
      .filter((p) => p !== currentPeerId)
      .slice(0, 2);
    const allParticipants = [currentPeerId, ...peersToInclude];

    // Corrected conditional block
    if (allParticipants.length < 3) {
      error = "Need at least 3 participants for a session";
      return;
    }

    const sessionId =
      proposedSessionIdInput.trim() || `wallet_2of3_${Date.now()}`;
    chrome.runtime.sendMessage({
      type: "proposeSession",
      sessionId, // Use the user-provided or generated ID
      total: 3,
      threshold: 2, // Removed duplicate threshold
      participants: allParticipants,
    });
    // Add message to UI for feedback
    serverMessages = [
      ...serverMessages,
      {
        type: "info",
        message: `Proposing session ${sessionId} with participants: ${allParticipants.join(", ")}`,
      } as any,
    ];
  }

  function acceptInvite(sessionId: string) {
    chrome.runtime.sendMessage({
      type: "acceptSession",
      sessionId,
    });
    // Add message to UI for feedback
    serverMessages = [
      ...serverMessages,
      {
        type: "info",
        message: `Accepting session invite: ${sessionId}`,
      } as any,
    ];
  }

  function relayTestData() {
    if (peersList.length > 0) {
      const recipient = peersList.find((p) => p !== currentPeerId);
      if (recipient) {
        chrome.runtime.sendMessage({
          type: "relay",
          to: recipient,
          data: {
            greeting: "Hello from " + currentPeerId,
            timestamp: new Date().toISOString(),
          },
        });
        serverMessages = [
          ...serverMessages,
          { type: "info", message: `Sent relay to ${recipient}` } as any,
        ];
      }
    }
  }

  // Helper function to run diagnostics
  async function runDiagnostics() {
    await logDiagnosticInfo(sessionInfo, peersList, currentPeerId);
    serverMessages = [
      ...serverMessages,
      {
        type: "info",
        message: "Diagnostics logged to console. Check developer tools.",
      } as any,
    ];
  }

  // Helper function to force refresh state from background
  function forceRefreshState() {
    chrome.runtime.sendMessage({
      type: "getState",
      forceRefresh: true,
    });
    serverMessages = [
      ...serverMessages,
      { type: "info", message: "Forced state refresh from background" } as any,
    ];
  }

  // Helper function to force refresh peers specifically
  function forcePeerListRefresh() {
    chrome.runtime.sendMessage({ type: "listPeers", forceRefresh: true });
    serverMessages = [
      ...serverMessages,
      { type: "info", message: "Forced peer list refresh" } as any,
    ];
  }

  // Helper to reset WebRTC connections if they get stuck
  async function resetConnections() {
    await resetSessionState();
    // Clear local session info
    if (sessionInfo && !confirm("Reset current session?")) {
      return;
    }
    serverMessages = [
      ...serverMessages,
      { type: "info", message: "Resetting WebRTC connections..." } as any,
    ];
    // Wait a moment then refresh state
    setTimeout(() => {
      forceRefreshState();
    }, 1000);
  }

  // Function to clear error messages
  function clearErrors() {
    error = "";
    wsError = "";
  }
</script>

<main>
  <div>
    <a href="https://svelte.dev" target="_blank" rel="noreferrer">
      <img src={svelteLogo} class="logo svelte" alt="Svelte Logo" />
    </a>
    <button
      class="border-0 bg-transparent cursor-pointer"
      on:click={() => (isSettings = !isSettings)}
    >
      settings
    </button>
    <button
      class="border-0 bg-transparent cursor-pointer ml-2"
      on:click={() => chrome.runtime.openOptionsPage()}
      title="Open persistent WebRTC page"
    >
      Open WebRTC Console
    </button>
  </div>
  <h1 class="text-4xl font-bold underline">Starlab Crypto Wallet</h1>

  <div class="mt-4">
    <label for="chain-select" class="font-bold mr-2">Chain:</label>
    <select id="chain-select" bind:value={chain} class="border p-2 rounded">
      <option value="ethereum">Ethereum (secp256k1)</option>
      <option value="solana">Solana (ed25519)</option>
    </select>
  </div>

  <div class="mt-8">
    <button
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
      on:click={fetchAddress}
    >
      Show Wallet Address
    </button>
    {#if address}
      <div class="mt-2">
        <strong>Address:</strong>
        <code class="bg-gray-100 px-1">{address}</code>
      </div>
    {/if}
  </div>

  <div class="mt-4">
    <input type="text" bind:value={message} class="border p-2 rounded w-3/4" />
    <button
      class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded ml-2"
      on:click={signDemoMessage}
      disabled={!private_key}
    >
      Sign Message
    </button>
    {#if signature}
      <div class="mt-2">
        <strong>Signature:</strong>
        <code class="bg-gray-100 p-2 block break-all">{signature}</code>
      </div>
    {/if}
  </div>

  {#if error}
    <div class="text-red-600 mt-2 flex justify-between items-center">
      <span>{error}</span>
      <button
        class="text-sm bg-gray-200 px-2 py-1 rounded"
        on:click={clearErrors}
      >
        Clear
      </button>
    </div>
  {/if}

  <!-- WebSocket Section -->
  <div class="mt-8 p-4 border rounded">
    <h2 class="text-2xl font-semibold">WebSocket Signaling</h2>
    <p>
      <strong>My Peer ID:</strong>
      <code class="bg-gray-100 px-1">{currentPeerId || "Not connected"}</code>
      <span class="ml-2 {wsConnected ? 'text-green-500' : 'text-red-500'}">
        {wsConnected ? "● Connected" : "● Disconnected"}
      </span>
    </p>

    <!-- Debug information (can be removed later) -->
    <div class="text-xs text-gray-500 mt-1">
      Last peers data update timestamp: {new Date().toLocaleTimeString()}
    </div>

    {#if wsError}
      <div class="text-red-600 mt-2 flex justify-between items-center">
        <span>{wsError}</span>
        <button
          class="text-sm bg-gray-200 px-2 py-1 rounded"
          on:click={clearErrors}
        >
          Clear
        </button>
      </div>
    {/if}

    <div class="mt-4">
      <button
        class="bg-purple-500 hover:bg-purple-700 text-white font-bold py-2 px-4 rounded mr-2"
        on:click={requestPeerList}
        disabled={!wsConnected}
      >
        List Peers
      </button>
      <button
        class="bg-indigo-500 hover:bg-indigo-700 text-white font-bold py-2 px-4 rounded mr-2"
        on:click={relayTestData}
        disabled={!wsConnected ||
          peersList.filter((p) => p !== currentPeerId).length === 0}
      >
        Relay Test Data
      </button>
      <!-- Close the Relay Test Data button properly -->

      <!-- Session ID input and Propose Session button should be siblings, not nested -->
      <input
        type="text"
        bind:value={proposedSessionIdInput}
        class="border p-2 rounded w-auto inline-block mr-2"
        placeholder="Optional Session ID"
      />
      <button
        class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded"
        on:click={proposeSession}
        disabled={!wsConnected ||
          peersList.filter((p) => p !== currentPeerId).length < 2}
      >
        Propose Session
      </button>
    </div>

    <!-- Peers and Messages Display -->
    <div class="mt-4 grid grid-cols-2 gap-4">
      <div>
        <div class="flex justify-between items-center mb-2">
          <h3 class="font-semibold">Connected Peers:</h3>
          <button
            class="text-sm bg-blue-100 px-2 py-1 rounded"
            on:click={forcePeerListRefresh}
            title="Force refresh peers list"
          >
            Refresh Peers
          </button>
        </div>
        {#if peersList && peersList.length > 0}
          <ul class="list-disc list-inside bg-gray-50 p-2 rounded">
            {#each peersList as peer}
              <li class={peer === currentPeerId ? "font-bold" : ""}>
                {peer}{peer === currentPeerId ? " (self)" : ""}
              </li>
            {/each}
          </ul>
          <p class="text-xs text-gray-500 mt-1">
            Peers count: {peersList.length}
          </p>
        {:else}
          <p class="text-gray-500">No peers connected yet.</p>
          {#if wsConnected}
            <button
              class="mt-2 text-sm bg-blue-100 px-2 py-1 rounded"
              on:click={forcePeerListRefresh}
            >
              Refresh peers list
            </button>
          {/if}
        {/if}
      </div>

      <div>
        <h3 class="font-semibold">Session State:</h3>
        {#if sessionInfo}
          <div class="bg-gray-50 p-2 rounded">
            <p><strong>Session:</strong> {sessionInfo.session_id}</p>
            <p><strong>Proposer:</strong> {sessionInfo.proposer_id}</p>
            <p>
              <strong>Threshold:</strong>
              {sessionInfo.threshold} of {sessionInfo.total}
            </p>
            <p>
              <strong>Participants:</strong>
              {sessionInfo.participants.join(", ")}
            </p>
            <p>
              <strong>Accepted:</strong>
              {sessionInfo.accepted_peers?.join(", ") || "None"}
            </p>
            <p>
              <strong>Mesh Status:</strong>
              {MeshStatusType[meshStatus?.type] || "Unknown"}
            </p>
            <p><strong>DKG State:</strong> {DkgState[dkgState] || "Unknown"}</p>
            {#if sessionInfo.participants && !sessionInfo.participants.includes(currentPeerId)}
              <div class="text-red-600 mt-2">
                Warning: You are not a participant in this session. WebRTC will
                not establish.
              </div>
            {/if}
          </div>
        {:else if invites && invites.length > 0}
          <div>
            <p><strong>Pending Invites:</strong></p>
            {#each invites as invite}
              <div class="bg-yellow-50 p-2 rounded mb-2">
                <p><strong>Session:</strong> {invite.session_id}</p>
                <p><strong>From:</strong> {invite.proposer_id}</p>
                <p>
                  <strong>Type:</strong>
                  {invite.threshold} of {invite.total} threshold
                </p>
                <p>
                  <strong>Participants:</strong>
                  {invite.participants?.join(", ") || "Unknown"}
                </p>
                <button
                  class="bg-green-500 hover:bg-green-700 text-white font-bold py-1 px-2 rounded text-sm mt-1"
                  on:click={() => acceptInvite(invite.session_id)}
                >
                  Accept Invitation
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-gray-500">No active session or pending invites.</p>
          {#if peersList && peersList.length >= 3}
            <button
              class="mt-2 text-sm bg-green-100 px-2 py-1 rounded"
              on:click={proposeSession}
            >
              Create a new session
            </button>
          {/if}
        {/if}
      </div>
    </div>

    <!-- Server Messages -->
    <div class="mt-4">
      <div class="flex justify-between items-center">
        <h3 class="font-semibold">Server Messages:</h3>
        <div class="flex gap-2">
          <button
            class="text-sm bg-blue-200 px-2 py-1 rounded"
            on:click={forceRefreshState}
            title="Force refresh state from background"
          >
            Refresh
          </button>
          <button
            class="text-sm bg-blue-200 px-2 py-1 rounded"
            on:click={runDiagnostics}
            title="Run connection diagnostics"
          >
            Diagnose
          </button>
          <button
            class="text-sm bg-red-200 px-2 py-1 rounded"
            on:click={resetConnections}
            title="Reset WebRTC connections"
          >
            Reset
          </button>
          <button
            class="text-sm bg-gray-200 px-2 py-1 rounded"
            on:click={() => (serverMessages = [])}
          >
            Clear Log
          </button>
        </div>
      </div>
      {#if serverMessages && serverMessages.length > 0}
        <div class="bg-gray-50 p-2 rounded max-h-60 overflow-y-auto">
          {#each serverMessages as msg, i (i)}
            <pre
              class="text-xs whitespace-pre-wrap break-all mb-1 p-1 border-b {msg.type ===
              'error'
                ? 'text-red-600'
                : ''}">{JSON.stringify(msg, null, 2)}</pre>
          {/each}
        </div>
      {:else}
        <p class="text-gray-500">No messages from server yet.</p>
      {/if}
    </div>
  </div>

  {#if isSettings}
    <Settings
      on:close={() => {
        isSettings = false;
      }}
    />
  {/if}

  <p class="text-gray-500 mt-8">
    Click on the WXT and Svelte logos to learn more
  </p>
</main>

<style>
  :global(body) {
    width: 800px;
    height: 600px;
    overflow: auto;
  }
</style>
