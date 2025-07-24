use std::net::Ipv4Addr;
use macaddr::MacAddr6;

/// Represents a network host with its identifying information
#[derive(Debug, Clone, PartialEq)]
pub struct Host {
    pub ip: Ipv4Addr,
    pub mac: MacAddr6,
    pub hostname: Option<String>,
}

impl Host {
    /// Create a new Host instance
    pub fn new(ip: Ipv4Addr, mac: MacAddr6, hostname: Option<String>) -> Self {
        Host { ip, mac, hostname }
    }
    
    /// Check if this host is likely the gateway
    pub fn is_gateway(&self, gateway_ip: Ipv4Addr) -> bool {
        self.ip == gateway_ip
    }
}