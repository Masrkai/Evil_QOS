use crate::networking::Host;
use std::collections::HashMap;

/// Display a chart of host information
pub fn display_host_chart(hosts: &[Host], bandwidth_data: &HashMap<String, u64>) {
    if hosts.is_empty() {
        println!("No hosts to display");
        return;
    }

    println!("\n{:=<80}", "");
    println!("{}{:^78}{}", "|", "Network Hosts Information", "|");
    println!("{:=<80}", "");
    
    // Header
    println!(
        "{:<15} {:<17} {:<20} {:<15} {:<8}",
        "IP Address", "MAC Address", "Hostname", "Status", "Speed"
    );
    println!("{:-<80}", "");
    
    // Host data
    for host in hosts {
        let status = if host.is_active() { "Active" } else { "Inactive" };
        let bandwidth = bandwidth_data.get(&host.ip).copied().unwrap_or(0);
        let speed = format_bandwidth(bandwidth);
        
        println!(
            "{:<15} {:<17} {:<20} {:<15} {:<8}",
            host.ip,
            host.mac.as_deref().unwrap_or("Unknown"),
            host.hostname.as_deref().unwrap_or("Unknown"),
            status,
            speed
        );
    }
    
    println!("{:=<80}\n", "");
}

/// Format bandwidth in a human-readable way
fn format_bandwidth(bytes_per_sec: u64) -> String {
    if bytes_per_sec == 0 {
        "0 B/s".to_string()
    } else if bytes_per_sec < 1000 {
        format!("{} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1_000_000 {
        format!("{:.1} KB/s", bytes_per_sec as f64 / 1000.0)
    } else if bytes_per_sec < 1_000_000_000 {
        format!("{:.1} MB/s", bytes_per_sec as f64 / 1_000_000.0)
    } else {
        format!("{:.1} GB/s", bytes_per_sec as f64 / 1_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bandwidth() {
        assert_eq!(format_bandwidth(0), "0 B/s");
        assert_eq!(format_bandwidth(500), "500 B/s");
        assert_eq!(format_bandwidth(1500), "1.5 KB/s");
        assert_eq!(format_bandwidth(2_500_000), "2.5 MB/s");
        assert_eq!(format_bandwidth(3_000_000_000), "3.0 GB/s");
    }
}