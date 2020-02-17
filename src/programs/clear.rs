use crate::shell::{executable::Builtin, Arguments, Executables, Variables};
use crate::terminal::Terminal;

pub struct Clear;

impl Default for Clear {
    fn default() -> Self {
        Clear {}
    }
}

impl Builtin for Clear {
    fn run(
        &self,
        terminal: &Terminal,
        _: Option<&mut Executables>,
        _: Option<&mut Variables>,
        _: Option<Arguments>,
    ) -> u8 {
        terminal.clear();
        0
    }
}
