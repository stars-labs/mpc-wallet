import { describe, it, expect } from 'bun:test';
import { readFileSync } from 'fs';
import { join } from 'path';

// Test decrypting and importing a CLI keystore
describe('CLI Keystore Import', () => {
    it('should decrypt and parse CLI keystore correctly', async () => {
        // Sample encrypted keystore from the logs
        const encryptedKeystore = {
            "version": "2.0",
            "encrypted": true,
            "algorithm": "AES-256-GCM-PBKDF2",
            "data": "kaXAylLm0dBgcPLPb9OEXzUKBr16KdGWMkIkeKtpP1Nyy2+40nfeJyLPcxNzLv/gFn225XSB2R5yUQ8+h08+A2Z3UbTsr417BozMGNcyQynJKMXGLhJJowi5u8bYwgicfYEsEmDMWSwnI8IL+qWvRKvCb20fPN3lPObb/wORvsg4MMErAHOzXI3oz9EunrADZXWhjJE2upIbddl94F6oSSe3xP1uI72Vduk/wJzQourskjANok01ufH0zSkrHVUz1RZrnpNpv49pHDJ07I1KZLItagQhMAADBSdgT2Brst8AcKPSYFt15ba9U2LPXepz1iGFyZJWvBoLZbuk6YkGxLoAXoYE4uSYBTc6kmkmBUrjzJNySm5UArTTo4ZuaG1awffF4OKXM84pN2R2OLizgMu25zq8kYSvd+kBycwMIbvrSmt81HB9Rgr5+0teXkWpco0gLFRqBs6nJ6wIhiPTl1PfLemjbYnH9bmm1a/4AN/Yb0l9pslYNoPmY9saifrZZ5/PVa29nXaadqecSszE0VdUxyxzQyDR5QLjVFlRrWS29QXN0U4RIGJ5b/9/m2YrR8flYU7Rs9TzxWG+hpdBhhfr1mL+4uyyp+sD/6TE4uctBfJ3q+1lusv5UQ4ATzb0//I7VZok/o3Wm005lQ7rIICRL89CrdpuqyawnYj87gtf1IwAX0BObDweMyJeaQ2MUaaoNchXbkKepZ4/HT89QipS0+c8OcP/1nL+5cILBJDn7TqpKvV5bEiAmNfa/SJ20wRhAN3LoE7KNgxdR6ReN2XYsRXZ6zngjnFJDUlVNPOaznW33F107gcL1HwuELq9uCMpH2xXKLB/JJA2qksjGo2eFg+wbjLLKV2b7dIrck4v/d76Nuy90jTFFkkFoxiWUOV8cJ2mMd6uA4V4SYY5J2rhSZu6UQKL/JLTZRwp+Db0msVVzqaNQL0LIroSHrmrLix3L47nkdBuK3xcZEZrez4Twnys/r4DYGF0HnV1JLffShzj9yZbVwj34nHdv/UgWlIF0utOg9Db4+ENIQYO+71rqHQmM0klDh+p6xHw8tS9GGmaHDOz6tGRIn331smKTuCb07WIGJrHiX0OtjACMQEMLKD9RDooOE7hFczlMCIb4u53L7/VooL3401Ruh4+FcPn1KZZPxFaH0R9GDyACaZKXgESaOoiynJ3GmkWvV4+hoYKOFyb84BD+EkP2nQV0KwNQbGRCJq1IPAOzO97WLjf9KEWDf7pLAz8CofPMk6902QuGLrnZDSCwO2XCJd3o1eYQgPX43n5SZFuvDi0wUxonLXGHXq5TrL666K+8bTYGwYmr41UXnul/A+ESpcokVki1//CcFovXt0IZmB43+mq/ezUVtuOBh8oalou71AirYlV8E+YFgWjXRU4G9VtEp5heeFq1E3Lb8E6EkCilRLG0CIhpCnJCDkzQS+6alQNMELJSlhvkim9kWnkBLcLLxcoUlcWz4UUlbn9XtVGLaCc31/jatHF9T9yVyAE/IK9i1tWupcq4WtCOk5NjS3QNsN3LFBP8Rv7ho0uKjLjyolI1llDP2X6zmbXFw==",
            "metadata": {
                "session_id": "wallet_2of3",
                "device_id": "mpc-2",
                "curve_type": "secp256k1",
                "blockchains": [
                    {
                        "blockchain": "ethereum",
                        "network": "mainnet",
                        "chain_id": 1,
                        "address": "0x5e11e955316ef83fa38a0afa545bd17ff27e12f0",
                        "address_format": "EIP-55",
                        "enabled": true
                    }
                ],
                "threshold": 2,
                "total_participants": 3,
                "participant_index": 2,
                "group_public_key": "{\"header\":{\"version\":0,\"ciphersuite\":\"FROST-secp256k1-SHA256-v1\"},\"verifying_shares\":{\"0000000000000000000000000000000000000000000000000000000000000001\":\"03bb6389cae1f284982174c7cd153576773a7dd4d69e32554539e6e4285ce37a2d\",\"0000000000000000000000000000000000000000000000000000000000000002\":\"026ae61d48541299bac7ac8fd68fa2744975d1f7ae32c48e8f5e906e2f085725ac\",\"0000000000000000000000000000000000000000000000000000000000000003\":\"0306bfb4102d45645bc1b77f585c22ec57e39f813864f21f95e8ea6ac4ce454189\"},\"verifying_key\":\"02e07d7f83cda7dbbe14eebdd54a1f8f0f0b8e1001c79d5955990bd1074570330b\"}",
                "created_at": "2025-06-28T08:58:23.247674381+00:00",
                "last_modified": "2025-06-28T08:58:23.247680038+00:00"
            }
        };

        const password = "mpc-2";

        // Helper function to decrypt CLI keystore (same as in offscreen/index.ts)
        async function decryptCLIKeystore(base64Data: string, password: string): Promise<string> {
            // CLI uses base64 encoding for the entire encrypted blob
            const encryptedData = Uint8Array.from(atob(base64Data), c => c.charCodeAt(0));
            
            // Extract salt (first 16 bytes), IV (next 12 bytes), and ciphertext + tag
            const salt = encryptedData.slice(0, 16);
            const iv = encryptedData.slice(16, 28);
            const ciphertextWithTag = encryptedData.slice(28);
            
            // Derive key using PBKDF2 (CLI uses 100,000 iterations)
            const encoder = new TextEncoder();
            const passwordKey = await crypto.subtle.importKey(
                'raw',
                encoder.encode(password),
                'PBKDF2',
                false,
                ['deriveKey']
            );
            
            const key = await crypto.subtle.deriveKey(
                {
                    name: 'PBKDF2',
                    salt: salt,
                    iterations: 100000,
                    hash: 'SHA-256'
                },
                passwordKey,
                { name: 'AES-GCM', length: 256 },
                false,
                ['decrypt']
            );
            
            // Decrypt using AES-GCM
            const decrypted = await crypto.subtle.decrypt(
                { name: 'AES-GCM', iv: iv },
                key,
                ciphertextWithTag
            );
            
            return new TextDecoder().decode(decrypted);
        }

        try {
            // Decrypt the keystore
            const decryptedData = await decryptCLIKeystore(encryptedKeystore.data, password);
            console.log("Decrypted data:", decryptedData);
            
            // Parse the decrypted JSON
            const keyData = JSON.parse(decryptedData);
            console.log("Parsed key data structure:", Object.keys(keyData));
            console.log("Full decrypted keystore:", JSON.stringify(keyData, null, 2));
            
            // Check what fields are present
            expect(keyData).toBeDefined();
            
            // Log the structure to understand what's missing
            if (!keyData.key_package) {
                console.error("Missing key_package field in decrypted data");
                console.log("Available fields:", Object.keys(keyData));
            }
            
        } catch (error) {
            console.error("Decryption error:", error);
            throw error;
        }
    });
});