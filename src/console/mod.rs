// Declare submodules - tells Rust these files are modules
pub mod io;
pub mod chart;
pub mod shell;
pub mod banner;

// Re-export items that actually exist in your modules
// This allows users to write `use console::display_banner` instead of `use console::banner::display_banner`

// From banner.rs - both functions exist
pub use banner::{display_banner, get_banner};

// From chart.rs - re-export the main struct and related types
pub use chart::{BarChart, BarValue};

// From io.rs - re-export the main struct (IO has static methods)
pub use io::IO;

// From shell.rs - re-export the main struct
pub use shell::Shell;

// Optional: Create convenience functions that wrap the actual implementations
// This provides a cleaner API for common operations

// /// Convenience function to print success messages
// pub fn print_success(msg: &str) {
//     IO::ok(msg);
// }

// /// Convenience function to print error messages  
// pub fn print_error(msg: &str) {
//     IO::error(msg);
// }

// /// Convenience function to print info messages (regular print)
// pub fn print_info(msg: &str) {
//     IO::print(msg, "\n", true);
// }

// /// Convenience function to get user input
// pub fn get_user_input(prompt: &str) -> String {
//     IO::input(prompt)
// }

// /// Convenience function to clear screen
// pub fn clear_screen() {
//     IO::clear();
// }

// /// Convenience function to create and display a host chart
// pub fn display_host_chart(data: Vec<(f64, String, String)>, reverse: bool) -> String {
//     let mut chart = BarChart::default();
    
//     for (value, prefix, suffix) in data {
//         chart.add_value(value, &prefix, &suffix);
//     }
    
//     chart.get(reverse)
// }