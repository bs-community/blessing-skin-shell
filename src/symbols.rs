use ansi_term::Color;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Symbol;

#[wasm_bindgen]
impl Symbol {
    pub fn success(message: String) -> String {
        format!("{} {}", Color::Green.paint("✔"), message)
    }

    pub fn info(message: String) -> String {
        format!("{} {}", Color::Blue.paint("ℹ"), message)
    }

    pub fn warning(message: String) -> String {
        format!("{} {}", Color::Yellow.paint("⚠"), message)
    }

    pub fn error(message: String) -> String {
        format!("{} {}", Color::Red.paint("✖"), message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success() {
        assert_eq!(
            Symbol::success("msg".to_string()),
            format!("{} {}", Color::Green.paint("✔"), "msg")
        );
    }

    #[test]
    fn test_info() {
        assert_eq!(
            Symbol::info("msg".to_string()),
            format!("{} {}", Color::Blue.paint("ℹ"), "msg")
        );
    }

    #[test]
    fn test_warning() {
        assert_eq!(
            Symbol::warning("msg".to_string()),
            format!("{} {}", Color::Yellow.paint("⚠"), "msg")
        );
    }

    #[test]
    fn test_error() {
        assert_eq!(
            Symbol::error("msg".to_string()),
            format!("{} {}", Color::Red.paint("✖"), "msg")
        );
    }
}
