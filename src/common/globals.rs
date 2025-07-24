use std::time::Duration;

// Application metadata
pub const APP_NAME: &str = "Evil QoS";
pub const APP_VERSION: &str = "1.0.0";
pub const APP_DESCRIPTION: &str = "Network Traffic Shaping Tool";

// Network settings
pub const DEFAULT_INTERFACE: &str = "auto"; // Automatically detect interface
pub const DEFAULT_SCAN_TIMEOUT: Duration = Duration::from_secs(5);
pub const DEFAULT_ARP_SCAN_TIMEOUT: Duration = Duration::from_millis(100);
pub const DEFAULT_PING_TIMEOUT: Duration = Duration::from_secs(1);
pub const DEFAULT_PORT_SCAN_TIMEOUT: Duration = Duration::from_millis(500);

// Performance settings
pub const MAX_PACKETS_PER_SECOND: u32 = 10000; // Maximum packets per second to process
pub const BUFFER_SIZE: usize = 4096; // Buffer size for packet processing
pub const CHANNEL_SIZE: usize = 1024; // Size of async channels

// Bandwidth settings
pub const DEFAULT_BANDWIDTH_LIMIT: u64 = 0; // No limit by default (in bytes per second)
pub const MIN_BANDWIDTH_LIMIT: u64 = 0; // 0 means no traffic allowed
pub const MAX_BANDWIDTH_LIMIT: u64 = u64::MAX; // Unlimited

// Host monitoring settings
pub const HOST_WATCH_INTERVAL: Duration = Duration::from_secs(10);
pub const HOST_TIMEOUT: Duration = Duration::from_secs(60);
pub const MAX_HOSTS: usize = 256; // Maximum number of hosts to track

// Logging settings
pub const LOG_FILE: &str = "evil_qos.log";
pub const LOG_LEVEL: &str = "INFO"; // Can be DEBUG, INFO, WARN, ERROR

// Error messages
pub const ERROR_PERMISSION_DENIED: &str = "Permission denied. Try running with sudo.";
pub const ERROR_NO_INTERFACE: &str = "No network interface found.";
pub const ERROR_INVALID_IP: &str = "Invalid IP address provided.";
pub const ERROR_INVALID_MAC: &str = "Invalid MAC address provided.";

// Default ports for scanning
pub const DEFAULT_SCAN_PORTS: &[u16] = &[22, 80, 443, 8080];

// Colors for TUI (ANSI codes)
pub const COLOR_RESET: &str = "\x1b[0m";
pub const COLOR_RED: &str = "\x1b[31m";
pub const COLOR_GREEN: &str = "\x1b[32m";
pub const COLOR_YELLOW: &str = "\x1b[33m";
pub const COLOR_BLUE: &str = "\x1b[34m";
pub const COLOR_MAGENTA: &str = "\x1b[35m";
pub const COLOR_CYAN: &str = "\x1b[36m";
pub const COLOR_WHITE: &str = "\x1b[37m";
pub const COLOR_BOLD: &str = "\x1b[1m";

// Configuration file
pub const CONFIG_FILE: &str = "evil_qos.conf";

/// Get the application name with version
pub fn app_name_with_version() -> String {
    format!("{} v{}", APP_NAME, APP_VERSION)
}

/// Get the user agent string for network requests
pub fn user_agent() -> String {
    format!("{}/{}", APP_NAME, APP_VERSION)
}

/// Convert bytes to human readable format
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Convert seconds to human readable format
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{}h {}m {}s", hours, minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
    }
}