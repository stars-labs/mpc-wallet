<script lang="ts">
    import { onMount } from "svelte";
    import NetworkService from "../services/networkService";
    import { mainnet } from "viem/chains";
    import type { Chain } from "../types/network";
    import type { SupportedChain } from "../types/appstate";
    import {
        CURVE_COMPATIBLE_CHAINS,
        getCompatibleChains,
        getRequiredCurve,
    } from "../types/appstate";
    import { createPublicClient, http } from "viem";
    import { createEventDispatcher } from "svelte";
    const dispatch = createEventDispatcher<{
        backToWallet: { chain: string; curve: string };
    }>();

    // Theme settings
    let isDarkMode = false;

    onMount(() => {
        // Check for saved theme preference or system preference
        const savedTheme = localStorage.getItem("theme");
        const prefersDark = window.matchMedia(
            "(prefers-color-scheme: dark)",
        ).matches;

        isDarkMode = savedTheme === "dark" || (!savedTheme && prefersDark);
        updateTheme();
    });

    function toggleDarkMode() {
        isDarkMode = !isDarkMode;
        updateTheme();
    }

    function updateTheme() {
        const htmlElement = document.documentElement;
        if (isDarkMode) {
            htmlElement.classList.add("dark");
            localStorage.setItem("theme", "dark");
        } else {
            htmlElement.classList.remove("dark");
            localStorage.setItem("theme", "light");
        }
    }

    // Wallet configuration
    let curve: "secp256k1" | "ed25519" = "secp256k1";
    let chain: SupportedChain = "ethereum";
    let networks: Chain[] = [];
    let currentNetwork: Chain | undefined;
    let networkService: NetworkService;

    // Custom network form
    let showCustomNetworkForm = false;
    let customNetworkName = "";
    let customNetworkRpcUrl = "";
    let customNetworkChainId = "";
    let customNetworkSymbol = "";
    let customNetworkExplorer = "";
    let customNetworkError = "";

    // Initialize network service and load data
    onMount(async () => {
        networkService = NetworkService.getInstance();

        try {
            // Sync chain with current settings
            const message = await chrome.runtime.sendMessage({
                type: "getState",
            });
            if (message && message.blockchain) {
                chain = message.blockchain;
                // Get curve from state if available, otherwise use sensible default
                if (message.curve) {
                    curve = message.curve;
                } else {
                    // Legacy support: default curve for existing chain selection
                    curve = chain === "ethereum" ? "secp256k1" : "ed25519";
                }
            }

            // Get networks for current chain - use new method that supports Layer 2 chains
            const chainNetworks = networkService.getNetworksForChain(chain);
            if (Array.isArray(chainNetworks)) {
                networks = chainNetworks;
                currentNetwork =
                    networkService.getCurrentNetworkForChain(chain);
            } else {
                console.error(
                    "[Settings] Networks is not an array:",
                    chainNetworks,
                );
                networks = [];
            }
        } catch (error) {
            console.error("[Settings] Error initializing:", error);
        }
    });

    // Handle curve change - validate compatibility and set sensible defaults
    function handleCurveChange() {
        console.log("[Settings] Curve changed to:", curve);

        // Check if current chain is compatible with new curve
        const compatibleChains = getCompatibleChains(curve);
        if (!compatibleChains.includes(chain)) {
            // Current chain is not compatible, switch to the first compatible chain
            chain = compatibleChains[0];
            console.log("[Settings] Switched to compatible chain:", chain);
        }

        updateBlockchainSelection();
    }

    // Handle chain change - validate compatibility and set sensible defaults
    function handleChainChange() {
        console.log("[Settings] Chain changed to:", chain);

        // Ensure curve is compatible with the selected chain
        const requiredCurve = getRequiredCurve(chain);
        if (curve !== requiredCurve) {
            curve = requiredCurve;
            console.log("[Settings] Updated curve to compatible:", curve);
        }

        updateBlockchainSelection();

        try {
            // Update networks list when chain changes - use new method that supports Layer 2 chains
            const chainNetworks = networkService.getNetworksForChain(chain);
            if (Array.isArray(chainNetworks)) {
                networks = chainNetworks;
                currentNetwork =
                    networkService.getCurrentNetworkForChain(chain);
            } else {
                console.error(
                    "[Settings] Networks is not an array:",
                    chainNetworks,
                );
                networks = [];
            }
        } catch (error) {
            console.error("[Settings] Error updating networks:", error);
            networks = [];
        }
    }

    // Update blockchain selection in background
    // Event dispatcher for parent components

    function updateBlockchainSelection() {
        chrome.runtime.sendMessage(
            {
                type: "setBlockchain",
                blockchain: chain,
            },
            (response) => {
                if (chrome.runtime.lastError) {
                    console.error(
                        "[Settings] Error setting blockchain:",
                        chrome.runtime.lastError.message,
                    );
                } else {
                    console.log(
                        "[Settings] Blockchain selection saved:",
                        chain,
                    );
                    // Only update locally without closing settings
                    // We don't want to automatically return to wallet page anymore
                    console.log(
                        "[Settings] Blockchain selection saved locally:",
                        chain,
                    );
                }
            },
        );
    }

    // Handle network change
    async function handleNetworkChange(event: Event) {
        const select = event.target as HTMLSelectElement;
        const networkId = parseInt(select.value, 10);
        try {
            await networkService.setCurrentNetworkForChain(chain, networkId);
            currentNetwork = networkService.getCurrentNetworkForChain(chain);
        } catch (error) {
            console.error("[Settings] Failed to change network:", error);
        }
    }

    // Toggle custom network form
    function toggleCustomNetworkForm() {
        showCustomNetworkForm = !showCustomNetworkForm;
        resetCustomNetworkForm();
    }

    // Reset custom network form
    function resetCustomNetworkForm() {
        customNetworkName = "";
        customNetworkRpcUrl = "";
        customNetworkChainId = "";
        customNetworkSymbol = "";
        customNetworkExplorer = "";
        customNetworkError = "";
    }

    // Add custom network
    async function addCustomNetwork() {
        if (
            !customNetworkName ||
            !customNetworkRpcUrl ||
            !customNetworkChainId ||
            !customNetworkSymbol
        ) {
            customNetworkError = "Please fill out all required fields";
            return;
        }

        const chainId = parseInt(customNetworkChainId, 10);
        if (isNaN(chainId)) {
            customNetworkError = "Chain ID must be a valid number";
            return;
        }

        try {
            // Create a custom chain configuration
            const customChain: Chain = {
                id: chainId,
                name: customNetworkName,
                network: customNetworkName.toLowerCase().replace(/\s+/g, "-"),
                nativeCurrency: {
                    name: customNetworkName,
                    symbol: customNetworkSymbol,
                    decimals: 18,
                },
                rpcUrls: {
                    default: {
                        http: [customNetworkRpcUrl],
                    },
                    public: {
                        http: [customNetworkRpcUrl],
                    },
                },
                blockExplorers: customNetworkExplorer
                    ? {
                          default: {
                              name: "Explorer",
                              url: customNetworkExplorer,
                          },
                      }
                    : undefined,
            };

            // Add the custom network - use new method that supports Layer 2 chains
            await networkService.addNetworkForChain(chain, customChain);

            // Refresh the networks list - use new method that supports Layer 2 chains
            networks = networkService.getNetworksForChain(chain);

            // Switch to the new network - use new method that supports Layer 2 chains
            await networkService.setCurrentNetworkForChain(chain, chainId);
            currentNetwork = networkService.getCurrentNetworkForChain(chain);

            // Hide the form
            showCustomNetworkForm = false;
            resetCustomNetworkForm();
        } catch (error) {
            console.error("[Settings] Failed to add custom network:", error);
            customNetworkError = `Failed to add network: ${error instanceof Error ? error.message : String(error)}`;
        }
    }
</script>

<div class="p-4">
    <div class="flex justify-between items-center mb-4">
        <h2 class="text-2xl font-bold">Wallet Settings</h2>
        <button
            class="text-sm px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
            on:click={() => dispatch("backToWallet", { chain, curve })}
            >Back to Wallet</button
        >
    </div>

    <!-- Theme Settings -->
    <div class="mb-6 p-4 border rounded">
        <h3 class="text-lg font-semibold mb-3">Theme</h3>
        <div class="flex justify-between items-center">
            <span>Dark Mode</span>
            <button
                class={`relative inline-flex items-center h-6 rounded-full w-11 transition-colors focus:outline-none ${isDarkMode ? "bg-blue-600" : "bg-gray-300"}`}
                on:click={toggleDarkMode}
                aria-label="Toggle dark mode"
            >
                <span
                    class={`inline-block w-4 h-4 transform bg-white rounded-full transition-transform ${isDarkMode ? "translate-x-6" : "translate-x-1"}`}
                ></span>
            </button>
        </div>
    </div>

    <!-- Blockchain Configuration -->
    <div class="mb-6 p-4 border rounded">
        <h3 class="text-lg font-semibold mb-3">Blockchain Configuration</h3>

        <!-- Curve Selection -->
        <div class="mb-4">
            <label for="curve-select" class="block font-bold mb-2"
                >Curve Type:</label
            >
            <select
                id="curve-select"
                bind:value={curve}
                on:change={handleCurveChange}
                class="w-full border p-2 rounded"
            >
                <option value="secp256k1">secp256k1</option>
                <option value="ed25519">ed25519</option>
            </select>
        </div>

        <!-- Chain Selection -->
        <div class="mb-4">
            <label for="chain-select" class="block font-bold mb-2"
                >Blockchain:</label
            >
            <select
                id="chain-select"
                bind:value={chain}
                on:change={handleChainChange}
                class="w-full border p-2 rounded"
            >
                <!-- secp256k1-based chains -->
                <optgroup label="secp256k1">
                    <option value="ethereum">Ethereum</option>
                    <option value="polygon">Polygon</option>
                    <option value="arbitrum">Arbitrum</option>
                    <option value="optimism">Optimism</option>
                    <option value="base">Base</option>
                </optgroup>
                <!-- ed25519-based chains -->
                <optgroup label="ed25519">
                    <option value="solana">Solana</option>
                    <option value="sui">Sui</option>
                </optgroup>
            </select>
        </div>

        <!-- Network Selection (for EVM chains) -->
        {#if ["ethereum", "polygon", "arbitrum", "optimism", "base"].includes(chain) && networks.length > 0}
            <div class="mb-4">
                <label for="network-select" class="block font-bold mb-2"
                    >Network:</label
                >
                <select
                    id="network-select"
                    on:change={handleNetworkChange}
                    class="w-full border p-2 rounded"
                    value={currentNetwork?.id}
                >
                    {#each networks as network}
                        <option value={network.id}>{network.name}</option>
                    {/each}
                </select>

                <button
                    class="mt-2 bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded text-sm w-full"
                    on:click={toggleCustomNetworkForm}
                >
                    {showCustomNetworkForm
                        ? "Hide Custom Network Form"
                        : "Add Custom Network"}
                </button>
            </div>

            <!-- Network Details -->
            {#if currentNetwork}
                <div class="mt-2 p-3 bg-gray-50 rounded text-sm">
                    <p><strong>Chain ID:</strong> {currentNetwork.id}</p>
                    <p><strong>Network Name:</strong> {currentNetwork.name}</p>
                    {#if currentNetwork.rpcUrls?.default?.http}
                        <p>
                            <strong>RPC URL:</strong>
                            {currentNetwork.rpcUrls.default.http[0]}
                        </p>
                    {/if}
                </div>
            {/if}
        {/if}
    </div>

    <!-- Add Custom Network Form -->
    {#if showCustomNetworkForm}
        <div class="mb-6 p-4 border rounded">
            <h3 class="text-lg font-semibold mb-3">Add Custom Network</h3>

            {#if customNetworkError}
                <div
                    class="mb-4 p-2 bg-red-50 border border-red-200 rounded text-red-700 text-sm"
                >
                    {customNetworkError}
                </div>
            {/if}

            <div class="mb-4">
                <label for="network-name" class="block font-bold mb-2"
                    >Network Name:</label
                >
                <input
                    id="network-name"
                    type="text"
                    class="w-full border p-2 rounded"
                    placeholder="My Custom Network"
                    bind:value={customNetworkName}
                />
            </div>

            <div class="mb-4">
                <label for="network-rpc" class="block font-bold mb-2"
                    >RPC URL:</label
                >
                <input
                    id="network-rpc"
                    type="text"
                    class="w-full border p-2 rounded"
                    placeholder="https://my-custom-network.example.com"
                    bind:value={customNetworkRpcUrl}
                />
            </div>

            <div class="mb-4">
                <label for="network-chainid" class="block font-bold mb-2"
                    >Chain ID:</label
                >
                <input
                    id="network-chainid"
                    type="text"
                    class="w-full border p-2 rounded"
                    placeholder="1"
                    bind:value={customNetworkChainId}
                />
            </div>

            <div class="mb-4">
                <label for="network-symbol" class="block font-bold mb-2"
                    >Currency Symbol:</label
                >
                <input
                    id="network-symbol"
                    type="text"
                    class="w-full border p-2 rounded"
                    placeholder="ETH"
                    bind:value={customNetworkSymbol}
                />
            </div>

            <div class="mb-4">
                <label for="network-explorer" class="block font-bold mb-2"
                    >Block Explorer URL: (optional)</label
                >
                <input
                    id="network-explorer"
                    type="text"
                    class="w-full border p-2 rounded"
                    placeholder="https://etherscan.io"
                    bind:value={customNetworkExplorer}
                />
            </div>

            <div class="flex justify-between gap-2">
                <button
                    class="w-1/2 bg-gray-400 hover:bg-gray-500 text-white font-bold py-2 px-4 rounded"
                    on:click={toggleCustomNetworkForm}
                >
                    Cancel
                </button>

                <button
                    class="w-1/2 bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded"
                    on:click={addCustomNetwork}
                >
                    Save Network
                </button>
            </div>
        </div>
    {/if}
</div>
