//! Simple demo of the Elm architecture TUI
//!
//! Run with: cargo run --example elm_demo

use tui_node::elm::{ElmApp, Model, Message, Screen};
use tui_node::utils::appstate_compat::AppState;
use frost_secp256k1::Secp256k1Sha256;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create app state
    let app_state = Arc::new(Mutex::new(AppState::<Secp256k1Sha256>::new()));
    
    // Create Elm app
    let mut elm_app = ElmApp::new(
        "demo-device".to_string(),
        app_state,
    )?;
    
    println!("Starting Elm architecture demo...");
    println!("Press Ctrl+Q to quit");
    println!("Use arrow keys to navigate, Enter to select");
    println!("Esc key will go back (not exit!)");
    
    // Run the app
    elm_app.run().await?;
    
    println!("Goodbye!");
    
    Ok(())
}