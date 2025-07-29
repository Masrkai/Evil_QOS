use once_cell::sync::Lazy;
use crate::console::shell::Shell;

pub static BROADCAST: &str = "ff:ff:ff:ff:ff:ff";
pub static IP_FORWARD_LOC: &str = "net.ipv4.ip_forward";

// Use `Lazy` so the shell lookup only happens once and on demand
pub static BIN_TC: Lazy<Option<String>> = Lazy::new(|| Shell::locate_bin("tc"));
pub static BIN_IPTABLES: Lazy<Option<String>> = Lazy::new(|| Shell::locate_bin("iptables"));
pub static BIN_SYSCTL: Lazy<Option<String>> = Lazy::new(|| Shell::locate_bin("sysctl"));
