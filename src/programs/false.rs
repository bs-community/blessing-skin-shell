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
