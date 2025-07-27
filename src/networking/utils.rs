use std::net::Ipv4Addr;
use std::process::Command;
use std::str;
use regex::Regex;
use pnet::datalink;
use crate::shell::Shell;
use crate::global::{BIN_IPTABLES, BIN_TC, BIN_SYSCTL, IP_FORWARD_LOC};

pub fn get_default_interface() -> Option<String> {
    datalink::interfaces().into_iter().find(|iface| iface.is_up() && !iface.ips.is_empty()).map(|iface| iface.name)
}

pub fn get_default_gateway() -> Option<String> {
    Shell::output("ip route show default", false)
        .and_then(|output| output.split_whitespace().nth(2).map(|s| s.to_string()))
}

pub fn get_default_netmask(interface: &str) -> Option<String> {
    datalink::interfaces().into_iter()
        .find(|iface| iface.name == interface)
        .and_then(|iface| iface.ips.iter().find(|ip| ip.is_ipv4()).map(|ip| ip.netmask().to_string()))
}

pub fn exists_interface(interface: &str) -> bool {
    datalink::interfaces().iter().any(|iface| iface.name == interface)
}

pub fn flush_network_settings(interface: &str) {
    if let Some(ref bin_iptables) = *BIN_IPTABLES {
        Shell::execute_suppressed(&format!("{} -P INPUT ACCEPT", bin_iptables), true);
        Shell::execute_suppressed(&format!("{} -P OUTPUT ACCEPT", bin_iptables), true);
        Shell::execute_suppressed(&format!("{} -P FORWARD ACCEPT", bin_iptables), true);
        Shell::execute_suppressed(&format!("{} -t mangle -F", bin_iptables), true);
        Shell::execute_suppressed(&format!("{} -t nat -F", bin_iptables), true);
        Shell::execute_suppressed(&format!("{} -F", bin_iptables), true);
        Shell::execute_suppressed(&format!("{} -X", bin_iptables), true);
    }

    if let Some(ref bin_tc) = *BIN_TC {
        Shell::execute_suppressed(&format!("{} qdisc del dev {} root", bin_tc, interface), true);
    }
}

pub fn validate_ip_address(ip: &str) -> bool {
    Regex::new(r"^(\d{1,3}\.){3}(\d{1,3})$").unwrap().is_match(ip)
}

pub fn validate_mac_address(mac: &str) -> bool {
    Regex::new(r"^([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}$").unwrap().is_match(mac)
}

pub fn create_qdisc_root(interface: &str) -> bool {
    if let Some(ref bin_tc) = *BIN_TC {
        return Shell::execute_suppressed(&format!("{} qdisc add dev {} root handle 1:0 htb", bin_tc, interface), true) == 0;
    }
    false
}

pub fn delete_qdisc_root(interface: &str) -> i32 {
    if let Some(ref bin_tc) = *BIN_TC {
        return Shell::execute_suppressed(&format!("{} qdisc del dev {} root handle 1:0 htb", bin_tc, interface), true);
    }
    1
}

pub fn enable_ip_forwarding() -> bool {
    if let Some(ref bin_sysctl) = *BIN_SYSCTL {
        return Shell::execute_suppressed(&format!("{} -w {}=1", bin_sysctl, IP_FORWARD_LOC), true) == 0;
    }
    false
}

pub fn disable_ip_forwarding() -> bool {
    if let Some(ref bin_sysctl) = *BIN_SYSCTL {
        return Shell::execute_suppressed(&format!("{} -w {}=0", bin_sysctl, IP_FORWARD_LOC), true) == 0;
    }
    false
}

pub struct ValueConverter;

impl ValueConverter {
    pub fn byte_to_bit(v: u64) -> u64 {
        v * 8
    }
}

#[derive(Debug, Clone)]
pub struct BitRate {
    pub rate: u64,
}

impl BitRate {
    pub fn new(rate: u64) -> Self {
        BitRate { rate }
    }

    pub fn from_rate_string(rate_string: &str) -> Self {
        BitRate { rate: Self::bit_value(rate_string) }
    }

    fn bit_value(rate_string: &str) -> u64 {
        let mut number = 0u64;
        let mut offset = 0;

        for c in rate_string.chars() {
            if c.is_digit(10) {
                number = number * 10 + c.to_digit(10).unwrap() as u64;
                offset += 1;
            } else {
                break;
            }
        }

        let unit = &rate_string[offset..].to_lowercase();

        match unit.as_str() {
            "bit" => number,
            "kbit" => number * 1000,
            "mbit" => number * 1000_u64.pow(2),
            "gbit" => number * 1000_u64.pow(3),
            _ => panic!("Invalid bitrate"),
        }
    }

    pub fn fmt(&self, fmt: &str) -> String {
        let s = self.to_string();
        let num_len = s.chars().take_while(|c| c.is_digit(10)).count();
        let (num, unit) = s.split_at(num_len);
        format!("{}{}", format!(fmt, num.parse::<u64>().unwrap()), unit)
    }
}

impl std::fmt::Display for BitRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = self.rate as f64;
        let mut counter = 0;
        while r >= 1000.0 {
            r /= 1000.0;
            counter += 1;
            if counter > 3 {
                return Err(std::fmt::Error);
            }
        }
        let unit = match counter {
            0 => "bit",
            1 => "kbit",
            2 => "mbit",
            3 => "gbit",
            _ => return Err(std::fmt::Error),
        };
        write!(f, "{}{}", r as u64, unit)
    }
}

#[derive(Debug, Clone)]
pub struct ByteValue {
    pub value: u64,
}

impl ByteValue {
    pub fn new(value: u64) -> Self {
        ByteValue { value }
    }

    pub fn from_byte_string(byte_string: &str) -> Self {
        ByteValue { value: Self::byte_value(byte_string) }
    }

    fn byte_value(byte_string: &str) -> u64 {
        let mut number = 0u64;
        let mut offset = 0;

        for c in byte_string.chars() {
            if c.is_digit(10) {
                number = number * 10 + c.to_digit(10).unwrap() as u64;
                offset += 1;
            } else {
                break;
            }
        }

        let unit = &byte_string[offset..].to_lowercase();

        match unit.as_str() {
            "b" => number,
            "kb" => number * 1024,
            "mb" => number * 1024_u64.pow(2),
            "gb" => number * 1024_u64.pow(3),
            "tb" => number * 1024_u64.pow(4),
            _ => panic!("Invalid byte string"),
        }
    }

    pub fn fmt(&self, fmt: &str) -> String {
        let s = self.to_string();
        let num_len = s.chars().take_while(|c| c.is_digit(10)).count();
        let (num, unit) = s.split_at(num_len);
        format!("{}{}", format!(fmt, num.parse::<u64>().unwrap()), unit)
    }
}

impl std::fmt::Display for ByteValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = self.value as f64;
        let mut counter = 0;
        while v >= 1024.0 {
            v /= 1024.0;
            counter += 1;
            if counter > 4 {
                return Err(std::fmt::Error);
            }
        }
        let unit = match counter {
            0 => "b",
            1 => "kb",
            2 => "mb",
            3 => "gb",
            4 => "tb",
            _ => return Err(std::fmt::Error),
        };
        write!(f, "{}{}", v as u64, unit)
    }
}

use pnet::datalink::NetworkInterface;

pub fn get_mac_by_ip(interface: &str, ip: &str) -> Option<String> {
    // Would require raw packet crafting similar to scapy
    // Placeholder: shell-based ARP query (not equivalent but okay for fallback logic)
    Shell::output(&format!("arp -an | grep {} | grep {}", ip, interface), false)
        .and_then(|out| {
            let parts: Vec<&str> = out.split_whitespace().collect();
            parts.get(3).map(|s| s.trim_matches('(').trim_matches(')').to_string())
        })
}
