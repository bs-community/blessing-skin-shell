pub(crate) mod executable;
mod history;
mod renderer;
pub(crate) mod transform;

use crate::parser::{self, ast::Command};
use crate::programs;
use crate::terminal::Terminal;
use crate::utils;
use ansi_term::Color;
use executable::Program;
use history::History;
use std::collections::HashMap;
pub use transform::Argument;
use transform::Transformer;
use wasm_bindgen::prelude::*;

pub type Executables = HashMap<String, Box<dyn Fn() -> Program>>;
pub type Vars = HashMap<String, String>;
pub type Arguments = Vec<transform::Argument>;

#[wasm_bindgen]
pub struct Shell {
    line: String,
    line_cursor: usize,
    terminal: Terminal,
    executables: Executables,
    globals: Vars,
    history: History,
    suggestion: Option<String>,
}

#[wasm_bindgen]
impl Shell {
    #[wasm_bindgen(constructor)]
    pub fn new(terminal: Terminal) -> Shell {
        let mut executables: HashMap<String, Box<dyn Fn() -> Program>> = HashMap::with_capacity(10);
        executables.insert(
            "clear".to_string(),
            Box::new(|| Program::Builtin(Box::new(programs::Clear::default()))),
        );
        executables.insert(
            "echo".to_string(),
            Box::new(|| Program::Builtin(Box::new(programs::Echo::default()))),
        );
        executables.insert(
            "true".to_string(),
            Box::new(|| Program::Builtin(Box::new(programs::True::default()))),
        );
        executables.insert(
            "false".to_string(),
            Box::new(|| Program::Builtin(Box::new(programs::False::default()))),
        );
        executables.insert(
            "export".to_string(),
            Box::new(|| Program::Builtin(Box::new(programs::Export::default()))),
        );

        let mut globals = HashMap::with_capacity(3);
        globals.insert("?".to_string(), "0".to_string());

        let shell = Shell {
            terminal,
            line: String::with_capacity(12),
            line_cursor: 0,
            executables,
            globals,
            history: History::new(),
            suggestion: None,
        };

        let greet = Color::Fixed(127)
            .paint("Welcome to Blessing Skin Shell!")
            .to_string();
        shell.print(&greet);
        shell.new_line();
        shell.new_line();
        shell.prompt();

        shell
    }

    pub fn input(&mut self, data: &str) {
        utils::set_panic_hook();

        let line_len = self.line.len();

        match data.as_bytes() {
            // line break
            [10] | [13] => {
                self.commit();
            }
            // the key "Esc", do nothing
            [27] => {}
            // backspace
            [127] => {
                if self.line_cursor > 0 {
                    self.line_cursor -= 1;
                    self.line.remove(self.line_cursor);
                }
            }
            // arrow left
            [27, 91, 68] => {
                if self.line_cursor > 0 {
                    self.line_cursor -= 1;
                }
            }
            // arrow right
            [27, 91, 67] => {
                if line_len > 0 && self.line_cursor < line_len {
                    self.line_cursor += 1;
                } else if let Some(suggestion) = &self.suggestion {
                    self.line.push_str(&*suggestion);
                    self.line_cursor += suggestion.len();
                    self.suggestion = None;
                }
            }
            // arrow up
            [27, 91, 65] => {
                if let Some(history) = self.history.up() {
                    self.line_cursor = history.len();
                    self.line = history;
                }
            }
            // arrow down
            [27, 91, 66] => match self.history.down() {
                Some(history) => {
                    self.line_cursor = history.len();
                    self.line = history;
                }
                None => {
                    self.line.clear();
                    self.line_cursor = 0;
                }
            },
            // the key "Delete"
            [27, 91, 51, 126] => {
                if self.line_cursor < line_len {
                    self.line.remove(self.line_cursor);
                }
            }
            // the key "Home"
            [27, 91, 72] => {
                self.line_cursor = 0;
            }
            // the key "End"
            [27, 91, 70] => {
                self.line_cursor = line_len;
            }
            // double quote
            [34] => {
                self.insert_text("\"\"");
                self.line_cursor += 1;
            }
            // single quote
            [39] => {
                self.insert_text("''");
                self.line_cursor += 1;
            }
            _ => {
                self.insert_text(data);
                self.line_cursor += data.len();
            }
        }

        self.output();
    }

    fn insert_text(&mut self, text: &str) {
        if self.line.len() == self.line_cursor {
            self.line.push_str(text);
        } else {
            self.line.insert_str(self.line_cursor, text);
        }
    }

    fn output(&mut self) {
        // Move cursor to left edge
        self.print("\u{001b}[1000D");
        // Clear line
        self.print("\u{001b}[0K");

        // Write line
        self.prompt();
        match parser::parse(&self.line) {
            Ok((command, rest)) => {
                self.render_command(command);
                self.print(rest);
            }
            Err(_) => {
                self.print(&self.line);
            }
        }

        if self.line.is_empty() {
            self.suggestion = None;
        } else if let Some(history) = self.history.find(&self.line) {
            let rest = history.trim_start_matches(&self.line);
            self.print(&Color::Fixed(8).paint(rest).to_string());
            self.suggestion = Some(rest.to_string());
        }

        // Move cursor to left edge again
        self.print("\u{001b}[1000D");
        // Move cursor to current position
        self.print(&format!("\u{001b}[{}C", self.line_cursor + 2));
    }

    fn render_command(&self, command: Command) {
        self.print(&renderer::command(&command, &self.executables));
        if command.parameters.is_none() {
            self.white_space(self.line_cursor - command.program.span.end.index);
        }
    }

    fn commit(&mut self) {
        // If we're going to clear screen, don't send new line.
        if !self.line.starts_with("clear") {
            self.new_line();
        }

        if &self.line != "" {
            self.history.commit(self.line.clone());

            match parser::parse(&self.line) {
                Ok((command, _)) => {
                    self.run_command(command);
                }
                Err(e) => {
                    self.println(&format!("bsh: syntax error: {}", e));
                }
            }
        }

        self.line.clear();
        self.line_cursor = 0;
    }

    pub(crate) fn print(&self, data: &str) {
        self.terminal.write(data);
    }

    pub(crate) fn println(&self, data: &str) {
        self.terminal.write(data);
        self.new_line();
    }

    fn prompt(&self) {
        self.print(Color::Purple.paint("❯ ").to_string().as_ref());
    }

    pub(crate) fn new_line(&self) {
        self.print("\r\n");
    }

    fn white_space(&self, size: usize) {
        self.print(&" ".repeat(size));
    }

    fn run_command(&mut self, command: Command) {
        let name = &command.program.id.name;
        let program = self.executables.get(name).map(|getter| getter());
        match program {
            Some(program) => {
                let exit_code = {
                    match program {
                        Program::Builtin(program) => {
                            let transformer = Transformer::new(&self.globals, false);
                            let arguments = command
                                .parameters
                                .map(|p| transformer.transform(p))
                                .unwrap_or_default();
                            program.run(
                                &self.terminal,
                                &mut self.executables,
                                &mut self.globals,
                                arguments,
                            )
                        }
                        Program::Internal(_) => 0,
                        Program::External(_) => 0,
                    }
                };
                if let Some(var) = self.globals.get_mut("?") {
                    *var = format!("{}", exit_code);
                }
            }
            None => {
                self.println(&format!(
                    "bsh: command not found: {}",
                    Color::Red.paint(name)
                ));
                if let Some(var) = self.globals.get_mut("?") {
                    *var = String::from("127");
                }
            }
        }
    }
}
