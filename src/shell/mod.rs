mod buffer;
pub(crate) mod executable;
mod history;
mod renderer;
pub(crate) mod transform;

use crate::parser::{self, ast::Command};
use crate::programs;
use crate::terminal::Terminal;
use crate::utils;
use ansi_term::Color;
use buffer::Buffer;
use executable::Program;
use futures::channel::oneshot::channel;
use history::History;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
pub use transform::Argument;
use transform::Transformer;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub type Executables = HashMap<String, Box<dyn Fn() -> Program>>;
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
    running: Rc<Cell<bool>>,
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
        executables.insert(
            "curl".to_string(),
            Box::new(|| Program::Internal(Box::new(programs::Curl::default()))),
        );

        let mut globals = HashMap::with_capacity(3);
        globals.insert("?".to_string(), "0".to_string());

        let shell = Shell {
            terminal: Rc::new(terminal),
            buffer: Buffer::new(),
            executables,
            globals,
            history: History::new(),
            suggestion: None,
            running: Rc::new(Cell::new(false)),
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

        match data.as_bytes() {
            // line break
            [10] | [13] => {
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

        if !&self.running.get() {
            self.output();
        }
    }

    fn output(&mut self) {
        // Move cursor to left edge
        self.print("\u{001b}[1000D");
        // Clear line
        self.print("\u{001b}[0K");

        // Write line
        self.prompt();
        match parser::parse(self.buffer.get()) {
            Ok((command, rest)) => {
                self.render_command(command);
                self.print(rest);
            }
            Err(_) => {
                self.print(self.buffer.get());
            }
        }

        if self.buffer.is_empty() {
            self.suggestion = None;
        } else if let Some(history) = self.history.find(self.buffer.get()) {
            let rest = history.trim_start_matches(self.buffer.get());
            self.print(&Color::Fixed(8).paint(rest).to_string());
            self.suggestion = Some(rest.to_string());
        }

        // Move cursor to left edge again
        self.print("\u{001b}[1000D");
        // Move cursor to current position
        self.print(&format!("\u{001b}[{}C", self.buffer.get_cursor() + 2));
    }

    fn render_command(&self, command: Command) {
        self.print(&renderer::command(&command, &self.executables));
        if command.parameters.is_none() {
            self.white_space(self.buffer.len() - command.program.span.end.index);
        }
    }

    fn commit(&mut self) {
        // If we're going to clear screen, don't send new line.
        if !self.buffer.get().starts_with("clear") {
            self.new_line();
        }

        if !self.buffer.is_empty() {
            self.history.commit(self.buffer.get().to_string());

            match parser::parse(&self.buffer.get()) {
                Ok((command, _)) => {
                    self.run_command(command);
                }
                Err(e) => {
                    self.println(&format!("bsh: syntax error: {}", e));
                }
            }
        }

        self.buffer.clear();
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

    fn set_exit_code(&mut self, code: u8) {
        if let Some(var) = self.globals.get_mut("?") {
            *var = format!("{}", code);
        }
    }

    fn run_command(&mut self, command: Command) {
        let name = &command.program.id.name;
        let program = self.executables.get(name).map(|getter| getter());
        match program {
            Some(program) => match program {
                Program::Builtin(program) => {
                    let transformer = Transformer::new(&self.globals, false);
                    let arguments = command
                        .parameters
                        .map(|p| transformer.transform(p))
                        .unwrap_or_default();
                    let exit_code = program.run(
                        &self.terminal,
                        &mut self.executables,
                        &mut self.globals,
                        arguments,
                    );
                    self.set_exit_code(exit_code);
                }
                Program::Internal(program) => {
                    let transformer = Transformer::new(&self.globals, false);
                    let arguments = command
                        .parameters
                        .map(|p| transformer.transform(p))
                        .unwrap_or_default();
                    self.running.set(true);
                    let (sender, receiver) = channel::<u8>();
                    let running = Rc::clone(&self.running);
                    spawn_local(async move {
                        receiver.await.expect("channel receiver failure");
                        running.set(false);
                    });
                    let stdout = executable::Stdio::new(Rc::clone(&self.terminal));
                    program.run(stdout, arguments, sender);
                }
                Program::External(_) => {}
            },
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
