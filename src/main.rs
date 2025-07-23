// src/main.rs

use clap::Parser;
use env_logger; // Import the logger backend
use log::{info, warn, error}; // Import log macros

mod cli;
mod network;
mod limiter;

use cli::Cli;

#[tokio::main]
async fn main() {
    // Initialize the logger
    // You can control log level via RUST_LOG environment variable
    // e.g., RUST_LOG=debug cargo run
    env_logger::init();

    info!("Starting Evil_QOS...");

    // Parse command-line arguments using Clap
    let cli_args = Cli::parse();

    info!("Parsed CLI arguments: {:?}", cli_args); // Debug print, remove later

    // TODO: Add logic to validate arguments (e.g., target IPs, interface existence)

    // TODO: Add logic to get network interface details based on cli_args.interface
    // This might involve calling functions from `network` module

    // TODO: Add logic for host discovery if needed (network::discovery module)

    // TODO: Add logic to resolve target IPs to MAC addresses (network::arp module)

    // TODO: Add logic to enable IP forwarding on the attacker machine (platform specific)

    // TODO: Add logic to start the ARP spoofing loop (network::arp module)

    // TODO: Add logic to start the traffic interception and limiting (limiter::qos module)

    info!("Evil_QOS initialized. Press Ctrl+C to stop.");
    // Keep the application running (placeholder)
    // In reality, the spoofing loop and traffic handling will keep it busy.
    // You might use a Tokio signal handler here to gracefully shut down.
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        // This is just a placeholder to keep the main task alive.
        // Real implementation will have the spoofing/limiting running concurrently.
    }
}