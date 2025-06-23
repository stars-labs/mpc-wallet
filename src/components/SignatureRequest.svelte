<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    
    export let signingId: string;
    export let message: string;
    export let origin: string;
    export let fromAddress: string;
    
    const dispatch = createEventDispatcher();
    
    // Track approval state
    let approving = false;
    let rejecting = false;
    
    // Format message for display
    function formatMessage(msg: string): string {
        // Check if it's hex data
        if (msg.startsWith('0x')) {
            // If it's a long hex string, truncate it
            if (msg.length > 66) {
                return msg.slice(0, 20) + '...' + msg.slice(-20);
            }
            return msg;
        }
        // Regular text message
        return msg;
    }
    
    // Format address for display
    function formatAddress(addr: string): string {
        if (addr.length > 10) {
            return addr.slice(0, 6) + '...' + addr.slice(-4);
        }
        return addr;
    }
    
    async function handleApprove() {
        approving = true;
        try {
            chrome.runtime.sendMessage({
                type: 'approveMessageSignature',
                requestId: signingId,
                approved: true
            }, (response) => {
                if (chrome.runtime.lastError) {
                    console.error('[SignatureRequest] Error approving signature:', chrome.runtime.lastError.message);
                    return;
                }
                console.log('[SignatureRequest] Signature approved');
                dispatch('complete');
            });
        } catch (error) {
            console.error('[SignatureRequest] Error approving signature:', error);
        } finally {
            approving = false;
        }
    }
    
    async function handleReject() {
        rejecting = true;
        try {
            chrome.runtime.sendMessage({
                type: 'approveMessageSignature',
                requestId: signingId,
                approved: false
            }, (response) => {
                if (chrome.runtime.lastError) {
                    console.error('[SignatureRequest] Error rejecting signature:', chrome.runtime.lastError.message);
                    return;
                }
                console.log('[SignatureRequest] Signature rejected');
                dispatch('complete');
            });
        } catch (error) {
            console.error('[SignatureRequest] Error rejecting signature:', error);
        } finally {
            rejecting = false;
        }
    }
</script>

<div class="bg-gradient-to-r from-yellow-50 to-orange-50 border-2 border-yellow-300 rounded-lg p-4 shadow-lg mb-4">
    <div class="flex items-center justify-between mb-3">
        <h3 class="text-lg font-bold text-yellow-800">
            🔏 Signature Request
        </h3>
        <span class="text-xs bg-yellow-200 text-yellow-800 px-2 py-1 rounded-full">
            Pending
        </span>
    </div>
    
    <div class="space-y-3 text-sm">
        <div>
            <span class="font-semibold text-gray-700">Origin:</span>
            <span class="ml-2 text-gray-600 font-mono text-xs bg-gray-100 px-2 py-1 rounded">{origin}</span>
        </div>
        
        <div>
            <span class="font-semibold text-gray-700">From Address:</span>
            <span class="ml-2 text-gray-600 font-mono text-xs bg-gray-100 px-2 py-1 rounded">{formatAddress(fromAddress)}</span>
        </div>
        
        <div>
            <span class="font-semibold text-gray-700">Message:</span>
            <div class="mt-1 p-3 bg-gray-100 rounded-lg font-mono text-xs break-all">
                {formatMessage(message)}
            </div>
        </div>
    </div>
    
    <div class="border-t border-yellow-200 pt-3 mt-4">
        <p class="text-sm text-gray-600 mb-3">
            The dapp at <strong>{origin}</strong> is requesting you to sign this message using your MPC wallet.
        </p>
        
        <div class="flex gap-2">
            <button
                class="flex-1 bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 text-white font-bold py-2 px-4 rounded-lg shadow-md transform transition hover:scale-105 flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
                on:click={handleReject}
                disabled={approving || rejecting}
            >
                {#if rejecting}
                    <svg class="animate-spin h-5 w-5 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Rejecting...
                {:else}
                    ❌ Reject
                {/if}
            </button>
            
            <button
                class="flex-1 bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 text-white font-bold py-2 px-4 rounded-lg shadow-md transform transition hover:scale-105 flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
                on:click={handleApprove}
                disabled={approving || rejecting}
            >
                {#if approving}
                    <svg class="animate-spin h-5 w-5 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Approving...
                {:else}
                    ✅ Sign
                {/if}
            </button>
        </div>
    </div>
</div>

<style>
    /* Add any component-specific styles here */
</style>