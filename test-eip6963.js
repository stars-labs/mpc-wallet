// Test script to verify EIP-6963 implementation
// Run this in the browser console after loading the extension

console.log('=== EIP-6963 Test Script ===');

// Test 1: Check if window.ethereum exists
console.log('1. window.ethereum exists:', !!window.ethereum);
if (window.ethereum) {
    console.log('   - isStarLabWallet:', window.ethereum.isStarLabWallet);
    console.log('   - chainId:', window.ethereum.chainId);
    console.log('   - networkVersion:', window.ethereum.networkVersion);
}

// Test 2: Check if window.starlabEthereum exists
console.log('2. window.starlabEthereum exists:', !!window.starlabEthereum);

// Test 3: Request EIP-6963 providers
console.log('3. Requesting EIP-6963 providers...');
const providers = new Map();

window.addEventListener("eip6963:announceProvider", (event) => {
    const detail = event.detail;
    console.log(`   Provider announced: ${detail.info.name}`);
    console.log(`   - UUID: ${detail.info.uuid}`);
    console.log(`   - RDNS: ${detail.info.rdns}`);
    console.log(`   - Icon: ${detail.info.icon ? 'Present' : 'Missing'}`);
    providers.set(detail.info.uuid, detail);
});

window.dispatchEvent(new Event("eip6963:requestProvider"));

// Test 4: Test basic RPC methods after a delay
setTimeout(async () => {
    console.log('\n4. Testing RPC methods...');
    
    if (!window.ethereum) {
        console.error('   No ethereum provider found!');
        return;
    }
    
    try {
        // Test eth_chainId
        const chainId = await window.ethereum.request({ method: 'eth_chainId' });
        console.log('   eth_chainId:', chainId);
        
        // Test eth_accounts (should return empty if not connected)
        const accounts = await window.ethereum.request({ method: 'eth_accounts' });
        console.log('   eth_accounts:', accounts);
        
        // Test net_version
        const netVersion = await window.ethereum.request({ method: 'net_version' });
        console.log('   net_version:', netVersion);
        
        console.log('\n✅ Basic tests passed!');
    } catch (error) {
        console.error('   Error during RPC tests:', error);
    }
    
    console.log('\n5. Summary:');
    console.log(`   - Providers discovered: ${providers.size}`);
    console.log(`   - MPC Wallet found: ${Array.from(providers.values()).some(p => p.info.name === 'MPC Wallet')}`);
    
}, 1000);

console.log('\nTo test connection, run: window.ethereum.request({ method: "eth_requestAccounts" })');