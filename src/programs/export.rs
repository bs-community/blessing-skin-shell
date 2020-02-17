use crate::shell::{executable::Builtin, Argument, Arguments, Executables, Variables};
use crate::terminal::Terminal;
use ansi_term::Color;

pub struct Export;

impl Export {
    fn print_warning(&self, terminal: &Terminal, message: String) {
        terminal.write(&format!("{}\n", Color::Yellow.paint(message)));
    }
}

impl Default for Export {
    fn default() -> Self {
        Export {}
    }
}

impl Builtin for Export {
    fn run(
        &self,
        terminal: &Terminal,
        _: Option<&mut Executables>,
        globals: Option<&mut Variables>,
        arguments: Option<Arguments>,
    ) -> u8 {
        match globals {
            Some(globals) => {
                if let Some(arguments) = arguments {
                    for argument in arguments {
                        match argument {
                            Argument::Text(text) => {
                                let mut parts = text.splitn(2, '=');
                                let name = parts.next().unwrap_or_default();
                                if name == "" {
                                    self.print_warning(
                                        terminal,
                                        "Variable name isn't provided.".to_string(),
                                    );
                                    return 1;
                                }
                                match parts.next() {
                                    Some(value) => {
                                        if value == "" {
                                            self.print_warning(
                                                terminal,
                                                "Variable name isn't provided.".to_string(),
                                            );
                                            return 1;
                                        } else {
                                            globals.insert(name.to_string(), value.to_string());
                                        }
                                    }
                                    None => {
                                        self.print_warning(
                                            terminal,
                                            "Missing variable value.".to_string(),
                                        );
                                        return 1;
                                    }
                                }
                            }
                            Argument::Switch(key, _) => {
                                self.print_warning(terminal, format!("Invalid argument: {}", key));
                            }
                        }
                    }
                }
                0
            }
            None => 1,
        }
    }
}
