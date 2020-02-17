pub struct History {
    cursor: usize,
    commands: Vec<String>,
}

impl History {
    pub fn new() -> History {
        History {
            cursor: 0,
            commands: vec![],
        }
    }

    pub fn up(&mut self) -> Option<String> {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.current()
    }

    pub fn down(&mut self) -> Option<String> {
        if self.cursor < self.commands.len() {
            self.cursor += 1;
        }
        self.current()
    }

    fn current(&self) -> Option<String> {
        self.commands.get(self.cursor).cloned()
    }

    pub fn commit(&mut self, command: String) {
        self.commands.push(command);
        self.cursor = self.commands.len();
    }

    pub fn find(&self, command: &str) -> Option<String> {
        self.commands
            .iter()
            .rev()
            .find(|c| c.starts_with(command))
            .map(|c| c.into())
    }
}

#[test]
fn history_up() {
    let mut history = History::new();

    assert_eq!(history.up(), None);

    history.commit("kumiko".into());
    history.commit("reina".into());

    assert_eq!(history.up(), Some("reina".into()));
    assert_eq!(history.up(), Some("kumiko".into()));
    assert_eq!(history.up(), Some("kumiko".into()));
}

#[test]
fn history_down() {
    let mut history = History::new();

    assert_eq!(history.down(), None);

    history.commit("kumiko".into());
    history.commit("reina".into());

    assert_eq!(history.down(), None);
    history.up();
    history.up();
    assert_eq!(history.down(), Some("reina".into()));
    assert_eq!(history.down(), None);
}

#[test]
fn find_history() {
    let mut history = History::new();
    history.commit("a123".into());
    history.commit("a456".into());

    assert_eq!(history.find("b"), None);
    assert_eq!(history.find("a"), Some("a456".into()));
    assert_eq!(history.find("a1"), Some("a123".into()));
}
