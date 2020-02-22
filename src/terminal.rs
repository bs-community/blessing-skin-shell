#[cfg(test)]
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

#[cfg(not(test))]
#[wasm_bindgen]
extern "C" {
    pub type Terminal;

    #[wasm_bindgen(method)]
    pub fn write(this: &Terminal, data: &str);

    #[wasm_bindgen(method)]
    pub fn clear(this: &Terminal);
}

#[cfg(test)]
#[wasm_bindgen]
pub struct Terminal {
    buffer: RefCell<String>,
}

#[cfg(test)]
#[wasm_bindgen]
impl Terminal {
    pub fn new() -> Self {
        Terminal {
            buffer: RefCell::new(String::new()),
        }
    }

    pub fn write(&self, data: &str) {
        let mut buffer = self.buffer.borrow_mut();
        buffer.push_str(data);
    }

    pub fn clear(&self) {
        let mut buffer = self.buffer.borrow_mut();
        buffer.clear();
    }

    pub fn get(&self) -> String {
        self.buffer.borrow().clone()
    }
}
