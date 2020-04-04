mod buffer;
pub(crate) mod executable;
mod history;
mod renderer;
pub(crate) mod transform;

use crate::parser::{self, ast::Command};
use crate::programs;
use crate::stdio::Stdio;
use crate::terminal::Terminal;
use crate::utils;
use ansi_term::Color;
use buffer::Buffer;
use executable::{Program, Runner};
use history::History;
use js_sys::Function;
use std::collections::HashMap;
use std::rc::Rc;
pub use transform::Argument;
use wasm_bindgen::prelude::*;

pub type Executables = HashMap<String, Program>;
pub type Vars = HashMap<String, String>;
pub type Arguments = Vec<transform::Argument>;

#[wasm_bindgen]
pub struct Shell {
    buffer: Buffer,
    terminal: Rc<Terminal>,
    executables: Executables,
    globals: Vars,
    history: History,
    suggestion: Option<String>,
    runner: Runner,
    stdio: Rc<Stdio>,
}

#[wasm_bindgen]
impl Shell {
    #[wasm_bindgen(constructor)]
    pub fn new(terminal: Terminal) -> Shell {
        let mut executables = HashMap::with_capacity(10);
        executables.insert(
            "clear".to_string(),
            Program::Builtin(Box::new(|| Box::new(programs::Clear::default()))),
        );
        executables.insert(
            "echo".to_string(),
            Program::Builtin(Box::new(|| Box::new(programs::Echo::default()))),
        );
        executables.insert(
            "export".to_string(),
            Program::Builtin(Box::new(|| Box::new(programs::Export::default()))),
        );
        executables.insert(
            "curl".to_string(),
            Program::Internal(Box::new(|| Box::new(programs::Curl::default()))),
        );

        let terminal = Rc::new(terminal);
        let stdio = Rc::new(Stdio::new(Rc::clone(&terminal)));

        let shell = Shell {
            terminal,
            buffer: Buffer::new(),
            executables,
            globals: HashMap::with_capacity(3),
            history: History::new(),
            suggestion: None,
            runner: Runner::new(),
            stdio,
        };

        let greet = Color::Fixed(127)
            .paint("Welcome to Blessing Skin Shell!\r\n")
            .to_string();
        shell.stdio.println(&greet);
        shell.stdio.prompt();

        shell
    }

    /// Send input data to the Shell.
    pub fn input(&mut self, data: &str) {
        utils::set_panic_hook();

        if self.runner.is_running() {
            return;
        }

        match data.as_bytes() {
            // line break
            [10] | [13] => {
                self.stdio.reset();
                self.stdio.prompt();
                match parser::parse_interactive(self.buffer.get()) {
                    Ok((command, rest)) => {
                        self.render_command(command);
                        self.stdio.print(rest);
                    }
                    Err(_) => {
                        self.stdio.print(self.buffer.get());
                    }
                }

                self.commit();
            }
            // the key "Esc", do nothing
            [27] => {}
            // backspace
            [127] => {
                self.buffer.delete_left();
            }
            // arrow left
            [27, 91, 68] => {
                self.buffer.move_left();
            }
            // arrow right
            [27, 91, 67] => {
                if !self.buffer.is_empty() && self.buffer.get_cursor() < self.buffer.len() {
                    self.buffer.move_right();
                } else if let Some(suggestion) = &self.suggestion {
                    self.buffer.insert(&suggestion);
                    self.suggestion = None;
                }
            }
            // arrow up
            [27, 91, 65] => {
                if let Some(history) = self.history.up() {
                    self.buffer.set(history);
                }
            }
            // arrow down
            [27, 91, 66] => match self.history.down() {
                Some(history) => {
                    self.buffer.set(history);
                }
                None => {
                    self.buffer.clear();
                }
            },
            // the key "Delete"
            [27, 91, 51, 126] => {
                self.buffer.delete_right();
            }
            // the key "Home"
            [27, 91, 72] => {
                self.buffer.move_to_start();
            }
            // the key "End"
            [27, 91, 70] => {
                self.buffer.move_to_end();
            }
            // double quote
            [34] => {
                self.buffer.insert_without_moving("\"\"");
                self.buffer.move_right();
            }
            // single quote
            [39] => {
                self.buffer.insert_without_moving("''");
                self.buffer.move_right();
            }
            _ => {
                self.buffer.insert(data);
            }
        }

        if !self.runner.is_running() {
            self.output();
        }
    }

    fn output(&mut self) {
        self.stdio.reset();

        // Write line
        self.stdio.prompt();
        match parser::parse_interactive(self.buffer.get()) {
            Ok((command, rest)) => {
                self.render_command(command);
                self.stdio.print(rest);
            }
            Err(_) => {
                self.stdio.print(self.buffer.get());
            }
        }

        if self.buffer.is_empty() {
            self.suggestion = None;
        } else if let Some(history) = self.history.find(self.buffer.get()) {
            let rest = history.trim_start_matches(self.buffer.get());
            self.stdio.print(&Color::Fixed(8).paint(rest).to_string());
            self.suggestion = Some(rest.to_string());
        }

        // Move cursor to left edge again
        self.stdio.print("\u{001b}[1000D");
        // Move cursor to current position
        self.stdio
            .print(&format!("\u{001b}[{}C", self.buffer.get_cursor() + 2));
    }

    fn render_command(&self, command: Command) {
        self.stdio
            .print(&renderer::command(&command, &self.executables));
        if command.parameters.is_none() {
            self.white_space(self.buffer.len() - command.program.span.end.index);
        }
    }

    fn commit(&mut self) {
        // If we're going to clear screen, don't send new line.
        if !self.buffer.get().starts_with("clear") {
            self.stdio.println("");
        }

        if !self.buffer.is_empty() {
            self.history.commit(self.buffer.get().to_string());

            match parser::parse_interactive(&self.buffer.get()) {
                Ok((command, _)) => {
                    self.run_command(command);
                }
                Err(err) => {
                    let unexpected = err
                        .errors
                        .iter()
                        .take(1)
                        .map(|e| {
                            let mut msg = format!("{}", e);
                            msg.make_ascii_lowercase();
                            msg
                        })
                        .fold(String::new(), |output, msg| output + &msg);
                    self.stdio.println(&format!(
                        "bsh: syntax error at {}, {}",
                        err.position, unexpected
                    ));
                }
            }
        }

        self.buffer.clear();
    }

    fn white_space(&self, size: usize) {
        self.stdio.print(&" ".repeat(size));
    }

    fn run_command(&mut self, command: Command) {
        let name = &command.program.id.name;
        let program = self.executables.get(name);
        match program {
            Some(program) => match program {
                Program::Builtin(program) => {
                    self.runner.run_builtin(
                        program(),
                        command.parameters,
                        &Rc::clone(&self.terminal),
                        &mut self.executables,
                        &mut self.globals,
                    );
                }
                Program::Internal(program) => {
                    self.runner.run_internal(
                        program(),
                        command.parameters,
                        &self.globals,
                        Rc::clone(&self.stdio),
                    );
                }
                Program::External(program) => {
                    self.runner.run_external(
                        &program,
                        command.parameters,
                        Rc::clone(&self.terminal),
                        &self.globals,
                        Rc::clone(&self.stdio),
                    );
                }
            },
            None => {
                self.stdio.println(&format!(
                    "bsh: command not found: {}",
                    Color::Red.paint(name)
                ));
            }
        }
    }

    #[wasm_bindgen(js_name = "addExternal")]
    /// Register a new external JavaScript function.
    pub fn add_external(&mut self, name: String, func: Function) {
        let external = Program::External(executable::External::new(func));
        self.executables.insert(name, external);
    }
}
