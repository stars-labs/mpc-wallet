// End-to-end demonstration of offline DKG + Signing process
// Simulates complete workflow: DKG ceremony followed by transaction signing

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use tempfile::TempDir;
use serde_json::json;

/// Simulated SD card for offline data exchange
#[derive(Clone)]
struct MockSDCard {
    base_dir: PathBuf,
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    round_counter: Arc<Mutex<usize>>,
}

impl MockSDCard {
    fn new(base_dir: PathBuf) -> Self {
        fs::create_dir_all(&base_dir).unwrap();
        Self {
            base_dir,
            files: Arc::new(Mutex::new(HashMap::new())),
            round_counter: Arc::new(Mutex::new(0)),
        }
    }
    
    fn export(&self, filename: &str, data: Vec<u8>) {
        let mut files = self.files.lock().unwrap();
        files.insert(filename.to_string(), data.clone());
        let filepath = self.base_dir.join(filename);
        fs::write(filepath, data).unwrap();
        println!("  📤 Exported: {}", filename);
    }
    
    fn import(&self, filename: &str) -> Option<Vec<u8>> {
        let files = self.files.lock().unwrap();
        if let Some(data) = files.get(filename) {
            println!("  📥 Imported: {}", filename);
            Some(data.clone())
        } else {
            None
        }
    }
    
    fn clear_signing_data(&self) {
        // Clear previous signing session data
        let mut files = self.files.lock().unwrap();
        let signing_files: Vec<String> = files.keys()
            .filter(|k| k.contains("signing") || k.contains("transaction") || k.contains("signature"))
            .cloned()
            .collect();
        
        for file in signing_files {
            files.remove(&file);
        }
        println!("  🗑️ Cleared previous signing data");
    }
    
    fn next_round(&self) -> usize {
        let mut counter = self.round_counter.lock().unwrap();
        *counter += 1;
        *counter
    }
}

/// Key share holder after DKG completion
#[derive(Clone)]
struct KeyShareHolder {
    participant_id: String,
    is_coordinator: bool,
    key_share: String,  // Simulated key share from DKG
    public_key: String,  // Group public key
    wallet_address: String,
}

/// Participant with both DKG and signing capabilities
struct Participant {
    id: String,
    is_coordinator: bool,
    sd_card: MockSDCard,
    key_holder: Option<KeyShareHolder>,
}

impl Participant {
    fn new(id: String, is_coordinator: bool, sd_card: MockSDCard) -> Self {
        Self { 
            id, 
            is_coordinator, 
            sd_card,
            key_holder: None,
        }
    }
    
    // ============= DKG PROCESS =============
    
    fn setup_phase(&self) {
        println!("\n[{}] 📋 DKG Setup Phase", self.id);
        
        if self.is_coordinator {
            let params = json!({
                "session_id": "DKG-DEMO-001",
                "threshold": 2,
                "participants": 3,
                "curve": "secp256k1"
            });
            
            self.sd_card.export("session_params.json", serde_json::to_vec(&params).unwrap());
            println!("  ✅ Created session parameters");
        } else {
            thread::sleep(Duration::from_millis(100));
            if let Some(_data) = self.sd_card.import("session_params.json") {
                println!("  ✅ Imported session parameters");
            }
        }
    }
    
    fn round1_commitments(&self) {
        println!("\n[{}] 🔑 DKG Round 1: Commitments", self.id);
        
        let commitment = json!({
            "participant": self.id,
            "commitment": format!("commitment_{}", self.id)
        });
        
        let filename = format!("round1_{}_commitment.json", self.id);
        self.sd_card.export(&filename, serde_json::to_vec(&commitment).unwrap());
        
        if self.is_coordinator {
            thread::sleep(Duration::from_millis(200));
            
            let aggregated = json!({
                "round": 1, 
                "all_commitments": ["P1", "P2", "P3"]
            });
            self.sd_card.export("round1_aggregated.json", serde_json::to_vec(&aggregated).unwrap());
            println!("  ✅ Aggregated all commitments");
        } else {
            thread::sleep(Duration::from_millis(300));
            if let Some(_data) = self.sd_card.import("round1_aggregated.json") {
                println!("  ✅ Imported aggregated commitments");
            }
        }
    }
    
    fn round2_shares(&self) {
        println!("\n[{}] 🔐 DKG Round 2: Share Distribution", self.id);
        
        let others = match self.id.as_str() {
            "P1" => vec!["P2", "P3"],
            "P2" => vec!["P1", "P3"],
            "P3" => vec!["P1", "P2"],
            _ => vec![],
        };
        
        for other in others {
            let filename = format!("round2_{}_to_{}.enc", self.id, other);
            let share = format!("encrypted_share_{}_to_{}", self.id, other);
            self.sd_card.export(&filename, share.into_bytes());
        }
        
        if self.is_coordinator {
            thread::sleep(Duration::from_millis(200));
            println!("  ✅ Redistributed shares to participants");
        }
    }
    
    fn finalize_dkg(&mut self) -> KeyShareHolder {
        println!("\n[{}] ✨ DKG Finalization", self.id);
        
        let key_holder = KeyShareHolder {
            participant_id: self.id.clone(),
            is_coordinator: self.is_coordinator,
            key_share: format!("key_share_{}", self.id),
            public_key: "0x04a7b8c9d2e3f4...".to_string(),
            wallet_address: "0x742d35Cc6634C053...".to_string(),
        };
        
        let public_data = json!({
            "participant": self.id,
            "public_key": &key_holder.public_key,
            "address": &key_holder.wallet_address,
        });
        
        let filename = format!("final_{}_public.json", self.id);
        self.sd_card.export(&filename, serde_json::to_vec(&public_data).unwrap());
        
        if self.is_coordinator {
            thread::sleep(Duration::from_millis(200));
            
            let wallet_data = json!({
                "wallet_id": "MPC_WALLET_001",
                "threshold": "2-of-3",
                "public_key": &key_holder.public_key,
                "address": &key_holder.wallet_address,
                "participants": ["P1", "P2", "P3"],
                "status": "SUCCESS"
            });
            
            self.sd_card.export("final_wallet.json", serde_json::to_vec(&wallet_data).unwrap());
            println!("  ✅ Created final wallet package");
        }
        
        self.key_holder = Some(key_holder.clone());
        println!("  ✅ Stored key share securely");
        
        key_holder
    }
    
    // ============= SIGNING PROCESS =============
    
    fn initiate_signing(&self, transaction: &serde_json::Value) {
        println!("\n[{}] 📝 Initiating Transaction Signing", self.id);
        
        if !self.is_coordinator {
            panic!("Only coordinator can initiate signing!");
        }
        
        // Create signing request
        let signing_request = json!({
            "request_id": format!("SIGN-{}", self.sd_card.next_round()),
            "transaction": transaction,
            "wallet_address": self.key_holder.as_ref().unwrap().wallet_address,
            "threshold": 2,
            "required_signers": ["P1", "P2", "P3"],
            "coordinator": self.id,
            "created_at": "2025-01-05T16:00:00Z",
        });
        
        self.sd_card.export("signing_request.json", serde_json::to_vec(&signing_request).unwrap());
        println!("  ✅ Created signing request");
        println!("  📋 Transaction details:");
        println!("     To: {}", transaction["to"]);
        println!("     Value: {}", transaction["value"]);
        println!("     Data: {}", transaction["data"]);
    }
    
    fn generate_signing_commitment(&self) -> String {
        println!("\n[{}] 🎲 Generating Signing Commitment", self.id);
        
        // Import signing request
        thread::sleep(Duration::from_millis(100));
        if let Some(request_data) = self.sd_card.import("signing_request.json") {
            let request: serde_json::Value = serde_json::from_slice(&request_data).unwrap();
            println!("  📋 Processing request: {}", request["request_id"]);
        }
        
        // Generate nonce commitment for this signing round
        let nonce_commitment = format!("nonce_commitment_{}_round_{}", 
            self.id, 
            self.sd_card.round_counter.lock().unwrap()
        );
        
        let commitment = json!({
            "participant": self.id,
            "commitment": &nonce_commitment,
            "timestamp": "2025-01-05T16:05:00Z",
        });
        
        let filename = format!("signing_commitment_{}.json", self.id);
        self.sd_card.export(&filename, serde_json::to_vec(&commitment).unwrap());
        println!("  ✅ Generated signing commitment");
        
        nonce_commitment
    }
    
    fn aggregate_commitments(&self) {
        if !self.is_coordinator {
            return;
        }
        
        println!("\n[{}] 📊 Aggregating Signing Commitments", self.id);
        thread::sleep(Duration::from_millis(300)); // Wait for all commitments
        
        let mut all_commitments = Vec::new();
        for participant in ["P1", "P2", "P3"] {
            let filename = format!("signing_commitment_{}.json", participant);
            if let Some(data) = self.sd_card.import(&filename) {
                all_commitments.push(participant.to_string());
            }
        }
        
        let aggregated = json!({
            "round": self.sd_card.round_counter.lock().unwrap().clone(),
            "commitments": all_commitments,
            "status": "ready_for_shares",
        });
        
        self.sd_card.export("signing_commitments_aggregated.json", 
            serde_json::to_vec(&aggregated).unwrap());
        println!("  ✅ Aggregated {} commitments", all_commitments.len());
    }
    
    fn generate_signature_share(&self, message_hash: &str) -> String {
        println!("\n[{}] ✍️ Generating Signature Share", self.id);
        
        // Import aggregated commitments
        thread::sleep(Duration::from_millis(100));
        if let Some(_data) = self.sd_card.import("signing_commitments_aggregated.json") {
            println!("  ✅ Imported aggregated commitments");
        }
        
        // Generate signature share using key share and commitments
        let key_share = &self.key_holder.as_ref().unwrap().key_share;
        // Safely handle hash truncation
        let hash_suffix = if message_hash.len() >= 8 {
            &message_hash[0..8]
        } else {
            message_hash
        };
        let signature_share = format!("sig_share_{}_{}", self.id, hash_suffix);
        
        let share_data = json!({
            "participant": self.id,
            "signature_share": &signature_share,
            "message_hash": message_hash,
            "timestamp": "2025-01-05T16:10:00Z",
        });
        
        let filename = format!("signature_share_{}.json", self.id);
        self.sd_card.export(&filename, serde_json::to_vec(&share_data).unwrap());
        println!("  ✅ Generated signature share");
        println!("  🔐 Share: {}...", &signature_share[0..20]);
        
        signature_share
    }
    
    fn aggregate_signatures(&self) -> String {
        if !self.is_coordinator {
            return String::new();
        }
        
        println!("\n[{}] 🔗 Aggregating Signature Shares", self.id);
        thread::sleep(Duration::from_millis(300)); // Wait for all shares
        
        let mut signature_shares = Vec::new();
        let mut participants_signed = Vec::new();
        
        // Collect signature shares (we need at least 2 for 2-of-3)
        for participant in ["P1", "P2"] {  // Simulating 2-of-3 threshold
            let filename = format!("signature_share_{}.json", participant);
            if let Some(data) = self.sd_card.import(&filename) {
                let share_data: serde_json::Value = serde_json::from_slice(&data).unwrap();
                signature_shares.push(share_data["signature_share"].as_str().unwrap().to_string());
                participants_signed.push(participant.to_string());
            }
        }
        
        // Aggregate shares into final signature
        let final_signature = format!("0x3045022100{}...", 
            signature_shares.join("").chars().take(40).collect::<String>()
        );
        
        let signature_data = json!({
            "transaction_signature": &final_signature,
            "participants_signed": participants_signed,
            "threshold_met": true,
            "signature_type": "ECDSA",
            "status": "COMPLETE",
        });
        
        self.sd_card.export("final_signature.json", serde_json::to_vec(&signature_data).unwrap());
        println!("  ✅ Aggregated {} signature shares", signature_shares.len());
        println!("  ✅ Final signature: {}...", &final_signature[0..20]);
        
        final_signature
    }
    
    fn broadcast_transaction(&self, signature: &str) {
        if !self.is_coordinator {
            return;
        }
        
        println!("\n[{}] 📡 Broadcasting Transaction", self.id);
        
        // In real scenario, this would submit to blockchain
        let broadcast_data = json!({
            "status": "BROADCAST",
            "signature": signature,
            "tx_hash": "0xabcd1234...",
            "block_explorer": "https://etherscan.io/tx/0xabcd1234...",
            "timestamp": "2025-01-05T16:15:00Z",
        });
        
        self.sd_card.export("transaction_broadcast.json", 
            serde_json::to_vec(&broadcast_data).unwrap());
        
        println!("  ✅ Transaction broadcast successfully!");
        println!("  🔗 Transaction hash: 0xabcd1234...");
        println!("  🌐 View on explorer: https://etherscan.io/tx/0xabcd1234...");
    }
}

/// Complete offline DKG + Signing demonstration
fn run_complete_offline_flow() {
    println!("🚀 Complete Offline DKG + Signing Process");
    println!("==========================================\n");
    
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let sd_card = MockSDCard::new(temp_dir.path().to_path_buf());
    
    // Create participants
    let mut p1 = Participant::new("P1".to_string(), true, sd_card.clone());
    let mut p2 = Participant::new("P2".to_string(), false, sd_card.clone());
    let mut p3 = Participant::new("P3".to_string(), false, sd_card.clone());
    
    println!("📊 Configuration:");
    println!("  • Threshold: 2-of-3");
    println!("  • Coordinator: P1");
    println!("  • Participants: P1, P2, P3");
    println!("  • Mode: Offline (SD Card Exchange)");
    
    // ============================================
    // PART 1: DKG CEREMONY
    // ============================================
    
    println!("\n╔════════════════════════════════════════╗");
    println!("║        PART 1: DKG CEREMONY            ║");
    println!("╚════════════════════════════════════════╝");
    
    // DKG Phase 1: Setup
    println!("\n━━━━━━━━━━ DKG PHASE 1: SETUP ━━━━━━━━━━");
    p1.setup_phase();
    p2.setup_phase();
    p3.setup_phase();
    
    // DKG Phase 2: Round 1
    println!("\n━━━━━━━━━━ DKG PHASE 2: ROUND 1 ━━━━━━━━━━");
    p1.round1_commitments();
    p2.round1_commitments();
    p3.round1_commitments();
    
    // DKG Phase 3: Round 2
    println!("\n━━━━━━━━━━ DKG PHASE 3: ROUND 2 ━━━━━━━━━━");
    p1.round2_shares();
    p2.round2_shares();
    p3.round2_shares();
    
    // DKG Phase 4: Finalization
    println!("\n━━━━━━━━━━ DKG PHASE 4: FINALIZATION ━━━━━━━━━━");
    let key1 = p1.finalize_dkg();
    let key2 = p2.finalize_dkg();
    let key3 = p3.finalize_dkg();
    
    println!("\n✅ DKG COMPLETE - Wallet Ready!");
    println!("  • Address: {}", key1.wallet_address);
    println!("  • Public Key: {}", key1.public_key);
    
    // Wait before signing
    thread::sleep(Duration::from_millis(500));
    
    // ============================================
    // PART 2: TRANSACTION SIGNING
    // ============================================
    
    println!("\n╔════════════════════════════════════════╗");
    println!("║      PART 2: TRANSACTION SIGNING       ║");
    println!("╚════════════════════════════════════════╝");
    
    // Clear previous signing data
    sd_card.clear_signing_data();
    
    // Create transaction to sign
    let transaction = json!({
        "to": "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
        "value": "1000000000000000000",  // 1 ETH in wei
        "data": "0x",
        "nonce": 42,
        "gas": 21000,
        "gasPrice": "20000000000",
    });
    
    // Signing Phase 1: Request Creation
    println!("\n━━━━━━━━━━ SIGNING PHASE 1: REQUEST ━━━━━━━━━━");
    p1.initiate_signing(&transaction);
    
    // Signing Phase 2: Commitment Round
    println!("\n━━━━━━━━━━ SIGNING PHASE 2: COMMITMENTS ━━━━━━━━━━");
    let _c1 = p1.generate_signing_commitment();
    let _c2 = p2.generate_signing_commitment();
    // P3 is offline/unavailable - simulating 2-of-3 threshold
    println!("\n[P3] ⚠️ Participant offline - proceeding with 2-of-3");
    
    p1.aggregate_commitments();
    
    // Signing Phase 3: Share Generation
    println!("\n━━━━━━━━━━ SIGNING PHASE 3: SHARES ━━━━━━━━━━");
    let message_hash = "0x1234567890abcdef..."; // Simulated transaction hash
    println!("\n📋 Message to sign: {}", message_hash);
    
    let _share1 = p1.generate_signature_share(message_hash);
    let _share2 = p2.generate_signature_share(message_hash);
    
    // Signing Phase 4: Aggregation
    println!("\n━━━━━━━━━━ SIGNING PHASE 4: AGGREGATION ━━━━━━━━━━");
    let final_signature = p1.aggregate_signatures();
    
    // Signing Phase 5: Broadcast
    println!("\n━━━━━━━━━━ SIGNING PHASE 5: BROADCAST ━━━━━━━━━━");
    p1.broadcast_transaction(&final_signature);
    
    // ============================================
    // SUMMARY
    // ============================================
    
    println!("\n╔════════════════════════════════════════╗");
    println!("║              SUMMARY                   ║");
    println!("╚════════════════════════════════════════╝");
    
    println!("\n🎉 COMPLETE OFFLINE WORKFLOW SUCCESS!");
    println!("\n📊 DKG Results:");
    println!("  ✅ 3 participants completed DKG");
    println!("  ✅ 2-of-3 threshold wallet created");
    println!("  ✅ Key shares distributed securely");
    
    println!("\n📊 Signing Results:");
    println!("  ✅ Transaction signed with 2-of-3 threshold");
    println!("  ✅ Signature shares aggregated successfully");
    println!("  ✅ Transaction broadcast to network");
    
    // Show all files created
    println!("\n📁 SD Card Contents:");
    let files = sd_card.files.lock().unwrap();
    let mut file_list: Vec<_> = files.keys().collect();
    file_list.sort();
    
    println!("\n  DKG Files:");
    for file in file_list.iter().filter(|f| !f.contains("sign") && !f.contains("transaction")) {
        println!("    • {}", file);
    }
    
    println!("\n  Signing Files:");
    for file in file_list.iter().filter(|f| f.contains("sign") || f.contains("transaction")) {
        println!("    • {}", file);
    }
    
    println!("\n⏱️ Time Estimates (Real Scenario):");
    println!("  • DKG Ceremony: 3-4 hours");
    println!("  • Transaction Signing: 1-2 hours");
    println!("  • Total SD Card Exchanges: ~20");
    
    println!("\n🔒 Security Notes:");
    println!("  • All operations performed air-gapped");
    println!("  • No network connectivity required");
    println!("  • Physical SD card exchanges only");
    println!("  • Suitable for high-value treasury operations");
}

fn main() {
    run_complete_offline_flow();
}

// ============================================
// TEST MODULE
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complete_offline_flow() {
        // Run the complete flow
        run_complete_offline_flow();
        
        // Test passes if no panics occur
        assert!(true);
    }
    
    #[test]
    fn test_sd_card_operations() {
        let temp_dir = TempDir::new().unwrap();
        let sd_card = MockSDCard::new(temp_dir.path().to_path_buf());
        
        // Test export and import
        let test_data = b"test data".to_vec();
        sd_card.export("test.txt", test_data.clone());
        
        let imported = sd_card.import("test.txt").unwrap();
        assert_eq!(imported, test_data);
        
        // Test clearing
        sd_card.export("signing_test.json", b"signing".to_vec());
        sd_card.clear_signing_data();
        assert!(sd_card.import("signing_test.json").is_none());
    }
    
    #[test]
    fn test_threshold_signing() {
        let temp_dir = TempDir::new().unwrap();
        let sd_card = MockSDCard::new(temp_dir.path().to_path_buf());
        
        // Setup participants with key shares
        let mut p1 = Participant::new("P1".to_string(), true, sd_card.clone());
        let mut p2 = Participant::new("P2".to_string(), false, sd_card.clone());
        
        // Simulate key holders
        p1.key_holder = Some(KeyShareHolder {
            participant_id: "P1".to_string(),
            is_coordinator: true,
            key_share: "key_1".to_string(),
            public_key: "pubkey".to_string(),
            wallet_address: "0xabc...".to_string(),
        });
        
        p2.key_holder = Some(KeyShareHolder {
            participant_id: "P2".to_string(),
            is_coordinator: false,
            key_share: "key_2".to_string(),
            public_key: "pubkey".to_string(),
            wallet_address: "0xabc...".to_string(),
        });
        
        // Test signing
        let tx = json!({"to": "0x123", "value": "100"});
        p1.initiate_signing(&tx);
        
        let c1 = p1.generate_signing_commitment();
        let c2 = p2.generate_signing_commitment();
        
        assert!(!c1.is_empty());
        assert!(!c2.is_empty());
        
        p1.aggregate_commitments();
        
        let share1 = p1.generate_signature_share("0x1234567890abcdef");
        let share2 = p2.generate_signature_share("0x1234567890abcdef");
        
        assert!(!share1.is_empty());
        assert!(!share2.is_empty());
        
        let signature = p1.aggregate_signatures();
        assert!(!signature.is_empty());
        assert!(signature.starts_with("0x"));
    }
}