use crate::io::IO;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct Host {
    pub ip: String,
    pub mac: String,
    pub name: String,
    pub spoofed: bool,
    pub limited: bool,
    pub blocked: bool,
    pub watched: bool,
}

impl Host {
    pub fn new(ip: &str, mac: &str, name: &str) -> Self {
        Host {
            ip: ip.to_string(),
            mac: mac.to_string(),
            name: name.to_string(),
            spoofed: false,
            limited: false,
            blocked: false,
            watched: false,
        }
    }

    pub fn pretty_status(&self) -> String {
        if self.limited {
            format!(
                "{}Limited{}",
                IO::Fore::LIGHTRED_EX,
                IO::Style::RESET_ALL
            )
        } else if self.blocked {
            format!("{}Blocked{}", IO::Fore::RED, IO::Style::RESET_ALL)
        } else {
            "Free".to_string()
        }
    }
}

// Implement PartialEq for comparison
impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
    }
}

// Implement Eq because we implement PartialEq
impl Eq for Host {}

// Implement Hash for use in HashMaps/Sets
impl Hash for Host {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        self.mac.hash(state);
    }
}

// Optional: Implement Debug or Display for pretty-printing
impl fmt::Debug for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Host(ip={}, mac={}, name={}, status={})",
            self.ip,
            self.mac,
            self.name,
            self.pretty_status()
        )
    }
}
