//! Networking module for Evil_QOS
//!
//! This module handles core networking functionalities including:
//! - Network interface and host information (`utils`, `host`)
//! - Host discovery (`scan`)
//! - ARP spoofing (`spoof`)
//! - Traffic limiting and blocking (`limit`)
//! - Bandwidth monitoring (`monitor`)
//! - Host reconnection watching (`watch`)
//!
//! Key types and functions are re-exported here for easier access.

// Declare submodules
pub mod host;
pub mod scan;
pub mod spoof;
pub mod limit;
pub mod monitor;
pub mod watch;
pub mod utils;

// Re-export key components for convenience.
// Grouped by submodule for better organization and clarity.

// --- Host ---
pub use host::Host;

// --- Scanning ---
pub use scan::discover_hosts;
// Consider if other items from `scan` (like `ProbeResult`, `ProbeStatus`) should be public

// // --- Spoofing ---
// pub use spoof::{start_arp_spoofing, stop_arp_spoofing};
// // Consider re-exporting `ArpSpoofer` if direct access is needed

// // --- Limiting ---
// pub use limit::{apply_bandwidth_limit, Direction}; // Export Direction enum as it's used with limit functions
// // Consider re-exporting `Limiter` if direct access is needed

// // --- Monitoring ---
// pub use monitor::{start_monitoring, BitRate, ByteValue}; // Export utility types used in monitoring results
// // Consider re-exporting `BandwidthMonitor` if direct access is needed

// // --- Watching ---
// pub use watch::watch_hosts;
// // Consider re-exporting `HostWatcher` or `HostReconnectLog` if direct access is needed

// // --- Utilities ---
// // Re-export specific utility functions likely needed at the module level
// pub use utils::{
//     get_network_info,    // Might encompass interface/gateway/netmask discovery
//     parse_bandwidth,     // Useful for parsing user input
//     // Add other specific utility functions if needed, e.g.:
//     // get_default_interface,
//     // get_default_gateway,
//     // validate_ip_address,
//     // validate_mac_address,
//     // BitRate, // Already exported from monitor, but could be from utils if it's the primary definition
//     // ByteValue, // Same as BitRate
// };
