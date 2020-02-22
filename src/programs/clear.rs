use crate::shell::{executable::Builtin, Arguments, Executables, Vars};
use crate::terminal::Terminal;

pub struct Clear;

impl Default for Clear {
    fn default() -> Self {
        Clear {}
    }
}

impl Builtin for Clear {
    fn run(&self, terminal: &Terminal, _: &mut Executables, _: &mut Vars, _: Arguments) -> u8 {
        terminal.clear();
        0
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

        terminal.write("text");
        assert_eq!("text", &terminal.get());

        let program = Clear::default();
        assert_eq!(
            0,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
        assert_eq!("", &terminal.get());
    }
}
