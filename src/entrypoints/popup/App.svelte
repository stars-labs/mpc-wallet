<script lang="ts">
  import svelteLogo from "../../assets/svelte.svg";
  import init, {
    generate_priv_key,
    get_eth_address,
    personal_sign,
  } from "../../../pkg/mpc_wallet.js"; // Import WASM init
  import { onMount } from "svelte";

  let private_key: string = "";
  let address: string = "";
  let message: string = "Hello from MPC Wallet!";
  let signature: string = "";
  let error: string = "";

  // Ensure WASM is initialized before any provider requests

  onMount(async () => {
    await init();
    private_key = generate_priv_key();
    console.log("Private Key:", private_key);
  });

  // Replace these with your actual wallet's API calls
  async function fetchAddress() {
    error = "";
    signature = "";
    try {
      // Example: Replace with your wallet's address retrieval logic
      // address = await myWalletApi.getAddress();
      address = get_eth_address(private_key);
      console.log("Address:", address);
      if (address.startsWith("0x")) {
        address = address.slice(2); // Remove '0x' prefix
      }
      if (address.length !== 40) {
        error = "Invalid address length.";
      }
      if (address === "0x") {
        error = "Address cannot be just '0x'.";
      }
      if (!address) {
        error = "No address returned.";
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
      console.log("Signing with private_key:", private_key);
      let pk = private_key.startsWith("0x")
        ? private_key.slice(2)
        : private_key;
      signature = personal_sign(pk, message);
      if (!signature) {
        error = "Signing failed. Check private key and message.";
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
  </div>
  <h1>Demo Crypto Wallet</h1>

  <!-- Provider detection removed -->

  <div style="margin-top:2em;">
    <button on:click={fetchAddress}>Show Wallet Address</button>
    {#if address}
      <div><strong>Address:</strong> <code>{address}</code></div>
    {/if}
  </div>

  <div style="margin-top:1em;">
    <input type="text" bind:value={message} style="width:70%;" />
    <button on:click={signDemoMessage} disabled={!private_key}>
      Sign Message
    </button>
    {#if signature}
      <div>
        <strong>Signature:</strong>
        <code style="word-break:break-all;">{signature}</code>
      </div>
    {/if}
  </div>

  {#if error}
    <div style="color:red;">{error}</div>
  {/if}

  <p class="read-the-docs">Click on the WXT and Svelte logos to learn more</p>
</main>

<style>
  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: filter 300ms;
  }
  .logo:hover {
    filter: drop-shadow(0 0 2em #54bc4ae0);
  }
  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00aa);
  }
  .read-the-docs {
    color: #888;
  }
</style>
