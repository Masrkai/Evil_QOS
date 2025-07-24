use super::menu::{Menu, MenuResult};
use crate::networking::{Host, NetworkScanner};
use std::collections::HashMap;

/// Main menu of the application
pub struct MainMenu {
    hosts: Vec<Host>,
    selected_hosts: HashMap<String, usize>, // IP -> index in hosts
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            hosts: Vec::new(),
            selected_hosts: HashMap::new(),
        }
    }

    /// Scan for hosts on the network
    fn scan_network(&mut self) {
        match NetworkScanner::new() {
            Ok(scanner) => {
                println!("Scanning network...");
                self.hosts = scanner.discover_hosts();
                println!("Found {} hosts", self.hosts.len());
            }
            Err(e) => {
                eprintln!("Failed to initialize network scanner: {}", e);
            }
        }
    }

    /// Display all discovered hosts
    fn display_hosts(&self) {
        if self.hosts.is_empty() {
            println!("No hosts discovered. Run scan first.");
            return;
        }

        println!("\nDiscovered Hosts:");
        println!("{:-<60}", "");
        println!("{:<15} {:<17} {:<20}", "IP Address", "MAC Address", "Hostname");
        println!("{:-<60}", "");

        for (index, host) in self.hosts.iter().enumerate() {
            let selected = if self.selected_hosts.contains_key(&host.ip) {
                "*"
            } else {
                " "
            };
            println!(
                "{}{:<14} {:<17} {:<20}",
                selected,
                host.ip,
                host.mac.as_deref().unwrap_or("Unknown"),
                host.hostname.as_deref().unwrap_or("Unknown")
            );
        }
        println!("{:-<60}\n", "");
    }

    /// Select a host by IP
    fn select_host(&mut self, ip: &str) {
        for (index, host) in self.hosts.iter().enumerate() {
            if host.ip == ip {
                self.selected_hosts.insert(ip.to_string(), index);
                println!("Selected host: {}", ip);
                return;
            }
        }
        println!("Host with IP {} not found", ip);
    }

    /// Deselect a host by IP
    fn deselect_host(&mut self, ip: &str) {
        if self.selected_hosts.remove(ip).is_some() {
            println!("Deselected host: {}", ip);
        } else {
            println!("Host with IP {} was not selected", ip);
        }
    }

    /// Select all hosts
    fn select_all_hosts(&mut self) {
        for (index, host) in self.hosts.iter().enumerate() {
            self.selected_hosts
                .insert(host.ip.clone(), index);
        }
        println!("Selected all {} hosts", self.selected_hosts.len());
    }

    /// Clear all selections
    fn clear_selections(&mut self) {
        self.selected_hosts.clear();
        println!("Cleared all selections");
    }

    /// Apply bandwidth limit to selected hosts
    fn apply_bandwidth_limit(&self, limit: &str) {
        if self.selected_hosts.is_empty() {
            println!("No hosts selected. Use 'select' command first.");
            return;
        }

        println!("Applying bandwidth limit '{}' to {} hosts...", 
                 limit, self.selected_hosts.len());
        
        // In a real implementation, this would call the networking module
        // to apply the actual bandwidth limits
        for ip in self.selected_hosts.keys() {
            println!("  - Applied to {}", ip);
        }
    }
}

impl Menu for MainMenu {
    fn display(&self) {
        println!("\n{:=<50}", "");
        println!("{}{:^48}{}", "|", "Evil QoS - Main Menu", "|");
        println!("{:=<50}", "");
        println!("Commands:");
        println!("  scan                  - Scan network for hosts");
        println!("  show                  - Show discovered hosts");
        println!("  select <ip>           - Select a host by IP");
        println!("  deselect <ip>         - Deselect a host by IP");
        println!("  select-all            - Select all discovered hosts");
        println!("  clear                 - Clear all selections");
        println!("  limit <value>         - Apply bandwidth limit to selected hosts");
        println!("  help                  - Show this help message");
        println!("  exit                  - Exit the application");
        println!("{:=<50}\n", "");
    }

    fn title(&self) -> &str {
        "Main Menu"
    }

    fn handle_input(&mut self, input: &str) -> MenuResult {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return MenuResult::Continue;
        }

        match parts[0].to_lowercase().as_str() {
            "scan" => {
                self.scan_network();
                MenuResult::Continue
            }
            "show" => {
                self.display_hosts();
                MenuResult::Continue
            }
            "select" => {
                if parts.len() < 2 {
                    return MenuResult::Error("Usage: select <ip>".to_string());
                }
                self.select_host(parts[1]);
                MenuResult::Continue
            }
            "deselect" => {
                if parts.len() < 2 {
                    return MenuResult::Error("Usage: deselect <ip>".to_string());
                }
                self.deselect_host(parts[1]);
                MenuResult::Continue
            }
            "select-all" => {
                self.select_all_hosts();
                MenuResult::Continue
            }
            "clear" => {
                self.clear_selections();
                MenuResult::Continue
            }
            "limit" => {
                if parts.len() < 2 {
                    return MenuResult::Error("Usage: limit <value>".to_string());
                }
                let limit = parts[1..].join(" ");
                self.apply_bandwidth_limit(&limit);
                MenuResult::Continue
            }
            "help" => {
                self.display();
                MenuResult::Continue
            }
            "exit" => MenuResult::Exit,
            _ => MenuResult::Error(format!("Unknown command: {}", parts[0])),
        }
    }

    fn available_commands(&self) -> Vec<&str> {
        vec![
            "scan",
            "show",
            "select",
            "deselect",
            "select-all",
            "clear",
            "limit",
            "help",
            "exit",
        ]
    }
}