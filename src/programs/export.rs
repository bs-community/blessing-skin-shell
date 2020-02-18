use crate::shell::{executable::Builtin, Argument, Arguments, Executables, Vars};
use crate::terminal::Terminal;
use ansi_term::Color;

pub struct Export;

impl Export {
    fn print_warning(&self, terminal: &Terminal, message: String) {
        terminal.write(&format!("{}\r\n", Color::Yellow.paint(message)));
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
        _: &mut Executables,
        globals: &mut Vars,
        arguments: Arguments,
    ) -> u8 {
        arguments.iter().fold(0, |exit_code, argument| {
            let e = match argument {
                Argument::Text(text) => {
                    let mut parts = text.splitn(2, '=');
                    let name = parts.next().unwrap_or_default();
                    if name == "" {
                        self.print_warning(terminal, "Variable name isn't provided.".to_string());
                        return exit_code + 1;
                    }
                    match parts.next() {
                        Some(value) => {
                            if value == "" {
                                self.print_warning(
                                    terminal,
                                    "Variable value isn't provided.".to_string(),
                                );
                                1
                            } else {
                                globals.insert(name.to_string(), value.to_string());
                                0
                            }
                        }
                        None => {
                            self.print_warning(terminal, "Missing variable value.".to_string());
                            1
                        }
                    }
                }
                Argument::Switch(key, _) => {
                    self.print_warning(terminal, format!("Invalid argument: {}", key));
                    0
                }
            };
            exit_code + e
        })
    }
}
