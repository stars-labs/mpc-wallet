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
    import AccountManager from "../../components/AccountManager.svelte";

    // Application state (consolidated from background) - the single source of truth
    let appState: AppState = { ...INITIAL_APP_STATE };

    // Keep connection to background script
    let port: chrome.runtime.Port;
    
    // UI state flags
    let acceptingSession = false; // Prevent multiple session accept clicks

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
                    console.log(
                        "[UI] Loaded UI preferences from localStorage:",
                        uiState,
                    );
                    return {
                        showSettings: uiState.showSettings || false,
                        proposedSessionIdInput:
                            uiState.proposedSessionIdInput || "",
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
        appState.sessionInfo &&
        appState.meshStatus?.type === MeshStatusType.Ready;
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
        console.log(
            "[UI] Background message received - Type:",
            message.type,
            "Data:",
            message,
        );

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
                    meshStatus: message.meshStatus || {
                        type: MeshStatusType.Incomplete,
                    },
                    dkgState: message.dkgState || DkgState.Idle,
                    webrtcConnections: message.webrtcConnections || {},
                    curve:
                        message.curve ||
                        (message.blockchain === "ethereum"
                            ? "secp256k1"
                            : "ed25519"),
                    uiPreferences: message.uiPreferences || {
                        darkMode: false,
                        language: "en",
                        showAdvanced: false,
                    },
                    // UI preferences - preserve from current popup state or use localStorage
                    showSettings: currentUIState.showSettings,
                    chain:
                        message.blockchain ||
                        currentUIState.chain ||
                        "ethereum",
                    proposedSessionIdInput:
                        currentUIState.proposedSessionIdInput,
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
                };
                console.log(
                    "[UI] App state updated from initialState:",
                    appState,
                );
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
                    // Device list updates are handled via "deviceList" messages
                    // No need to handle them here
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
                // Only update if we have a valid devices array
                if (Array.isArray(message.devices)) {
                    appState.connecteddevices = [...message.devices];
                    console.log("[UI] Updated connected devices:", appState.connecteddevices);
                } else {
                    console.warn("[UI] Invalid device list received:", message.devices);
                }
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
                if (
                    appState.sessionInfo &&
                    appState.sessionInfo.accepted_devices
                ) {
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

                if (
                    message.deviceId &&
                    typeof message.connected === "boolean"
                ) {
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
                    console.warn(
                        "[UI] Invalid webrtcConnectionUpdate message:",
                        message,
                    );
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
                console.log(
                    "[UI] Processing dataChannelStatusUpdate:",
                    message,
                );
                if (message.deviceId && message.channelName && message.state) {
                    console.log(
                        `[UI] Data channel ${message.channelName} for ${message.deviceId}: ${message.state}`,
                    );
                }
                break;

            case "peerConnectionStatusUpdate":
                console.log(
                    "[UI] Processing peerConnectionStatusUpdate:",
                    message,
                );
                if (message.deviceId && message.connectionState) {
                    console.log(
                        `[UI] Peer connection for ${message.deviceId}: ${message.connectionState}`,
                    );
                }
                break;

            case "dkgAddressUpdate":
                console.log("[UI] Processing dkgAddressUpdate:", message);
                if (message.address && message.blockchain) {
                    console.log(
                        "[UI] DKG address automatically fetched:",
                        message.address,
                        "for",
                        message.blockchain,
                    );
                    appState.dkgAddress = message.address;
                    appState.dkgError = "";
                    // Trigger reactivity
                    appState = { ...appState };
                }
                break;

            default:
                console.log(
                    "[UI] Unhandled message type:",
                    message.type,
                    message,
                );
        }
    }

    // Removed ensurePrivateKey() and ensureOffscreenDocument() - this is now an MPC-only wallet
    // Offscreen document management is handled entirely by the background script

    onMount(async () => {
        console.log("[UI] Component mounting");

        // Set up state tracking before connecting port
        let stateReceived = false;
        let fallbackTimeoutId: ReturnType<typeof setTimeout>;

        // Initialize as false to prevent reactive statements from running
        initialStateLoaded = false;

        port = chrome.runtime.connect({ name: "popup" });
        console.log(
            "[UI] Port connected to background, waiting for initial state...",
        );

        port.onMessage.addListener((message) => {
            console.log("[UI] Port message received:", message.type, message);
            // Track when initial state is received
            if (message.type === "initialState" && !stateReceived) {
                stateReceived = true;
                console.log(
                    "[UI] Initial state received successfully from StateManager",
                );
                // Clear fallback timeout since we received state
                if (fallbackTimeoutId) {
                    clearTimeout(fallbackTimeoutId);
                }
            }
            handleBackgroundMessage(message);
        });

        port.onDisconnect.addListener(() => {
            console.error("[UI] Port disconnected from background");
            appState.wsConnected = false;
            // Clear fallback timeout if port disconnects
            if (fallbackTimeoutId) {
                clearTimeout(fallbackTimeoutId);
            }
        });

        console.log(
            "[UI] Port connected, StateManager should automatically send state...",
        );

        // Add fallback in case automatic state is delayed (StateManager still loading, etc.)
        fallbackTimeoutId = setTimeout(() => {
            if (!stateReceived) {
                console.warn(
                    "[UI] Automatic state not received within 2 seconds, requesting manually as fallback",
                );
                chrome.runtime.sendMessage({ type: "getState" }, (response) => {
                    if (chrome.runtime.lastError) {
                        console.error(
                            "[UI] Fallback getState error:",
                            chrome.runtime.lastError.message,
                        );
                        return;
                    }
                    if (response && !stateReceived) {
                        console.log(
                            "[UI] Fallback state response received:",
                            response,
                        );
                        handleBackgroundMessage({
                            type: "initialState",
                            ...response,
                        });
                        stateReceived = true;
                    }
                });
            } else {
                console.log(
                    "[UI] State was received automatically, no fallback needed",
                );
            }
        }, 2000); // Increased to 2 seconds to account for async state loading

        // Removed ensurePrivateKey() call - this is now an MPC-only wallet
        // Removed ensureOffscreenDocument() call - offscreen management is handled by background script
    });

    onDestroy(() => {
        console.log("[UI] Component destroying, cleaning up port connection");
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
            chain: appState.chain,
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
            console.error(
                "Threshold cannot be greater than total participants",
            );
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
            blockchain: appState.chain, // Include blockchain selection
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
        // Prevent multiple clicks
        if (acceptingSession) {
            console.warn("[UI] Already processing a session acceptance");
            return;
        }
        
        // Check if invite still exists before accepting
        const invite = appState.invites.find(inv => inv.session_id === sessionId);
        if (!invite) {
            console.warn("[UI] Session invite not found:", sessionId);
            return;
        }
        
        // Check if we already have an active session
        if (appState.sessionInfo && appState.sessionInfo.session_id === sessionId) {
            console.warn("[UI] Session already accepted:", sessionId);
            return;
        }
        
        // Set flag to prevent multiple clicks
        acceptingSession = true;
        
        chrome.runtime.sendMessage({
            type: "acceptSession",
            session_id: sessionId,
            accepted: true,
            blockchain: appState.chain, // Include blockchain selection
        }, (response) => {
            acceptingSession = false; // Reset flag
            console.log("[UI] Accept session response:", response);
            if (!response || !response.success) {
                console.error("[UI] Failed to accept session:", response?.error || "Unknown error");
                // Restore the invite if acceptance failed
                appState.invites = [...appState.invites, invite];
            }
        });
        console.log(
            "[UI] Accepting session invite:",
            sessionId,
            "with blockchain:",
            appState.chain,
        );
        
        // Optimistically update UI to show processing state
        appState.invites = appState.invites.filter(inv => inv.session_id !== sessionId);
    }

    function rejectInvite(sessionId: string) {
        // Remove the invite from local state
        appState.invites = appState.invites.filter(inv => inv.session_id !== sessionId);
        console.log("[UI] Rejected session invite:", sessionId);
        
        // Optionally send rejection to background (for future implementation)
        chrome.runtime.sendMessage({
            type: "rejectSession",
            session_id: sessionId
        });
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
                        console.error(
                            `Failed to send message: ${response.error}`,
                        );
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

    // Test MPC signing function
    function testMPCSigning() {
        console.log("[UI] Testing MPC signing");
        
        // Generate a test signing ID and transaction data
        const signingId = `test_signing_${Date.now()}`;
        const testTransactionData = appState.chain === "ethereum" 
            ? "0x" + "00".repeat(32) // Ethereum test transaction hash
            : "test_solana_transaction_" + Date.now(); // Solana test data
        
        // Use the threshold from the current session
        const requiredSigners = appState.sessionInfo?.threshold || 2;
        
        chrome.runtime.sendMessage({
            type: "requestSigning",
            signingId: signingId,
            transactionData: testTransactionData,
            requiredSigners: requiredSigners
        }, (response) => {
            if (chrome.runtime.lastError) {
                console.error("[UI] Error requesting signing:", chrome.runtime.lastError.message);
                return;
            }
            console.log("[UI] Signing request response:", response);
            if (!response.success) {
                console.error(`Failed to initiate signing: ${response.error}`);
            }
        });
        
        console.log("[UI] Sent signing request:", {
            signingId,
            transactionData: testTransactionData,
            requiredSigners
        });
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
                if (
                    detail.curve === "secp256k1" ||
                    detail.curve === "ed25519"
                ) {
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
                        🔄 DKG in progress - MPC addresses will be available
                        when complete
                    </p>
                </div>
            {/if}
        </div>
    {/if}

    <!-- Account Manager - Multi-account support -->
    {#if appState.dkgState === DkgState.Complete}
        <div class="mb-4">
            <AccountManager 
                currentAccount={null}
                blockchain={appState.chain}
            />
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
                    <li
                        class="flex items-center justify-between p-3 bg-gray-50 rounded"
                    >
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
                                <span class="text-xs text-gray-500"
                                    >WebRTC:</span
                                >
                                {#if getWebRTCStatus(peer) === "connected"}
                                    <span
                                        class="text-xs bg-green-100 text-green-800 px-2 py-1 rounded"
                                        >Connected</span
                                    >
                                    {#if appState.sessionInfo && appState.meshStatus?.type === MeshStatusType.Ready}
                                        <button
                                            class="text-xs bg-blue-500 hover:bg-blue-700 text-white px-2 py-1 rounded"
                                            on:click={() =>
                                                sendDirectMessage(peer)}
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
            <div class="bg-gradient-to-r from-green-50 to-emerald-50 border-2 border-green-300 rounded-lg p-4 shadow-md">
                <div class="flex items-center justify-between mb-3">
                    <h3 class="text-lg font-bold text-green-800 flex items-center">
                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                        </svg>
                        Active Session
                    </h3>
                    <div class="flex items-center space-x-2">
                        {#if appState.meshStatus?.type === MeshStatusType.Ready}
                            <span class="text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full font-semibold">
                                Mesh Ready
                            </span>
                        {:else if appState.meshStatus?.type === MeshStatusType.PartiallyReady}
                            <span class="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded-full font-semibold animate-pulse">
                                Connecting...
                            </span>
                        {:else}
                            <span class="text-xs bg-red-100 text-red-800 px-2 py-1 rounded-full font-semibold">
                                Not Ready
                            </span>
                        {/if}
                    </div>
                </div>
                
                <div class="bg-white bg-opacity-70 rounded p-3 mb-3">
                    <div class="grid grid-cols-2 gap-3 text-sm">
                        <div>
                            <span class="text-gray-600">Session ID:</span>
                            <p class="font-mono text-xs bg-gray-100 px-2 py-1 rounded mt-1 truncate">
                                {appState.sessionInfo.session_id}
                            </p>
                        </div>
                        <div>
                            <span class="text-gray-600">Threshold:</span>
                            <p class="font-bold text-green-700 text-lg">
                                {appState.sessionInfo.threshold} of {appState.sessionInfo.total}
                            </p>
                        </div>
                        <div>
                            <span class="text-gray-600">DKG Status:</span>
                            <p class="font-semibold {appState.dkgState === DkgState.Complete ? 'text-green-700' : appState.dkgState === DkgState.Failed ? 'text-red-700' : 'text-yellow-700'}">
                                {DkgState[appState.dkgState] || "Unknown"}
                            </p>
                        </div>
                        <div>
                            <span class="text-gray-600">Proposer:</span>
                            <p class="font-mono text-xs truncate {appState.sessionInfo.proposer_id === appState.deviceId ? 'text-blue-700 font-semibold' : ''}">
                                {appState.sessionInfo.proposer_id}{appState.sessionInfo.proposer_id === appState.deviceId ? ' (you)' : ''}
                            </p>
                        </div>
                    </div>
                </div>

                <div class="mb-3">
                    <div class="flex justify-between items-center mb-2">
                        <span class="text-sm font-semibold text-gray-700">Participants:</span>
                        <span class="text-xs text-gray-500">
                            {appState.sessionInfo.accepted_devices?.length || 0}/{appState.sessionInfo.participants.length} accepted
                        </span>
                    </div>
                    <div class="space-y-1">
                        {#each appState.sessionInfo.participants as participant}
                            {@const isAccepted = appState.sessionInfo.accepted_devices?.includes(participant)}
                            {@const isConnected = appState.webrtcConnections[participant]}
                            <div class="flex items-center justify-between p-2 rounded {participant === appState.deviceId ? 'bg-blue-50' : 'bg-gray-50'}">
                                <span class="text-sm font-mono {participant === appState.deviceId ? 'text-blue-700 font-semibold' : ''}">
                                    {participant}{participant === appState.deviceId ? ' (you)' : ''}
                                </span>
                                <div class="flex items-center space-x-2">
                                    {#if isAccepted}
                                        <span class="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded">
                                            Accepted
                                        </span>
                                    {:else}
                                        <span class="text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded">
                                            Pending
                                        </span>
                                    {/if}
                                    {#if participant !== appState.deviceId}
                                        {#if isConnected}
                                            <span class="w-2 h-2 bg-green-500 rounded-full" title="Connected"></span>
                                        {:else}
                                            <span class="w-2 h-2 bg-gray-300 rounded-full" title="Not connected"></span>
                                        {/if}
                                    {/if}
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>

                {#if appState.meshStatus?.type === MeshStatusType.Ready && appState.dkgState === DkgState.Idle}
                    <div class="border-t border-green-200 pt-3">
                        <p class="text-sm text-gray-600 mb-2">
                            All participants are connected. Ready to start the Distributed Key Generation process.
                        </p>
                        <button class="w-full bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white font-bold py-2 px-4 rounded-lg shadow-md">
                            Start DKG Process
                        </button>
                    </div>
                {:else if appState.dkgState === DkgState.Complete}
                    <div class="border-t border-green-200 pt-3">
                        <p class="text-sm text-green-700 font-semibold mb-2">
                            ✅ DKG Complete - Ready for threshold signatures
                        </p>
                        <button
                            class="w-full bg-gradient-to-r from-purple-500 to-purple-600 hover:from-purple-600 hover:to-purple-700 text-white font-bold py-2 px-4 rounded-lg shadow-md"
                            on:click={testMPCSigning}
                        >
                            Test MPC Signing
                        </button>
                    </div>
                {/if}
            </div>
        {:else if appState.invites && appState.invites.length > 0}
            <!-- Pending Invitations -->
            <div class="space-y-3">
                {#each appState.invites as invite}
                    <div class="bg-gradient-to-r from-yellow-50 to-orange-50 border-2 border-yellow-300 rounded-lg p-4 shadow-lg">
                        <div class="flex items-center justify-between mb-3">
                            <h3 class="text-lg font-bold text-yellow-900 flex items-center">
                                <svg class="w-5 h-5 mr-2 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"></path>
                                </svg>
                                New Session Invitation
                            </h3>
                            <span class="text-xs text-gray-500">
                                {new Date().toLocaleTimeString()}
                            </span>
                        </div>
                        
                        <div class="bg-white bg-opacity-70 rounded p-3 mb-3">
                            <div class="text-sm space-y-2">
                                <div class="flex justify-between">
                                    <span class="font-semibold text-gray-600">Session ID:</span>
                                    <span class="font-mono text-xs bg-gray-100 px-2 py-1 rounded">{invite.session_id}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="font-semibold text-gray-600">Proposer:</span>
                                    <span class="font-mono text-xs {invite.proposer_id === appState.deviceId ? 'bg-blue-100 text-blue-800' : 'bg-gray-100'} px-2 py-1 rounded">
                                        {invite.proposer_id}{invite.proposer_id === appState.deviceId ? ' (you)' : ''}
                                    </span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="font-semibold text-gray-600">Threshold:</span>
                                    <span class="font-bold text-orange-600">{invite.threshold} of {invite.total}</span>
                                </div>
                            </div>
                        </div>

                        <div class="mb-3">
                            <p class="text-sm font-semibold text-gray-700 mb-2">Participants ({invite.participants?.length || 0}):</p>
                            <div class="flex flex-wrap gap-1">
                                {#each invite.participants || [] as participant}
                                    <span class="text-xs px-2 py-1 rounded-full {participant === appState.deviceId ? 'bg-blue-100 text-blue-800 font-semibold' : 'bg-gray-100 text-gray-700'}">
                                        {participant}{participant === appState.deviceId ? ' (you)' : ''}
                                    </span>
                                {/each}
                            </div>
                        </div>

                        <div class="border-t border-yellow-200 pt-3">
                            <p class="text-sm text-gray-600 mb-3">
                                You have been invited to join a {invite.threshold}-of-{invite.total} threshold signature session. 
                                This will allow any {invite.threshold} participants to create valid signatures together.
                            </p>
                            
                            <div class="flex gap-2">
                                <button
                                    class="flex-1 bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 text-white font-bold py-2 px-4 rounded-lg shadow-md transform transition hover:scale-105 flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
                                    on:click={() => acceptInvite(invite.session_id)}
                                    disabled={acceptingSession}
                                >
                                    {#if acceptingSession}
                                        <svg class="animate-spin h-5 w-5 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                        </svg>
                                        Processing...
                                    {:else}
                                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                                        </svg>
                                        Accept & Join
                                    {/if}
                                </button>
                                <button
                                    class="flex-1 bg-gray-300 hover:bg-gray-400 text-gray-700 font-bold py-2 px-4 rounded-lg shadow-md transform transition hover:scale-105"
                                    on:click={() => rejectInvite(invite.session_id)}
                                >
                                    <svg class="w-5 h-5 mr-2 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                    </svg>
                                    Decline
                                </button>
                            </div>
                        </div>
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
                        <label
                            for="total-participants"
                            class="block font-bold mb-1"
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
                        <label
                            for="threshold-input"
                            class="block font-bold mb-1">Threshold:</label
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
                        appState.connecteddevices.filter(
                            (p) => p !== appState.deviceId,
                        ).length <
                            appState.totalParticipants - 1 ||
                        appState.threshold > appState.totalParticipants ||
                        appState.threshold < 1}
                >
                    Propose New Session ({appState.threshold}-of-{appState.totalParticipants})
                </button>

                {#if appState.connecteddevices.filter((p) => p !== appState.deviceId).length < appState.totalParticipants - 1}
                    <p class="text-sm text-gray-500 text-center">
                        Need at least {appState.totalParticipants - 1} other devices
                        for a {appState.totalParticipants}-participant session
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
