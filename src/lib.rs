use getrandom::getrandom;
use hex;
use k256::ecdsa::{SigningKey as Secp256k1SigningKey, signature::Signer};
use sha3::{Digest, Keccak256};
use wasm_bindgen::prelude::*; // use getrandom instead of rand

// Add new imports for Ed25519 and Solana address generation
use ed25519_dalek::{SECRET_KEY_LENGTH, SigningKey};
use rand::rngs::OsRng; // For Ed25519 key generation
// bs58 will be used in get_sol_address

// Import the `window.alert` function from the Web.
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// Export a `greet` function from Rust to JavaScript, that alerts a
// hello message.
#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn generate_priv_key(curve: &str) -> String {
    if curve == "secp256k1" {
        loop {
            let mut priv_bytes = [0u8; 32];
            getrandom(&mut priv_bytes).expect("random generation failed for secp256k1");
            if Secp256k1SigningKey::from_bytes((&priv_bytes).into()).is_ok() {
                return format!("0x{}", hex::encode(priv_bytes));
            }
        }
    } else if curve == "ed25519" {
        let mut csprng = OsRng {};
        let mut priv_bytes = [0u8; SECRET_KEY_LENGTH];
        use rand::RngCore;
        csprng.fill_bytes(&mut priv_bytes);
        let signing_key = SigningKey::from_bytes(&priv_bytes);
        return format!("0x{}", hex::encode(signing_key.to_bytes()));
    } else {
        alert(&format!("Unsupported curve type: {}", curve));
        return String::from("Error: Unsupported curve type");
    }
}

#[wasm_bindgen]
pub fn get_eth_address(priv_hex: &str) -> String {
    // Remove optional "0x" prefix if present
    let priv_hex = priv_hex.strip_prefix("0x").unwrap_or(priv_hex);
    let priv_bytes = match hex::decode(priv_hex) {
        Ok(bytes) => bytes,
        Err(_) => return String::from(""),
    };
    if priv_bytes.len() != 32 {
        return String::from("");
    }
    let priv_bytes: [u8; 32] = match priv_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => return String::from(""),
    };
    let signing_key = match Secp256k1SigningKey::from_bytes((&priv_bytes).into()) {
        Ok(sk) => sk,
        Err(_) => return String::from(""),
    };
    let verify_key = signing_key.verifying_key();
    let pubkey = verify_key.to_encoded_point(false);
    let pubkey_bytes = pubkey.as_bytes();
    // Ethereum address: keccak256(pubkey[1..])[12..]
    let hash = Keccak256::digest(&pubkey_bytes[1..]);
    let address = &hash[12..];
    format!("0x{}", hex::encode(address))
}

#[wasm_bindgen]
pub fn eth_sign(priv_hex: &str, message: &str) -> String {
    // Remove optional "0x" prefix if present
    let priv_hex_no_prefix = priv_hex.strip_prefix("0x").unwrap_or(priv_hex);
    let priv_bytes = match hex::decode(priv_hex_no_prefix) {
        Ok(bytes) => bytes,
        Err(_) => {
            alert("Failed to decode private key hex");
            return String::from("");
        }
    };
    let priv_bytes: [u8; 32] = match priv_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            alert("Private key is not 32 bytes");
            return String::from("");
        }
    };
    let signing_key = match Secp256k1SigningKey::from_bytes((&priv_bytes).into()) {
        Ok(sk) => sk,
        Err(_) => {
            alert("Failed to create signing key");
            return String::from("");
        }
    };
    // Ethereum signed message prefix
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut eth_message = prefix.into_bytes();
    eth_message.extend_from_slice(message.as_bytes());
    let hash = Keccak256::digest(&eth_message);
    let signature: k256::ecdsa::Signature = signing_key.sign(&hash);
    format!("0x{}", hex::encode(signature.to_bytes()))
}

#[wasm_bindgen]
pub fn get_sol_address(priv_hex: &str) -> String {
    // Remove optional "0x" prefix if present
    let priv_hex_no_prefix = priv_hex.strip_prefix("0x").unwrap_or(priv_hex);
    let priv_bytes = match hex::decode(priv_hex_no_prefix) {
        Ok(bytes) => bytes,
        Err(_) => {
            // alert("Failed to decode private key hex for Solana"); // Optional alert
            return String::from("Error: Failed to decode private key hex for Solana");
        }
    };

    if priv_bytes.len() != SECRET_KEY_LENGTH {
        // alert("Invalid private key length for Ed25519"); // Optional alert
        return String::from("Error: Invalid private key length for Ed25519");
    }

    // Convert priv_bytes to [u8; 32] for SecretKey::from_bytes
    let priv_bytes_array: [u8; SECRET_KEY_LENGTH] = match priv_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return String::from("Error: Failed to convert private key bytes to array");
        }
    };

    let signing_key = SigningKey::from_bytes(&priv_bytes_array);
    let verifying_key = signing_key.verifying_key();
    bs58::encode(verifying_key.as_bytes()).into_string()
}

#[wasm_bindgen]
pub fn sol_sign(priv_hex: &str, message: &str) -> String {
    // Remove optional "0x" prefix if present
    let priv_hex_no_prefix = priv_hex.strip_prefix("0x").unwrap_or(priv_hex);
    let priv_bytes = match hex::decode(priv_hex_no_prefix) {
        Ok(bytes) => bytes,
        Err(_) => {
            // alert("Failed to decode private key hex for Solana"); // Optional alert
            return String::from("Error: Failed to decode private key hex for Solana");
        }
    };

    if priv_bytes.len() != SECRET_KEY_LENGTH {
        // alert("Invalid private key length for Ed25519"); // Optional alert
        return String::from("Error: Invalid private key length for Ed25519");
    }

    let priv_bytes_array: [u8; SECRET_KEY_LENGTH] = match priv_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return String::from("Error: Failed to convert private key bytes to array");
        }
    };

    let signing_key = SigningKey::from_bytes(&priv_bytes_array);
    let signature = signing_key.sign(message.as_bytes());
    format!("0x{}", hex::encode(signature.to_bytes()))
}
