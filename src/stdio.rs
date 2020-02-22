use crate::terminal::Terminal;
use ansi_term::Color;
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

    pub fn prompt(&self) {
        self.print(&Color::Purple.paint("❯ ").to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print() {
        let terminal = Rc::new(Terminal::new());
        let stdio = Stdio::new(Rc::clone(&terminal));

        stdio.print("text");
        assert_eq!("text", &terminal.get());
    }

    #[test]
    fn print_with_linebreak() {
        let terminal = Rc::new(Terminal::new());
        let stdio = Stdio::new(Rc::clone(&terminal));

        stdio.println("text");
        assert_eq!("text\r\n", &terminal.get());
    }
}
