// src/cli.rs

use clap::Parser;
use std::net::IpAddr;

/// Evil_QOS - A tool for network limitation using ARP spoofing.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Target IP address(es) or CIDR range(s) to limit (e.g., 192.168.1.10, 192.168.1.0/24)
    #[arg(short, long, value_name = "TARGET", required = true)]
    pub targets: Vec<String>, // Using String to allow for both IPs and CIDR easily parsed later

    /// Network interface to use (e.g., eth0, wlan0)
    #[arg(short, long, value_name = "INTERFACE", required = true)]
    pub interface: String,

    /// Gateway IP address (if not specified, it might be auto-detected)
    #[arg(short, long, value_name = "GATEWAY")]
    pub gateway: Option<IpAddr>,

    /// Mode of limitation: 'drop' or 'bandwidth'
    #[arg(short, long, value_name = "MODE", default_value = "drop")]
    pub mode: LimitMode,

    /// Bandwidth limit in KB/s (only used if mode is 'bandwidth')
    #[arg(short, long, value_name = "LIMIT_KBPS", requires = "bandwidth")]
    pub bandwidth_limit: Option<u32>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LimitMode {
    Drop,
    Bandwidth,
}

// Implementing FromStr for LimitMode might be needed if clap's derive doesn't cover it,
// but usually `clap::ValueEnum` handles it.