//! Networking module for Evil_QOS
//! Handles network interface detection, host discovery, ARP spoofing, and traffic limiting

pub mod host;
pub mod scan;
pub mod spoof;
pub mod limit;
pub mod monitor;
pub mod watch;
pub mod utils;

// Re-export key components
pub use host::Host;
pub use scan::discover_hosts;
pub use spoof::{start_arp_spoofing, stop_arp_spoofing};
pub use limit::apply_bandwidth_limit;
pub use monitor::start_monitoring;
pub use watch::watch_hosts;
pub use utils::{get_network_info, parse_bandwidth};