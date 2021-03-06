use crate::shell::{executable::Internal, Argument, Arguments};
use crate::stdio::Stdio;
use futures::channel::oneshot::Sender;
use js_sys::Reflect;
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::Response;

async fn fetch(url: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().expect("window should exist");
    let resp = JsFuture::from(window.fetch_with_str(url)).await?;
    let resp: Response = resp.dyn_into().expect("failed to convert response");

    JsFuture::from(resp.text()?).await
}

pub struct Curl;

impl Default for Curl {
    fn default() -> Self {
        Curl
    }
}

impl Internal for Curl {
    fn run(&self, stdio: Rc<Stdio>, arguments: Arguments, exit: Sender<()>) {
        spawn_local(async move {
            let url = match arguments.get(0) {
                Some(url) => url,
                None => {
                    stdio.println("No URL is provided.");
                    exit.send(()).expect("sender failure");
                    return;
                }
            };
            let url = match url {
                Argument::Text(t) => t.clone(),
                Argument::Switch(_, _) => {
                    stdio.println("No URL is provided.");
                    exit.send(()).expect("sender failure");
                    return;
                }
            };

            match fetch(&url).await {
                Ok(text) => match text.as_string() {
                    Some(text) => {
                        let text = text.replace("\n", "\r\n");
                        stdio.reset();
                        stdio.println(&text);
                    }
                    None => {
                        stdio.println("conversion failed");
                    }
                },
                Err(e) => {
                    let message = Reflect::get(&e, &JsValue::from("message"))
                        .ok()
                        .and_then(|message| message.as_string())
                        .unwrap_or_else(|| "unknown error".to_string());
                    stdio.println(&message);
                }
            };
            if exit.send(()).is_err() {
                stdio.println("Program is hang up...Please refresh the page.");
            }
        });
    }
}
