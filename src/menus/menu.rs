use std::fmt::Display;

/// Base trait for all menus in the application
pub trait Menu: Send {
    /// Display the menu to the user
    fn display(&self);
    
    /// Get the title of the menu
    fn title(&self) -> &str;
    
    /// Handle user input for this menu
    fn handle_input(&mut self, input: &str) -> MenuResult;
    
    /// Get available commands for this menu
    fn available_commands(&self) -> Vec<&str>;
}

/// Result type for menu operations
#[derive(Debug)]
pub enum MenuResult {
    /// Continue showing the current menu
    Continue,
    /// Exit the application
    Exit,
    /// Display an error message
    Error(String),
    /// Display a success message
    Success(String),
}

impl Display for MenuResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuResult::Continue => write!(f, "Continue"),
            MenuResult::Exit => write!(f, "Exit"),
            MenuResult::Error(msg) => write!(f, "Error: {}", msg),
            MenuResult::Success(msg) => write!(f, "Success: {}", msg),
        }
    }
}