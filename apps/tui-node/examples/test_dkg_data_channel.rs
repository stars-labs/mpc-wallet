use cli_node::network::webrtc::{WEBRTC_API, WEBRTC_CONFIG};
use cli_node::network::websocket::handle_websocket_connection;
use cli_node::protocal::signal::{SessionInfo, SessionType};
/// Test example for DKG with data channel debugging
/// Run with: cargo run --example test_dkg_data_channel
use cli_node::utils::state::{AppState, InternalCommand, MeshStatus};
use frost_core::Ciphersuite;
use frost_ed25519::Ed25519Sha512;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{error, info};

const WEBSOCKET_URL: &str = "wss://auto-life.tech";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging with debug level
    tracing_subscriber::fmt().with_env_filter("debug").init();

    info!("Starting DKG Data Channel Test...");

    // Create three nodes
    let nodes = vec!["test-node-1", "test-node-2", "test-node-3"];
    let mut handles = vec![];

    // Start each node
    for (idx, node_id) in nodes.iter().enumerate() {
        let node_id = node_id.to_string();
        let is_creator = idx == 0;

        let handle = tokio::spawn(async move {
            if let Err(e) = run_node(node_id.clone(), is_creator).await {
                error!("Node {} failed: {}", node_id, e);
            }
        });

        handles.push(handle);

        // Give nodes time to start
        sleep(Duration::from_secs(2)).await;
    }

    // Wait for all nodes
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

async fn run_node(device_id: String, is_creator: bool) -> anyhow::Result<()> {
    info!("[{}] Starting node (creator: {})", device_id, is_creator);

    // Create app state
    let app_state: Arc<Mutex<AppState<Ed25519Sha512>>> = Arc::new(Mutex::new(AppState {
        connected: false,
        device_id: device_id.clone(),
        online_devices: vec![],
        session: None,
        device_connections: Arc::new(Mutex::new(HashMap::new())),
        data_channels: HashMap::new(),
        device_statuses: HashMap::new(),
        dkg_part1_public_package: None,
        dkg_part1_secret_package: None,
        dkg_part2_secret_package: None,
        generated_address: None,
        keystore: None,
        log: vec![],
        received_dkg_packages: HashMap::new(),
        received_dkg_round2_packages: HashMap::new(),
        dkg_state: cli_node::utils::state::DkgState::Idle,
        signing_packages: HashMap::new(),
        signing_commitments: HashMap::new(),
        signing_requests: HashMap::new(),
        pending_signing_acceptances: HashMap::new(),
        signature_shares: HashMap::new(),
        aggregated_signatures: HashMap::new(),
        sent_messages: HashSet::new(),
        session_invitations: Vec::new(),
        wallet_keystores: Vec::new(),
        selected_wallet: None,
        received_all_dkg_round1: false,
        reconnection_tracker: cli_node::utils::state::ReconnectionTracker::new(),
        pending_ice_candidates: HashMap::new(),
        identifier_map: None,
        mesh_status: MeshStatus::Incomplete,
        pending_mesh_ready_signals: Vec::new(),
        own_mesh_ready_sent: false,
    }));

    // Create command channel
    let (internal_cmd_tx, mut internal_cmd_rx) =
        mpsc::unbounded_channel::<InternalCommand<Ed25519Sha512>>();

    // Clone for WebSocket handler
    let ws_state = app_state.clone();
    let ws_cmd_tx = internal_cmd_tx.clone();
    let ws_device_id = device_id.clone();

    // Start WebSocket connection
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = handle_websocket_connection(
            WEBSOCKET_URL.to_string(),
            ws_device_id,
            ws_state,
            ws_cmd_tx,
            WEBRTC_API.clone(),
            WEBRTC_CONFIG.clone(),
        )
        .await
        {
            error!("WebSocket connection failed: {}", e);
        }
    });

    // Wait for connection
    sleep(Duration::from_secs(3)).await;

    // Create or join session
    if is_creator {
        info!("[{}] Creating DKG session...", device_id);

        // Create session
        let session_info = SessionInfo {
            session_id: "test-dkg-session".to_string(),
            session_type: SessionType::DKG,
            curve_type: "ed25519".to_string(),
            proposer_id: device_id.clone(),
            participants: vec![device_id.clone()],
            threshold: 2,
            accepted_devices: vec![device_id.clone()],
            description: Some("Test DKG session".to_string()),
        };

        // Set session in state
        {
            let mut state = app_state.lock().await;
            state.session = Some(session_info.clone());
            state
                .log
                .push(format!("📢 Created session: test-dkg-session"));
        }

        // Send AnnounceSession command
        internal_cmd_tx.send(InternalCommand::AnnounceSession {
            session_info: serde_json::to_value(&session_info).unwrap(),
        })?;
    } else {
        // Wait for session to be created
        sleep(Duration::from_secs(5)).await;

        info!("[{}] Joining DKG session...", device_id);

        // Send JoinSession command
        internal_cmd_tx.send(InternalCommand::JoinSession {
            session_id: "test-dkg-session".to_string(),
        })?;
    }

    // Process commands for 60 seconds
    let timeout = tokio::time::timeout(Duration::from_secs(60), async {
        while let Some(cmd) = internal_cmd_rx.recv().await {
            // Log important commands
            match &cmd {
                InternalCommand::ReportChannelOpen { device_id: remote } => {
                    info!("[{}] 📊 DATA CHANNEL OPEN with {}", device_id, remote);
                }
                InternalCommand::ProcessMeshReady { .. } => {
                    info!("[{}] 🌐 MESH READY signal received", device_id);
                }
                InternalCommand::CheckAndTriggerDkg => {
                    info!("[{}] 🎲 Checking DKG conditions...", device_id);
                }
                InternalCommand::TriggerDkgRound1 => {
                    info!("[{}] 🚀 Starting DKG Round 1!", device_id);
                }
                _ => {}
            }
        }
    })
    .await;

    // Print final state
    {
        let state = app_state.lock().await;
        info!("[{}] Final state:", device_id);
        info!("  - Connected: {}", state.connected);
        info!("  - Session: {}", state.session.is_some());
        info!("  - Data channels: {}", state.data_channels.len());
        info!("  - Mesh status: {:?}", state.mesh_status);
        info!("  - DKG state: {:?}", state.dkg_state);
        info!("  - Identifier map: {}", state.identifier_map.is_some());

        // Print last 10 log entries
        info!("  Last logs:");
        for log in state.log.iter().rev().take(10) {
            info!("    {}", log);
        }
    }

    // Cleanup
    drop(ws_handle);

    Ok(())
}
