<script lang="ts">
  import svelteLogo from "../../assets/svelte.svg";
  import init, {
    generate_priv_key,
    get_eth_address,
    get_sol_address,
    eth_sign,
    sol_sign,
  } from "../../../pkg/mpc_wallet.js";
  import { onMount } from "svelte";
  import Settings from "@/components/Settings.svelte";
  import { storage } from "#imports";

  let private_key: string = "";
  let address: string = "";
  let message: string = "Hello from MPC Wallet!";
  let signature: string = "";
  let error: string = "";
  let isSettings: boolean = false;
  let chain: "ethereum" | "solana" = "ethereum";

  // Generate or load the private key for the selected chain
  async function ensurePrivateKey() {
    const curve = chain === "ethereum" ? "secp256k1" : "ed25519";
    const keyName = `local:private_key_${curve}`;
    const storedKey = await storage.getItem<string>(keyName);
    if (storedKey) {
      private_key = storedKey;
    } else {
      private_key = generate_priv_key(curve);
      await storage.setItem(keyName, private_key);
    }
    address = "";
    signature = "";
  }

  onMount(async () => {
    await init();
    await ensurePrivateKey();
    console.log("Private Key:", private_key);
  });

  $: if (chain) {
    // When chain changes, reload key/address/signature
    ensurePrivateKey();
  }

  async function fetchAddress() {
    error = "";
    signature = "";
    try {
      if (chain === "ethereum") {
        address = get_eth_address(private_key);
        if (address.startsWith("0x")) {
          address = address.slice(2);
        }
        if (address.length !== 40) {
          error = "Invalid Ethereum address length.";
        }
        if (!address) {
          error = "No address returned.";
        }
      } else if (chain === "solana") {
        address = get_sol_address(private_key);
        if (!address || address.startsWith("Error")) {
          error = "Failed to get Solana address.";
        }
      }
    } catch (e: any) {
      error = `Failed to fetch address: ${e.message || e}`;
    }
  }

  async function signDemoMessage() {
    error = "";
    signature = "";
    if (!private_key) {
      error = "Private key is not set.";
      return;
    }
    if (!address) {
      error = "Please fetch address first.";
      return;
    }
    try {
      if (chain === "ethereum") {
        // Prefer eth_sign for Ethereum
        signature = eth_sign(private_key, message);
        if (!signature) {
          error = "Signing failed. Check private key and message.";
        }
      } else if (chain === "solana") {
        signature = sol_sign(private_key, message);
        if (!signature || signature.startsWith("Error")) {
          error = "Solana signing failed.";
        }
      }
    } catch (e: any) {
      error = `Failed to sign message: ${e.message || e}`;
    }
  }
</script>

<main>
  <div>
    <a href="https://svelte.dev" target="_blank" rel="noreferrer">
      <img src={svelteLogo} class="logo svelte" alt="Svelte Logo" />
    </a>
    <button
      class="border-0 bg-transparent cursor-pointer"
      on:click={() => (isSettings = !isSettings)}
    >
      settings
    </button>
  </div>
  <h1 class="text-4xl font-bold underline">Starlab Crypto Wallet</h1>

  <div class="mt-4">
    <label for="chain-select" class="font-bold mr-2">Chain:</label>
    <select id="chain-select" bind:value={chain} class="border p-2 rounded">
      <option value="ethereum">Ethereum (secp256k1)</option>
      <option value="solana">Solana (ed25519)</option>
    </select>
  </div>

  <div class="mt-8">
    <button
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
      on:click={fetchAddress}>Show Wallet Address</button
    >
    {#if address}
      <div class="mt-2">
        <strong>Address:</strong>
        <code class="bg-gray-100 px-1">{address}</code>
      </div>
    {/if}
  </div>

  <div class="mt-4">
    <input type="text" bind:value={message} class="border p-2 rounded w-3/4" />
    <button
      class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded ml-2"
      on:click={signDemoMessage}
      disabled={!private_key}
    >
      Sign Message
    </button>
    {#if signature}
      <div class="mt-2">
        <strong>Signature:</strong>
        <code class="bg-gray-100 p-2 block break-all">{signature}</code>
      </div>
    {/if}
  </div>

  {#if error}
    <div class="text-red-600 mt-2">{error}</div>
  {/if}

  {#if isSettings}
    <Settings
      on:close={() => {
        isSettings = false;
      }}
    />
  {/if}

  <p class="text-gray-500 mt-8">
    Click on the WXT and Svelte logos to learn more
  </p>
</main>

<style>
  :global(body) {
    width: 800px;
    height: 600px;
    overflow: auto;
  }
</style>
