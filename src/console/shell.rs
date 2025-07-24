use crate::console::{clear_screen, get_user_input, display_banner};
use crate::menus::{MainMenu, Menu, MenuResult};

/// Interactive shell for the application
pub struct InteractiveShell {
    running: bool,
}

impl InteractiveShell {
    /// Create a new interactive shell
    pub fn new() -> Self {
        Self { running: true }
    }

    /// Start the interactive shell
    pub fn start(&mut self) {
        clear_screen();
        
        // Display banner
        display_banner();
        
        // Initialize main menu
        let mut menu = MainMenu::new();
        menu.display();

        while self.running {
            let input = get_user_input("> ");
            
            if input.is_empty() {
                continue;
            }

            match menu.handle_input(&input) {
                MenuResult::Continue => {
                    // Continue with the same menu
                }
                MenuResult::Exit => {
                    self.running = false;
                    println!("Exiting Evil QoS. Goodbye!");
                }
                MenuResult::Error(msg) => {
                    eprintln!("Error: {}", msg);
                }
                MenuResult::Success(msg) => {
                    println!("Success: {}", msg);
                }
            }
        }
    }

    /// Stop the interactive shell
    pub fn stop(&mut self) {
        self.running = false;
    }
}