use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::host::Host;
use crate::shell::Shell;
use crate::globals::{BIN_TC, BIN_IPTABLES};

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    None = 0,
    Outgoing = 1,
    Incoming = 2,
    Both = 3,
}

impl Direction {
    pub fn pretty(direction: Direction) -> &'static str {
        match direction {
            Direction::Outgoing => "upload",
            Direction::Incoming => "download",
            Direction::Both => "upload / download",
            _ => "-",
        }
    }
}

pub struct Limiter {
    interface: String,
    host_map: Arc<Mutex<HashMap<Host, HostLimitInfo>>>,
}

struct HostLimitInfo {
    ids: HostLimitIDs,
    rate: Option<u32>,
    direction: Direction,
}

#[derive(Clone)]
struct HostLimitIDs {
    upload_id: u32,
    download_id: u32,
}

impl Limiter {
    pub fn new(interface: &str) -> Self {
        Limiter {
            interface: interface.to_string(),
            host_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn limit(&self, host: &mut Host, direction: Direction, rate: u32) {
        let host_ids = self.new_host_limit_ids(host, direction);

        if (direction as u8 & Direction::Outgoing as u8) != 0 {
            if let Some(tc) = &*BIN_TC {
                let _ = Shell::execute_suppressed(
                    &format!("{tc} class add dev {} parent 1:0 classid 1:{} htb rate {} burst {}", self.interface, host_ids.upload_id, rate, rate as f32 * 1.1),
                    true,
                );
                let _ = Shell::execute_suppressed(
                    &format!("{tc} filter add dev {} parent 1:0 protocol ip prio {} handle {} fw flowid 1:{}", self.interface, host_ids.upload_id, host_ids.upload_id, host_ids.upload_id),
                    true,
                );
            }
            if let Some(iptables) = &*BIN_IPTABLES {
                let _ = Shell::execute_suppressed(
                    &format!("{iptables} -t mangle -A POSTROUTING -s {} -j MARK --set-mark {}", host.ip, host_ids.upload_id),
                    true,
                );
            }
        }

        if (direction as u8 & Direction::Incoming as u8) != 0 {
            if let Some(tc) = &*BIN_TC {
                let _ = Shell::execute_suppressed(
                    &format!("{tc} class add dev {} parent 1:0 classid 1:{} htb rate {} burst {}", self.interface, host_ids.download_id, rate, rate as f32 * 1.1),
                    true,
                );
                let _ = Shell::execute_suppressed(
                    &format!("{tc} filter add dev {} parent 1:0 protocol ip prio {} handle {} fw flowid 1:{}", self.interface, host_ids.download_id, host_ids.download_id, host_ids.download_id),
                    true,
                );
            }
            if let Some(iptables) = &*BIN_IPTABLES {
                let _ = Shell::execute_suppressed(
                    &format!("{iptables} -t mangle -A PREROUTING -d {} -j MARK --set-mark {}", host.ip, host_ids.download_id),
                    true,
                );
            }
        }

        host.limited = true;
        self.host_map.lock().unwrap().insert(host.clone(), HostLimitInfo { ids: host_ids, rate: Some(rate), direction });
    }

    pub fn block(&self, host: &mut Host, direction: Direction) {
        let host_ids = self.new_host_limit_ids(host, direction);

        if let Some(iptables) = &*BIN_IPTABLES {
            if (direction as u8 & Direction::Outgoing as u8) != 0 {
                let _ = Shell::execute_suppressed(&format!("{iptables} -t filter -A FORWARD -s {} -j DROP", host.ip), true);
            }
            if (direction as u8 & Direction::Incoming as u8) != 0 {
                let _ = Shell::execute_suppressed(&format!("{iptables} -t filter -A FORWARD -d {} -j DROP", host.ip), true);
            }
        }

        host.blocked = true;
        self.host_map.lock().unwrap().insert(host.clone(), HostLimitInfo { ids: host_ids, rate: None, direction });
    }

    pub fn unlimit(&self, host: &mut Host, direction: Direction) {
        if !host.limited && !host.blocked {
            return;
        }

        let mut map = self.host_map.lock().unwrap();
        if let Some(info) = map.remove(host) {
            if (direction as u8 & Direction::Outgoing as u8) != 0 {
                self.delete_tc_class(info.ids.upload_id);
                self.delete_iptables_entries(host, Direction::Outgoing, info.ids.upload_id);
            }
            if (direction as u8 & Direction::Incoming as u8) != 0 {
                self.delete_tc_class(info.ids.download_id);
                self.delete_iptables_entries(host, Direction::Incoming, info.ids.download_id);
            }
        }

        host.limited = false;
        host.blocked = false;
    }

    fn new_host_limit_ids(&self, host: &Host, direction: Direction) -> HostLimitIDs {
        let mut map = self.host_map.lock().unwrap();
        if map.contains_key(host) {
            drop(map);
            self.unlimit(&mut host.clone(), direction);
        }

        let id1 = self.generate_id(&[]);
        let id2 = self.generate_id(&[id1]);
        HostLimitIDs { upload_id: id1, download_id: id2 }
    }

    fn generate_id(&self, exclude: &[u32]) -> u32 {
        let map = self.host_map.lock().unwrap();
        let mut id = 1;
        loop {
            if exclude.contains(&id) {
                id += 1;
                continue;
            }
            if map.values().all(|x| x.ids.upload_id != id && x.ids.download_id != id) {
                return id;
            }
            id += 1;
        }
    }

    fn delete_tc_class(&self, id: u32) {
        if let Some(tc) = &*BIN_TC {
            let _ = Shell::execute_suppressed(&format!("{tc} filter del dev {} parent 1:0 prio {}", self.interface, id), true);
            let _ = Shell::execute_suppressed(&format!("{tc} class del dev {} parent 1:0 classid 1:{}", self.interface, id), true);
        }
    }

    fn delete_iptables_entries(&self, host: &Host, direction: Direction, id: u32) {
        if let Some(iptables) = &*BIN_IPTABLES {
            if (direction as u8 & Direction::Outgoing as u8) != 0 {
                let _ = Shell::execute_suppressed(&format!("{iptables} -t mangle -D POSTROUTING -s {} -j MARK --set-mark {}", host.ip, id), true);
                let _ = Shell::execute_suppressed(&format!("{iptables} -t filter -D FORWARD -s {} -j DROP", host.ip), true);
            }
            if (direction as u8 & Direction::Incoming as u8) != 0 {
                let _ = Shell::execute_suppressed(&format!("{iptables} -t mangle -D PREROUTING -d {} -j MARK --set-mark {}", host.ip, id), true);
                let _ = Shell::execute_suppressed(&format!("{iptables} -t filter -D FORWARD -d {} -j DROP", host.ip), true);
            }
        }
    }
}
