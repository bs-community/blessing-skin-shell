use crate::shell::{executable::Builtin, Argument, Arguments, Executables, Vars};
use crate::terminal::Terminal;

pub struct Echo;

impl Default for Echo {
    fn default() -> Self {
        Echo {}
    }
}

impl Builtin for Echo {
    fn run(&self, terminal: &Terminal, _: &mut Executables, _: &mut Vars, arguments: Arguments) {
        arguments.iter().for_each(|argument| {
            match argument {
                Argument::Text(value) => terminal.write(&value),
                Argument::Switch(key, value) => {
                    terminal.write(key);
                    if let Some(value) = value {
                        terminal.write("=");
                        terminal.write(&value);
                    }
                }
            }
            terminal.write(" ");
        });
        terminal.write("\r\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn run() {
        let terminal = Terminal::new();
        let mut executables = HashMap::new();
        let mut globals = HashMap::new();
        let arguments = vec![];

        let program = Echo::default();
        program.run(&terminal, &mut executables, &mut globals, arguments);
        assert_eq!("\r\n", &terminal.get());

        terminal.clear();

        let arguments = vec![
            Argument::Text("text".to_string()),
            Argument::Switch("switch".to_string(), None),
            Argument::Switch("key".to_string(), Some("value".to_string())),
        ];
        program.run(&terminal, &mut executables, &mut globals, arguments);
        assert_eq!("text switch key=value \r\n", &terminal.get());
    }
}
