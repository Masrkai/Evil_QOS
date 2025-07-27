use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::host::Host;
use crate::io::IO;
use tokio::time;
use pcap::{Capture, Device};

pub struct BandwidthMonitorResult {
    pub upload_rate: BitRate,
    pub upload_total_size: ByteValue,
    pub upload_total_count: usize,
    pub download_rate: BitRate,
    pub download_total_size: ByteValue,
    pub download_total_count: usize,
    upload_temp_size: ByteValue,
    download_temp_size: ByteValue,
    last_now: Instant,
}

pub struct BandwidthMonitor {
    interface: String,
    host_results: Arc<Mutex<HashMap<Host, BandwidthMonitorResult>>>,
    running: Arc<Mutex<bool>>,
}

impl BandwidthMonitor {
    pub fn new(interface: &str) -> Self {
        Self {
            interface: interface.to_string(),
            host_results: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn add(&self, host: Host) {
        let mut map = self.host_results.lock().unwrap();
        map.entry(host).or_insert_with(|| BandwidthMonitorResult {
            upload_rate: BitRate::zero(),
            upload_total_size: ByteValue::zero(),
            upload_total_count: 0,
            download_rate: BitRate::zero(),
            download_total_size: ByteValue::zero(),
            download_total_count: 0,
            upload_temp_size: ByteValue::zero(),
            download_temp_size: ByteValue::zero(),
            last_now: Instant::now(),
        });
    }

    pub fn remove(&self, host: &Host) {
        let mut map = self.host_results.lock().unwrap();
        map.remove(host);
    }

    pub fn replace(&self, old: &Host, new: Host) {
        let mut map = self.host_results.lock().unwrap();
        if let Some(entry) = map.remove(old) {
            map.insert(new, entry);
        }
    }

    pub fn get(&self, host: &Host) -> Option<BandwidthMonitorResult> {
        let mut map = self.host_results.lock().unwrap();
        if let Some(r) = map.get_mut(host) {
            let now = Instant::now();
            let elapsed = now.duration_since(r.last_now).as_secs_f64().max(0.0001);

            r.upload_rate = BitRate::from_bits((r.upload_temp_size.to_bits() as f64 / elapsed) as u64);
            r.download_rate = BitRate::from_bits((r.download_temp_size.to_bits() as f64 / elapsed) as u64);

            r.upload_temp_size = ByteValue::zero();
            r.download_temp_size = ByteValue::zero();
            r.last_now = now;

            return Some(r.clone());
        }
        None
    }

    pub async fn start(&self) {
        let iface = Device::list()
            .unwrap()
            .into_iter()
            .find(|d| d.name == self.interface)
            .expect("Interface not found")
            .clone();

        let mut cap = Capture::from_device(iface)
            .unwrap()
            .immediate_mode(true)
            .promisc(true)
            .open()
            .unwrap();

        *self.running.lock().unwrap() = true;
        let hr = self.host_results.clone();
        let run = self.running.clone();

        tokio::spawn(async move {
            while *run.lock().unwrap() {
                if let Ok(packet) = cap.next_packet() {
                    let len = packet.header.len as usize;
                    // parse IP src/dst from packet.data if IPv4
                    if let Some((src, dst)) = parse_ipv4_addrs(&packet.data) {
                        let mut map = hr.lock().unwrap();
                        for (host, result) in map.iter_mut() {
                            if host.ip == src {
                                result.upload_total_size += ByteValue::from_bytes(len as u64);
                                result.upload_total_count += 1;
                                result.upload_temp_size += ByteValue::from_bytes(len as u64);
                            } else if host.ip == dst {
                                result.download_total_size += ByteValue::from_bytes(len as u64);
                                result.download_total_count += 1;
                                result.download_temp_size += ByteValue::from_bytes(len as u64);
                            }
                        }
                    }
                }
                time::sleep(Duration::from_millis(10)).await;
            }
        });
        IO.ok("Bandwidth monitor started");
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
        IO.ok("Bandwidth monitor stopped");
    }
}

// Helper to parse IPv4 addresses from raw packet
fn parse_ipv4_addrs(data: &[u8]) -> Option<(String, String)> {
    if data.len() >= 34 && data[12] == 0x08 && data[13] == 0x00 {
        let src = std::net::Ipv4Addr::new(data[26], data[27], data[28], data[29]).to_string();
        let dst = std::net::Ipv4Addr::new(data[30], data[31], data[32], data[33]).to_string();
        return Some((src, dst));
    }
    None
}
