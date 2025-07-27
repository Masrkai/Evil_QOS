// //! Evil QoS - Network Traffic Shaping Tool

// mod menus;
// mod common;
// mod console;
// mod networking;
mod check_sudo;

use std::process;
// use menus::{MainMenu, CommandParser};
// use console::shell::InteractiveShell;
// use console::{display_banner, clear_screen};


fn main() {
    let is_running_as_root = check_sudo::is_root();


    if !is_running_as_root {
        eprintln!("Error: This program requires root privileges.");
        eprintln!("Please run with: sudo ./Program");
        process::exit(1);
    } else {
        println!("Running with root privileges!");

    }

    // // Or get it as integer (1/0)
    // let root_int = check_sudo::bool_to_int(is_running_as_root);
    // println!("Root status as int: {}", root_int);
}