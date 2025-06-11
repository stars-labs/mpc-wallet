<script lang="ts">
  import svelteLogo from "../../assets/svelte.svg";
  // Removed single-party WASM functions - this is now an MPC-only wallet
  import { onMount, onDestroy } from "svelte";
  import { storage } from "#imports";
  import type { AppState } from "../../types/appstate";
  import { MeshStatusType } from "../../types/mesh";
  import { DkgState } from "../../types/dkg";
  import { INITIAL_APP_STATE } from "../../types/appstate";
  import Settings from "../../components/Settings.svelte";

  // Application state (consolidated from background) - the single source of truth
  let appState: AppState = { ...INITIAL_APP_STATE };

  // Keep connection to background script
  let port: chrome.runtime.Port;

  // Local storage for UI preferences persistence (not real-time connection state)
  const UI_STATE_KEY = "mpc_wallet_ui_preferences";

  // Save ONLY UI preferences to localStorage (not real-time connection states)
  function saveUIState() {
    const uiState = {
      showSettings: appState.showSettings,
      proposedSessionIdInput: appState.proposedSessionIdInput,
      totalParticipants: appState.totalParticipants,
      threshold: appState.threshold,
      chain: appState.chain,
      timestamp: Date.now(),
    };
    try {
      localStorage.setItem(UI_STATE_KEY, JSON.stringify(uiState));
      console.log("[UI] Saved UI preferences to localStorage:", uiState);
    } catch (error) {
      console.warn("[UI] Failed to save UI preferences:", error);
    }
  }

  // Load ONLY UI preferences from localStorage (not real-time connection states)
  function loadUIState(): Partial<AppState> {
    try {
      const stored = localStorage.getItem(UI_STATE_KEY);
      if (stored) {
        const uiState = JSON.parse(stored);
        // Check if state is not too old (1 hour)
        if (Date.now() - uiState.timestamp < 60 * 60 * 1000) {
          console.log("[UI] Loaded UI preferences from localStorage:", uiState);
          return {
            showSettings: uiState.showSettings || false,
            proposedSessionIdInput: uiState.proposedSessionIdInput || "",
            totalParticipants: uiState.totalParticipants || 3,
            threshold: uiState.threshold || 2,
            chain: uiState.chain || "ethereum",
          };
        } else {
          console.log("[UI] UI preferences expired, using defaults");
          localStorage.removeItem(UI_STATE_KEY);
        }
      }
    } catch (error) {
      console.warn("[UI] Failed to load UI preferences:", error);
      localStorage.removeItem(UI_STATE_KEY);
    }
    return {};
  }

  // Debug logging to console (throttled to prevent spam)
  let lastDebugLog = "";
  $: {
    const debugInfo = JSON.stringify({
      dkgState: appState.dkgState,
      chainChanged: appState.chain,
      sessionActive: !!appState.sessionInfo,
      meshReady: appState.meshStatus?.type === MeshStatusType.Ready,
    });

    // Only log when significant state changes occur
    if (debugInfo !== lastDebugLog) {
      console.log("[UI Debug] Significant state change:", {
        dkgState: appState.dkgState,
        chain: appState.chain,
        hasSession: !!appState.sessionInfo,
        meshReady: appState.meshStatus?.type === MeshStatusType.Ready,
      });
      lastDebugLog = debugInfo;
    }
  }

  // Reactive computation for WebRTC connection status
  $: webrtcConnected =
    appState.sessionInfo && appState.meshStatus?.type === MeshStatusType.Ready;
  $: webrtcConnecting =
    appState.sessionInfo &&
    appState.meshStatus?.type === MeshStatusType.PartiallyReady;

  // Add reactive statement to log WebRTC connection changes (throttled)
  let lastWebRTCState = "";
  $: {
    // Only log when WebRTC connection state actually changes
    const webrtcState = JSON.stringify(appState.webrtcConnections);
    if (webrtcState !== lastWebRTCState) {
      console.log(
        "[UI] WebRTC Connections updated:",
        appState.webrtcConnections,
      );
      if (Object.keys(appState.webrtcConnections).length > 0) {
        console.log(
          "[UI] Active WebRTC connections:",
          Object.entries(appState.webrtcConnections).filter(
            ([_, connected]) => connected,
          ),
        );
      }
      lastWebRTCState = webrtcState;
    }
  }

  // Common handler for messages from the background script
  function handleBackgroundMessage(message: any) {
    console.log("[UI] Background message received - Type:", message.type, "Data:", message);

    switch (message.type) {
      case "initialState":
        console.log(
          "[UI] Processing initialState - state restoration from background",
        );

        // Load persisted UI preferences from localStorage (ONLY UI state, not real-time)
        const persistedUIState = loadUIState();

        // Preserve current UI-specific preferences that aren't managed by background
        const currentUIState = {
          showSettings: appState.showSettings,
          proposedSessionIdInput: appState.proposedSessionIdInput,
          totalParticipants: appState.totalParticipants,
          threshold: appState.threshold,
          ...persistedUIState, // Override with persisted preferences if available
        };

        // Update the entire app state from background - real-time state comes from background
        appState = {
          // Real-time state from background (never from localStorage)
          deviceId: message.deviceId || "",
          connecteddevices: [...(message.connecteddevices || [])],
          wsConnected: message.wsConnected || false,
          sessionInfo: message.sessionInfo || null,
          invites: message.invites ? [...message.invites] : [],
          meshStatus: message.meshStatus || { type: MeshStatusType.Incomplete },
          dkgState: message.dkgState || DkgState.Idle,
          webrtcConnections: message.webrtcConnections || {},
          curve:
            message.curve ||
            (message.blockchain === "ethereum" ? "secp256k1" : "ed25519"),
          uiPreferences: message.uiPreferences || {
            darkMode: false,
            language: "en",
            showAdvanced: false,
          },
          // UI preferences - preserve from current popup state or use localStorage
          showSettings: currentUIState.showSettings,
          chain: message.blockchain || currentUIState.chain || "ethereum",
          proposedSessionIdInput: currentUIState.proposedSessionIdInput,
          totalParticipants: currentUIState.totalParticipants,
          threshold: currentUIState.threshold,
          // Other state fields
          dkgAddress: appState.dkgAddress || "",
          dkgError: appState.dkgError || "",
          sessionAcceptanceStatus:
            message.sessionAcceptanceStatus ||
            appState.sessionAcceptanceStatus ||
            {},
          wsError: message.wsError || appState.wsError || "",
          isInitializing: message.isInitializing,
          globalError: message.globalError,
          setupComplete: message.setupComplete,
        };        console.log("[UI] App state updated from initialState:", appState);
        initialStateLoaded = true; // Enable reactive saving after state is loaded
        
        // Save the current UI preferences immediately after loading to ensure persistence
        saveUIState();
        
        // NOTE: No need for fresh state update - all real-time state comes from background automatically
        // WebSocket updates, device lists, etc. are pushed via port messages, not pulled via getState
        break;

      case "wsStatus":
        console.log("[UI] Processing wsStatus:", message);
        appState.wsConnected = message.connected || false;
        if (!message.connected && message.reason) {
          appState.wsError = `WebSocket disconnected: ${message.reason}`;
        } else if (message.connected) {
          appState.wsError = "";
        }
        // Trigger reactivity
        appState = { ...appState };
        break;

      case "wsMessage":
        console.log("[UI] Processing wsMessage:", message);
        if (message.message) {
          console.log("[UI] Server message:", message.message);
          if (message.message.type === "devices") {
            appState.connecteddevices = [...(message.message.devices || [])];
            appState = { ...appState };
          }
        }
        break;

      case "wsError":
        console.log("[UI] Processing wsError:", message);
        appState.wsError = message.error;
        console.error("[UI] WebSocket error:", message.error);
        // Trigger reactivity
        appState = { ...appState };
        break;

      case "deviceList":
        console.log("[UI] Processing deviceList:", message);
        appState.connecteddevices = [...(message.devices || [])];
        appState = { ...appState };
        break;

      case "sessionUpdate":
        console.log("[UI] Processing sessionUpdate:", message);
        appState.sessionInfo = message.sessionInfo || null;
        appState.invites = message.invites ? [...message.invites] : [];
        console.log("[UI] Session update:", {
          sessionInfo: appState.sessionInfo,
          invites: appState.invites,
        });

        // Log accepted devices for debugging
        if (appState.sessionInfo && appState.sessionInfo.accepted_devices) {
          console.log(
            "[UI] Session accepted devices:",
            appState.sessionInfo.accepted_devices,
          );
          // Filter out any null/undefined values that might have been added
          appState.sessionInfo.accepted_devices =
            appState.sessionInfo.accepted_devices.filter(
              (peer) => peer != null && peer !== undefined,
            );
        }
        // Trigger reactivity
        appState = { ...appState };
        break;

      case "meshStatusUpdate":
        console.log("[UI] Processing meshStatusUpdate:", message);
        appState.meshStatus = message.status || {
          type: MeshStatusType.Incomplete,
        };
        console.log("[UI] Mesh status update:", appState.meshStatus);
        // Trigger reactivity
        appState = { ...appState };
        break;

      case "webrtcConnectionUpdate":
        console.log("[UI] Processing webrtcConnectionUpdate:", message);

        if (message.deviceId && typeof message.connected === "boolean") {
          console.log(
            "[UI] Updating peer connection:",
            message.deviceId,
            "->",
            message.connected,
          );

          // Update WebRTC connections in app state
          appState.webrtcConnections = {
            ...appState.webrtcConnections,
            [message.deviceId]: message.connected,
          };

          console.log(
            "[UI] Updated webrtcConnections:",
            appState.webrtcConnections,
          );

          // Trigger reactivity
          appState = { ...appState };
        } else {
          console.warn("[UI] Invalid webrtcConnectionUpdate message:", message);
        }
        break;

      case "dkgStateUpdate":
        console.log("[UI] Processing dkgStateUpdate:", message);
        appState.dkgState = message.state || DkgState.Idle;
        console.log("[UI] DKG state update:", appState.dkgState);
        // Trigger reactivity
        appState = { ...appState };
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
        if (message.deviceId && message.status) {
          console.log(
            `[UI] WebRTC status for ${message.deviceId}: ${message.status}`,
          );
          // Update UI state based on WebRTC status if needed
        }
        break;

      case "dataChannelStatusUpdate":
        console.log("[UI] Processing dataChannelStatusUpdate:", message);
        if (message.deviceId && message.channelName && message.state) {
          console.log(
            `[UI] Data channel ${message.channelName} for ${message.deviceId}: ${message.state}`,
          );
        }
        break;

      case "peerConnectionStatusUpdate":
        console.log("[UI] Processing peerConnectionStatusUpdate:", message);
        if (message.deviceId && message.connectionState) {
          console.log(
            `[UI] Peer connection for ${message.deviceId}: ${message.connectionState}`,
          );
        }
        break;

      case "dkgAddressUpdate":
        console.log("[UI] Processing dkgAddressUpdate:", message);
        if (message.address && message.blockchain) {
          console.log("[UI] DKG address automatically fetched:", message.address, "for", message.blockchain);
          appState.dkgAddress = message.address;
          appState.dkgError = "";
          // Trigger reactivity
          appState = { ...appState };
        }
        break;

      default:
        console.log("[UI] Unhandled message type:", message.type, message);
    }
  }

  // Removed ensurePrivateKey() and ensureOffscreenDocument() - this is now an MPC-only wallet
  // Offscreen document management is handled entirely by the background script

  onMount(async () => {
    console.log("[UI] Component mounting");

    // Set up state tracking before connecting port
    let stateReceived = false;

    // Initialize as false to prevent reactive statements from running
    initialStateLoaded = false;

    port = chrome.runtime.connect({ name: "popup" });
    console.log("[UI] Port connected to background, waiting for initial state...");
    
    port.onMessage.addListener((message) => {
      console.log("[UI] Port message received:", message.type, message);
      // Track when initial state is received
      if (message.type === "initialState" && !stateReceived) {
        stateReceived = true;
        console.log("[UI] Initial state received successfully from StateManager");
      }
      handleBackgroundMessage(message);
    });

    port.onDisconnect.addListener(() => {
      console.error("[UI] Port disconnected from background");
      appState.wsConnected = false;
    });

    console.log("[UI] Port connected, StateManager should automatically send state...");

    // Add fallback in case automatic state is delayed (network issues, etc.)
    setTimeout(() => {
      if (!stateReceived) {
        console.warn("[UI] Automatic state not received within 1 second, requesting manually as fallback");
        chrome.runtime.sendMessage({ type: "getState" }, (response) => {
          if (chrome.runtime.lastError) {
            console.error("[UI] Fallback getState error:", chrome.runtime.lastError.message);
            return;
          }
          if (response && !stateReceived) {
            console.log("[UI] Fallback state response received:", response);
            handleBackgroundMessage({
              type: "initialState",
              ...response,
            });
            stateReceived = true;
          }
        });
      } else {
        console.log("[UI] State was received automatically, no fallback needed");
      }
    }, 1000); // 1 second timeout

    // Removed ensurePrivateKey() call - this is now an MPC-only wallet
    // Removed ensureOffscreenDocument() call - offscreen management is handled by background script
  });

  onDestroy(() => {
    if (port) {
      port.disconnect();
    }
  });

  // Removed reactive statement for ensurePrivateKey() - this is now an MPC-only wallet

  // Removed single-party reactive statements - this is now an MPC-only wallet

  // REMOVED: All reactive business logic moved to background script
  // The popup should ONLY contain pure UI reactive statements
  // All blockchain selection, state management, etc. happens in background
  // DKG address fetching is now handled automatically by StateManager

  // Reactive statements to save UI preferences to localStorage (UI-only)
  // Only save after initial state has been loaded to prevent premature saves
  let initialStateLoaded = false;
  let lastSavedState = "";

  // Throttled save function to prevent excessive localStorage writes (UI preferences only)
  function throttledSaveUIState() {
    const currentStateStr = JSON.stringify({
      showSettings: appState.showSettings,
      proposedSessionIdInput: appState.proposedSessionIdInput,
      totalParticipants: appState.totalParticipants,
      threshold: appState.threshold,
      chain: appState.chain
    });
    
    // Only save if UI preferences actually changed
    if (currentStateStr !== lastSavedState) {
      saveUIState();
      lastSavedState = currentStateStr;
    }
  }

  // Save showSettings changes
  $: if (initialStateLoaded && typeof appState.showSettings !== "undefined") {
    throttledSaveUIState();
  }

  // Save proposedSessionIdInput changes
  $: if (
    initialStateLoaded &&
    typeof appState.proposedSessionIdInput !== "undefined"
  ) {
    throttledSaveUIState();
  }

  // Save totalParticipants changes
  $: if (
    initialStateLoaded &&
    typeof appState.totalParticipants !== "undefined"
  ) {
    throttledSaveUIState();
  }

  // Save threshold changes
  $: if (initialStateLoaded && typeof appState.threshold !== "undefined") {
    throttledSaveUIState();
  }

  // Save chain changes (in addition to the blockchain message sending)
  $: if (initialStateLoaded && typeof appState.chain !== "undefined") {
    throttledSaveUIState();
  }

  // Removed fetchAddress() and fetchDkgAddress() - this is now an MPC-only wallet
  // DKG address fetching is now handled automatically by StateManager when DKG completes

  // Removed signDemoMessage() - this is now an MPC-only wallet

  function requestdeviceList() {
    console.log("[UI] Requesting peer list");
    chrome.runtime.sendMessage({ type: "listdevices" }, (response) => {
      if (chrome.runtime.lastError) {
        console.error(
          "[UI] Error requesting peer list:",
          chrome.runtime.lastError.message,
        );
        return;
      }
      console.log("[UI] listdevices response:", response);
    });
  }

  function proposeSession() {
    const availabledevices = appState.connecteddevices.filter(
      (p) => p !== appState.deviceId,
    );

    if (availabledevices.length < appState.totalParticipants - 1) {
      console.error(
        `Need at least ${appState.totalParticipants - 1} other devices for a ${appState.totalParticipants}-participant session`,
      );
      return;
    }

    if (appState.threshold > appState.totalParticipants) {
      console.error("Threshold cannot be greater than total participants");
      return;
    }

    if (appState.threshold < 1) {
      console.error("Threshold must be at least 1");
      return;
    }

    const devicesToInclude = availabledevices.slice(
      0,
      appState.totalParticipants - 1,
    );
    const allParticipants = [appState.deviceId, ...devicesToInclude];

    const sessionId =
      appState.proposedSessionIdInput.trim() ||
      `wallet_${appState.threshold}of${appState.totalParticipants}_${Date.now()}`;

    chrome.runtime.sendMessage({
      type: "proposeSession",
      session_id: sessionId,
      total: appState.totalParticipants,
      threshold: appState.threshold,
      participants: allParticipants,
    });
    console.log(
      "[UI] Proposing session:",
      sessionId,
      `(${appState.threshold}-of-${appState.totalParticipants})`,
      "with participants:",
      allParticipants,
    );
  }

  function acceptInvite(sessionId: string) {
    chrome.runtime.sendMessage({
      type: "acceptSession",
      session_id: sessionId,
      accepted: true,
      blockchain: appState.chain, // Include blockchain selection
    });
    console.log(
      "[UI] Accepting session invite:",
      sessionId,
      "with blockchain:",
      appState.chain,
    );
  }

  // Add function to send direct message for testing
  function sendDirectMessage(todeviceId: string) {
    const testMessage = `Hello from ${appState.deviceId} at ${new Date().toLocaleTimeString()}`;
    chrome.runtime.sendMessage(
      {
        type: "sendDirectMessage",
        todeviceId: todeviceId,
        message: testMessage,
      },
      (response) => {
        if (chrome.runtime.lastError) {
          console.error(
            "[UI] Error sending direct message:",
            chrome.runtime.lastError.message,
          );
        } else {
          console.log("[UI] Direct message response:", response);
          if (!response.success) {
            console.error(`Failed to send message: ${response.error}`);
          }
        }
      },
    );
    console.log(
      "[UI] Sending direct message to:",
      todeviceId,
      "Message:",
      testMessage,
    );
  }

  // Helper function to get WebRTC status for a peer
  function getWebRTCStatus(
    deviceId: string,
  ): "connected" | "connecting" | "disconnected" {
    console.log(
      "[UI] Getting WebRTC status for peer:",
      deviceId,
      "from webrtcConnections:",
      appState.webrtcConnections,
    );

    // Check direct connection status first
    if (appState.webrtcConnections[deviceId] === true) {
      return "connected";
    } else if (
      appState.sessionInfo &&
      appState.sessionInfo.participants.includes(deviceId) &&
      appState.meshStatus?.type === MeshStatusType.PartiallyReady
    ) {
      return "connecting";
    } else {
      return "disconnected";
    }
  }

  // Helper function to get session acceptance status
  function getSessionAcceptanceStatus(
    sessionId: string,
    deviceId: string,
  ): boolean | undefined {
    if (!appState.sessionAcceptanceStatus[sessionId]) {
      return undefined;
    }
    return appState.sessionAcceptanceStatus[sessionId][deviceId];
  }
</script>

<main class="p-4 max-w-2xl mx-auto">
  <div class="text-center mb-6 flex justify-between items-center">
    <img src={svelteLogo} class="logo svelte mb-2" alt="Svelte Logo" />
    <h1 class="text-3xl font-bold flex-grow text-center">MPC Wallet</h1>
    <button
      class="bg-blue-500 hover:bg-blue-600 text-white p-2 rounded-full"
      on:click={() => {
        appState.showSettings = !appState.showSettings;
        appState = { ...appState };
      }}
      aria-label="Settings"
      title="Settings"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-6 w-6"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
        />
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
        />
      </svg>
    </button>
  </div>

  {#if appState.showSettings}
    <Settings
      on:backToWallet={({ detail }) => {
        if (detail.chain === "ethereum" || detail.chain === "solana") {
          appState.chain = detail.chain;
        }
        if (detail.curve === "secp256k1" || detail.curve === "ed25519") {
          appState.curve = detail.curve;
        }
        appState.showSettings = false;
        appState = { ...appState };
      }}
    />
  {:else}
    <!-- Wallet Status Banner -->
    <div class="mb-4 p-3 border rounded">
      <div class="mb-2">
        <div class="font-bold">Current Network:</div>
      </div>

      <div class="p-2 bg-blue-50 border border-blue-200 rounded mb-2">
        <p class="text-blue-700">
          {appState.chain === "ethereum"
            ? "Ethereum (secp256k1)"
            : "Solana (ed25519)"}
        </p>
      </div>

      {#if appState.sessionInfo && appState.dkgState === DkgState.Complete}
        <div class="p-2 bg-green-50 border border-green-200 rounded">
          <p class="text-sm text-green-700">
            ✓ DKG Complete - MPC addresses available for {appState.chain}
          </p>
        </div>
      {:else if appState.sessionInfo && appState.dkgState !== DkgState.Idle}
        <div class="p-2 bg-yellow-50 border border-yellow-200 rounded">
          <p class="text-sm text-yellow-700">
            🔄 DKG in progress - MPC addresses will be available when complete
          </p>
        </div>
      {/if}
    </div>
  {/if}

  <!-- DKG Address Display (Moved from MPC Wallet Operations) -->
  {#if appState.dkgAddress}
    <div class="mb-4 p-3 border rounded">
      <h2 class="text-xl font-semibold mb-2">MPC Address</h2>
      <div>
        <span class="block font-bold mb-1"
          >{appState.chain === "ethereum" ? "Ethereum" : "Solana"} Address:</span
        >
        <code
          class="block bg-purple-50 border border-purple-200 p-2 rounded break-all"
          >{appState.dkgAddress}</code
        >
        <p class="text-xs text-purple-600 mt-1">
          ✓ Generated using {appState.sessionInfo?.threshold}-of-{appState
            .sessionInfo?.total} threshold signature
        </p>
      </div>

      {#if appState.dkgError}
        <div class="mt-2 p-2 bg-red-50 border border-red-200 rounded">
          <span class="text-red-600 text-sm">{appState.dkgError}</span>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Network Status -->
  <div class="mb-4 p-3 border rounded">
    <h2 class="text-xl font-semibold mb-3">Network Status</h2>

    <div class="grid grid-cols-1 gap-3 mb-3">
      <div>
        <span class="block font-bold mb-1">Peer ID:</span>
        <code class="block bg-gray-100 p-2 rounded text-sm">
          {appState.deviceId || "Not connected"}
        </code>
      </div>
      <div>
        <span class="block font-bold mb-1">WebSocket:</span>
        <span
          class="inline-block px-2 py-1 rounded text-sm {appState.wsConnected
            ? 'bg-green-100 text-green-800'
            : 'bg-red-100 text-red-800'}"
        >
          {appState.wsConnected ? "Connected" : "Disconnected"}
        </span>
      </div>
    </div>
  </div>

  <!-- Connected devices with Individual WebRTC Status -->
  <div class="mb-4 p-3 border rounded">
    <h2 class="text-xl font-semibold mb-3">
      Connected devices ({appState.connecteddevices.length})
    </h2>

    {#if appState.connecteddevices && appState.connecteddevices.length > 0}
      <ul class="space-y-2">
        {#each appState.connecteddevices as peer}
          <li class="flex items-center justify-between p-3 bg-gray-50 rounded">
            <div class="flex items-center gap-3">
              <code class="text-sm font-mono">{peer}</code>
              {#if peer === appState.deviceId}
                <span
                  class="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded"
                  >You</span
                >
              {/if}
            </div>

            {#if peer !== appState.deviceId}
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-500">WebRTC:</span>
                {#if getWebRTCStatus(peer) === "connected"}
                  <span
                    class="text-xs bg-green-100 text-green-800 px-2 py-1 rounded"
                    >Connected</span
                  >
                  {#if appState.sessionInfo && appState.meshStatus?.type === MeshStatusType.Ready}
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
      <p class="text-gray-500 text-center py-4">No devices connected</p>
    {/if}
  </div>

  <!-- MPC Session Management -->
  <div class="mb-4 p-3 border rounded">
    <h2 class="text-xl font-semibold mb-3">MPC Session</h2>

    {#if appState.sessionInfo}
      <!-- Active Session -->
      <div class="bg-green-50 border border-green-200 rounded p-3 mb-3">
        <h3 class="font-bold text-green-800 mb-2">Active Session</h3>
        <div class="grid grid-cols-2 gap-2 text-sm">
          <div><strong>ID:</strong> {appState.sessionInfo.session_id}</div>
          <div>
            <strong>Proposer:</strong>
            {appState.sessionInfo.proposer_id}
          </div>
          <div>
            <strong>Threshold:</strong>
            {appState.sessionInfo.threshold} of
            {appState.sessionInfo.total}
          </div>
          <div>
            <strong>Mesh:</strong>
            {MeshStatusType[appState.meshStatus?.type] || "Unknown"}
          </div>
          <div>
            <strong>DKG:</strong>
            {DkgState[appState.dkgState] || "Unknown"}
          </div>
        </div>
        <div class="mt-2">
          <span class="font-bold">Participants:</span>
          <div class="flex flex-wrap gap-1 mt-1">
            {#each appState.sessionInfo.participants as participant}
              <span
                class="text-xs bg-gray-100 px-2 py-1 rounded {participant ===
                appState.deviceId
                  ? 'bg-blue-100'
                  : ''}"
              >
                {participant}{participant === appState.deviceId ? " (you)" : ""}
              </span>
            {/each}
          </div>
        </div>
        <div class="mt-2">
          <span class="font-bold">Accepted:</span>
          <div class="flex flex-wrap gap-1 mt-1">
            {#each appState.sessionInfo.accepted_devices || [] as accepted}
              <span
                class="text-xs bg-green-100 text-green-800 px-2 py-1 rounded"
              >
                {accepted}
              </span>
            {/each}
          </div>
        </div>
      </div>
    {:else if appState.invites && appState.invites.length > 0}
      <!-- Pending Invitations -->
      <div class="space-y-3">
        {#each appState.invites as invite}
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
            bind:value={appState.proposedSessionIdInput}
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
              bind:value={appState.totalParticipants}
              min="2"
              max={appState.connecteddevices.length}
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
              bind:value={appState.threshold}
              min="1"
              max={appState.totalParticipants}
              class="w-full border p-2 rounded"
            />
          </div>
        </div>

        <button
          class="w-full bg-indigo-500 hover:bg-indigo-700 text-white font-bold py-2 px-4 rounded"
          on:click={proposeSession}
          disabled={!appState.wsConnected ||
            appState.connecteddevices.filter((p) => p !== appState.deviceId)
              .length <
              appState.totalParticipants - 1 ||
            appState.threshold > appState.totalParticipants ||
            appState.threshold < 1}
        >
          Propose New Session ({appState.threshold}-of-{appState.totalParticipants})
        </button>

        {#if appState.connecteddevices.filter((p) => p !== appState.deviceId).length < appState.totalParticipants - 1}
          <p class="text-sm text-gray-500 text-center">
            Need at least {appState.totalParticipants - 1} other devices for a {appState.totalParticipants}-participant
            session
          </p>
        {:else if appState.threshold > appState.totalParticipants || appState.threshold < 1}
          <p class="text-sm text-red-500 text-center">
            Invalid threshold: must be between 1 and {appState.totalParticipants}
          </p>
        {/if}
      </div>
    {/if}
  </div>

  <!-- WebSocket Error Display -->
  {#if appState.wsError}
    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded">
      <div class="flex justify-between items-center">
        <span class="text-red-600">{appState.wsError}</span>
        <button
          class="text-sm bg-red-100 hover:bg-red-200 px-2 py-1 rounded"
          on:click={() => {
            appState.wsError = "";
            appState = { ...appState };
          }}
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

  /* Dark mode styles */
  :global(.dark) {
    color-scheme: dark;
  }

  :global(.dark body) {
    background-color: #1a1a1a;
    color: #e5e5e5;
  }

  :global(.dark .border) {
    border-color: #333333;
  }

  :global(.dark .bg-gray-50) {
    background-color: #262626;
  }

  :global(.dark .bg-gray-100) {
    background-color: #333333;
  }

  :global(.dark .bg-green-50) {
    background-color: #064e3b;
    color: #a7f3d0;
  }

  :global(.dark .bg-yellow-50) {
    background-color: #78350f;
    color: #fde68a;
  }

  :global(.dark .bg-blue-50) {
    background-color: #082f49;
    color: #bae6fd;
  }

  :global(.dark .text-green-700) {
    color: #a7f3d0;
  }

  :global(.dark .text-yellow-700) {
    color: #fde68a;
  }

  :global(.dark .text-blue-700) {
    color: #bae6fd;
  }

  :global(.dark .text-blue-600) {
    color: #60a5fa;
  }

  :global(.dark .bg-blue-500) {
    background-color: #2563eb;
  }

  :global(.dark .bg-blue-600) {
    background-color: #1d4ed8;
  }

  :global(.dark .border-green-200) {
    border-color: #065f46;
  }

  :global(.dark .border-yellow-200) {
    border-color: #92400e;
  }

  :global(.dark .border-blue-200) {
    border-color: #0c4a6e;
  }

  .logo {
    height: 40px;
    width: 40px;
  }
</style>
