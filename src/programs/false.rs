use crate::shell::{executable::Builtin, Arguments, EnvVars, Executables};
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
        _: Option<&mut EnvVars>,
        _: Option<Arguments>,
    ) -> u8 {
        1
    }
}
