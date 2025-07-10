<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { AppState } from "@mpc-wallet/types/appstate";
    import { MeshStatusType } from "@mpc-wallet/types/mesh";
    import { DkgState } from "@mpc-wallet/types/dkg";

    export let appState: AppState;
    export let proposedSessionIdInput: string;
    export let totalParticipants: number;
    export let threshold: number;
    export let processingInvites: Set<string>;

    const dispatch = createEventDispatcher<{
        proposeSession: void;
        acceptInvite: { sessionId: string };
    }>();

    function handleProposeSession() {
        dispatch("proposeSession");
    }

    function handleAcceptInvite(sessionId: string) {
        dispatch("acceptInvite", { sessionId });
    }
</script>

<!-- Session Management -->
<div class="mb-6">
    <!-- Active Session Display -->
    {#if appState.sessionInfo}
        <div class="p-4 border rounded mb-4">
            <h3 class="text-lg font-semibold mb-3">Active Session</h3>
            <div class="space-y-2">
                <p>
                    <strong>Session ID:</strong>
                    {appState.sessionInfo.session_id}
                </p>
                <p>
                    <strong>Proposer:</strong>
                    {appState.sessionInfo.proposer_id}
                </p>
                <p>
                    <strong>Total Participants:</strong>
                    {appState.sessionInfo.total}
                </p>
                <p>
                    <strong>Threshold:</strong>
                    {appState.sessionInfo.threshold}
                </p>
                <p>
                    <strong>Status:</strong>
                    {appState.sessionInfo.status || "active"}
                </p>

                <!-- Participants List -->
                <div class="mt-3">
                    <p class="font-semibold">Participants:</p>
                    <div class="flex flex-wrap gap-1 mt-1">
                        {#each appState.sessionInfo.participants as participant}
                            <span
                                class="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded"
                            >
                                {participant}
                            </span>
                        {/each}
                    </div>
                </div>

                <!-- Accepted Devices -->
                <div class="mt-3">
                    <p class="font-semibold">
                        Accepted ({appState.sessionInfo.accepted_devices
                            .length}/{appState.sessionInfo.total}):
                    </p>
                    <div class="flex flex-wrap gap-1 mt-1">
                        {#each appState.sessionInfo.accepted_devices as device}
                            <span
                                class="px-2 py-1 text-xs bg-green-100 text-green-800 rounded"
                            >
                                {device}
                            </span>
                        {/each}
                    </div>
                </div>

                <!-- Mesh Status -->
                <div class="mt-3">
                    <p class="font-semibold">Mesh Status:</p>
                    {#if appState.meshStatus?.type === MeshStatusType.Ready}
                        <span
                            class="px-2 py-1 text-xs bg-green-100 text-green-800 rounded"
                        >
                            Ready - All participants connected
                        </span>
                    {:else if appState.meshStatus?.type === MeshStatusType.PartiallyReady}
                        <span
                            class="px-2 py-1 text-xs bg-yellow-100 text-yellow-800 rounded"
                        >
                            Partially Ready - Waiting for connections
                        </span>
                    {:else}
                        <span
                            class="px-2 py-1 text-xs bg-gray-100 text-gray-800 rounded"
                        >
                            Incomplete - Setting up connections
                        </span>
                    {/if}
                </div>

                <!-- DKG Status -->
                <div class="mt-3">
                    <p class="font-semibold">DKG Status:</p>
                    <span
                        class="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded"
                    >
                        {appState.dkgState}
                    </span>
                </div>
            </div>
        </div>
    {/if}

    <!-- Session Invites -->
    {#if appState.invites && appState.invites.length > 0}
        <div class="p-4 border rounded mb-4">
            <h3 class="text-lg font-semibold mb-3">
                Session Invitations ({appState.invites.length})
            </h3>
            <div class="space-y-3">
                {#each appState.invites as invite}
                    <div
                        class="p-3 bg-yellow-50 border border-yellow-200 rounded"
                    >
                        <div class="flex justify-between items-start mb-2">
                            <div>
                                <p class="font-semibold">
                                    Session ID: {invite.session_id}
                                </p>
                                <p class="text-sm text-gray-600">
                                    From: {invite.proposer_id}
                                </p>
                                <p class="text-sm">
                                    Participants: {invite.total} | Threshold: {invite.threshold}
                                </p>
                            </div>
                            <button
                                class="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600 text-sm"
                                disabled={processingInvites.has(
                                    invite.session_id,
                                )}
                                on:click={() =>
                                    handleAcceptInvite(invite.session_id)}
                            >
                                {processingInvites.has(invite.session_id)
                                    ? "Processing..."
                                    : "Accept"}
                            </button>
                        </div>

                        <!-- Participants Preview -->
                        <div class="mt-2">
                            <p class="text-xs text-gray-600">Participants:</p>
                            <div class="flex flex-wrap gap-1 mt-1">
                                {#each invite.participants as participant}
                                    <span
                                        class="px-1 py-0.5 text-xs bg-gray-100 text-gray-700 rounded"
                                    >
                                        {participant}
                                    </span>
                                {/each}
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    <!-- Create New Session -->
    {#if !appState.sessionInfo}
        <div class="p-4 border rounded">
            <h3 class="text-lg font-semibold mb-3">Create New Session</h3>
            <div class="space-y-4">
                <div>
                    <label for="session-id" class="block font-bold mb-2"
                        >Session ID:</label
                    >
                    <input
                        id="session-id"
                        type="text"
                        class="w-full border p-2 rounded"
                        placeholder="Enter session ID"
                        bind:value={proposedSessionIdInput}
                    />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label
                            for="total-participants"
                            class="block font-bold mb-2"
                            >Total Participants:</label
                        >
                        <input
                            id="total-participants"
                            type="number"
                            min="2"
                            max="10"
                            class="w-full border p-2 rounded"
                            bind:value={totalParticipants}
                        />
                    </div>

                    <div>
                        <label for="threshold" class="block font-bold mb-2"
                            >Threshold:</label
                        >
                        <input
                            id="threshold"
                            type="number"
                            min="2"
                            max={totalParticipants}
                            class="w-full border p-2 rounded"
                            bind:value={threshold}
                        />
                    </div>
                </div>

                <button
                    class="w-full bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded"
                    disabled={!proposedSessionIdInput ||
                        !appState.wsConnected ||
                        appState.connecteddevices.length === 0}
                    on:click={handleProposeSession}
                >
                    {#if !appState.wsConnected}
                        WebSocket Disconnected
                    {:else if appState.connecteddevices.length === 0}
                        No Connected Devices
                    {:else}
                        Propose Session
                    {/if}
                </button>

                {#if appState.connecteddevices.length > 0}
                    <p class="text-sm text-gray-600">
                        Available devices: {appState.connecteddevices.length}
                    </p>
                {/if}
            </div>
        </div>
    {/if}
</div>
