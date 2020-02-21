use crate::shell::{
    executable::{Internal, Stdio},
    Argument, Arguments,
};
use futures::channel::oneshot::Sender;
use js_sys::Reflect;
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
    fn run(&self, stdio: Stdio, arguments: Arguments, sender: Sender<u8>) {
        spawn_local(async move {
            let url = match arguments.get(0) {
                Some(url) => url,
                None => {
                    stdio.println("No URL is provided.");
                    sender.send(1).expect("sender failure");
                    return;
                }
            };
            let url = match url {
                Argument::Text(t) => t.clone(),
                Argument::Switch(_, _) => {
                    stdio.println("No URL is provided.");
                    sender.send(1).expect("sender failure");
                    return;
                }
            };

            let exit_code = match fetch(&url).await {
                Ok(text) => match text.as_string() {
                    Some(text) => {
                        let text = text.replace("\n", "\r\n");
                        stdio.reset();
                        stdio.println(&text);
                        0
                    }
                    None => {
                        stdio.println("conversion failed");
                        1
                    }
                },
                Err(e) => {
                    let message =
                        Reflect::get(&e, &JsValue::from("message")).expect("should have message");
                    let message = message
                        .as_string()
                        .unwrap_or_else(|| String::from("connection failed"));
                    stdio.println(&message);
                    1
                }
            };
            stdio.complete();
            sender.send(exit_code).expect("sender failure");
        });
    }
}
