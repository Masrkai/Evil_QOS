/// Parses user input into commands and arguments
pub struct CommandParser;

impl CommandParser {
    /// Parse a command string into a command and its arguments
    pub fn parse(input: &str) -> (String, Vec<String>) {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        
        if parts.is_empty() {
            return (String::new(), Vec::new());
        }
        
        let command = parts[0].to_lowercase();
        let arguments = parts[1..].iter().map(|s| s.to_string()).collect();
        
        (command, arguments)
    }

    /// Parse a bandwidth limit expression into bytes per second
    pub fn parse_bandwidth_limit(input: &str) -> Option<u64> {
        let input = input.trim().to_lowercase();
        
        // Handle special cases
        if input == "none" || input == "0" {
            return Some(0);
        }
        
        if input == "full" || input == "unlimited" {
            return Some(u64::MAX);
        }
        
        // Parse numerical values with units
        let mut number_str = String::new();
        let mut unit_str = String::new();
        let mut in_number = true;
        
        for c in input.chars() {
            if in_number && (c.is_ascii_digit() || c == '.') {
                number_str.push(c);
            } else {
                in_number = false;
                if c.is_alphabetic() || c == '/' {
                    unit_str.push(c);
                }
            }
        }
        
        let number: f64 = number_str.parse().ok()?;
        let unit = unit_str.trim();
        
        // Convert to bytes per second
        let multiplier = match unit {
            "b" | "bps" => 1.0,
            "k" | "kb" | "kbit" | "kbits" => 1000.0,
            "kbps" => 1000.0,
            "m" | "mb" | "mbit" | "mbits" => 1_000_000.0,
            "mbps" => 1_000_000.0,
            "g" | "gb" | "gbit" | "gbits" => 1_000_000_000.0,
            "gbps" => 1_000_000_000.0,
            "b/s" => 1.0,
            "kb/s" => 1000.0,
            "mb/s" => 1_000_000.0,
            "gb/s" => 1_000_000_000.0,
            "bytes/s" => 8.0, // Convert bytes to bits
            "kbytes/s" => 8_000.0,
            "mbytes/s" => 8_000_000.0,
            "gbytes/s" => 8_000_000_000.0,
            _ => return None,
        };
        
        Some((number * multiplier) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bandwidth_limit() {
        assert_eq!(CommandParser::parse_bandwidth_limit("none"), Some(0));
        assert_eq!(CommandParser::parse_bandwidth_limit("0"), Some(0));
        assert_eq!(CommandParser::parse_bandwidth_limit("full"), Some(u64::MAX));
        assert_eq!(CommandParser::parse_bandwidth_limit("unlimited"), Some(u64::MAX));
        assert_eq!(CommandParser::parse_bandwidth_limit("10k"), Some(10_000));
        assert_eq!(CommandParser::parse_bandwidth_limit("10 kb"), Some(10_000));
        assert_eq!(CommandParser::parse_bandwidth_limit("10 kbps"), Some(10_000));
        assert_eq!(CommandParser::parse_bandwidth_limit("10 mb"), Some(10_000_000));
        assert_eq!(CommandParser::parse_bandwidth_limit("10 mbps"), Some(10_000_000));
        assert_eq!(CommandParser::parse_bandwidth_limit("10 mb/s"), Some(10_000_000));
        assert_eq!(CommandParser::parse_bandwidth_limit("10 mbytes/s"), Some(80_000_000));
        assert_eq!(CommandParser::parse_bandwidth_limit("invalid"), None);
    }
}