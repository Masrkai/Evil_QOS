use std::io::{self, Write};

/// Clear the terminal screen
pub fn clear_screen() {
    // ANSI escape code to clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Get input from the user
pub fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

/// Print an informational message
pub fn print_info(message: &str) {
    println!("[*] {}", message);
}

/// Print a success message
pub fn print_success(message: &str) {
    println!("[+] {}", message);
}

/// Print an error message
pub fn print_error(message: &str) {
    eprintln!("[-] {}", message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("[!] {}", message);
}

/// Display a progress bar
pub fn display_progress(current: usize, total: usize, label: &str) {
    let percentage = if total > 0 {
        (current as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    
    let bar_width = 40;
    let filled = (percentage / 100.0 * bar_width as f64) as usize;
    let empty = bar_width - filled;
    
    let bar: String = std::iter::repeat('=')
        .take(filled)
        .chain(std::iter::repeat(' '))
        .take(bar_width)
        .collect();
    
    print!("\r{} [{}] {:.1}%", label, bar, percentage);
    io::stdout().flush().unwrap();
}

/// Finish the progress bar display
pub fn finish_progress(label: &str) {
    println!("\r{} [========================================] 100.0%", label);
}