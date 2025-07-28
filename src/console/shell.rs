use crate::IO;
use std::io::{self};
use std::process::{Command, Stdio};

pub struct Shell;

impl Shell {
    pub fn check_doas_sudo() -> String {
        match Command::new("which")
            .arg("doas")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) if status.success() => "doas ".to_string(),
            _ => "sudo ".to_string(),
        }
    }

    pub fn execute(command: &str, root: bool) -> i32 {
        let full_cmd = if root {
            format!("{}{}", Shell::check_doas_sudo(), command)
        } else {
            command.to_string()
        };

        Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1)
    }

    pub fn execute_suppressed(command: &str, root: bool) -> i32 {
        let full_cmd = if root {
            format!("{}{}", Shell::check_doas_sudo(), command)
        } else {
            command.to_string()
        };

        Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1)
    }

    pub fn output(command: &str, root: bool) -> Option<String> {
        let full_cmd = if root {
            format!("{}{}", Shell::check_doas_sudo(), command)
        } else {
            command.to_string()
        };

        Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
    }

    pub fn output_suppressed(command: &str, root: bool) -> Option<String> {
        let full_cmd = if root {
            format!("{}{}", Shell::check_doas_sudo(), command)
        } else {
            command.to_string()
        };

        Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .stderr(Stdio::null())
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
    }

    pub fn locate_bin(name: &str) -> Option<String> {
        match Shell::output_suppressed(&format!("which {}", name), true) {
            Some(path) if !path.trim().is_empty() => Some(path.trim().to_string()),
            _ => {
                IO::error(&format!("missing util: {}, check your PATH", name));
                None
            }
        }
    }
}
