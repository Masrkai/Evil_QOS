//! Entry point: Translated from Python's evillimiter main

mod cli;
mod utils;
mod menus;
mod banner;
mod console;
mod networking;

use crate::banner::get_main_banner;
use crate::cli::parser::parse_arguments;
use crate::console::io::IO;
use crate::menus::main_menu::MainMenu;
use crate::networking::utils as netutils;

use std::path::PathBuf;
use std::process;
use std::fs;
use std::env;

fn get_init_content() -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/init.rs");
    fs::read_to_string(path).expect("Could not read init.rs")
}

fn get_version() -> String {
    let content = get_init_content();
    let version_regex = regex::Regex::new(r"__version__\s*=\s*\"(\d+\.\d+\.\d+)\"").unwrap();
    version_regex
        .captures(&content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| panic!("Unable to locate version string."))
}

fn get_description() -> String {
    let content = get_init_content();
    let desc_regex = regex::Regex::new(r"__description__\s*=\s*\"(.+?)\"").unwrap();
    desc_regex
        .captures(&content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| panic!("Unable to locate description string."))
}

fn is_privileged() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

fn initialize(interface: &str) -> bool {
    if !netutils::create_qdisc_root(interface) {
        IO::spacer();
        IO::error("qdisc root handle could not be created. maybe flush network settings (--flush).", None);
        return false;
    }

    if !netutils::enable_ip_forwarding() {
        IO::spacer();
        IO::error("ip forwarding could not be enabled.", None);
        return false;
    }

    true
}

fn cleanup(interface: &str) {
    netutils::delete_qdisc_root(interface);
    netutils::disable_ip_forwarding();
}

fn run() {
    let version = get_version();
    let args = parse_arguments(&get_description());

    IO::initialize(args.colorless);
    IO::print(&get_main_banner(&version));

    if !is_linux() {
        IO::error("run under linux.", None);
        return;
    }

    if !is_privileged() {
        IO::error("run as root.", None);
        return;
    }

    let args = match cli::process_arguments(&args) {
        Some(val) => val,
        None => return,
    };

    if initialize(&args.interface) {
        IO::spacer();
        let mut menu = MainMenu::new(
            &version,
            &args.interface,
            &args.gateway_ip,
            &args.gateway_mac,
            &args.netmask,
        );
        menu.start();
        cleanup(&args.interface);
    }
}

fn main() {
    run();
}
