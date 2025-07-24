//! Evil QoS - Network Traffic Shaping Tool

mod common;
mod console;
mod menus;
mod networking;

use console::{display_banner, clear_screen};
use console::shell::InteractiveShell;
use menus::{MainMenu, CommandParser};

/// Main entry point for the application
fn main() {
    // Setup panic handler for better error messages
    setup_panic_handler();
    
    // For now, always start interactive mode
    run_interactive_mode();
}

/// Run the application in interactive TUI mode
fn run_interactive_mode() {
    // Check for root privileges (required for packet manipulation)
    if !is_root() {
        eprintln!("{}{}", common::globals::COLOR_RED, common::globals::ERROR_PERMISSION_DENIED);
        eprintln!("{}Please run with sudo privileges.", common::globals::COLOR_RESET);
        std::process::exit(1);
    }
    
    // Display banner
    clear_screen();
    display_banner();
    
    // Start interactive shell
    let mut shell = InteractiveShell::new();
    shell.start();
}

/// Check if the application is running with root privileges
fn is_root() -> bool {
    unsafe { Uid::effective().is_root() == 0 }
}

/// Setup custom panic handler for better error messages
fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!(
            "{}{}An unexpected error occurred:{}",
            common::globals::COLOR_RED,
            common::globals::COLOR_BOLD,
            common::globals::COLOR_RESET
        );
        
        eprintln!(
            "{}Please report this issue with the following information:{}",
            common::globals::COLOR_YELLOW,
            common::globals::COLOR_RESET
        );
        
        eprintln!("{}", panic_info);
    }));
}