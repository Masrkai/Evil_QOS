use std::net::{Ipv4Addr, IpAddr};
use pnet::datalink;
use macaddr::MacAddr6;
use regex::Regex;
use crate::common::globals;

/// Get information about the default network interface
pub fn get_network_info() -> Result<(String, Ipv4Addr, Ipv4Addr), String> {
    // Find the default interface (usually the one with internet access)
    for interface in datalink::interfaces() {
        if !interface.is_loopback() && interface.is_up() {
            for ip in interface.ips {
                if let IpAddr::V4(ipv4) = ip.ip() {
                    // Calculate network address from IP and subnet mask
                    let netmask = ip.netmask();
                    if let IpAddr::V4(netmask_v4) = netmask {
                        let network = apply_netmask(ipv4, netmask_v4);
                        return Ok((interface.name, ipv4, network));
                    }
                }
            }
        }
    }
    Err("No suitable network interface found".to_string())
}

/// Apply a subnet mask to an IP address to get the network address
fn apply_netmask(ip: Ipv4Addr, netmask: Ipv4Addr) -> Ipv4Addr {
    let ip_octets = ip.octets();
    let mask_octets = netmask.octets();
    
    Ipv4Addr::new(
        ip_octets[0] & mask_octets[0],
        ip_octets[1] & mask_octets[1],
        ip_octets[2] & mask_octets[2],
        ip_octets[3] & mask_octets[3],
    )
}

/// Parse natural language bandwidth expressions into bits per second
pub fn parse_bandwidth(input: &str) -> Result<u64, String> {
    let re = Regex::new(r"(?i)^(\d+(?:\.\d+)?)\s*(kb|kbit|kbits|mb|mbit|mbits|gb|gbit|gbits|kb/s|mb/s|gb/s|b|bit|bits)$")
        .map_err(|e| format!("Regex error: {}", e))?;
    
    let caps = re.captures(input).ok_or("Invalid bandwidth format")?;
    let value: f64 = caps.get(1).unwrap().as_str().parse()
        .map_err(|_| "Invalid number format")?;
    let unit = caps.get(2).unwrap().as_str().to_lowercase();
    
    let bits = match unit.as_str() {
        "b" | "bit" | "bits" => value,
        "kb" | "kbit" | "kbits" | "kb/s" => value * 1000.0,
        "mb" | "mbit" | "mbits" | "mb/s" => value * 1000.0 * 1000.0,
        "gb" | "gbit" | "gbits" | "gb/s" => value * 1000.0 * 1000.0 * 1000.0,
        _ => return Err("Unknown unit".to_string()),
    };
    
    Ok(bits as u64)
}

/// Convert a string to a MAC address
pub fn parse_mac(mac: &str) -> Result<MacAddr6, String> {
    mac.parse::<MacAddr6>()
        .map_err(|_| "Invalid MAC address format".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_bandwidth() {
        assert_eq!(parse_bandwidth("10 kbit").unwrap(), 10_000);
        assert_eq!(parse_bandwidth("200 Kbits").unwrap(), 200_000);
        assert_eq!(parse_bandwidth("10 Mb").unwrap(), 10_000_000);
        assert_eq!(parse_bandwidth("4 MB/s").unwrap(), 4_000_000);
        assert!(parse_bandwidth("invalid").is_err());
    }
}