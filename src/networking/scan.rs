use std::time::Duration;
use tokio::task;
use async_arp::{Client, ClientConfigBuilder, ProbeStatus, Result};
use std::net::IpAddr;
use futures::future::join_all;
use crate::networking::host::Host;
use crate::IO; // assume IO implements some trait with .print
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

    pub async fn scan(&self) -> Result<Vec<Host>> {
        let config = ClientConfigBuilder::new(&self.interface)
            .with_response_timeout(Duration::from_secs_f64(1.5))
            .build()?; // no unwrap
        let client = Client::new(config)?; // returns Result

        let mut hosts = Vec::new();

        for batch in self.iprange.chunks(self.batch_size) {
            let mut probes = Vec::new();
            for ipstr in batch {
                if let Ok(addr) = ipstr.parse::<IpAddr>() {
                    probes.push(client.probe(addr));
                } else {
                    // skip bad IP
                }
            }

            let outcomes = join_all(probes).await;

            for outcome in outcomes.into_iter().flatten() {
                if outcome.status == ProbeStatus::Occupied {
                    let mut host = Host::new(
                        &outcome.ip.to_string(),
                        &outcome.mac.to_string(),
                        "",
                    );
                    host = self.resolve_name(host).await;
                    hosts.push(host);
                }
            }

            IO.print(&format!(
                "Scanned {}/{}\n",
                hosts.len(),
                self.iprange.len()
            ), "", true);
        }

        Ok(hosts)
    }

    async fn resolve_name(&self, mut host: Host) -> Host {
        let ip = host.ip.clone();
        if let Ok(lookup) = task::spawn_blocking(move || std::net::lookup_host((ip.as_str(), 0))).await {
            if let Ok(mut iter) = lookup {
                if let Some(socket) = iter.next() {
                    if let Ok(name) = std::net::lookup_addr(&socket.ip()) {
                        host.name = name;
                    }
                }
            }
        }
        host
    }

    /// Returns mapping from old-host (by MAC) to updated Host where IP changed
    pub fn scan_for_reconnects(
        &self,
        previous: &[Host],
        current: &[Host],
    ) -> HashMap<String, Host> {
        let mut mac_to_current: HashMap<String, &Host> = HashMap::new();
        for c in current {
            mac_to_current.insert(c.mac.clone(), c);
        }
        let mut reconnected = HashMap::new();
        for old in previous {
            if let Some(&cur) = mac_to_current.get(&old.mac) {
                if old.ip != cur.ip {
                    let mut updated = cur.clone();
                    updated.name = old.name.clone();
                    reconnected.insert(old.mac.clone(), updated);
                }
            }
        }
        reconnected
    }
}
