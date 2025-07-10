<script lang="ts">
  import type { AppState } from "@mpc-wallet/types/appstate";

  export let appState: AppState;
</script>

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
