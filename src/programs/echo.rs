use crate::shell::{executable::Builtin, Argument, Arguments, EnvVars, Executables};
use crate::terminal::Terminal;

pub struct Echo;

impl Default for Echo {
    fn default() -> Self {
        Echo {}
    }
}

impl Builtin for Echo {
    fn run(
        &self,
        terminal: &Terminal,
        _: &mut Executables,
        _: &mut EnvVars,
        arguments: Option<Arguments>,
    ) -> u8 {
        if let Some(arguments) = arguments {
            for argument in &arguments {
                match argument {
                    Argument::Text(value) => terminal.write(&value),
                    Argument::Switch(key, value) => {
                        terminal.write(key);
                        terminal.write("=");
                        terminal.write(value.as_deref().unwrap_or(""));
                    }
                }
                terminal.write(" ");
            }
            terminal.write("\r\n");
        }
        0
    }
}
