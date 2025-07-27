use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::common::globals::{BIN_IPTABLES, BIN_TC};
use crate::host::Host;
use crate::shell::Shell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    None = 0,
    Outgoing = 1,
    Incoming = 2,
    Both = 3,
}

impl Direction {
    pub fn pretty_direction(direction: Direction) -> &'static str {
        match direction {
            Direction::Outgoing => "upload",
            Direction::Incoming => "download",
            Direction::Both => "upload / download",
            _ => "-",
        }
    }

    pub fn contains(self, flag: Direction) -> bool {
        (self as u8 & flag as u8) == flag as u8
    }
}

pub struct Limiter {
    interface: String,
    host_dict: Arc<Mutex<HashMap<Host, HostLimitEntry>>>,
}

#[derive(Clone)]
struct HostLimitIDs {
    upload_id: u32,
    download_id: u32,
}

#[derive(Clone)]
struct HostLimitEntry {
    ids: HostLimitIDs,
    rate: Option<u32>,
    direction: Direction,
}

impl Limiter {
    pub fn new(interface: &str) -> Self {
        Self {
            interface: interface.to_string(),
            host_dict: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn limit(&self, host: &mut Host, direction: Direction, rate: u32) {
        let host_ids = self.new_host_limit_ids(host, direction);

        if direction.contains(Direction::Outgoing) {
            if let Some(bin_tc) = &*BIN_TC {
                Shell::execute_suppressed(
                    &format!(
                        "{} class add dev {} parent 1:0 classid 1:{} htb rate {} burst {}",
                        bin_tc,
                        self.interface,
                        host_ids.upload_id,
                        rate,
                        (rate as f32 * 1.1) as u32
                    ),
                    true,
                );
                Shell::execute_suppressed(
                    &format!(
                        "{} filter add dev {} parent 1:0 protocol ip prio {} handle {} fw flowid 1:{}",
                        bin_tc,
                        self.interface,
                        host_ids.upload_id,
                        host_ids.upload_id,
                        host_ids.upload_id
                    ),
                    true,
                );
            }
            if let Some(bin_iptables) = &*BIN_IPTABLES {
                Shell::execute_suppressed(
                    &format!(
                        "{} -t mangle -A POSTROUTING -s {} -j MARK --set-mark {}",
                        bin_iptables, host.ip, host_ids.upload_id
                    ),
                    true,
                );
            }
        }

        if direction.contains(Direction::Incoming) {
            if let Some(bin_tc) = &*BIN_TC {
                Shell::execute_suppressed(
                    &format!(
                        "{} class add dev {} parent 1:0 classid 1:{} htb rate {} burst {}",
                        bin_tc,
                        self.interface,
                        host_ids.download_id,
                        rate,
                        (rate as f32 * 1.1) as u32
                    ),
                    true,
                );
                Shell::execute_suppressed(
                    &format!(
                        "{} filter add dev {} parent 1:0 protocol ip prio {} handle {} fw flowid 1:{}",
                        bin_tc,
                        self.interface,
                        host_ids.download_id,
                        host_ids.download_id,
                        host_ids.download_id
                    ),
                    true,
                );
            }
            if let Some(bin_iptables) = &*BIN_IPTABLES {
                Shell::execute_suppressed(
                    &format!(
                        "{} -t mangle -A PREROUTING -d {} -j MARK --set-mark {}",
                        bin_iptables, host.ip, host_ids.download_id
                    ),
                    true,
                );
            }
        }

        host.limited = true;

        let mut dict = self.host_dict.lock().unwrap();
        dict.insert(
            host.clone(),
            HostLimitEntry {
                ids: host_ids,
                rate: Some(rate),
                direction,
            },
        );
    }

    pub fn block(&self, host: &mut Host, direction: Direction) {
        let host_ids = self.new_host_limit_ids(host, direction);

        if let Some(bin_iptables) = &*BIN_IPTABLES {
            if direction.contains(Direction::Outgoing) {
                Shell::execute_suppressed(
                    &format!("{} -t filter -A FORWARD -s {} -j DROP", bin_iptables, host.ip),
                    true,
                );
            }
            if direction.contains(Direction::Incoming) {
                Shell::execute_suppressed(
                    &format!("{} -t filter -A FORWARD -d {} -j DROP", bin_iptables, host.ip),
                    true,
                );
            }
        }

        host.blocked = true;

        let mut dict = self.host_dict.lock().unwrap();
        dict.insert(
            host.clone(),
            HostLimitEntry {
                ids: host_ids,
                rate: None,
                direction,
            },
        );
    }

    pub fn unlimit(&self, host: &mut Host, direction: Direction) {
        if !host.limited && !host.blocked {
            return;
        }

        let mut dict = self.host_dict.lock().unwrap();
        if let Some(entry) = dict.remove(host) {
            if direction.contains(Direction::Outgoing) {
                self.delete_tc_class(entry.ids.upload_id);
                self.delete_iptables_entries(host, direction, entry.ids.upload_id);
            }
            if direction.contains(Direction::Incoming) {
                self.delete_tc_class(entry.ids.download_id);
                self.delete_iptables_entries(host, direction, entry.ids.download_id);
            }
        }

        host.limited = false;
        host.blocked = false;
    }

    pub fn replace(&self, old_host: &Host, new_host: &mut Host) {
        let info = {
            let dict = self.host_dict.lock().unwrap();
            dict.get(old_host).cloned()
        };

        if let Some(entry) = info {
            let mut old_host_clone = old_host.clone();
            self.unlimit(&mut old_host_clone, Direction::Both);

            if let Some(rate) = entry.rate {
                self.limit(new_host, entry.direction, rate);
            } else {
                self.block(new_host, entry.direction);
            }
        }
    }

    fn new_host_limit_ids(&self, host: &Host, direction: Direction) -> HostLimitIDs {
        let present = self.host_dict.lock().unwrap().contains_key(host);

        if present {
            let mut host_clone = host.clone();
            self.unlimit(&mut host_clone, direction);
        }

        let (id1, id2) = self.create_ids();
        HostLimitIDs {
            upload_id: id1,
            download_id: id2,
        }
    }

    fn create_ids(&self) -> (u32, u32) {
        fn generate_id(dict: &HashMap<Host, HostLimitEntry>, exclude: &[u32]) -> u32 {
            let mut id = 1;
            loop {
                if !exclude.contains(&id)
                    && !dict.values().any(|entry| {
                        entry.ids.upload_id == id || entry.ids.download_id == id
                    })
                {
                    return id;
                }
                id += 1;
            }
        }

        let dict = self.host_dict.lock().unwrap();
        let id1 = generate_id(&dict, &[]);
        let id2 = generate_id(&dict, &[id1]);
        (id1, id2)
    }

    fn delete_tc_class(&self, id: u32) {
        if let Some(bin_tc) = &*BIN_TC {
            Shell::execute_suppressed(
                &format!("{} filter del dev {} parent 1:0 prio {}", bin_tc, self.interface, id),
                true,
            );
            Shell::execute_suppressed(
                &format!("{} class del dev {} parent 1:0 classid 1:{}", bin_tc, self.interface, id),
                true,
            );
        }
    }

    fn delete_iptables_entries(&self, host: &Host, direction: Direction, id: u32) {
        if let Some(bin_iptables) = &*BIN_IPTABLES {
            if direction.contains(Direction::Outgoing) {
                Shell::execute_suppressed(
                    &format!(
                        "{} -t mangle -D POSTROUTING -s {} -j MARK --set-mark {}",
                        bin_iptables, host.ip, id
                    ),
                    true,
                );
                Shell::execute_suppressed(
                    &format!("{} -t filter -D FORWARD -s {} -j DROP", bin_iptables, host.ip),
                    true,
                );
            }
            if direction.contains(Direction::Incoming) {
                Shell::execute_suppressed(
                    &format!(
                        "{} -t mangle -D PREROUTING -d {} -j MARK --set-mark {}",
                        bin_iptables, host.ip, id
                    ),
                    true,
                );
                Shell::execute_suppressed(
                    &format!("{} -t filter -D FORWARD -d {} -j DROP", bin_iptables, host.ip),
                    true,
                );
            }
        }
    }
}
