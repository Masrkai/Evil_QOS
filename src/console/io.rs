use colored::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{self, Write};
use std::process::Command;

pub struct IO;

lazy_static! {
    static ref ANSI_CSI_RE: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
}

impl IO {
    // Mutable global flag to toggle color output
    pub static mut COLORLESS: bool = false;

    pub fn initialize(colorless: bool) {
        unsafe {
            IO::COLORLESS = colorless;
        }
    }

    pub fn print(text: &str, end: &str, flush: bool) {
        let output = if unsafe { IO::COLORLESS } {
            IO::remove_colors(text)
        } else {
            text.to_string()
        };

        print!("{}{}", output, end);
        if flush {
            io::stdout().flush().unwrap();
        }
    }

    pub fn ok(text: &str) {
        let prefix = format!("{}OK{}   ", Style::Bright.fg(Color::Green), Style::Reset);
        IO::print(&(prefix + text), "\n", false);
    }

    pub fn error(text: &str) {
        let prefix = format!("{}ERR{}  ", Style::Bright.fg(Color::Red), Style::Reset);
        IO::print(&(prefix + text), "\n", false);
    }

    pub fn spacer() {
        IO::print("", "\n", false);
    }

    pub fn input(prompt: &str) -> String {
        let prompt_text = if unsafe { IO::COLORLESS } {
            IO::remove_colors(prompt)
        } else {
            prompt.to_string()
        };

        print!("{}", prompt_text);
        io::stdout().flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        buffer.trim_end().to_string()
    }

    pub fn clear() {
        let _ = Command::new("clear").status();
    }

    fn remove_colors(text: &str) -> String {
        ANSI_CSI_RE.replace_all(text, "").to_string()
    }
}
