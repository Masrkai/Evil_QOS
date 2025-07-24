use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use pnet::datalink;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::Packet;
use crate::networking::host::Host;
use crate::networking::utils::get_network_info;

/// Monitor network traffic
pub struct TrafficMonitor {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl TrafficMonitor {
    /// Create a new traffic monitor
    pub fn new() -> Self {
        TrafficMonitor {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
    
    /// Start monitoring traffic
    pub fn start<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: Fn(Host, usize) + Send + 'static,
    {
        if self.running.load(Ordering::Relaxed) {
            return Err("Monitor already running".to_string());
        }
        
        let (interface_name, _, _) = get_network_info()?;
        let interfaces = datalink::interfaces();
        let interface = interfaces.into_iter()
            .find(|iface| iface.name == interface_name)
            .ok_or("Interface not found")?;
        
        let running = Arc::clone(&self.running);
        
        self.running.store(true, Ordering::Relaxed);
        let handle = thread::spawn(move || {
            let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
                Ok(datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
                Ok(_) => {
                    eprintln!("Unhandled channel type");
                    return;
                }
                Err(e) => {
                    eprintln!("Error creating channel: {}", e);
                    return;
                }
            };
            
            while running.load(Ordering::Relaxed) {
                match rx.next() {
                    Ok(packet) => {
                        let ethernet = match EthernetPacket::new(packet) {
                            Some(ethernet) => ethernet,
                            None => continue,
                        };
                        
                        // In a real implementation, we would parse the packet to identify
                        // the source host and call the callback with traffic information
                        // For now, we'll just simulate this
                        let packet_size = ethernet.packet().len();
                        
                        // This is a simplified example - in practice you'd extract
                        // actual host information from the packet
                        // callback(host, packet_size);
                    }
                    Err(e) => {
                        eprintln!("Error receiving packet: {:?}", e);
                    }
                }
            }
        });
        
        self.handle = Some(handle);
        Ok(())
    }
    
    /// Stop monitoring
    pub fn stop(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            self.running.store(false, Ordering::Relaxed);
            
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for TrafficMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Start traffic monitoring (simplified interface)
pub fn start_monitoring<F>(callback: F) -> Result<(), String>
where
    F: Fn(Host, usize) + Send + 'static,
{
    static mut MONITOR: Option<TrafficMonitor> = None;
    
    unsafe {
        if MONITOR.is_none() {
            MONITOR = Some(TrafficMonitor::new());
        }
        
        if let Some(ref mut monitor) = MONITOR {
            monitor.start(callback)
        } else {
            Err("Failed to create monitor".to_string())
        }
    }
}