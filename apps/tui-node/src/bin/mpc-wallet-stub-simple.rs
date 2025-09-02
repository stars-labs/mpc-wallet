use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "mpc-wallet-tui")]
#[command(about = "MPC Wallet TUI Node (Simple Stub for Deployment)")]
struct Args {
    #[arg(long, default_value = "ws://localhost:9000")]
    signal_server: String,
    
    #[arg(long, default_value = "mpc-node")]
    device_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("==========================================");
    println!("    MPC Wallet TUI Node (Stub Version)   ");
    println!("==========================================");
    println!("Signal Server: {}", args.signal_server);
    println!("Device ID: {}", args.device_id);
    println!("");
    println!("This is a stub version for deployment testing.");
    println!("The full DKG implementation is temporarily disabled.");
    println!("All MPC/FROST functionality will return stub responses.");
    println!("");
    println!("Status: RUNNING (stub mode)");
    println!("==========================================");
    
    let mut counter = 0;
    // Keep the process running and simulate activity
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        counter += 1;
        println!("[{}] Node {} heartbeat #{} (stub mode)", 
                 chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                 args.device_id, 
                 counter);
        
        // Simulate some periodic status updates
        if counter % 5 == 0 {
            println!("[{}] Status check: Signal server {}, Device status: Active (stub)", 
                     chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                     args.signal_server);
        }
    }
}