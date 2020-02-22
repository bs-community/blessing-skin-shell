use crate::shell::{executable::Builtin, Arguments, Executables, Vars};
use crate::terminal::Terminal;

pub struct True;

impl Default for True {
    fn default() -> Self {
        True {}
    }
}

impl Builtin for True {
    fn run(&self, _: &Terminal, _: &mut Executables, _: &mut Vars, _: Arguments) -> u8 {
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

        let program = True::default();
        assert_eq!(
            0,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
    }
}
