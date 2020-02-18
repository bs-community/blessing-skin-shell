use super::{Arguments, EnvVars, Executables};
use crate::terminal::Terminal;

#[allow(dead_code)]
pub enum Program {
    Builtin(Box<dyn Builtin>),
    Internal(Box<dyn Internal>),
    External(Box<dyn External>),
}

pub trait Builtin {
    fn run(
        &self,
        terminal: &Terminal,
        executables: &mut Executables,
        globals: &mut EnvVars,
        arguments: Option<Arguments>,
    ) -> u8;
}

pub trait Internal {
    fn run(&self, arguments: Option<Arguments>) -> u8;
}

pub trait External {
    fn run(&self, arguments: Vec<String>) -> u8;
}
