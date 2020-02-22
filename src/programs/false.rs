use crate::shell::{executable::Builtin, Arguments, Executables, Vars};
use crate::terminal::Terminal;

pub struct False;

impl Default for False {
    fn default() -> Self {
        False {}
    }
}

impl Builtin for False {
    fn run(&self, _: &Terminal, _: &mut Executables, _: &mut Vars, _: Arguments) -> u8 {
        1
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

        let program = False::default();
        assert_eq!(
            1,
            program.run(&terminal, &mut executables, &mut globals, arguments)
        );
    }
}
