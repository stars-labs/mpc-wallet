use hex;
use wasm_bindgen::prelude::*;

// FROST DKG imports for Ed25519 and Secp256k1 curves
use frost_ed25519::{
    Identifier as Ed25519Identifier, Signature as Ed25519Signature,
    keys::{
        KeyPackage as Ed25519KeyPackage, PublicKeyPackage as Ed25519PublicKeyPackage,
        dkg as ed25519_dkg,
    },
    round1::{
        SigningCommitments as Ed25519SigningCommitments, SigningNonces as Ed25519SigningNonces,
    },
    round2::SignatureShare as Ed25519SignatureShare,
};

use frost_secp256k1::{
    Identifier as Secp256k1Identifier, Signature as Secp256k1Signature,
    keys::{
        KeyPackage as Secp256k1KeyPackage, PublicKeyPackage as Secp256k1PublicKeyPackage,
        dkg as secp256k1_dkg,
    },
    round1::{
        SigningCommitments as Secp256k1SigningCommitments, SigningNonces as Secp256k1SigningNonces,
    },
    round2::SignatureShare as Secp256k1SignatureShare,
};

// Required imports for MPC functions
use rand::rngs::OsRng;
use sha3::{Digest, Keccak256};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Error types for WASM
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

impl From<String> for WasmError {
    fn from(message: String) -> Self {
        WasmError { message }
    }
}

impl From<&str> for WasmError {
    fn from(message: &str) -> Self {
        WasmError {
            message: message.to_string(),
        }
    }
}

// Generic trait for FROST curve operations
trait FrostCurve {
    type Identifier: Copy + Clone + Serialize + for<'de> Deserialize<'de> + Ord;
    type KeyPackage: Clone + Serialize + for<'de> Deserialize<'de>;
    type PublicKeyPackage: Clone + Serialize + for<'de> Deserialize<'de>;
    type Round1SecretPackage: Clone;
    type Round2SecretPackage: Clone;
    type Round1Package: Clone + Serialize + for<'de> Deserialize<'de>;
    type Round2Package: Clone + Serialize + for<'de> Deserialize<'de>;
    type VerifyingKey;
    // FROST signing types
    type SigningNonces: Clone;
    type SigningCommitments: Clone + Serialize + for<'de> Deserialize<'de>;
    type SignatureShare: Clone + Serialize + for<'de> Deserialize<'de>;
    type Signature: Clone + Serialize + for<'de> Deserialize<'de>;
    type SigningPackage;

    fn identifier_from_u16(value: u16) -> Result<Self::Identifier, String>;
    fn dkg_part1(
        identifier: Self::Identifier,
        total: u16,
        threshold: u16,
        rng: &mut OsRng,
    ) -> Result<(Self::Round1SecretPackage, Self::Round1Package), String>;
    fn dkg_part2(
        round1_secret: Self::Round1SecretPackage,
        round1_packages: &BTreeMap<Self::Identifier, Self::Round1Package>,
    ) -> Result<
        (
            Self::Round2SecretPackage,
            BTreeMap<Self::Identifier, Self::Round2Package>,
        ),
        String,
    >;
    fn dkg_part3(
        round2_secret: &Self::Round2SecretPackage,
        round1_packages: &BTreeMap<Self::Identifier, Self::Round1Package>,
        round2_packages: &BTreeMap<Self::Identifier, Self::Round2Package>,
    ) -> Result<(Self::KeyPackage, Self::PublicKeyPackage), String>;
    fn verifying_key(public_key_package: &Self::PublicKeyPackage) -> Self::VerifyingKey;
    fn serialize_verifying_key(key: &Self::VerifyingKey) -> Result<Vec<u8>, String>;
    fn get_address(key: &Self::VerifyingKey) -> String;

    // FROST signing methods
    fn signing_commit(
        key_package: &Self::KeyPackage,
        rng: &mut OsRng,
    ) -> Result<(Self::SigningNonces, Self::SigningCommitments), String>;
    fn sign(
        signing_package: &Self::SigningPackage,
        nonces: &Self::SigningNonces,
        key_package: &Self::KeyPackage,
    ) -> Result<Self::SignatureShare, String>;
    fn aggregate_signature(
        signing_package: &Self::SigningPackage,
        signature_shares: &BTreeMap<Self::Identifier, Self::SignatureShare>,
        public_key_package: &Self::PublicKeyPackage,
    ) -> Result<Self::Signature, String>;
    fn create_signing_package(
        commitments: &BTreeMap<Self::Identifier, Self::SigningCommitments>,
        message: &[u8],
    ) -> Result<Self::SigningPackage, String>;
    fn serialize_signature(signature: &Self::Signature) -> Result<Vec<u8>, String>;
}

// Ed25519 implementation
struct Ed25519Curve;

impl FrostCurve for Ed25519Curve {
    type Identifier = Ed25519Identifier;
    type KeyPackage = Ed25519KeyPackage;
    type PublicKeyPackage = Ed25519PublicKeyPackage;
    type Round1SecretPackage = ed25519_dkg::round1::SecretPackage;
    type Round2SecretPackage = ed25519_dkg::round2::SecretPackage;
    type Round1Package = ed25519_dkg::round1::Package;
    type Round2Package = ed25519_dkg::round2::Package;
    type VerifyingKey = frost_ed25519::VerifyingKey;
    // FROST signing types
    type SigningNonces = Ed25519SigningNonces;
    type SigningCommitments = Ed25519SigningCommitments;
    type SignatureShare = Ed25519SignatureShare;
    type Signature = Ed25519Signature;
    type SigningPackage = frost_ed25519::SigningPackage;

    fn identifier_from_u16(value: u16) -> Result<Self::Identifier, String> {
        Ed25519Identifier::try_from(value).map_err(|e| e.to_string())
    }

    fn dkg_part1(
        identifier: Self::Identifier,
        total: u16,
        threshold: u16,
        rng: &mut OsRng,
    ) -> Result<(Self::Round1SecretPackage, Self::Round1Package), String> {
        ed25519_dkg::part1(identifier, total, threshold, rng).map_err(|e| e.to_string())
    }

    fn dkg_part2(
        round1_secret: Self::Round1SecretPackage,
        round1_packages: &BTreeMap<Self::Identifier, Self::Round1Package>,
    ) -> Result<
        (
            Self::Round2SecretPackage,
            BTreeMap<Self::Identifier, Self::Round2Package>,
        ),
        String,
    > {
        ed25519_dkg::part2(round1_secret, round1_packages).map_err(|e| e.to_string())
    }

    fn dkg_part3(
        round2_secret: &Self::Round2SecretPackage,
        round1_packages: &BTreeMap<Self::Identifier, Self::Round1Package>,
        round2_packages: &BTreeMap<Self::Identifier, Self::Round2Package>,
    ) -> Result<(Self::KeyPackage, Self::PublicKeyPackage), String> {
        ed25519_dkg::part3(round2_secret, round1_packages, round2_packages)
            .map_err(|e| e.to_string())
    }

    fn verifying_key(public_key_package: &Self::PublicKeyPackage) -> Self::VerifyingKey {
        *public_key_package.verifying_key()
    }

    fn serialize_verifying_key(key: &Self::VerifyingKey) -> Result<Vec<u8>, String> {
        key.serialize().map_err(|e| e.to_string())
    }

    fn get_address(key: &Self::VerifyingKey) -> String {
        let pubkey_bytes = key.serialize().unwrap_or_default();
        bs58::encode(pubkey_bytes).into_string()
    }

    // FROST signing method implementations
    fn signing_commit(
        key_package: &Self::KeyPackage,
        rng: &mut OsRng,
    ) -> Result<(Self::SigningNonces, Self::SigningCommitments), String> {
        let (nonces, commitments) = frost_ed25519::round1::commit(key_package.signing_share(), rng);
        Ok((nonces, commitments))
    }

    fn sign(
        signing_package: &Self::SigningPackage,
        nonces: &Self::SigningNonces,
        key_package: &Self::KeyPackage,
    ) -> Result<Self::SignatureShare, String> {
        frost_ed25519::round2::sign(signing_package, nonces, key_package).map_err(|e| e.to_string())
    }

    fn aggregate_signature(
        signing_package: &Self::SigningPackage,
        signature_shares: &BTreeMap<Self::Identifier, Self::SignatureShare>,
        public_key_package: &Self::PublicKeyPackage,
    ) -> Result<Self::Signature, String> {
        frost_ed25519::aggregate(signing_package, signature_shares, public_key_package)
            .map_err(|e| e.to_string())
    }

    fn create_signing_package(
        commitments: &BTreeMap<Self::Identifier, Self::SigningCommitments>,
        message: &[u8],
    ) -> Result<Self::SigningPackage, String> {
        Ok(frost_ed25519::SigningPackage::new(
            commitments.clone(),
            message,
        ))
    }

    fn serialize_signature(signature: &Self::Signature) -> Result<Vec<u8>, String> {
        signature
            .serialize()
            .map(|bytes| bytes.to_vec())
            .map_err(|e| e.to_string())
    }
}

// Secp256k1 implementation
struct Secp256k1Curve;

impl FrostCurve for Secp256k1Curve {
    type Identifier = Secp256k1Identifier;
    type KeyPackage = Secp256k1KeyPackage;
    type PublicKeyPackage = Secp256k1PublicKeyPackage;
    type Round1SecretPackage = secp256k1_dkg::round1::SecretPackage;
    type Round2SecretPackage = secp256k1_dkg::round2::SecretPackage;
    type Round1Package = secp256k1_dkg::round1::Package;
    type Round2Package = secp256k1_dkg::round2::Package;
    type VerifyingKey = frost_secp256k1::VerifyingKey;
    // FROST signing types
    type SigningNonces = Secp256k1SigningNonces;
    type SigningCommitments = Secp256k1SigningCommitments;
    type SignatureShare = Secp256k1SignatureShare;
    type Signature = Secp256k1Signature;
    type SigningPackage = frost_secp256k1::SigningPackage;

    fn identifier_from_u16(value: u16) -> Result<Self::Identifier, String> {
        Secp256k1Identifier::try_from(value).map_err(|e| e.to_string())
    }

    fn dkg_part1(
        identifier: Self::Identifier,
        total: u16,
        threshold: u16,
        rng: &mut OsRng,
    ) -> Result<(Self::Round1SecretPackage, Self::Round1Package), String> {
        secp256k1_dkg::part1(identifier, total, threshold, rng).map_err(|e| e.to_string())
    }

    fn dkg_part2(
        round1_secret: Self::Round1SecretPackage,
        round1_packages: &BTreeMap<Self::Identifier, Self::Round1Package>,
    ) -> Result<
        (
            Self::Round2SecretPackage,
            BTreeMap<Self::Identifier, Self::Round2Package>,
        ),
        String,
    > {
        secp256k1_dkg::part2(round1_secret, round1_packages).map_err(|e| e.to_string())
    }

    fn dkg_part3(
        round2_secret: &Self::Round2SecretPackage,
        round1_packages: &BTreeMap<Self::Identifier, Self::Round1Package>,
        round2_packages: &BTreeMap<Self::Identifier, Self::Round2Package>,
    ) -> Result<(Self::KeyPackage, Self::PublicKeyPackage), String> {
        secp256k1_dkg::part3(round2_secret, round1_packages, round2_packages)
            .map_err(|e| e.to_string())
    }

    fn verifying_key(public_key_package: &Self::PublicKeyPackage) -> Self::VerifyingKey {
        *public_key_package.verifying_key()
    }

    fn serialize_verifying_key(key: &Self::VerifyingKey) -> Result<Vec<u8>, String> {
        key.serialize().map_err(|e| e.to_string())
    }

    fn get_address(key: &Self::VerifyingKey) -> String {
        let pubkey_bytes = key.serialize().unwrap_or_default();

        // Convert from compressed to uncompressed format for Ethereum address computation
        if let Ok(k256_key) = k256::ecdsa::VerifyingKey::from_sec1_bytes(&pubkey_bytes) {
            let pubkey_point = k256_key.to_encoded_point(false);
            let pubkey_uncompressed = pubkey_point.as_bytes();
            let hash = Keccak256::digest(&pubkey_uncompressed[1..]);
            let address = &hash[12..];
            format!("0x{}", hex::encode(address))
        } else {
            // Fallback if conversion fails
            format!("0x{}", hex::encode(&[0u8; 20]))
        }
    }

    // FROST signing method implementations
    fn signing_commit(
        key_package: &Self::KeyPackage,
        rng: &mut OsRng,
    ) -> Result<(Self::SigningNonces, Self::SigningCommitments), String> {
        let (nonces, commitments) =
            frost_secp256k1::round1::commit(key_package.signing_share(), rng);
        Ok((nonces, commitments))
    }

    fn sign(
        signing_package: &Self::SigningPackage,
        nonces: &Self::SigningNonces,
        key_package: &Self::KeyPackage,
    ) -> Result<Self::SignatureShare, String> {
        frost_secp256k1::round2::sign(signing_package, nonces, key_package)
            .map_err(|e| e.to_string())
    }

    fn aggregate_signature(
        signing_package: &Self::SigningPackage,
        signature_shares: &BTreeMap<Self::Identifier, Self::SignatureShare>,
        public_key_package: &Self::PublicKeyPackage,
    ) -> Result<Self::Signature, String> {
        frost_secp256k1::aggregate(signing_package, signature_shares, public_key_package)
            .map_err(|e| e.to_string())
    }

    fn create_signing_package(
        commitments: &BTreeMap<Self::Identifier, Self::SigningCommitments>,
        message: &[u8],
    ) -> Result<Self::SigningPackage, String> {
        Ok(frost_secp256k1::SigningPackage::new(
            commitments.clone(),
            message,
        ))
    }

    fn serialize_signature(signature: &Self::Signature) -> Result<Vec<u8>, String> {
        signature
            .serialize()
            .map(|bytes| bytes.to_vec())
            .map_err(|e| e.to_string())
    }
}

// Generic DKG implementation
struct FrostDkgGeneric<C: FrostCurve> {
    identifier: Option<C::Identifier>,
    total_participants: Option<u16>,
    threshold: Option<u16>,
    round1_secret_package: Option<C::Round1SecretPackage>,
    round2_secret_package: Option<C::Round2SecretPackage>,
    round1_packages: BTreeMap<C::Identifier, C::Round1Package>,
    round2_packages: BTreeMap<C::Identifier, C::Round2Package>,
    key_package: Option<C::KeyPackage>,
    public_key_package: Option<C::PublicKeyPackage>,
    // FROST signing fields
    signing_nonces: Option<C::SigningNonces>,
    signing_commitments: BTreeMap<C::Identifier, C::SigningCommitments>,
    signature_shares: BTreeMap<C::Identifier, C::SignatureShare>,
}

impl<C: FrostCurve> FrostDkgGeneric<C> {
    fn new() -> Self {
        Self {
            identifier: None,
            total_participants: None,
            threshold: None,
            round1_secret_package: None,
            round2_secret_package: None,
            round1_packages: BTreeMap::new(),
            round2_packages: BTreeMap::new(),
            key_package: None,
            public_key_package: None,
            signing_nonces: None,
            signing_commitments: BTreeMap::new(),
            signature_shares: BTreeMap::new(),
        }
    }

    fn init_dkg(
        &mut self,
        participant_index: u16,
        total: u16,
        threshold: u16,
    ) -> Result<(), WasmError> {
        if threshold > total {
            return Err("Threshold cannot be greater than total participants".into());
        }
        if participant_index == 0 || participant_index > total {
            return Err("Participant index must be between 1 and total participants".into());
        }

        self.identifier = Some(C::identifier_from_u16(participant_index)?);
        self.total_participants = Some(total);
        self.threshold = Some(threshold);
        Ok(())
    }

    fn generate_round1(&mut self) -> Result<String, WasmError> {
        let identifier = self.identifier.ok_or("DKG not initialized")?;
        let total = self
            .total_participants
            .ok_or("Total participants not set")?;
        let threshold = self.threshold.ok_or("Threshold not set")?;

        let mut rng = OsRng;
        let (round1_secret_package, round1_package) =
            C::dkg_part1(identifier, total, threshold, &mut rng)?;

        self.round1_secret_package = Some(round1_secret_package);
        self.round1_packages
            .insert(identifier, round1_package.clone());

        console_log!(
            "🔍 WASM generate_round1: stored self package, total packages now: {}",
            self.round1_packages.len()
        );

        let serialized = serde_json::to_string(&round1_package)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        Ok(hex::encode(serialized.as_bytes()))
    }

    fn add_round1_package(
        &mut self,
        participant_index: u16,
        package_hex: &str,
    ) -> Result<(), WasmError> {
        let package_bytes =
            hex::decode(package_hex).map_err(|e| format!("Failed to decode hex: {}", e))?;
        let package_str = String::from_utf8(package_bytes)
            .map_err(|e| format!("Failed to convert bytes to string: {}", e))?;
        let round1_package: C::Round1Package = serde_json::from_str(&package_str)
            .map_err(|e| format!("Failed to deserialize round1 package: {}", e))?;

        let identifier = C::identifier_from_u16(participant_index)?;
        self.round1_packages.insert(identifier, round1_package);

        console_log!(
            "🔍 WASM add_round1_package: added package from participant {}, total packages now: {}",
            participant_index,
            self.round1_packages.len()
        );

        Ok(())
    }

    fn can_start_round2(&self) -> bool {
        let total = self.total_participants.unwrap_or(0);
        let packages_count = self.round1_packages.len();
        let can_start = packages_count == total as usize;

        console_log!(
            "🔍 WASM can_start_round2: packages_count={}, total={}, can_start={}",
            packages_count,
            total,
            can_start
        );

        can_start
    }

    fn generate_round2(&mut self) -> Result<String, WasmError> {
        if !self.can_start_round2() {
            return Err("Not all round 1 packages received".into());
        }

        let round1_secret_package = self
            .round1_secret_package
            .as_ref()
            .ok_or("Round 1 secret package not found")?;

        // Filter round1 packages to exclude self (dkg_part2 expects packages from other participants only)
        let self_identifier = self.identifier.ok_or("Self identifier not set")?;
        let round1_packages_from_others: std::collections::BTreeMap<_, _> = self
            .round1_packages
            .iter()
            .filter(|(id, _)| **id != self_identifier)
            .map(|(id, pkg)| (*id, pkg.clone()))
            .collect();

        console_log!(
            "Generating round 2: {} total packages, {} from others (excluding self)",
            self.round1_packages.len(),
            round1_packages_from_others.len()
        );

        // Generate round2 packages
        let (round2_secret_package, round2_packages) =
            C::dkg_part2(round1_secret_package.clone(), &round1_packages_from_others)?;

        self.round2_secret_package = Some(round2_secret_package);

        // All packages in round2_packages are created by us FOR other participants
        // We should send all of them to their respective recipients
        let serialized = serde_json::to_string(&round2_packages)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        console_log!(
            "Generated round 2 packages for {} participants",
            round2_packages.len()
        );
        Ok(hex::encode(serialized.as_bytes()))
    }

    fn add_round2_package(
        &mut self,
        sender_index: u16,
        package_hex: &str,
    ) -> Result<(), WasmError> {
        let package_bytes =
            hex::decode(package_hex).map_err(|e| format!("Failed to decode hex: {}", e))?;
        let package_str = String::from_utf8(package_bytes)
            .map_err(|e| format!("Failed to convert bytes to string: {}", e))?;

        // Try to deserialize directly first, then try as double-encoded JSON string
        let round2_package: C::Round2Package = serde_json::from_str(&package_str)
            .or_else(|_| {
                // If direct deserialization fails, try parsing as string first (double-encoded)
                let inner_str: String = serde_json::from_str(&package_str)
                    .map_err(|e| format!("Failed to parse as string: {}", e))?;
                serde_json::from_str(&inner_str)
                    .map_err(|e| format!("Failed to deserialize inner round2 package: {}", e))
            })
            .map_err(|e| format!("Failed to deserialize round2 package: {}", e))?;

        let sender_identifier = C::identifier_from_u16(sender_index)?;

        // Store the package from this sender
        self.round2_packages
            .insert(sender_identifier, round2_package);
        console_log!("Added round 2 package from participant {}", sender_index);

        Ok(())
    }

    fn can_finalize(&self) -> bool {
        let total = self.total_participants.unwrap_or(0);
        // We should have round2 packages from all other participants (excluding ourselves)
        self.round2_packages.len() == (total - 1) as usize
    }

    fn finalize_dkg(&mut self) -> Result<String, WasmError> {
        if !self.can_finalize() {
            return Err("Not all round 2 packages received".into());
        }

        let round2_secret_package = self
            .round2_secret_package
            .as_ref()
            .ok_or("Round 2 secret package not found")?;

        // Get self identifier to filter out our own packages
        let self_identifier = self.identifier.ok_or("DKG not initialized")?;

        // For part3, we need round1 packages from OTHER participants (excluding ourselves)
        let round1_packages_from_others: BTreeMap<C::Identifier, C::Round1Package> = self
            .round1_packages
            .iter()
            .filter(|(id, _)| **id != self_identifier)
            .map(|(id, pkg)| (*id, pkg.clone()))
            .collect();

        console_log!(
            "Finalizing DKG with {} round1 packages from others and {} round2 packages received",
            round1_packages_from_others.len(),
            self.round2_packages.len()
        );

        // Complete the DKG protocol
        // part3 expects: round1 packages from others, round2 packages received from others
        let (key_package, public_key_package) = C::dkg_part3(
            round2_secret_package,
            &round1_packages_from_others,
            &self.round2_packages,
        )?;

        // Store results
        self.key_package = Some(key_package);
        self.public_key_package = Some(public_key_package.clone());

        // Return the group public key
        let group_public_key = C::verifying_key(&public_key_package);
        let pubkey_bytes = C::serialize_verifying_key(&group_public_key)?;

        console_log!("DKG completed successfully");
        Ok(hex::encode(pubkey_bytes))
    }

    fn get_group_public_key(&self) -> Result<String, WasmError> {
        if let Some(ref public_key_package) = self.public_key_package {
            let group_public_key = C::verifying_key(public_key_package);
            let pubkey_bytes = C::serialize_verifying_key(&group_public_key)?;
            Ok(hex::encode(pubkey_bytes))
        } else {
            Err("DKG not completed yet".into())
        }
    }

    fn get_address(&self) -> Result<String, WasmError> {
        if let Some(ref public_key_package) = self.public_key_package {
            let group_public_key = C::verifying_key(public_key_package);
            Ok(C::get_address(&group_public_key))
        } else {
            Err("DKG not completed yet".into())
        }
    }

    fn is_dkg_complete(&self) -> bool {
        self.key_package.is_some() && self.public_key_package.is_some()
    }

    // FROST signing methods
    fn signing_commit(&mut self) -> Result<String, WasmError> {
        console_log!("🔍 signing_commit: key_package exists: {}", self.key_package.is_some());
        console_log!("🔍 signing_commit: identifier exists: {}", self.identifier.is_some());
        
        let key_package = self.key_package.as_ref().ok_or("DKG not completed")?;
        let mut rng = OsRng;

        let (nonces, commitments) = C::signing_commit(key_package, &mut rng)?;

        // Store nonces for later use in signing
        self.signing_nonces = Some(nonces.clone());

        // Also store our own commitment in the commitments map
        let our_identifier = self.identifier.ok_or("DKG not initialized")?;
        self.signing_commitments
            .insert(our_identifier, commitments.clone());

        // Return serialized commitments
        let serialized = serde_json::to_string(&commitments)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        Ok(hex::encode(serialized.as_bytes()))
    }

    fn add_signing_commitment(
        &mut self,
        participant_index: u16,
        commitment_hex: &str,
    ) -> Result<(), WasmError> {
        console_log!(
            "🔍 add_signing_commitment: participant_index={}, hex_length={}",
            participant_index,
            commitment_hex.len()
        );

        let commitment_bytes =
            hex::decode(commitment_hex).map_err(|e| format!("Failed to decode hex: {}", e))?;
        let commitment_str = String::from_utf8(commitment_bytes)
            .map_err(|e| format!("Failed to convert bytes to string: {}", e))?;
        let commitments: C::SigningCommitments = serde_json::from_str(&commitment_str)
            .map_err(|e| format!("Failed to deserialize commitments: {}", e))?;

        let identifier = C::identifier_from_u16(participant_index).map_err(|e| {
            format!(
                "Failed to create identifier from {}: {}",
                participant_index, e
            )
        })?;

        console_log!(
            "🔍 add_signing_commitment: storing commitment for participant {}",
            participant_index
        );
        self.signing_commitments.insert(identifier, commitments);

        console_log!(
            "🔍 add_signing_commitment: total commitments now: {}",
            self.signing_commitments.len()
        );
        Ok(())
    }

    fn sign(&mut self, message_hex: &str) -> Result<String, WasmError> {
        console_log!(
            "🔍 sign: starting with {} commitments",
            self.signing_commitments.len()
        );

        let nonces = self
            .signing_nonces
            .as_ref()
            .ok_or("No signing nonces available")?;
        let key_package = self.key_package.as_ref().ok_or("DKG not completed")?;

        // Decode message from hex
        let message =
            hex::decode(message_hex).map_err(|e| format!("Failed to decode message hex: {}", e))?;

        console_log!(
            "🔍 sign: creating signing package with {} commitments",
            self.signing_commitments.len()
        );

        // Create signing package from commitments
        let signing_package = C::create_signing_package(&self.signing_commitments, &message)
            .map_err(|e| format!("Failed to create signing package: {}", e))?;

        console_log!("🔍 sign: signing package created, generating signature share");

        // Generate signature share
        let signature_share = C::sign(&signing_package, nonces, key_package)?;

        // Store our own signature share for later aggregation
        let our_identifier = self.identifier.ok_or("DKG not initialized")?;
        console_log!(
            "🔍 sign: storing own share for participant"
        );
        self.signature_shares
            .insert(our_identifier, signature_share.clone());

        console_log!(
            "🔍 sign: stored our own signature share, total shares now: {}",
            self.signature_shares.len()
        );

        // Serialize and return
        let serialized = serde_json::to_string(&signature_share)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        Ok(hex::encode(serialized.as_bytes()))
    }

    fn add_signature_share(
        &mut self,
        participant_index: u16,
        share_hex: &str,
    ) -> Result<(), WasmError> {
        console_log!(
            "🔍 add_signature_share: participant_index={}, hex_length={}",
            participant_index,
            share_hex.len()
        );

        let share_bytes =
            hex::decode(share_hex).map_err(|e| format!("Failed to decode hex: {}", e))?;
        let share_str = String::from_utf8(share_bytes)
            .map_err(|e| format!("Failed to convert bytes to string: {}", e))?;
        let signature_share: C::SignatureShare = serde_json::from_str(&share_str)
            .map_err(|e| format!("Failed to deserialize signature share: {}", e))?;

        let identifier = C::identifier_from_u16(participant_index).map_err(|e| {
            format!(
                "Failed to create identifier from {}: {}",
                participant_index, e
            )
        })?;

        console_log!(
            "🔍 add_signature_share: storing share for participant {} (identifier index {})",
            participant_index, participant_index
        );
        self.signature_shares.insert(identifier, signature_share);

        console_log!(
            "🔍 add_signature_share: total shares now: {}",
            self.signature_shares.len()
        );
        Ok(())
    }

    fn aggregate_signature(&self, message_hex: &str) -> Result<String, WasmError> {
        console_log!(
            "🔍 aggregate_signature: starting with {} commitments and {} shares",
            self.signing_commitments.len(),
            self.signature_shares.len()
        );
        
        // Log commitment and share counts
        console_log!("🔍 aggregate_signature: checking participants...");
        
        // Check if we have matching commitments and shares
        let mut missing_shares = Vec::new();
        for (id, _) in &self.signing_commitments {
            if !self.signature_shares.contains_key(id) {
                missing_shares.push("missing");
            }
        }
        
        if !missing_shares.is_empty() {
            console_log!("🔍 aggregate_signature: ERROR - have commitments but missing {} shares", missing_shares.len());
        }

        let public_key_package = self
            .public_key_package
            .as_ref()
            .ok_or("DKG not completed")?;

        // Decode message from hex
        let message =
            hex::decode(message_hex).map_err(|e| format!("Failed to decode message hex: {}", e))?;

        console_log!("🔍 aggregate_signature: creating signing package");

        // Create signing package from commitments
        let signing_package = C::create_signing_package(&self.signing_commitments, &message)
            .map_err(|e| format!("Failed to create signing package for aggregation: {}", e))?;

        console_log!(
            "🔍 aggregate_signature: aggregating signature with {} shares",
            self.signature_shares.len()
        );

        // Aggregate signature
        let signature =
            C::aggregate_signature(&signing_package, &self.signature_shares, public_key_package)
                .map_err(|e| format!("Failed to aggregate signature: {}", e))?;

        console_log!("🔍 aggregate_signature: signature aggregation successful");

        // Serialize signature
        let signature_bytes = C::serialize_signature(&signature)?;
        Ok(hex::encode(signature_bytes))
    }
}

// WASM wrappers
#[wasm_bindgen]
pub struct FrostDkgEd25519 {
    inner: FrostDkgGeneric<Ed25519Curve>,
}

#[wasm_bindgen]
impl FrostDkgEd25519 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FrostDkgEd25519 {
        console_log!("Creating new FROST DKG Ed25519 instance");
        FrostDkgEd25519 {
            inner: FrostDkgGeneric::new(),
        }
    }

    #[wasm_bindgen]
    pub fn init_dkg(
        &mut self,
        participant_index: u16,
        total: u16,
        threshold: u16,
    ) -> Result<(), WasmError> {
        self.inner.init_dkg(participant_index, total, threshold)
    }

    #[wasm_bindgen]
    pub fn generate_round1(&mut self) -> Result<String, WasmError> {
        self.inner.generate_round1()
    }

    #[wasm_bindgen]
    pub fn add_round1_package(
        &mut self,
        participant_index: u16,
        package_hex: &str,
    ) -> Result<(), WasmError> {
        self.inner
            .add_round1_package(participant_index, package_hex)
    }

    #[wasm_bindgen]
    pub fn can_start_round2(&self) -> bool {
        self.inner.can_start_round2()
    }

    #[wasm_bindgen]
    pub fn generate_round2(&mut self) -> Result<String, WasmError> {
        self.inner.generate_round2()
    }

    #[wasm_bindgen]
    pub fn add_round2_package(
        &mut self,
        sender_index: u16,
        package_hex: &str,
    ) -> Result<(), WasmError> {
        self.inner.add_round2_package(sender_index, package_hex)
    }

    #[wasm_bindgen]
    pub fn can_finalize(&self) -> bool {
        self.inner.can_finalize()
    }

    #[wasm_bindgen]
    pub fn finalize_dkg(&mut self) -> Result<String, WasmError> {
        self.inner.finalize_dkg()
    }

    #[wasm_bindgen]
    pub fn get_group_public_key(&self) -> Result<String, WasmError> {
        self.inner.get_group_public_key()
    }

    #[wasm_bindgen]
    pub fn get_address(&self) -> Result<String, WasmError> {
        self.inner.get_address()
    }

    #[wasm_bindgen]
    pub fn is_dkg_complete(&self) -> bool {
        self.inner.is_dkg_complete()
    }

    // FROST signing methods
    #[wasm_bindgen]
    pub fn signing_commit(&mut self) -> Result<String, WasmError> {
        self.inner.signing_commit()
    }

    #[wasm_bindgen]
    pub fn add_signing_commitment(
        &mut self,
        participant_index: u16,
        commitment_hex: &str,
    ) -> Result<(), WasmError> {
        self.inner
            .add_signing_commitment(participant_index, commitment_hex)
    }

    #[wasm_bindgen]
    pub fn sign(&mut self, message_hex: &str) -> Result<String, WasmError> {
        self.inner.sign(message_hex)
    }

    #[wasm_bindgen]
    pub fn add_signature_share(
        &mut self,
        participant_index: u16,
        share_hex: &str,
    ) -> Result<(), WasmError> {
        self.inner.add_signature_share(participant_index, share_hex)
    }

    #[wasm_bindgen]
    pub fn aggregate_signature(&self, message_hex: &str) -> Result<String, WasmError> {
        self.inner.aggregate_signature(message_hex)
    }
}

#[wasm_bindgen]
pub struct FrostDkgSecp256k1 {
    inner: FrostDkgGeneric<Secp256k1Curve>,
}

#[wasm_bindgen]
impl FrostDkgSecp256k1 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FrostDkgSecp256k1 {
        console_log!("Creating new FROST DKG Secp256k1 instance");
        FrostDkgSecp256k1 {
            inner: FrostDkgGeneric::new(),
        }
    }

    #[wasm_bindgen]
    pub fn init_dkg(
        &mut self,
        participant_index: u16,
        total: u16,
        threshold: u16,
    ) -> Result<(), WasmError> {
        self.inner.init_dkg(participant_index, total, threshold)
    }

    #[wasm_bindgen]
    pub fn generate_round1(&mut self) -> Result<String, WasmError> {
        self.inner.generate_round1()
    }

    #[wasm_bindgen]
    pub fn add_round1_package(
        &mut self,
        participant_index: u16,
        package_hex: &str,
    ) -> Result<(), WasmError> {
        self.inner
            .add_round1_package(participant_index, package_hex)
    }

    #[wasm_bindgen]
    pub fn can_start_round2(&self) -> bool {
        self.inner.can_start_round2()
    }

    #[wasm_bindgen]
    pub fn generate_round2(&mut self) -> Result<String, WasmError> {
        self.inner.generate_round2()
    }

    #[wasm_bindgen]
    pub fn add_round2_package(
        &mut self,
        sender_index: u16,
        package_hex: &str,
    ) -> Result<(), WasmError> {
        self.inner.add_round2_package(sender_index, package_hex)
    }

    #[wasm_bindgen]
    pub fn can_finalize(&self) -> bool {
        self.inner.can_finalize()
    }

    #[wasm_bindgen]
    pub fn finalize_dkg(&mut self) -> Result<String, WasmError> {
        self.inner.finalize_dkg()
    }

    #[wasm_bindgen]
    pub fn get_group_public_key(&self) -> Result<String, WasmError> {
        self.inner.get_group_public_key()
    }

    #[wasm_bindgen]
    pub fn get_address(&self) -> Result<String, WasmError> {
        self.inner.get_address()
    }

    #[wasm_bindgen]
    pub fn get_eth_address(&self) -> Result<String, WasmError> {
        // For Secp256k1, get_address returns the Ethereum address
        self.inner.get_address()
    }

    #[wasm_bindgen]
    pub fn is_dkg_complete(&self) -> bool {
        self.inner.is_dkg_complete()
    }

    // FROST signing methods
    #[wasm_bindgen]
    pub fn signing_commit(&mut self) -> Result<String, WasmError> {
        self.inner.signing_commit()
    }

    #[wasm_bindgen]
    pub fn add_signing_commitment(
        &mut self,
        participant_index: u16,
        commitment_hex: &str,
    ) -> Result<(), WasmError> {
        self.inner
            .add_signing_commitment(participant_index, commitment_hex)
    }

    #[wasm_bindgen]
    pub fn sign(&mut self, message_hex: &str) -> Result<String, WasmError> {
        self.inner.sign(message_hex)
    }

    #[wasm_bindgen]
    pub fn add_signature_share(
        &mut self,
        participant_index: u16,
        share_hex: &str,
    ) -> Result<(), WasmError> {
        self.inner.add_signature_share(participant_index, share_hex)
    }

    #[wasm_bindgen]
    pub fn aggregate_signature(&self, message_hex: &str) -> Result<String, WasmError> {
        self.inner.aggregate_signature(message_hex)
    }
}

// Note: Removed FrostDkg wrapper struct to eliminate duplicate WASM exports
// Use FrostDkgEd25519 or FrostDkgSecp256k1 directly for specific curve implementations

// Initialize the library
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    console_log!("FROST DKG WASM library initialized");
}
