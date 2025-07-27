use nix::unistd::Uid;

/// Check if the current process is running as root
/// Returns true if running as root (UID 0), false otherwise
pub fn is_root() -> bool {
    Uid::effective().is_root()
}

/// Alternative: Check if the current process has root privileges
/// This checks the real UID instead of effective UID
pub fn is_real_root() -> bool {
    Uid::current().is_root()
}

/// Convert boolean to integer (1 for true, 0 for false)
pub fn bool_to_int(value: bool) -> u8 {
    if value { 1 } else { 0 }
}



// If you need an async version
pub fn ensure_root() -> bool {
    let is_root = Uid::effective().is_root();
    
    if is_root {
        println!("✅ Running as root (sudo)");
    } else {
        println!("❌ Not running as root");
    }
    
    is_root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_to_int() {
        assert_eq!(bool_to_int(true), 1);
        assert_eq!(bool_to_int(false), 0);
    }

    #[test]
    fn test_is_root_returns_bool() {
        let result = is_root();
        // Should return either true or false, never panic
        assert!(result == true || result == false);
    }
}