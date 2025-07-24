use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::networking::host::Host;
use crate::networking::scan::discover_hosts;

/// Watch for changes in network hosts
pub struct HostWatcher {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl HostWatcher {
    /// Create a new host watcher
    pub fn new() -> Self {
        HostWatcher {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
    
    /// Start watching for host changes
    pub fn start<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: Fn(Vec<Host>, Vec<Host>) + Send + 'static, // (new_hosts, disappeared_hosts)
    {
        if self.running.load(Ordering::Relaxed) {
            return Err("Watcher already running".to_string());
        }
        
        let running = Arc::clone(&self.running);
        
        self.running.store(true, Ordering::Relaxed);
        let handle = thread::spawn(move || {
            let mut previous_hosts = Vec::new();
            
            while running.load(Ordering::Relaxed) {
                match discover_hosts() {
                    Ok(current_hosts) => {
                        // Find new hosts
                        let new_hosts: Vec<Host> = current_hosts
                            .iter()
                            .filter(|host| !previous_hosts.contains(host))
                            .cloned()
                            .collect();
                        
                        // Find disappeared hosts
                        let disappeared_hosts: Vec<Host> = previous_hosts
                            .iter()
                            .filter(|host| !current_hosts.contains(host))
                            .cloned()
                            .collect();
                        
                        // Call callback if there are changes
                        if !new_hosts.is_empty() || !disappeared_hosts.is_empty() {
                            callback(new_hosts, disappeared_hosts);
                        }
                        
                        previous_hosts = current_hosts;
                    }
                    Err(e) => {
                        eprintln!("Error discovering hosts: {}", e);
                    }
                }
                
                // Check every 5 seconds
                thread::sleep(Duration::from_secs(5));
            }
        });
        
        self.handle = Some(handle);
        Ok(())
    }
    
    /// Stop watching
    pub fn stop(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            self.running.store(false, Ordering::Relaxed);
            
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for HostWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Start watching hosts (simplified interface)
pub fn watch_hosts<F>(callback: F) -> Result<(), String>
where
    F: Fn(Vec<Host>, Vec<Host>) + Send + 'static,
{
    static mut WATCHER: Option<HostWatcher> = None;
    
    unsafe {
        if WATCHER.is_none() {
            WATCHER = Some(HostWatcher::new());
        }
        
        if let Some(ref mut watcher) = WATCHER {
            watcher.start(callback)
        } else {
            Err("Failed to create watcher".to_string())
        }
    }
}