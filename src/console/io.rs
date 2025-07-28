use colored::*;
use regex::Regex;
use once_cell::sync::Lazy;

use std::process::Command;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// IO provides console input/output utilities with optional ANSI color support.
pub struct IO;

/// Global flag to enable/disable ANSI colors at runtime.
static COLORLESS: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Regex to strip ANSI escape sequences when colorless mode is active.
static ANSI_CSI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").expect("Failed to compile ANSI regex")
});

impl IO {
    /// Initializes the global color mode.
    ///
    /// When `colorless` is true, all ANSI sequences will be stripped from outputs.
    pub fn initialize(colorless: bool) {
        COLORLESS.store(colorless, Ordering::Relaxed);
    }

    /// Core print method handling optional color stripping and flush behavior.
    pub fn print(text: &str, end: &str, flush: bool) {
        let output = if COLORLESS.load(Ordering::Relaxed) {
            IO::remove_colors(text)
        } else {
            text.to_string()
        };

        print!("{}{}", output, end);
        if flush {
            io::stdout().flush().expect("Failed to flush stdout");
        }
    }

    /// Prints a success message prefixed with a green "OK" badge.
    pub fn ok(msg: &str) {
        let badge = "OK".bright_green().normal();
        IO::print(&format!("{}   {}", badge, msg), "\n", true);
    }

    /// Prints an error message prefixed with a red "ERR" badge.
    pub fn error(msg: &str) {
        let badge = "ERR".bright_red().normal();
        IO::print(&format!("{}  {}", badge, msg), "\n", true);
    }

    /// Prints an empty line.
    pub fn spacer() {
        IO::print("", "\n", true);
    }

    /// Prompts the user for input, returning the trimmed response.
    pub fn input(prompt: &str) -> String {
        let prompt_text = if COLORLESS.load(Ordering::Relaxed) {
            IO::remove_colors(prompt)
        } else {
            prompt.to_string()
        };

        print!("{}", prompt_text);
        io::stdout().flush().expect("Failed to flush stdout");

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Failed to read input");
        buffer.trim_end().to_string()
    }

    /// Clears the terminal using ANSI escape codes or fallback to `clear` command.
    pub fn clear() {
        // Try ANSI clear screen
        if !cfg!(target_os = "windows") {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().ok();
        } else {
            let _ = Command::new("cmd").args(&["/C", "cls"]).status();
        }
    }

    /// Removes ANSI escape sequences from the provided text.
    fn remove_colors(text: &str) -> String {
        ANSI_CSI_RE.replace_all(text, "").to_string()
    }
}
