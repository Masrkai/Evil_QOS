use crate::io::IO;
use crate::parser::CommandParser;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::io;

pub struct CommandMenu {
    prompt: String,
    parser: CommandParser,
    active: Arc<AtomicBool>,
}

impl CommandMenu {
    pub fn new() -> Self {
        CommandMenu {
            prompt: ">>> ".to_string(),
            parser: CommandParser::new(),
            active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn argument_handler(&self, _args: std::collections::HashMap<String, Option<String>>) {
        // Placeholder: user-defined command logic
    }

    pub fn interrupt_handler(&self) {
        self.stop();
    }

    pub fn start(&mut self) {
        let running = Arc::clone(&self.active);
        running.store(true, Ordering::SeqCst);

        let ctrlc_flag = Arc::clone(&self.active);
        let handler_self = self.clone_for_handler();

        ctrlc::set_handler(move || {
            ctrlc_flag.store(false, Ordering::SeqCst);
            handler_self.interrupt_handler();
        })
        .expect("Error setting Ctrl-C handler");

        while running.load(Ordering::SeqCst) {
            let input = match IO::input(&self.prompt) {
                Ok(line) => line,
                Err(_) => {
                    self.interrupt_handler();
                    break;
                }
            };

            let args: Vec<String> = input
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            if let Some(parsed) = self.parser.parse(args) {
                self.argument_handler(parsed);
            }
        }
    }

    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    // Needed to pass `self` into `ctrlc::set_handler`
    fn clone_for_handler(&self) -> Self {
        CommandMenu {
            prompt: self.prompt.clone(),
            parser: CommandParser::new(), // Empty parser, real one not used in handler
            active: Arc::clone(&self.active),
        }
    }
}
