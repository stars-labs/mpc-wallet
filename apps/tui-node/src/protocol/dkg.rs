//! FROST DKG Protocol Implementation
//!
//! Based on the frost-core examples/dkg.rs implementation
//! Adapted for WebRTC message passing in the TUI node

use frost_core::Ciphersuite;
use frost_core::Identifier;
use frost_core::keys::dkg::{part1, part2, part3};
use frost_core::keys::{KeyPackage, PublicKeyPackage};
use frost_ed25519::Ed25519Sha512;
use frost_secp256k1::Secp256K1Sha256;
use frost_ed25519::rand_core::{CryptoRng, OsRng, RngCore};
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use log::{info, error, debug};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Message types for DKG protocol over WebRTC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DKGMessage {
    Round1Package {
        from: String,
        package_json: String,
    },
    Round2Package {
        from: String,
        to: String,
        package_json: String,
    },
    DKGComplete {
        from: String,
        success: bool,
    },
}

/// Round-specific messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DKGRoundMessage {
    Round1(Vec<u8>),
    Round2(Vec<u8>),
}

/// DKG Participant for a specific ciphersuite
struct Participant<C: Ciphersuite> {
    id: Identifier<C>,
    device_id: String,
    round1_secret_package: Option<frost_core::keys::dkg::round1::SecretPackage<C>>,
    round1_package: Option<frost_core::keys::dkg::round1::Package<C>>,
    round2_secret_package: Option<frost_core::keys::dkg::round2::SecretPackage<C>>,
    round1_packages_received: BTreeMap<Identifier<C>, frost_core::keys::dkg::round1::Package<C>>,
    round2_packages_received: BTreeMap<Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
    key_package: Option<KeyPackage<C>>,
    pubkey_package: Option<PublicKeyPackage<C>>,
}

impl<C: Ciphersuite> Participant<C> {
    fn new(id: Identifier<C>, device_id: String) -> Self {
        Self {
            id,
            device_id,
            round1_secret_package: None,
            round1_package: None,
            round2_secret_package: None,
            round1_packages_received: BTreeMap::new(),
            round2_packages_received: BTreeMap::new(),
            key_package: None,
            pubkey_package: None,
        }
    }

    fn generate_round1(
        &mut self,
        max_signers: u16,
        min_signers: u16,
        rng: &mut (impl RngCore + CryptoRng),
    ) -> Result<frost_core::keys::dkg::round1::Package<C>, frost_core::Error<C>> {
        let (secret, package) = part1(self.id, max_signers, min_signers, rng)?;
        self.round1_secret_package = Some(secret);
        self.round1_package = Some(package.clone());
        // Add own package to received
        self.round1_packages_received.insert(self.id, package.clone());
        Ok(package)
    }

    fn add_round1_package(&mut self, sender_id: Identifier<C>, package_json: &str) -> Result<(), String> {
        let package: frost_core::keys::dkg::round1::Package<C> =
            serde_json::from_str(package_json)
            .map_err(|e| format!("Failed to deserialize Round 1 package: {}", e))?;
        self.round1_packages_received.insert(sender_id, package);
        Ok(())
    }

    fn generate_round2(
        &mut self,
    ) -> Result<BTreeMap<Identifier<C>, frost_core::keys::dkg::round2::Package<C>>, frost_core::Error<C>> {
        // Filter out own package
        let round1_packages_from_others: BTreeMap<_, _> = self
            .round1_packages_received
            .iter()
            .filter(|(id, _)| *id != &self.id)
            .map(|(id, pkg)| (*id, pkg.clone()))
            .collect();

        let (secret, packages) = part2(
            self.round1_secret_package.take().unwrap(),
            &round1_packages_from_others,
        )?;

        self.round2_secret_package = Some(secret);
        Ok(packages)
    }

    fn add_round2_package(&mut self, sender_id: Identifier<C>, package_json: &str) -> Result<(), String> {
        let package: frost_core::keys::dkg::round2::Package<C> =
            serde_json::from_str(package_json)
            .map_err(|e| format!("Failed to deserialize Round 2 package: {}", e))?;
        self.round2_packages_received.insert(sender_id, package);
        Ok(())
    }

    fn finalize_dkg(&mut self) -> Result<(KeyPackage<C>, PublicKeyPackage<C>), frost_core::Error<C>> {
        // Filter out own package for round1
        let round1_packages_from_others: BTreeMap<_, _> = self
            .round1_packages_received
            .iter()
            .filter(|(id, _)| *id != &self.id)
            .map(|(id, pkg)| (*id, pkg.clone()))
            .collect();

        let (key_package, pubkey_package) = part3(
            self.round2_secret_package.as_ref().unwrap(),
            &round1_packages_from_others,
            &self.round2_packages_received,
        )?;

        self.key_package = Some(key_package.clone());
        self.pubkey_package = Some(pubkey_package.clone());
        Ok((key_package, pubkey_package))
    }
}

/// DKG Coordinator that manages the protocol execution
pub struct DKGCoordinator {
    device_id: String,
    session_id: String,
    curve: crate::elm::model::CurveType,
    max_signers: u16,
    min_signers: u16,
    participants: Vec<String>,
    ui_tx: mpsc::UnboundedSender<crate::elm::message::Message>,
    app_state: Arc<Mutex<crate::utils::state::AppState>>,
    is_coordinator: bool,
}

impl DKGCoordinator {
    pub fn new(
        device_id: String,
        session_id: String,
        curve: crate::elm::model::CurveType,
        max_signers: u16,
        min_signers: u16,
        participants: Vec<String>,
        ui_tx: mpsc::UnboundedSender<crate::elm::message::Message>,
        app_state: Arc<Mutex<crate::utils::state::AppState>>,
        is_coordinator: bool,
    ) -> Self {
        Self {
            device_id,
            session_id,
            curve,
            max_signers,
            min_signers,
            participants,
            ui_tx,
            app_state,
            is_coordinator,
        }
    }

    /// Execute the complete DKG protocol
    pub async fn execute_dkg(&self) -> Result<(), String> {
        info!("🚀 Starting FROST DKG protocol for session {}", self.session_id);
        info!("📊 Configuration: {} participants, threshold {}", self.max_signers, self.min_signers);
        info!("👥 Participants: {:?}", self.participants);

        // Execute based on curve type
        match self.curve {
            crate::elm::model::CurveType::Secp256k1 => {
                self.execute_typed_dkg::<Secp256K1Sha256>().await
            }
            crate::elm::model::CurveType::Ed25519 => {
                self.execute_typed_dkg::<Ed25519Sha512>().await
            }
        }
    }

    /// Execute DKG for a specific curve type
    async fn execute_typed_dkg<C>(&self) -> Result<(), String>
    where
        C: Ciphersuite,
    {
        // Find our participant index (1-based for FROST)
        let our_index = self.participants
            .iter()
            .position(|p| p == &self.device_id)
            .ok_or("Device not found in participants")?;
        let our_id = Identifier::try_from((our_index + 1) as u16)
            .map_err(|e| format!("Invalid identifier: {:?}", e))?;

        info!("📍 Our FROST identifier: {:?} (index {})", our_id, our_index + 1);

        // Create our participant
        let mut participant = Participant::<C>::new(our_id, self.device_id.clone());
        let mut rng = OsRng;

        // === Round 1: Generate and broadcast commitments ===
        info!("\n=== DKG Round 1: Generating commitments ===");

        // Update UI
        let _ = self.ui_tx.send(crate::elm::message::Message::UpdateDKGProgress {
            round: crate::elm::message::DKGRound::Round1,
            progress: 0.2,
        });

        // Generate our Round 1 package
        let round1_package = participant.generate_round1(self.max_signers, self.min_signers, &mut rng)
            .map_err(|e| format!("Failed to generate Round 1: {:?}", e))?;

        let round1_json = serde_json::to_string(&round1_package)
            .map_err(|e| format!("Failed to serialize Round 1: {}", e))?;

        info!("✅ Generated Round 1 package (size: {} bytes)", round1_json.len());

        // Broadcast Round 1 package to all other participants via WebRTC
        self.broadcast_round1_package(round1_json.clone()).await?;

        // Wait to receive Round 1 packages from others
        info!("⏳ Waiting for Round 1 packages from other participants...");
        let round1_packages = self.collect_round1_packages(our_id, &mut participant).await?;

        info!("✅ Received {} Round 1 packages", round1_packages);

        // === Round 2: Process Round 1 and generate shares ===
        info!("\n=== DKG Round 2: Generating shares ===");

        // Update UI
        let _ = self.ui_tx.send(crate::elm::message::Message::UpdateDKGProgress {
            round: crate::elm::message::DKGRound::Round2,
            progress: 0.5,
        });

        // Generate Round 2 packages
        let round2_packages = participant.generate_round2()
            .map_err(|e| format!("Failed to generate Round 2: {:?}", e))?;

        info!("✅ Generated Round 2 packages for {} participants", round2_packages.len());

        // Send Round 2 packages to specific recipients
        for (recipient_id, package) in &round2_packages {
            let package_json = serde_json::to_string(package)
                .map_err(|e| format!("Failed to serialize Round 2: {}", e))?;

            // Find the device_id for this FROST identifier
            let recipient_index = (*recipient_id).try_into()
                .map_err(|_| "Invalid recipient ID")?;
            let recipient_device = self.participants.get((recipient_index as usize) - 1)
                .ok_or("Recipient not found")?;

            self.send_round2_package(recipient_device.clone(), package_json).await?;
        }

        // Wait to receive Round 2 packages from others
        info!("⏳ Waiting for Round 2 packages from other participants...");
        let round2_packages = self.collect_round2_packages(our_id, &mut participant).await?;

        info!("✅ Received {} Round 2 packages", round2_packages);

        // === Finalize DKG ===
        info!("\n=== Finalizing DKG ===");

        // Update UI
        let _ = self.ui_tx.send(crate::elm::message::Message::UpdateDKGProgress {
            round: crate::elm::message::DKGRound::Finalization,
            progress: 0.8,
        });

        let (key_package, pubkey_package) = participant.finalize_dkg()
            .map_err(|e| format!("Failed to finalize DKG: {:?}", e))?;

        info!("✅ DKG Complete!");
        info!("🔑 Generated KeyPackage: {:?}", key_package);
        info!("🔓 Group Verifying Key: {:?}", pubkey_package.verifying_key());

        // Store the key package
        self.store_key_package(key_package, pubkey_package).await?;

        // Update UI with completion
        let _ = self.ui_tx.send(crate::elm::message::Message::DKGComplete {
            result: crate::elm::message::DKGResult {
                wallet_id: self.session_id.clone(),
                public_key: format!("{:?}", pubkey_package.verifying_key()),
                threshold: self.min_signers,
                total_signers: self.max_signers,
            },
        });

        info!("🎉 DKG protocol completed successfully!");
        Ok(())
    }

    /// Broadcast Round 1 package to all participants via WebRTC
    async fn broadcast_round1_package(&self, package_json: String) -> Result<(), String> {
        let state = self.app_state.lock().await;

        // Create the DKG message
        let dkg_msg = DKGMessage::Round1Package {
            from: self.device_id.clone(),
            package_json: package_json.clone(),
        };

        let msg_json = serde_json::to_string(&dkg_msg)
            .map_err(|e| format!("Failed to serialize DKG message: {}", e))?;

        // Send via all data channels
        for (device_id, channel) in &state.data_channels {
            if device_id != &self.device_id {
                info!("📤 Sending Round 1 package to {}", device_id);

                // Send the message through WebRTC data channel
                if let Err(e) = channel.send_text(msg_json.clone()).await {
                    error!("Failed to send Round 1 to {}: {:?}", device_id, e);
                } else {
                    info!("✅ Sent Round 1 package to {}", device_id);
                }
            }
        }

        Ok(())
    }

    /// Send Round 2 package to specific recipient
    async fn send_round2_package(&self, recipient: String, package_json: String) -> Result<(), String> {
        let state = self.app_state.lock().await;

        // Create the DKG message
        let dkg_msg = DKGMessage::Round2Package {
            from: self.device_id.clone(),
            to: recipient.clone(),
            package_json,
        };

        let msg_json = serde_json::to_string(&dkg_msg)
            .map_err(|e| format!("Failed to serialize DKG message: {}", e))?;

        // Send via data channel to specific recipient
        if let Some(channel) = state.data_channels.get(&recipient) {
            info!("📤 Sending Round 2 package to {}", recipient);

            if let Err(e) = channel.send_text(msg_json).await {
                error!("Failed to send Round 2 to {}: {:?}", recipient, e);
                return Err(format!("Failed to send Round 2: {:?}", e));
            }

            info!("✅ Sent Round 2 package to {}", recipient);
        } else {
            return Err(format!("No data channel to recipient {}", recipient));
        }

        Ok(())
    }

    /// Collect Round 1 packages from other participants
    async fn collect_round1_packages<C>(&self, our_id: Identifier<C>, participant: &mut Participant<C>) -> Result<usize, String>
    where
        C: Ciphersuite,
    {
        let expected_packages = self.max_signers as usize - 1; // Excluding ourselves
        let mut received = 0;

        // TODO: Implement actual message collection from WebRTC
        // For now, this is a placeholder that would listen for incoming messages

        // In a real implementation, we would:
        // 1. Listen on the WebRTC data channels
        // 2. Parse incoming DKGMessage::Round1Package messages
        // 3. Add them to the participant using participant.add_round1_package()
        // 4. Continue until we have all expected packages or timeout

        info!("⚠️ TODO: Implement Round 1 package collection from WebRTC");

        // Simulate receiving packages for testing
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(expected_packages)
    }

    /// Collect Round 2 packages from other participants
    async fn collect_round2_packages<C>(&self, our_id: Identifier<C>, participant: &mut Participant<C>) -> Result<usize, String>
    where
        C: Ciphersuite,
    {
        let expected_packages = self.max_signers as usize - 1; // Excluding ourselves

        // TODO: Implement actual message collection from WebRTC
        info!("⚠️ TODO: Implement Round 2 package collection from WebRTC");

        // Simulate receiving packages for testing
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(expected_packages)
    }

    /// Store the generated key package
    async fn store_key_package<C>(&self, key_package: KeyPackage<C>, pubkey_package: PublicKeyPackage<C>) -> Result<(), String>
    where
        C: Ciphersuite,
    {
        // TODO: Implement proper key storage
        info!("⚠️ TODO: Implement key package storage");

        // This would:
        // 1. Serialize the key package
        // 2. Encrypt it with a user password
        // 3. Store it in the keystore
        // 4. Update the wallet database

        Ok(())
    }
}