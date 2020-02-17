use crate::shell::{executable::Builtin, Arguments, Executables, Variables};
use crate::terminal::Terminal;

pub struct False;

impl Default for False {
    fn default() -> Self {
        False {}
    }
}

impl Builtin for False {
    fn run(
        &self,
        _: &Terminal,
        _: Option<&mut Executables>,
        _: Option<&mut Variables>,
        _: Option<Arguments>,
    ) -> u8 {
        1
    }
}
