use std::process;
use std::sync::Arc;
use clap::{Arg, Command};
use std::collections::HashMap;

// Import all the modules
mod menus;
mod common;
mod console;
mod networking;

// Use the public exports
use console::{IO, display_banner, Shell};
use networking::{Host, discover_hosts};
use networking::limit::{Limiter, Direction};
use networking::spoof::ArpSpoofer;
use networking::monitor::BandwidthMonitor;
use networking::utils::*;
use menus::{CommandMenu, CommandParser};

#[tokio::main]
async fn main() {
    let matches = Command::new("Evil QoS")
        .version("1.0.0")
        .about("Network Traffic Shaping Tool")
        .author("Evil QoS Team")
        .arg(
            Arg::new("interface")
                .short('i')
                .long("interface")
                .value_name("INTERFACE")
                .help("Network interface to use")
                .required(false),
        )
        .arg(
            Arg::new("gateway")
                .short('g')
                .long("gateway")
                .value_name("GATEWAY_IP")
                .help("Gateway IP address")
                .required(false),
        )
        .arg(
            Arg::new("range")
                .short('r')
                .long("range")
                .value_name("IP_RANGE")
                .help("IP range to scan (e.g., 192.168.1.0/24)")
                .required(false),
        )
        .arg(
            Arg::new("colorless")
                .long("no-color")
                .help("Disable colored output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Initialize IO with color settings
    let colorless = matches.get_flag("colorless");
    IO::initialize(colorless);

    // Display banner
    display_banner();
    IO::spacer();

    // Check if running as root
    if std::env::var("USER").unwrap_or_default() != "root" && 
       std::env::var("SUDO_USER").is_err() {
        IO::error("This application requires root privileges to function properly.");
        IO::error("Please run with sudo or as root user.");
        process::exit(1);
    }

    // Get network interface
    let interface = match matches.get_one::<String>("interface") {
        Some(iface) => {
            if !exists_interface(iface) {
                IO::error(&format!("Interface '{}' not found", iface));
                process::exit(1);
            }
            iface.clone()
        }
        None => {
            match get_default_interface() {
                Some(iface) => {
                    IO::ok(&format!("Using default interface: {}", iface));
                    iface
                }
                None => {
                    IO::error("No suitable network interface found");
                    process::exit(1);
                }
            }
        }
    };

    // Get gateway IP
    let gateway_ip = match matches.get_one::<String>("gateway") {
        Some(gw) => {
            if !validate_ip_address(gw) {
                IO::error(&format!("Invalid gateway IP address: {}", gw));
                process::exit(1);
            }
            gw.clone()
        }
        None => {
            match get_default_gateway() {
                Some(gw) => {
                    IO::ok(&format!("Using default gateway: {}", gw));
                    gw
                }
                None => {
                    IO::error("No default gateway found");
                    process::exit(1);
                }
            }
        }
    };

    // Generate IP range if not provided
    let ip_range = match matches.get_one::<String>("range") {
        Some(range) => generate_ip_range(range),
        None => {
            // Generate range based on interface
            let netmask = get_default_netmask(&interface).unwrap_or_else(|| {
                IO::error("Could not determine network range");
                process::exit(1);
            });
            generate_ip_range_from_interface(&interface, &netmask)
        }
    };

    IO::ok(&format!("Network interface: {}", interface));
    IO::ok(&format!("Gateway IP: {}", gateway_ip));
    IO::ok(&format!("Scanning {} IP addresses", ip_range.len()));
    IO::spacer();

    // Initialize network settings
    flush_network_settings(&interface);
    
    if !enable_ip_forwarding() {
        IO::error("Failed to enable IP forwarding");
        process::exit(1);
    }

    if !create_qdisc_root(&interface) {
        IO::error("Failed to create traffic control root qdisc");
        process::exit(1);
    }

    IO::ok("Network settings initialized");
    IO::spacer();

    // Initialize components
    let limiter = Arc::new(Limiter::new(&interface));
    let monitor = Arc::new(BandwidthMonitor::new(&interface));
    
    // Get gateway MAC address (simplified - in real implementation you'd need ARP resolution)
    let gateway_mac = get_mac_by_ip(&interface, &gateway_ip)
        .unwrap_or_else(|| "00:00:00:00:00:00".to_string());
    
    let spoofer = Arc::new(ArpSpoofer::new(&interface, &gateway_ip, &gateway_mac));

    // Start the interactive menu
    start_interactive_menu(
        interface,
        gateway_ip,
        ip_range,
        limiter,
        monitor,
        spoofer,
    ).await;
}

async fn start_interactive_menu(
    interface: String,
    gateway_ip: String,
    ip_range: Vec<String>,
    limiter: Arc<Limiter>,
    monitor: Arc<BandwidthMonitor>,
    spoofer: Arc<ArpSpoofer>,
) {
    let mut hosts: Vec<Host> = Vec::new();
    
    // Set up command parser
    let mut parser = CommandParser::new();
    
    // Add main commands
    parser.add_parameter("command");
    
    // Scan command
    parser.add_subparser("scan", Some(|_args| {
        println!("Scanning for hosts...");
    }));
    
    // Limit command
    let limit_parser = parser.add_subparser("limit", None);
    limit_parser.add_parameter("host_id");
    limit_parser.add_parameterized_flag("-r", "rate");
    limit_parser.add_parameterized_flag("-d", "direction");
    
    // Block command
    let block_parser = parser.add_subparser("block", None);
    block_parser.add_parameter("host_id");
    block_parser.add_parameterized_flag("-d", "direction");
    
    // Unlimit command
    let unlimit_parser = parser.add_subparser("unlimit", None);
    unlimit_parser.add_parameter("host_id");
    
    // List command
    parser.add_subparser("list", Some(|_args| {
        println!("Listing hosts...");
    }));
    
    // Monitor command
    parser.add_subparser("monitor", Some(|_args| {
        println!("Starting bandwidth monitor...");
    }));
    
    // Spoof command
    parser.add_subparser("spoof", Some(|_args| {
        println!("Starting ARP spoofing...");
    }));
    
    // Help command
    parser.add_subparser("help", Some(|_args| {
        print_help();
    }));
    
    // Exit command
    parser.add_subparser("exit", Some(|_args| {
        std::process::exit(0);
    }));

    IO::ok("Evil QoS ready. Type 'help' for available commands.");
    IO::spacer();

    // Main command loop
    loop {
        match IO::input("evil-qos> ") {
            Ok(input) => {
                let args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
                
                if args.is_empty() {
                    continue;
                }
                
                match args[0].as_str() {
                    "scan" => {
                        IO::print("Scanning for hosts...", "\n", true);
                        match discover_hosts(&interface, ip_range.clone()).await {
                            Ok(discovered) => {
                                hosts = discovered;
                                IO::ok(&format!("Found {} hosts", hosts.len()));
                                for (i, host) in hosts.iter().enumerate() {
                                    println!("  [{}] {} ({}) - {}", i, host.ip, host.mac, 
                                           if host.name.is_empty() { "Unknown" } else { &host.name });
                                }
                            }
                            Err(e) => {
                                IO::error(&format!("Scan failed: {}", e));
                            }
                        }
                    }
                    "list" => {
                        if hosts.is_empty() {
                            IO::error("No hosts found. Run 'scan' first.");
                        } else {
                            println!("Discovered hosts:");
                            for (i, host) in hosts.iter().enumerate() {
                                println!("  [{}] {} ({}) - {} [{}]", 
                                       i, host.ip, host.mac, 
                                       if host.name.is_empty() { "Unknown" } else { &host.name },
                                       host.pretty_status());
                            }
                        }
                    }
                    "limit" => {
                        if args.len() < 4 {
                            IO::error("Usage: limit <host_id> -r <rate> [-d <direction>]");
                            continue;
                        }
                        
                        if let Ok(host_id) = args[1].parse::<usize>() {
                            if host_id < hosts.len() {
                                let rate_str = &args[3];
                                let direction = if args.len() > 5 && args[4] == "-d" {
                                    match args[5].as_str() {
                                        "upload" => Direction::Outgoing,
                                        "download" => Direction::Incoming,
                                        "both" => Direction::Both,
                                        _ => Direction::Both,
                                    }
                                } else {
                                    Direction::Both
                                };
                                
                                if let Ok(rate) = rate_str.parse::<u32>() {
                                    limiter.limit(&mut hosts[host_id], direction, rate);
                                    IO::ok(&format!("Limited {} to {} ({}) ", 
                                                   hosts[host_id].ip, rate_str, 
                                                   Direction::pretty_direction(direction)));
                                } else {
                                    IO::error("Invalid rate format");
                                }
                            } else {
                                IO::error("Invalid host ID");
                            }
                        } else {
                            IO::error("Invalid host ID");
                        }
                    }
                    "block" => {
                        if args.len() < 2 {
                            IO::error("Usage: block <host_id> [-d <direction>]");
                            continue;
                        }
                        
                        if let Ok(host_id) = args[1].parse::<usize>() {
                            if host_id < hosts.len() {
                                let direction = if args.len() > 3 && args[2] == "-d" {
                                    match args[3].as_str() {
                                        "upload" => Direction::Outgoing,
                                        "download" => Direction::Incoming,
                                        "both" => Direction::Both,
                                        _ => Direction::Both,
                                    }
                                } else {
                                    Direction::Both
                                };
                                
                                limiter.block(&mut hosts[host_id], direction);
                                IO::ok(&format!("Blocked {} ({})", 
                                               hosts[host_id].ip, 
                                               Direction::pretty_direction(direction)));
                            } else {
                                IO::error("Invalid host ID");
                            }
                        } else {
                            IO::error("Invalid host ID");
                        }
                    }
                    "unlimit" => {
                        if args.len() < 2 {
                            IO::error("Usage: unlimit <host_id>");
                            continue;
                        }
                        
                        if let Ok(host_id) = args[1].parse::<usize>() {
                            if host_id < hosts.len() {
                                limiter.unlimit(&mut hosts[host_id], Direction::Both);
                                IO::ok(&format!("Unlimited {}", hosts[host_id].ip));
                            } else {
                                IO::error("Invalid host ID");
                            }
                        } else {
                            IO::error("Invalid host ID");
                        }
                    }
                    "monitor" => {
                        if hosts.is_empty() {
                            IO::error("No hosts to monitor. Run 'scan' first.");
                        } else {
                            IO::ok("Starting bandwidth monitoring...");
                            for host in &hosts {
                                monitor.add(host.clone());
                            }
                            monitor.start().await;
                        }
                    }
                    "spoof" => {
                        if hosts.is_empty() {
                            IO::error("No hosts to spoof. Run 'scan' first.");
                        } else {
                            IO::ok("Starting ARP spoofing...");
                            for host in &hosts {
                                spoofer.add(host.clone());
                            }
                            spoofer.start();
                        }
                    }
                    "clear" => {
                        IO::clear();
                        display_banner();
                    }
                    "help" => {
                        print_help();
                    }
                    "exit" | "quit" => {
                        cleanup(&interface, &hosts, &spoofer, &monitor);
                        IO::ok("Goodbye!");
                        break;
                    }
                    _ => {
                        IO::error(&format!("Unknown command: {}. Type 'help' for available commands.", args[0]));
                    }
                }
            }
            Err(_) => {
                // Ctrl+C or EOF
                cleanup(&interface, &hosts, &spoofer, &monitor);
                IO::ok("Goodbye!");
                break;
            }
        }
        
        IO::spacer();
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  scan                           - Scan for hosts on the network");
    println!("  list                           - List discovered hosts");
    println!("  limit <id> -r <rate> [-d dir] - Limit bandwidth for a host");
    println!("                                   rate: bandwidth in kbit/mbit");
    println!("                                   dir: upload|download|both (default: both)");
    println!("  block <id> [-d dir]            - Block traffic for a host");
    println!("  unlimit <id>                   - Remove bandwidth limits for a host");
    println!("  monitor                        - Start bandwidth monitoring");
    println!("  spoof                          - Start ARP spoofing");
    println!("  clear                          - Clear screen");
    println!("  help                           - Show this help");
    println!("  exit/quit                      - Exit the application");
    println!();
    println!("Examples:");
    println!("  limit 0 -r 100kbit -d upload  - Limit host 0 to 100kbit upload");
    println!("  block 1 -d both               - Block all traffic for host 1");
    println!("  unlimit 0                      - Remove all limits from host 0");
}

fn cleanup(interface: &str, hosts: &[Host], spoofer: &ArpSpoofer, monitor: &BandwidthMonitor) {
    IO::print("Cleaning up...", "\n", true);
    
    // Stop monitoring and spoofing
    monitor.stop();
    spoofer.stop();
    
    // Restore network settings
    flush_network_settings(interface);
    disable_ip_forwarding();
    delete_qdisc_root(interface);
    
    IO::ok("Cleanup completed");
}

fn generate_ip_range(range: &str) -> Vec<String> {
    // Simple CIDR parsing - in a real implementation you'd want more robust parsing
    if let Some((network, prefix)) = range.split_once('/') {
        let prefix_len: u8 = prefix.parse().unwrap_or(24);
        let base_ip: std::net::Ipv4Addr = network.parse().unwrap_or_else(|_| {
            eprintln!("Invalid network address: {}", network);
            process::exit(1);
        });
        
        let host_bits = 32 - prefix_len;
        let num_hosts = (1u32 << host_bits) - 2; // Exclude network and broadcast
        
        let base = u32::from(base_ip);
        let mut ips = Vec::new();
        
        for i in 1..=num_hosts {
            let ip = std::net::Ipv4Addr::from(base + i);
            ips.push(ip.to_string());
        }
        
        ips
    } else {
        // Single IP
        vec![range.to_string()]
    }
}

fn generate_ip_range_from_interface(interface: &str, netmask: &str) -> Vec<String> {
    // Simplified - in real implementation you'd calculate based on interface IP and netmask
    // For now, assume common /24 network
    vec![]
}