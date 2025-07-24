/// Display the application banner
pub fn display_banner() {
    println!("{}", get_banner());
}

/// Get the banner string without printing it
pub fn get_banner() -> String {
    format!(
        r#"
  _______ _______ ___ ___ _______ _______ _______ _______ 
 |   _   |   _   |   Y   |   _   |   _   |   _   |   _   |
 |.  |   |.  |   |.  |   |.  |   |.  |   |.  |   |.  1   |
 |.  |   |.  |   |.  |   |.  |   |.  |   |.  |   |.  _   |
 |:  1   |:  1   |:  |   |:  1   |:  1   |:  1   |:  |   |
 |::.. . |::.. . |::.|:. |::.. . |::.. . |::.. . |::.|:. |
 `-------'`-------'--- ---'`-------'`-------'`-------'`---'
 
        Evil QoS - Network Traffic Shaping Tool
                   Version 1.0.0
"#
    )
}