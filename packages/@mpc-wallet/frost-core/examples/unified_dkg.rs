//! Example demonstrating unified DKG: one root secret → keys for both curves.
//!
//! Run with: cargo run --example unified_dkg

use mpc_wallet_frost_core::unified_dkg::{UnifiedDkg, UnifiedRound1Package};

fn main() {
    let max_signers: u16 = 3;
    let min_signers: u16 = 2;

    println!("=== Unified DKG: Single Root Secret → Ed25519 + Secp256k1 ===");
    println!("  Participants: {}", max_signers);
    println!("  Threshold: {}", min_signers);

    // Create participants, each with their own root secret
    let mut participants: Vec<UnifiedDkg> = (1..=max_signers)
        .map(|i| {
            let mut dkg = UnifiedDkg::new();
            dkg.init_dkg(i, max_signers, min_signers);
            dkg
        })
        .collect();

    // === Round 1 ===
    println!("\n--- Round 1: Generating commitments for both curves ---");
    let round1_packages: Vec<UnifiedRound1Package> = participants
        .iter_mut()
        .enumerate()
        .map(|(i, p)| {
            let pkg = p.generate_round1().expect("round1 failed");
            println!("  Participant {} generated round 1 packages", i + 1);
            pkg
        })
        .collect();

    // Distribute round 1 packages (each participant receives packages from others)
    for sender_idx in 0..max_signers as usize {
        for receiver_idx in 0..max_signers as usize {
            if sender_idx == receiver_idx {
                continue; // Don't send to self
            }
            let sender_id = (sender_idx + 1) as u16;
            participants[receiver_idx]
                .add_round1_package(sender_id, &round1_packages[sender_idx])
                .expect("add_round1_package failed");
        }
    }
    println!("  Round 1 packages distributed to all participants");

    // === Round 2 ===
    println!("\n--- Round 2: Generating shares for both curves ---");
    let round2_packages: Vec<_> = participants
        .iter_mut()
        .enumerate()
        .map(|(i, p)| {
            assert!(p.can_start_round2(), "participant {} not ready for round 2", i + 1);
            let pkgs = p.generate_round2().expect("round2 failed");
            println!("  Participant {} generated round 2 packages", i + 1);
            pkgs
        })
        .collect();

    // Distribute round 2 packages
    for sender_idx in 0..max_signers as usize {
        let sender_id = (sender_idx + 1) as u16;
        for receiver_idx in 0..max_signers as usize {
            let receiver_id = (receiver_idx + 1) as u16;
            if sender_id == receiver_id {
                continue;
            }
            let ed_hex = round2_packages[sender_idx]
                .ed25519
                .get(&receiver_id)
                .expect("missing ed25519 round2 package");
            let secp_hex = round2_packages[sender_idx]
                .secp256k1
                .get(&receiver_id)
                .expect("missing secp256k1 round2 package");
            participants[receiver_idx]
                .add_round2_package(sender_id, ed_hex, secp_hex)
                .expect("add_round2_package failed");
        }
    }
    println!("  Round 2 packages distributed to all participants");

    // === Finalize ===
    println!("\n--- Finalizing DKG for both curves ---");
    let mut sol_addresses = Vec::new();
    let mut eth_addresses = Vec::new();

    for (i, p) in participants.iter_mut().enumerate() {
        assert!(p.can_finalize(), "participant {} not ready to finalize", i + 1);
        let keystore = p.finalize_dkg().expect("finalize failed");
        println!("  Participant {} finalized DKG", i + 1);
        println!("    Ed25519 curve: {}", keystore.ed25519.curve);
        println!("    Secp256k1 curve: {}", keystore.secp256k1.curve);

        let sol = p.get_solana_address().expect("sol address failed");
        let eth = p.get_eth_address().expect("eth address failed");
        sol_addresses.push(sol);
        eth_addresses.push(eth);
    }

    // Verify all participants agree on the same addresses
    println!("\n=== Results ===");
    println!("  Solana address:   {}", sol_addresses[0]);
    println!("  Ethereum address: {}", eth_addresses[0]);

    for i in 1..sol_addresses.len() {
        assert_eq!(
            sol_addresses[0], sol_addresses[i],
            "Solana address mismatch for participant {}",
            i + 1
        );
        assert_eq!(
            eth_addresses[0], eth_addresses[i],
            "Ethereum address mismatch for participant {}",
            i + 1
        );
    }
    println!("\n  All participants agree on both addresses!");
    println!("\n=== Unified DKG complete: one root secret, two curves ===");
}
