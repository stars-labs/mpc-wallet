<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let error: string;

  const dispatch = createEventDispatcher<{
    dismiss: void;
  }>();

  function handleDismiss() {
    dispatch("dismiss");
  }
</script>

<!-- Error Display -->
{#if error}
  <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded">
    <div class="flex justify-between items-center">
      <span class="text-red-600">{error}</span>
      <button
        class="text-sm bg-red-100 hover:bg-red-200 px-2 py-1 rounded"
        on:click={handleDismiss}
        aria-label="Dismiss error"
      >
        ×
      </button>
    </div>
  </div>
{/if}
