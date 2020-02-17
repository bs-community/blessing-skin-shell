use crate::shell::{executable::Builtin, Arguments, EnvVars, Executables};
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
        _: Option<&mut EnvVars>,
        _: Option<Arguments>,
    ) -> u8 {
        0
    }
}
