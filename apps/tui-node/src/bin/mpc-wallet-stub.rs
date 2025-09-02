use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mpc-wallet-tui")]
#[command(about = "MPC Wallet TUI Node (Stub for Deployment)")]
struct Args {
    #[arg(long, default_value = "ws://localhost:9000")]
    signal_server: String,
    
    #[arg(long, default_value = "mpc-node")]
    device_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("MPC Wallet TUI Node (Stub Version)");
    println!("Signal Server: {}", args.signal_server);
    println!("Device ID: {}", args.device_id);
    println!("This is a stub version for deployment testing.");
    println!("The full DKG implementation is temporarily disabled.");
    
    // Keep the process running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        println!("Node {} is running (stub mode)...", args.device_id);
    }
}