use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::networking::host::Host;

pub struct HostWatcher {
    scanner: Arc<dyn HostScanner + Send + Sync>,
    reconnection_callback: Arc<dyn Fn(Host, Host) + Send + Sync>,
    hosts: Arc<Mutex<HashSet<Host>>>,
    interval: Arc<Mutex<u64>>, // in seconds
    iprange: Arc<Mutex<Option<String>>>,
    log_list: Arc<Mutex<Vec<HostReconnectLog>>>,
    running: Arc<Mutex<bool>>,
}

#[derive(Clone)]
pub struct HostReconnectLog {
    pub old: Host,
    pub new: Host,
    pub time: String,
}

pub trait HostScanner {
    fn scan_for_reconnects(
        &self,
        hosts: HashSet<Host>,
        iprange: Option<String>,
    ) -> HashMap<Host, Host>;
}

impl HostWatcher {
    pub fn new(
        scanner: Arc<dyn HostScanner + Send + Sync>,
        reconnection_callback: Arc<dyn Fn(Host, Host) + Send + Sync>,
    ) -> Self {
        HostWatcher {
            scanner,
            reconnection_callback,
            hosts: Arc::new(Mutex::new(HashSet::new())),
            interval: Arc::new(Mutex::new(45)),
            iprange: Arc::new(Mutex::new(None)),
            log_list: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn interval(&self) -> u64 {
        *self.interval.lock().unwrap()
    }

    pub fn set_interval(&self, value: u64) {
        *self.interval.lock().unwrap() = value;
    }

    pub fn iprange(&self) -> Option<String> {
        self.iprange.lock().unwrap().clone()
    }

    pub fn set_iprange(&self, value: Option<String>) {
        *self.iprange.lock().unwrap() = value;
    }

    pub fn hosts(&self) -> HashSet<Host> {
        self.hosts.lock().unwrap().clone()
    }

    pub fn log_list(&self) -> Vec<HostReconnectLog> {
        self.log_list.lock().unwrap().clone()
    }

    pub fn add(&self, mut host: Host) {
        host.watched = true;
        self.hosts.lock().unwrap().insert(host);
    }

    pub fn remove(&self, host: &Host) {
        self.hosts.lock().unwrap().remove(host);
    }

    pub fn start(&self) {
        let running = Arc::clone(&self.running);
        let hosts = Arc::clone(&self.hosts);
        let interval = Arc::clone(&self.interval);
        let iprange = Arc::clone(&self.iprange);
        let scanner = Arc::clone(&self.scanner);
        let callback = Arc::clone(&self.reconnection_callback);
        let log_list = Arc::clone(&self.log_list);

        *running.lock().unwrap() = true;

        thread::spawn(move || {
            while *running.lock().unwrap() {
                let hosts_copy = hosts.lock().unwrap().clone();

                if !hosts_copy.is_empty() {
                    let iprange_val = iprange.lock().unwrap().clone();
                    let reconnects = scanner.scan_for_reconnects(hosts_copy, iprange_val);

                    for (old_host, new_host) in reconnects {
                        callback(old_host.clone(), new_host.clone());
                        let log = HostReconnectLog {
                            old: old_host,
                            new: new_host,
                            time: format_time(SystemTime::now()),
                        };
                        log_list.lock().unwrap().push(log);
                    }
                }

                thread::sleep(Duration::from_secs(*interval.lock().unwrap()));
            }
        });
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }
}

fn format_time(system_time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = system_time.into();
    datetime.format("%Y-%m-%d %H:%M %p").to_string()
}
