use std::time::Duration;
use tokio::task;
use async_arp::{Client, ClientConfigBuilder, ProbeStatus};
use crate::host::Host;
use crate::io::IO;
use std::collections::HashMap;

pub struct HostScanner {
    interface: String,
    iprange: Vec<String>,
    batch_size: usize,
    resolve_timeout: Duration,
}

impl HostScanner {
    pub fn new(interface: &str, iprange: Vec<String>) -> Self {
        HostScanner {
            interface: interface.to_string(),
            iprange,
            batch_size: 75,
            resolve_timeout: Duration::from_secs_f64(1.0),
        }
    }

    pub async fn scan(&self) -> Vec<Host> {
        let client = Client::new(
            ClientConfigBuilder::new(&self.interface)
                .with_response_timeout(Duration::from_secs_f64(1.5))
                .build()
                .unwrap(),
        ).unwrap();

        let mut hosts = Vec::new();

        for batch in self.iprange.chunks(self.batch_size) {
            let ips = batch.to_vec();
            let mut futures = Vec::new();

            for ip in ips {
                futures.push(client.probe(ip.parse().unwrap()));
            }

            let results = futures::future::join_all(futures).await;

            for outcome in results.into_iter().flatten() {
                if outcome.status == ProbeStatus::Occupied {
                    let mut host = Host::new(
                        &outcome.ip.to_string(),
                        &outcome.mac.to_string(),
                        "",
                    );
                    hosts.push(self.resolve_name(host).await);
                }
            }

            IO.print(&format!("Scanned {}/{}\n", hosts.len(), self.iprange.len()), "", true);
        }

        hosts
    }

    async fn resolve_name(&self, mut host: Host) -> Host {
        let ip = host.ip.clone();
        let result = task::spawn_blocking(move || {
            std::net::lookup_host((ip.as_str(), 0))
        }).await;

        if let Ok(Ok(mut addrs)) = result {
            if let Some(entry) = addrs.next() {
                // attempt reverse lookup
                if let Ok((name, _)) = std::net::lookup_addr(&entry.ip()) {
                    host.name = name;
                }
            }
        }
        host
    }

    pub fn scan_for_reconnects(&self, previous: &[Host], current: &[Host]) -> HashMap<&Host, Host> {
        let mut mac_map: HashMap<String, &Host> = HashMap::new();
        for h in current {
            mac_map.insert(h.mac.clone(), h);
        }
        let mut reconnected = HashMap::new();
        for old in previous {
            if let Some(s) = mac_map.get(&old.mac) {
                if old.ip != s.ip {
                    let mut updated = (*s).clone();
                    updated.name = old.name.clone();
                    reconnected.insert(old, updated);
                }
            }
        }
        reconnected
    }
}
