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
