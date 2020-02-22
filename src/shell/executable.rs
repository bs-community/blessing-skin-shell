use super::{Arguments, Executables, Vars};
use crate::terminal::Terminal;
use ansi_term::Color;
use futures::channel::oneshot::Sender;
use std::rc::Rc;

pub struct Stdio {
    terminal: Rc<Terminal>,
}

impl Stdio {
    pub fn new(terminal: Rc<Terminal>) -> Stdio {
        Stdio { terminal }
    }

    pub fn print(&self, data: &str) {
        self.terminal.write(data);
    }

    pub fn println(&self, data: &str) {
        self.print(data);
        self.print("\r\n");
    }

    pub fn reset(&self) {
        // Move cursor to left edge
        self.print("\u{001b}[1000D");
        // Clear line
        self.print("\u{001b}[0K");
    }

    pub fn complete(&self) {
        self.print(&Color::Purple.paint("❯ ").to_string());
    }
}

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
        globals: &mut Vars,
        arguments: Arguments,
    );
}

pub trait Internal {
    fn run(&self, stdout: Stdio, arguments: Arguments, sender: Sender<u8>);
}

pub trait External {
    fn run(&self, stdout: Stdio, arguments: Vec<String>);
}
