use super::transform::Transformer;
use super::{Arguments, Executables, Vars};
use crate::parser::ast::Parameters;
use crate::stdio::Stdio;
use crate::terminal::Terminal;
use futures::channel::oneshot::{channel, Sender};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

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
    fn run(&self, stdout: Rc<Stdio>, arguments: Arguments, exit: Sender<()>);
}

pub trait External {
    fn run(&self, stdout: Stdio, arguments: Vec<String>);
}

pub struct Runner {
    running: Rc<Cell<bool>>,
}

impl Runner {
    pub fn new() -> Self {
        Runner {
            running: Rc::new(Cell::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.get()
    }

    pub fn run_builtin(
        &self,
        program: Box<dyn Builtin>,
        parameters: Option<Parameters>,
        terminal: &Terminal,
        executables: &mut Executables,
        globals: &mut Vars,
    ) {
        let transformer = Transformer::new(&globals, false);
        let arguments = parameters
            .map(|p| transformer.transform(p))
            .unwrap_or_default();
        program.run(terminal, executables, globals, arguments);
    }

    pub fn run_internal(
        &mut self,
        program: Box<dyn Internal>,
        parameters: Option<Parameters>,
        globals: &Vars,
        stdio: Rc<Stdio>,
    ) {
        self.running.set(true);

        let transformer = Transformer::new(&globals, false);
        let arguments = parameters
            .map(|p| transformer.transform(p))
            .unwrap_or_default();

        let (sender, receiver) = channel::<()>();
        program.run(Rc::clone(&stdio), arguments, sender);

        let running = Rc::clone(&self.running);
        spawn_local(async move {
            receiver.await.expect("channel receiver failure");
            running.set(false);
            stdio.prompt();
        });
    }
}
