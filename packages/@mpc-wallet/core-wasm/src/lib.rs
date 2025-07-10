use wasm_bindgen::prelude::*;
use mpc_wallet_frost_core::{
    FrostCurve, FrostError,
    ed25519::Ed25519Curve,
    secp256k1::Secp256k1Curve,
    keystore::{Keystore, KeystoreData},
};
use rand::rngs::OsRng;
use std::collections::BTreeMap;

// Re-export specific FROST types needed by WASM
use frost_ed25519::{
    Identifier as Ed25519Identifier,
    keys::{KeyPackage as Ed25519KeyPackage, PublicKeyPackage as Ed25519PublicKeyPackage},
    round1::{SigningCommitments as Ed25519SigningCommitments, SigningNonces as Ed25519SigningNonces},
    round2::SignatureShare as Ed25519SignatureShare,
};

use frost_secp256k1::{
    Identifier as Secp256k1Identifier,
    keys::{KeyPackage as Secp256k1KeyPackage, PublicKeyPackage as Secp256k1PublicKeyPackage},
    round1::{SigningCommitments as Secp256k1SigningCommitments, SigningNonces as Secp256k1SigningNonces},
    round2::SignatureShare as Secp256k1SignatureShare,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Error type for WASM
#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmError {
    message: String,
}

#[wasm_bindgen]
impl WasmError {
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Self {
        WasmError {
            message: message.to_string(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl From<FrostError> for WasmError {
    fn from(error: FrostError) -> Self {
        WasmError {
            message: error.to_string(),
        }
    }
}

// Ed25519 WASM wrapper
#[wasm_bindgen]
pub struct FrostDkgEd25519 {
    round1_secret: Option<frost_ed25519::keys::dkg::round1::SecretPackage>,
    round2_secret: Option<frost_ed25519::keys::dkg::round2::SecretPackage>,
    key_package: Option<Ed25519KeyPackage>,
    public_key_package: Option<Ed25519PublicKeyPackage>,
    round1_packages: BTreeMap<Ed25519Identifier, frost_ed25519::keys::dkg::round1::Package>,
    round2_packages: BTreeMap<Ed25519Identifier, frost_ed25519::keys::dkg::round2::Package>,
    signing_nonces: Option<Ed25519SigningNonces>,
    signing_commitments: BTreeMap<Ed25519Identifier, Ed25519SigningCommitments>,
    signature_shares: BTreeMap<Ed25519Identifier, Ed25519SignatureShare>,
    participant_indices: Vec<u16>,
    threshold: u16,
    total: u16,
    participant_index: u16,
}

#[wasm_bindgen]
impl FrostDkgEd25519 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            round1_secret: None,
            round2_secret: None,
            key_package: None,
            public_key_package: None,
            round1_packages: BTreeMap::new(),
            round2_packages: BTreeMap::new(),
            signing_nonces: None,
            signing_commitments: BTreeMap::new(),
            signature_shares: BTreeMap::new(),
            participant_indices: Vec::new(),
            threshold: 0,
            total: 0,
            participant_index: 0,
        }
    }

    pub fn init_dkg(&mut self, participant_index: u16, total: u16, threshold: u16) -> Result<(), WasmError> {
        self.participant_index = participant_index;
        self.total = total;
        self.threshold = threshold;
        self.participant_indices = (1..=total).collect();
        Ok(())
    }

    pub fn generate_round1(&mut self) -> Result<String, WasmError> {
        let identifier = Ed25519Curve::identifier_from_u16(self.participant_index)?;
        let mut rng = OsRng;
        
        let (round1_secret, round1_package) = Ed25519Curve::dkg_part1(
            identifier,
            self.total,
            self.threshold,
            &mut rng,
        )?;
        
        self.round1_secret = Some(round1_secret);
        let package_json = serde_json::to_string(&round1_package)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        Ok(hex::encode(package_json))
    }

    pub fn add_round1_package(&mut self, participant_index: u16, package_hex: &str) -> Result<(), WasmError> {
        let package_json = hex::decode(package_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        let package: frost_ed25519::keys::dkg::round1::Package = serde_json::from_slice(&package_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let identifier = Ed25519Curve::identifier_from_u16(participant_index)?;
        self.round1_packages.insert(identifier, package);
        Ok(())
    }

    pub fn can_start_round2(&self) -> bool {
        self.round1_packages.len() == self.total as usize && self.round1_secret.is_some()
    }

    pub fn generate_round2(&mut self) -> Result<String, WasmError> {
        let round1_secret = self.round1_secret.clone()
            .ok_or_else(|| WasmError::new("Round 1 secret not available"))?;
        
        let (round2_secret, round2_packages) = Ed25519Curve::dkg_part2(
            round1_secret,
            &self.round1_packages,
        )?;
        
        self.round2_secret = Some(round2_secret);
        
        let mut packages_map = BTreeMap::new();
        for (id, package) in round2_packages {
            let id_value = id.serialize()[31] as u16 | ((id.serialize()[30] as u16) << 8);
            packages_map.insert(id_value, hex::encode(serde_json::to_string(&package).unwrap()));
        }
        
        Ok(serde_json::to_string(&packages_map).unwrap())
    }

    pub fn add_round2_package(&mut self, sender_index: u16, package_hex: &str) -> Result<(), WasmError> {
        let package_json = hex::decode(package_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        let package: frost_ed25519::keys::dkg::round2::Package = serde_json::from_slice(&package_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let identifier = Ed25519Curve::identifier_from_u16(sender_index)?;
        self.round2_packages.insert(identifier, package);
        Ok(())
    }

    pub fn can_finalize(&self) -> bool {
        self.round2_packages.len() >= (self.threshold - 1) as usize && self.round2_secret.is_some()
    }

    pub fn finalize_dkg(&mut self) -> Result<String, WasmError> {
        let round2_secret = self.round2_secret.as_ref()
            .ok_or_else(|| WasmError::new("Round 2 secret not available"))?;
        
        let (key_package, public_key_package) = Ed25519Curve::dkg_part3(
            round2_secret,
            &self.round1_packages,
            &self.round2_packages,
        )?;
        
        self.key_package = Some(key_package.clone());
        self.public_key_package = Some(public_key_package.clone());
        
        let keystore_data = Keystore::export_keystore::<Ed25519Curve>(
            &key_package,
            &public_key_package,
            self.threshold,
            self.total,
            self.participant_index,
            self.participant_indices.clone(),
            "ed25519",
        )?;
        
        Ok(serde_json::to_string(&keystore_data).unwrap())
    }

    pub fn get_group_public_key(&self) -> Result<String, WasmError> {
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("DKG not complete"))?;
        
        let verifying_key = Ed25519Curve::verifying_key(public_key_package);
        let key_bytes = Ed25519Curve::serialize_verifying_key(&verifying_key)?;
        Ok(hex::encode(key_bytes))
    }

    pub fn get_address(&self) -> Result<String, WasmError> {
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("DKG not complete"))?;
        
        let verifying_key = Ed25519Curve::verifying_key(public_key_package);
        Ok(Ed25519Curve::get_address(&verifying_key))
    }

    pub fn is_dkg_complete(&self) -> bool {
        self.key_package.is_some() && self.public_key_package.is_some()
    }

    pub fn signing_commit(&mut self) -> Result<String, WasmError> {
        let key_package = self.key_package.as_ref()
            .ok_or_else(|| WasmError::new("Key package not available"))?;
        
        let (nonces, commitments) = Ed25519Curve::generate_signing_commitment(key_package)?;
        self.signing_nonces = Some(nonces);
        
        let commitment_hex = hex::encode(serde_json::to_string(&commitments).unwrap());
        Ok(commitment_hex)
    }

    pub fn add_signing_commitment(&mut self, participant_index: u16, commitment_hex: &str) -> Result<(), WasmError> {
        let commitment_json = hex::decode(commitment_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        let commitment: Ed25519SigningCommitments = serde_json::from_slice(&commitment_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let identifier = Ed25519Curve::identifier_from_u16(participant_index)?;
        self.signing_commitments.insert(identifier, commitment);
        Ok(())
    }

    pub fn sign(&mut self, message_hex: &str) -> Result<String, WasmError> {
        let message = hex::decode(message_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let signing_package = Ed25519Curve::create_signing_package(&self.signing_commitments, &message)?;
        
        let nonces = self.signing_nonces.as_ref()
            .ok_or_else(|| WasmError::new("Signing nonces not available"))?;
        let key_package = self.key_package.as_ref()
            .ok_or_else(|| WasmError::new("Key package not available"))?;
        
        let signature_share = Ed25519Curve::generate_signature_share(&signing_package, nonces, key_package)?;
        
        Ok(hex::encode(serde_json::to_string(&signature_share).unwrap()))
    }

    pub fn add_signature_share(&mut self, participant_index: u16, share_hex: &str) -> Result<(), WasmError> {
        let share_json = hex::decode(share_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        let share: Ed25519SignatureShare = serde_json::from_slice(&share_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let identifier = Ed25519Curve::identifier_from_u16(participant_index)?;
        self.signature_shares.insert(identifier, share);
        Ok(())
    }

    pub fn aggregate_signature(&self, message_hex: &str) -> Result<String, WasmError> {
        let message = hex::decode(message_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let signing_package = Ed25519Curve::create_signing_package(&self.signing_commitments, &message)?;
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("Public key package not available"))?;
        
        let signature = Ed25519Curve::aggregate_signature(&signing_package, &self.signature_shares, public_key_package)?;
        let sig_bytes = Ed25519Curve::serialize_signature(&signature)?;
        
        Ok(hex::encode(sig_bytes))
    }

    pub fn clear_signing_state(&mut self) {
        self.signing_nonces = None;
        self.signing_commitments.clear();
        self.signature_shares.clear();
    }

    pub fn has_signing_nonces(&self) -> bool {
        self.signing_nonces.is_some()
    }

    pub fn import_keystore(&mut self, keystore_json: &str) -> Result<(), WasmError> {
        let keystore_data: KeystoreData = serde_json::from_str(keystore_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let (key_package, public_key_package) = Keystore::import_keystore::<Ed25519Curve>(&keystore_data)?;
        
        self.key_package = Some(key_package);
        self.public_key_package = Some(public_key_package);
        self.threshold = keystore_data.min_signers;
        self.total = keystore_data.max_signers;
        self.participant_index = keystore_data.participant_index;
        self.participant_indices = keystore_data.participant_indices;
        
        Ok(())
    }

    pub fn export_keystore(&self) -> Result<String, WasmError> {
        let key_package = self.key_package.as_ref()
            .ok_or_else(|| WasmError::new("Key package not available"))?;
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("Public key package not available"))?;
        
        let keystore_data = Keystore::export_keystore::<Ed25519Curve>(
            key_package,
            public_key_package,
            self.threshold,
            self.total,
            self.participant_index,
            self.participant_indices.clone(),
            "ed25519",
        )?;
        
        Ok(serde_json::to_string(&keystore_data).unwrap())
    }
}

// Secp256k1 WASM wrapper
#[wasm_bindgen]
pub struct FrostDkgSecp256k1 {
    round1_secret: Option<frost_secp256k1::keys::dkg::round1::SecretPackage>,
    round2_secret: Option<frost_secp256k1::keys::dkg::round2::SecretPackage>,
    key_package: Option<Secp256k1KeyPackage>,
    public_key_package: Option<Secp256k1PublicKeyPackage>,
    round1_packages: BTreeMap<Secp256k1Identifier, frost_secp256k1::keys::dkg::round1::Package>,
    round2_packages: BTreeMap<Secp256k1Identifier, frost_secp256k1::keys::dkg::round2::Package>,
    signing_nonces: Option<Secp256k1SigningNonces>,
    signing_commitments: BTreeMap<Secp256k1Identifier, Secp256k1SigningCommitments>,
    signature_shares: BTreeMap<Secp256k1Identifier, Secp256k1SignatureShare>,
    participant_indices: Vec<u16>,
    threshold: u16,
    total: u16,
    participant_index: u16,
}

#[wasm_bindgen]
impl FrostDkgSecp256k1 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            round1_secret: None,
            round2_secret: None,
            key_package: None,
            public_key_package: None,
            round1_packages: BTreeMap::new(),
            round2_packages: BTreeMap::new(),
            signing_nonces: None,
            signing_commitments: BTreeMap::new(),
            signature_shares: BTreeMap::new(),
            participant_indices: Vec::new(),
            threshold: 0,
            total: 0,
            participant_index: 0,
        }
    }

    pub fn init_dkg(&mut self, participant_index: u16, total: u16, threshold: u16) -> Result<(), WasmError> {
        self.participant_index = participant_index;
        self.total = total;
        self.threshold = threshold;
        self.participant_indices = (1..=total).collect();
        Ok(())
    }

    pub fn generate_round1(&mut self) -> Result<String, WasmError> {
        let identifier = Secp256k1Curve::identifier_from_u16(self.participant_index)?;
        let mut rng = OsRng;
        
        let (round1_secret, round1_package) = Secp256k1Curve::dkg_part1(
            identifier,
            self.total,
            self.threshold,
            &mut rng,
        )?;
        
        self.round1_secret = Some(round1_secret);
        let package_json = serde_json::to_string(&round1_package)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        Ok(hex::encode(package_json))
    }

    pub fn add_round1_package(&mut self, participant_index: u16, package_hex: &str) -> Result<(), WasmError> {
        let package_json = hex::decode(package_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        let package: frost_secp256k1::keys::dkg::round1::Package = serde_json::from_slice(&package_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let identifier = Secp256k1Curve::identifier_from_u16(participant_index)?;
        self.round1_packages.insert(identifier, package);
        Ok(())
    }

    pub fn can_start_round2(&self) -> bool {
        self.round1_packages.len() == self.total as usize && self.round1_secret.is_some()
    }

    pub fn generate_round2(&mut self) -> Result<String, WasmError> {
        let round1_secret = self.round1_secret.clone()
            .ok_or_else(|| WasmError::new("Round 1 secret not available"))?;
        
        let (round2_secret, round2_packages) = Secp256k1Curve::dkg_part2(
            round1_secret,
            &self.round1_packages,
        )?;
        
        self.round2_secret = Some(round2_secret);
        
        let mut packages_map = BTreeMap::new();
        for (id, package) in round2_packages {
            let id_value = id.serialize()[31] as u16 | ((id.serialize()[30] as u16) << 8);
            packages_map.insert(id_value, hex::encode(serde_json::to_string(&package).unwrap()));
        }
        
        Ok(serde_json::to_string(&packages_map).unwrap())
    }

    pub fn add_round2_package(&mut self, sender_index: u16, package_hex: &str) -> Result<(), WasmError> {
        let package_json = hex::decode(package_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        let package: frost_secp256k1::keys::dkg::round2::Package = serde_json::from_slice(&package_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let identifier = Secp256k1Curve::identifier_from_u16(sender_index)?;
        self.round2_packages.insert(identifier, package);
        Ok(())
    }

    pub fn can_finalize(&self) -> bool {
        self.round2_packages.len() >= (self.threshold - 1) as usize && self.round2_secret.is_some()
    }

    pub fn finalize_dkg(&mut self) -> Result<String, WasmError> {
        let round2_secret = self.round2_secret.as_ref()
            .ok_or_else(|| WasmError::new("Round 2 secret not available"))?;
        
        let (key_package, public_key_package) = Secp256k1Curve::dkg_part3(
            round2_secret,
            &self.round1_packages,
            &self.round2_packages,
        )?;
        
        self.key_package = Some(key_package.clone());
        self.public_key_package = Some(public_key_package.clone());
        
        let keystore_data = Keystore::export_keystore::<Secp256k1Curve>(
            &key_package,
            &public_key_package,
            self.threshold,
            self.total,
            self.participant_index,
            self.participant_indices.clone(),
            "secp256k1",
        )?;
        
        Ok(serde_json::to_string(&keystore_data).unwrap())
    }

    pub fn get_group_public_key(&self) -> Result<String, WasmError> {
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("DKG not complete"))?;
        
        let verifying_key = Secp256k1Curve::verifying_key(public_key_package);
        let key_bytes = Secp256k1Curve::serialize_verifying_key(&verifying_key)?;
        Ok(hex::encode(key_bytes))
    }

    pub fn get_address(&self) -> Result<String, WasmError> {
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("DKG not complete"))?;
        
        let verifying_key = Secp256k1Curve::verifying_key(public_key_package);
        Ok(Secp256k1Curve::get_address(&verifying_key))
    }

    pub fn get_eth_address(&self) -> Result<String, WasmError> {
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("DKG not complete"))?;
        
        let verifying_key = Secp256k1Curve::verifying_key(public_key_package);
        Ok(Secp256k1Curve::get_eth_address(&verifying_key)?)
    }

    pub fn is_dkg_complete(&self) -> bool {
        self.key_package.is_some() && self.public_key_package.is_some()
    }

    pub fn signing_commit(&mut self) -> Result<String, WasmError> {
        let key_package = self.key_package.as_ref()
            .ok_or_else(|| WasmError::new("Key package not available"))?;
        
        let (nonces, commitments) = Secp256k1Curve::generate_signing_commitment(key_package)?;
        self.signing_nonces = Some(nonces);
        
        let commitment_hex = hex::encode(serde_json::to_string(&commitments).unwrap());
        Ok(commitment_hex)
    }

    pub fn add_signing_commitment(&mut self, participant_index: u16, commitment_hex: &str) -> Result<(), WasmError> {
        let commitment_json = hex::decode(commitment_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        let commitment: Secp256k1SigningCommitments = serde_json::from_slice(&commitment_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let identifier = Secp256k1Curve::identifier_from_u16(participant_index)?;
        self.signing_commitments.insert(identifier, commitment);
        Ok(())
    }

    pub fn sign(&mut self, message_hex: &str) -> Result<String, WasmError> {
        let message = hex::decode(message_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let signing_package = Secp256k1Curve::create_signing_package(&self.signing_commitments, &message)?;
        
        let nonces = self.signing_nonces.as_ref()
            .ok_or_else(|| WasmError::new("Signing nonces not available"))?;
        let key_package = self.key_package.as_ref()
            .ok_or_else(|| WasmError::new("Key package not available"))?;
        
        let signature_share = Secp256k1Curve::generate_signature_share(&signing_package, nonces, key_package)?;
        
        Ok(hex::encode(serde_json::to_string(&signature_share).unwrap()))
    }

    pub fn add_signature_share(&mut self, participant_index: u16, share_hex: &str) -> Result<(), WasmError> {
        let share_json = hex::decode(share_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        let share: Secp256k1SignatureShare = serde_json::from_slice(&share_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let identifier = Secp256k1Curve::identifier_from_u16(participant_index)?;
        self.signature_shares.insert(identifier, share);
        Ok(())
    }

    pub fn aggregate_signature(&self, message_hex: &str) -> Result<String, WasmError> {
        let message = hex::decode(message_hex)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let signing_package = Secp256k1Curve::create_signing_package(&self.signing_commitments, &message)?;
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("Public key package not available"))?;
        
        let signature = Secp256k1Curve::aggregate_signature(&signing_package, &self.signature_shares, public_key_package)?;
        let sig_bytes = Secp256k1Curve::serialize_signature(&signature)?;
        
        Ok(hex::encode(sig_bytes))
    }

    pub fn clear_signing_state(&mut self) {
        self.signing_nonces = None;
        self.signing_commitments.clear();
        self.signature_shares.clear();
    }

    pub fn has_signing_nonces(&self) -> bool {
        self.signing_nonces.is_some()
    }

    pub fn import_keystore(&mut self, keystore_json: &str) -> Result<(), WasmError> {
        let keystore_data: KeystoreData = serde_json::from_str(keystore_json)
            .map_err(|e| WasmError::new(&e.to_string()))?;
        
        let (key_package, public_key_package) = Keystore::import_keystore::<Secp256k1Curve>(&keystore_data)?;
        
        self.key_package = Some(key_package);
        self.public_key_package = Some(public_key_package);
        self.threshold = keystore_data.min_signers;
        self.total = keystore_data.max_signers;
        self.participant_index = keystore_data.participant_index;
        self.participant_indices = keystore_data.participant_indices;
        
        Ok(())
    }

    pub fn export_keystore(&self) -> Result<String, WasmError> {
        let key_package = self.key_package.as_ref()
            .ok_or_else(|| WasmError::new("Key package not available"))?;
        let public_key_package = self.public_key_package.as_ref()
            .ok_or_else(|| WasmError::new("Public key package not available"))?;
        
        let keystore_data = Keystore::export_keystore::<Secp256k1Curve>(
            key_package,
            public_key_package,
            self.threshold,
            self.total,
            self.participant_index,
            self.participant_indices.clone(),
            "secp256k1",
        )?;
        
        Ok(serde_json::to_string(&keystore_data).unwrap())
    }
}

#[wasm_bindgen]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    console_log!("MPC Wallet WASM initialized");
}

// Called when the WASM module is instantiated
#[wasm_bindgen(start)]
pub fn start() {
    main();
}