use super::transform::Transformer;
use super::{Arguments, Executables, Vars};
use crate::parser::ast::Parameters;
use crate::stdio::Stdio;
use crate::terminal::Terminal;
use futures::channel::oneshot::{channel, Sender};
use js_sys::{Promise, Reflect};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

pub enum Program {
    Builtin(Box<dyn Fn() -> Box<dyn Builtin>>),
    Internal(Box<dyn Fn() -> Box<dyn Internal>>),
    External(External),
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

#[wasm_bindgen]
extern "C" {
    pub type ExternalProgram;

    #[wasm_bindgen(catch, method)]
    pub fn run(this: &ExternalProgram, stdio: Stdio, args: JsValue) -> Result<JsValue, JsValue>;
}

pub struct External {
    program: ExternalProgram,
}

impl External {
    pub fn new(program: ExternalProgram) -> Self {
        External { program }
    }

    pub fn run(&self, terminal: Rc<Terminal>, arguments: Vec<String>, exit: Sender<()>) {
        let stdio = Stdio::new(Rc::clone(&terminal));
        let program = &self.program;
        let arguments =
            serde_wasm_bindgen::to_value(&arguments).expect("arguments conversion failed");
        let result = match program.run(stdio, arguments) {
            Ok(value) => value,
            Err(e) => {
                let message = Reflect::get(&e, &JsValue::from("message"))
                    .ok()
                    .and_then(|message| message.as_string())
                    .unwrap_or_else(|| "unknown error".to_string());
                terminal.write(&format!("{}\r\n", message));
                if exit.send(()).is_err() {
                    terminal.write("Program is hang up...Please refresh the page.\r\n");
                }
                return;
            }
        };
        spawn_local(async move {
            let future = JsFuture::from(Promise::resolve(&result));
            if future.await.is_err() {
                terminal.write("\r\n");
            };
            exit.send(()).expect("sender failure");
        });
    }
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

    pub fn run_external(
        &mut self,
        program: &External,
        parameters: Option<Parameters>,
        terminal: Rc<Terminal>,
        globals: &Vars,
        stdio: Rc<Stdio>,
    ) {
        self.running.set(true);

        let transformer = Transformer::new(&globals, true);
        let arguments = parameters
            .map(|p| transformer.to_texts(transformer.transform(p)))
            .unwrap_or_default();

        let (exit_sender, exit_receiver) = channel::<()>();
        program.run(terminal, arguments, exit_sender);

        let running = Rc::clone(&self.running);
        spawn_local(async move {
            exit_receiver.await.expect("channel receiver failure");
            running.set(false);
            stdio.prompt();
        });
    }
}
