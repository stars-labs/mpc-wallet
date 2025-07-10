<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { AppState } from "@mpc-wallet/types/appstate";

    export let appState: AppState;

    const dispatch = createEventDispatcher<{
        sendDirectMessage: { deviceId: string };
    }>();

    function handleSendDirectMessage(deviceId: string) {
        dispatch("sendDirectMessage", { deviceId });
    }
</script>

<!-- Connected Devices -->
{#if appState.connecteddevices && appState.connecteddevices.length > 0}
    <div class="mb-6 p-4 border rounded">
        <h3 class="text-lg font-semibold mb-3">
            Connected Devices ({appState.connecteddevices.length})
        </h3>
        <div class="space-y-2">
            {#each appState.connecteddevices as deviceId}
                <div
                    class="flex justify-between items-center p-2 bg-gray-50 rounded"
                >
                    <span class="font-mono text-sm">{deviceId}</span>
                    <div class="flex gap-2">
                        <!-- WebRTC Connection Status -->
                        {#if appState.webrtcConnections[deviceId]}
                            <span
                                class="px-2 py-1 text-xs bg-green-100 text-green-800 rounded"
                            >
                                WebRTC Connected
                            </span>
                        {:else}
                            <span
                                class="px-2 py-1 text-xs bg-gray-100 text-gray-800 rounded"
                            >
                                WebRTC Disconnected
                            </span>
                        {/if}
                        <button
                            class="px-3 py-1 text-xs bg-blue-500 text-white rounded hover:bg-blue-600"
                            on:click={() => handleSendDirectMessage(deviceId)}
                        >
                            Send Message
                        </button>
                    </div>
                </div>
            {/each}
        </div>
    </div>
{:else}
    <div class="mb-6 p-4 border rounded">
        <h3 class="text-lg font-semibold mb-3">Connected Devices (0)</h3>
        <p class="text-gray-500 text-sm">No other devices connected</p>
    </div>
{/if}
