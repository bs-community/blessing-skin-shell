use crate::shell::{executable::Builtin, Argument, Arguments, Executables, Vars};
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
        _: &mut Vars,
        arguments: Arguments,
    ) -> u8 {
        arguments.iter().for_each(|argument| {
            match argument {
                Argument::Text(value) => terminal.write(&value),
                Argument::Switch(key, value) => {
                    terminal.write(key);
                    terminal.write("=");
                    terminal.write(value.as_deref().unwrap_or(""));
                }
            }
            terminal.write(" ");
        });
        terminal.write("\r\n");
        0
    }
}
