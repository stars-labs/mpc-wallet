<script lang="ts">
  import { DkgState } from "../types/dkg";
  import type { AppState } from "../types/appstate";

  export let appState: AppState;
</script>

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
