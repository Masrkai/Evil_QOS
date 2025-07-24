use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::networking::host::Host;

/// Traffic limiting modes
#[derive(Debug, Clone)]
pub enum LimitMode {
    /// Completely block internet access
    Block,
    /// Limit to a specific bitrate
    RateLimit(u64), // bits per second
    /// Allow full speed
    Unlimited,
}

/// Manages bandwidth limiting for hosts
pub struct BandwidthLimiter {
    limits: Arc<Mutex<HashMap<String, LimitMode>>>,
    traffic_stats: Arc<Mutex<HashMap<String, TrafficStats>>>,
}

/// Tracks traffic statistics for a host
struct TrafficStats {
    last_reset: Instant,
    bytes_transferred: u64,
}

impl BandwidthLimiter {
    /// Create a new bandwidth limiter
    pub fn new() -> Self {
        BandwidthLimiter {
            limits: Arc::new(Mutex::new(HashMap::new())),
            traffic_stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Apply a bandwidth limit to a specific host
    pub fn apply_limit(&self, host: &Host, mode: LimitMode) -> Result<(), String> {
        let mut limits = self.limits.lock()
            .map_err(|_| "Failed to acquire limits lock")?;
        limits.insert(host.ip.to_string(), mode);
        Ok(())
    }
    
    /// Apply a bandwidth limit to multiple hosts
    pub fn apply_limit_to_group(&self, hosts: &[Host], mode: LimitMode) -> Result<(), String> {
        let mut limits = self.limits.lock()
            .map_err(|_| "Failed to acquire limits lock")?;
        
        for host in hosts {
            limits.insert(host.ip.to_string(), mode.clone());
        }
        Ok(())
    }
    
    /// Remove limits from a host (allow full speed)
    pub fn remove_limit(&self, host: &Host) -> Result<(), String> {
        let mut limits = self.limits.lock()
            .map_err(|_| "Failed to acquire limits lock")?;
        limits.remove(&host.ip.to_string());
        Ok(())
    }
    
    /// Check if traffic should be dropped for a host
    pub fn should_drop_packet(&self, host: &Host, packet_size: usize) -> bool {
        let limits = match self.limits.lock() {
            Ok(limits) => limits,
            Err(_) => return false, // If we can't check limits, don't drop
        };
        
        let mode = match limits.get(&host.ip.to_string()) {
            Some(mode) => mode,
            None => return false, // No limit applied
        };
        
        match mode {
            LimitMode::Block => true,
            LimitMode::RateLimit(max_bits_per_sec) => {
                self.check_rate_limit(host, packet_size, *max_bits_per_sec)
            },
            LimitMode::Unlimited => false,
        }
    }
    
    /// Check if sending a packet would exceed the rate limit
    fn check_rate_limit(&self, host: &Host, packet_size: usize, max_bits_per_sec: u64) -> bool {
        let mut stats = match self.traffic_stats.lock() {
            Ok(stats) => stats,
            Err(_) => return false, // If we can't track stats, don't drop
        };
        
        let host_key = host.ip.to_string();
        let now = Instant::now();
        
        let host_stats = stats.entry(host_key).or_insert(TrafficStats {
            last_reset: now,
            bytes_transferred: 0,
        });
        
        // Reset counter every second
        if now.duration_since(host_stats.last_reset) > Duration::from_secs(1) {
            host_stats.last_reset = now;
            host_stats.bytes_transferred = 0;
        }
        
        let bits_in_packet = packet_size as u64 * 8;
        let new_total = host_stats.bytes_transferred + bits_in_packet;
        
        if new_total > max_bits_per_sec {
            true // Drop the packet
        } else {
            host_stats.bytes_transferred = new_total;
            false // Allow the packet
        }
    }
}

/// Apply bandwidth limit to a host (simplified interface)
pub fn apply_bandwidth_limit(host: &Host, mode: LimitMode) -> Result<(), String> {
    static LIMITER: once_cell::sync::Lazy<BandwidthLimiter> = once_cell::sync::Lazy::new(BandwidthLimiter::new);
    LIMITER.apply_limit(host, mode)
}