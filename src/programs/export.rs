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
                        self.print_warning(terminal, "Missing variable name.".to_string());
                        return exit_code + 1;
                    }
                    match parts.next() {
                        Some(value) => {
                            globals.insert(name.to_string(), value.to_string());
                            0
                        }
                        None => {
                            self.print_warning(terminal, "Missing variable value.".to_string());
                            1
                        }
                    }
                }
                Argument::Switch(key, _) => {
                    self.print_warning(terminal, format!("Invalid argument: {}", key));
                    1
                }
            };
            exit_code + e
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn no_arguments() {
        let terminal = Terminal::new();
        let mut executables = HashMap::new();
        let mut globals = HashMap::new();
        let arguments = vec![];

        let program = Export::default();
        assert_eq!(
            0,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
    }

    #[test]
    fn pass_a_switch() {
        let terminal = Terminal::new();
        let mut executables = HashMap::new();
        let mut globals = HashMap::new();
        let arguments = vec![Argument::Switch("s".to_string(), None)];

        let program = Export::default();
        assert_eq!(
            1,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
        assert!(terminal.get().contains("Invalid argument: s"));
    }

    #[test]
    fn missing_name() {
        let terminal = Terminal::new();
        let mut executables = HashMap::new();
        let mut globals = HashMap::new();
        let arguments = vec![Argument::Text("".to_string())];

        let program = Export::default();
        assert_eq!(
            1,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
        assert!(terminal.get().contains("Missing variable name."));
    }

    #[test]
    fn missing_value() {
        let terminal = Terminal::new();
        let mut executables = HashMap::new();
        let mut globals = HashMap::new();
        let arguments = vec![Argument::Text("k".to_string())];

        let program = Export::default();
        assert_eq!(
            1,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
        assert!(terminal.get().contains("Missing variable value."));
    }

    #[test]
    fn allow_empty() {
        let terminal = Terminal::new();
        let mut executables = HashMap::new();
        let mut globals = HashMap::new();
        let arguments = vec![Argument::Text("k=".to_string())];

        let program = Export::default();
        assert_eq!(
            0,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
        assert_eq!(Some(&"".to_string()), globals.get("k"));
    }

    #[test]
    fn insert() {
        let terminal = Terminal::new();
        let mut executables = HashMap::new();
        let mut globals = HashMap::new();
        let arguments = vec![Argument::Text("k=v1".to_string())];

        let program = Export::default();
        assert_eq!(
            0,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
        assert_eq!(Some(&"v1".to_string()), globals.get("k"));

        let arguments = vec![Argument::Text("k=v2".to_string())];
        assert_eq!(
            0,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
        assert_eq!(Some(&"v2".to_string()), globals.get("k"));
    }
}
