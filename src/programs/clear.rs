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
