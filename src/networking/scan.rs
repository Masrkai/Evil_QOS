use std::net::{Ipv4Addr, IpAddr};
use std::time::Duration;
use pnet::datalink;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use macaddr::MacAddr6;
use crate::networking::host::Host;
use crate::networking::utils::get_network_info;

/// Discover hosts on the local network using ARP scanning
pub fn discover_hosts() -> Result<Vec<Host>, String> {
    let (interface_name, _, network) = get_network_info()?;
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or("Interface not found")?;
    
    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unhandled channel type".to_string()),
        Err(e) => return Err(format!("Error creating channel: {}", e)),
    };
    
    let my_mac = interface.mac.ok_or("Interface MAC not found")?;
    let mut hosts = Vec::new();
    
    // Scan common IPs in the subnet
    for i in 1..255 {
        let target_ip = Ipv4Addr::from(u32::from(network).wrapping_add(i));
        send_arp_request(&mut tx, my_mac, target_ip)?;
        
        // Wait for response
        if let Ok(host) = receive_arp_reply(&mut rx, target_ip) {
            hosts.push(host);
        }
    }
    
    Ok(hosts)
}

/// Send an ARP request to a target IP
fn send_arp_request(
    tx: &mut Box<dyn datalink::DataLinkSender>,
    source_mac: MacAddr6,
    target_ip: Ipv4Addr,
) -> Result<(), String> {
    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer)
        .ok_or("Failed to create Ethernet packet")?;
    
    ethernet_packet.set_destination(MacAddr6::broadcast());
    ethernet_packet.set_source(source_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);
    
    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer)
        .ok_or("Failed to create ARP packet")?;
    
    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_sender_proto_addr(Ipv4Addr::UNSPECIFIED); // We'll set this properly in a real implementation
    arp_packet.set_target_hw_addr(MacAddr6::zero());
    arp_packet.set_target_proto_addr(target_ip);
    
    ethernet_packet.set_payload(arp_packet.packet_mut());
    
    tx.send_to(ethernet_packet.packet(), None)
        .map_err(|e| format!("Failed to send ARP request: {:?}", e))?
        .map_err(|e| format!("Failed to send ARP request: {:?}", e))?;
    
    Ok(())
}

/// Receive an ARP reply and extract host information
fn receive_arp_reply(
    rx: &mut Box<dyn datalink::DataLinkReceiver>,
    expected_ip: Ipv4Addr,
) -> Result<Host, String> {
    let timeout = Duration::from_millis(100);
    
    match rx.next() {
        Ok(packet) => {
            let ethernet = EthernetPacket::new(packet)
                .ok_or("Failed to parse Ethernet packet")?;
            
            if ethernet.get_ethertype() != EtherTypes::Arp {
                return Err("Not an ARP packet".to_string());
            }
            
            let arp = ArpPacket::new(ethernet.payload())
                .ok_or("Failed to parse ARP packet")?;
            
            if arp.get_operation() != ArpOperations::Reply {
                return Err("Not an ARP reply".to_string());
            }
            
            if arp.get_target_proto_addr() != expected_ip {
                return Err("ARP reply for wrong IP".to_string());
            }
            
            Ok(Host::new(
                arp.get_sender_proto_addr(),
                arp.get_sender_hw_addr(),
                None, // We don't get hostname from ARP
            ))
        }
        Err(_) => Err("Timeout waiting for ARP reply".to_string()),
    }
}