<script lang="ts">
  import type { AppState } from "@mpc-wallet/types/appstate";

  export let appState: AppState;
  export let dkgAddress: string;
  export let dkgError: string;
</script>

<!-- DKG Address Display -->
{#if dkgAddress}
  <div class="mb-4 p-3 border rounded">
    <h2 class="text-xl font-semibold mb-2">MPC Address</h2>
    <div>
      <span class="block font-bold mb-1"
        >{appState.chain === "ethereum" ? "Ethereum" : "Solana"} Address:</span
      >
      <code
        class="block bg-purple-50 border border-purple-200 p-2 rounded break-all"
        >{dkgAddress}</code
      >
      <p class="text-xs text-purple-600 mt-1">
        ✓ Generated using {appState.sessionInfo?.threshold}-of-{appState
          .sessionInfo?.total} threshold signature
      </p>
    </div>

    {#if dkgError}
      <div class="mt-2 p-2 bg-red-50 border border-red-200 rounded">
        <span class="text-red-600 text-sm">{dkgError}</span>
      </div>
    {/if}
  </div>
{/if}
