use crate::shell::{executable::Builtin, Arguments, Executables, Variables};
use crate::terminal::Terminal;

pub struct True;

impl Default for True {
    fn default() -> Self {
        True {}
    }
}

impl Builtin for True {
    fn run(
        &self,
        _: &Terminal,
        _: Option<&mut Executables>,
        _: Option<&mut Variables>,
        _: Option<Arguments>,
    ) -> u8 {
        0
    }
}
