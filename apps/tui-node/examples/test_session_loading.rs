use tui_node::elm::command::Command;
use tui_node::elm::message::Message;
use tui_node::protocal::signal::{SessionInfo, SessionType};
use tui_node::utils::appstate_compat::AppState;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create a message channel
    let (tx, mut rx) = mpsc::channel::<Message>(100);
    
    // Create app state with a test session
    let mut app_state = AppState::<frost_core::Secp256k1Sha256>::new("test-device".to_string());
    
    // Add a test session to invites
    app_state.invites.push(SessionInfo {
        session_id: "TEST-SESSION-123".to_string(),
        proposer_id: "test-proposer".to_string(),
        total: 3,
        threshold: 2,
        participants: vec![],
        accepted_devices: vec!["test-proposer".to_string()],
        session_type: SessionType::DKG,
        curve_type: "secp256k1".to_string(),
        coordination_type: "Online".to_string(),
    });
    
    let app_state = Arc::new(Mutex::new(app_state));
    
    // Execute LoadSessions command
    println!("Executing LoadSessions command...");
    let cmd = Command::LoadSessions;
    cmd.execute(tx.clone(), &app_state).await.unwrap();
    
    // Try to receive the SessionsLoaded message
    println!("Waiting for SessionsLoaded message...");
    
    // Use try_recv with a small delay to allow the async task to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    match rx.try_recv() {
        Ok(Message::SessionsLoaded { sessions }) => {
            println!("✅ SUCCESS: Received SessionsLoaded with {} sessions", sessions.len());
            for session in &sessions {
                println!("  - Session: {} ({:?})", session.session_id, session.session_type);
            }
        }
        Ok(msg) => {
            println!("❌ FAIL: Received unexpected message: {:?}", msg);
        }
        Err(e) => {
            println!("❌ FAIL: Failed to receive message: {:?}", e);
        }
    }
}