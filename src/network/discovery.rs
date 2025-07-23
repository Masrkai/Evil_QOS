// src/network/discovery.rs

use ipnetwork::IpNetwork;
use log::{info, debug};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpStream;

/// Discovers active hosts using a simple TCP connect scan on common ports.
pub async fn discover_hosts(network_str: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
    let network: IpNetwork = network_str.parse()?;
    let mut active_hosts = Vec::new();

    // Define common ports to check
    let common_ports = vec![22, 80, 443, 53]; // SSH, HTTP, HTTPS, DNS

    info!("Starting host discovery on {}", network);

    // Iterate through all IPs in the network range
    for ip in network.iter() {
        // Skip network and broadcast addresses for IPv4
        if let IpAddr::V4(ipv4) = ip {
             if ipv4.is_broadcast() || ipv4.is_unspecified() {
                continue;
            }
            // Optionally skip the network address (first usable is usually .1)
            // if u32::from(*ipv4) == u32::from(network.network()) {
            //     continue;
            // }
        }

        debug!("Scanning {}", ip);

        // Check multiple common ports concurrently for this IP
        let mut port_checks = Vec::new();
        for &port in &common_ports {
            let addr = SocketAddr::new(ip, port);
            port_checks.push(tokio::spawn(check_port(addr)));
        }

        // Await the results for this IP
        for check in port_checks {
            if let Ok(Ok(true)) = check.await { // Check if task succeeded and port is open
                info!("Found active host: {}", ip);
                active_hosts.push(ip);
                break; // Found one open port, consider host alive, move to next IP
            }
        }
    }

    Ok(active_hosts)
}


/// Checks if a specific port is open on an IP address.
async fn check_port(addr: SocketAddr) -> Result<bool, Box<dyn std::error::Error>> {
    // Attempt to connect with a short timeout
    let timeout_duration = Duration::from_millis(500); // Adjust timeout as needed
    let result = tokio::time::timeout(timeout_duration, TcpStream::connect(addr)).await;

    match result {
        Ok(Ok(_stream)) => {
            // Connection successful
            debug!("Port {} is open", addr);
            Ok(true)
        }
        Ok(Err(_e)) => {
            // Connection failed (port closed, filtered, etc.)
            // debug!("Port {} is closed or filtered: {}", addr, e); // Uncomment for more verbose debugging
            Ok(false)
        }
        Err(_timeout) => {
            // Connection timed out
            // debug!("Timeout connecting to {}", addr);
            Ok(false)
        }
    }
}