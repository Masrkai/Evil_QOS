use std::collections::HashMap;
use crate::io::IO;

#[derive(Debug, PartialEq, Eq)]
pub enum CommandType {
    ParameterCommand,
    FlagCommand,
    ParameterizedFlagCommand,
}

#[derive(Debug)]
pub struct FlagCommand {
    pub kind: CommandType,
    pub identifier: String,
    pub name: String,
}

#[derive(Debug)]
pub struct ParameterCommand {
    pub kind: CommandType,
    pub name: String,
}

pub struct Subparser {
    pub identifier: String,
    pub parser: CommandParser,
    pub handler: Option<Box<dyn Fn(HashMap<String, Option<String>>)>>,
}

pub struct CommandParser {
    flag_commands: Vec<FlagCommand>,
    parameter_commands: Vec<ParameterCommand>,
    subparsers: Vec<Subparser>,
}

impl CommandParser {
    pub fn new() -> Self {
        CommandParser {
            flag_commands: Vec::new(),
            parameter_commands: Vec::new(),
            subparsers: Vec::new(),
        }
    }

    pub fn add_parameter(&mut self, name: &str) {
        self.parameter_commands.push(ParameterCommand {
            kind: CommandType::ParameterCommand,
            name: name.to_string(),
        });
    }

    pub fn add_flag(&mut self, identifier: &str, name: &str) {
        self.flag_commands.push(FlagCommand {
            kind: CommandType::FlagCommand,
            identifier: identifier.to_string(),
            name: name.to_string(),
        });
    }

    pub fn add_parameterized_flag(&mut self, identifier: &str, name: &str) {
        self.flag_commands.push(FlagCommand {
            kind: CommandType::ParameterizedFlagCommand,
            identifier: identifier.to_string(),
            name: name.to_string(),
        });
    }

    pub fn add_subparser<F>(&mut self, identifier: &str, handler: Option<F>) -> &mut CommandParser
    where
        F: Fn(HashMap<String, Option<String>>) + 'static,
    {
        let mut parser = CommandParser::new();
        let sp = Subparser {
            identifier: identifier.to_string(),
            parser,
            handler: handler.map(|f| Box::new(f) as Box<dyn Fn(HashMap<String, Option<String>>)>),
        };

        self.subparsers.push(sp);
        let last_index = self.subparsers.len() - 1;
        &mut self.subparsers[last_index].parser
    }

    pub fn parse(&self, command: Vec<String>) -> Option<HashMap<String, Option<String>>> {
        let mut result: HashMap<String, Option<String>> = self
            .flag_commands
            .iter()
            .map(|cmd| (cmd.name.clone(), None))
            .chain(
                self.parameter_commands
                    .iter()
                    .map(|cmd| (cmd.name.clone(), None)),
            )
            .collect();

        let mut skip_next = false;

        for (i, arg) in command.iter().enumerate() {
            if skip_next {
                skip_next = false;
                continue;
            }

            if i == 0 {
                for sp in &self.subparsers {
                    if sp.identifier == *arg {
                        let sub_args = command[i + 1..].to_vec();
                        let parsed = sp.parser.parse(sub_args)?;
                        if let Some(handler) = &sp.handler {
                            handler(parsed.clone());
                        }
                        return Some(parsed);
                    }
                }
            }

            let mut is_processed = false;

            for cmd in &self.flag_commands {
                if cmd.identifier == *arg {
                    match cmd.kind {
                        CommandType::FlagCommand => {
                            result.insert(cmd.name.clone(), Some("true".to_string()));
                            is_processed = true;
                            break;
                        }
                        CommandType::ParameterizedFlagCommand => {
                            if i + 1 >= command.len() {
                                IO::error(&format!(
                                    "parameter for flag {}{}{} is missing",
                                    IO::Fore::LIGHTYELLOW_EX,
                                    cmd.name,
                                    IO::Style::RESET_ALL
                                ));
                                return None;
                            }
                            result.insert(cmd.name.clone(), Some(command[i + 1].clone()));
                            skip_next = true;
                            is_processed = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }

            if !is_processed {
                for cmd in &self.parameter_commands {
                    if result.get(&cmd.name).unwrap().is_none() {
                        result.insert(cmd.name.clone(), Some(arg.clone()));
                        is_processed = true;
                        break;
                    }
                }
            }

            if !is_processed {
                IO::error(&format!(
                    "{}{}{} is an unknown command.",
                    IO::Fore::LIGHTYELLOW_EX,
                    arg,
                    IO::Style::RESET_ALL
                ));
                return None;
            }
        }

        for cmd in &self.parameter_commands {
            if result.get(&cmd.name).unwrap().is_none() {
                IO::error(&format!(
                    "parameter {}{}{} is missing",
                    IO::Fore::LIGHTYELLOW_EX,
                    cmd.name,
                    IO::Style::RESET_ALL
                ));
                return None;
            }
        }

        for cmd in &self.flag_commands {
            if cmd.kind == CommandType::FlagCommand && result.get(&cmd.name).unwrap().is_none() {
                result.insert(cmd.name.clone(), Some("false".to_string()));
            }
        }

        Some(result)
    }
}
