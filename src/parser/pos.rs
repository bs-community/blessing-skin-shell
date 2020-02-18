use combine::stream::position::{Positioner, RangePositioner};

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub index: usize,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            line: 1,
            column: 1,
            index: 0,
        }
    }
}

impl Position {
    pub fn new() -> Position {
        Position::default()
    }
}

impl Positioner<char> for Position {
    type Position = Position;
    type Checkpoint = Self;

    #[inline]
    fn position(&self) -> Position {
        self.clone()
    }

    #[inline]
    fn update(&mut self, token: &char) {
        self.index += 1;
        self.column += 1;
        if *token == '\n' {
            self.column = 1;
            self.line += 1;
        }
    }

    #[inline]
    fn checkpoint(&self) -> Self::Checkpoint {
        self.clone()
    }

    #[inline]
    fn reset(&mut self, checkpoint: Self::Checkpoint) {
        *self = checkpoint;
    }
}

impl<'a> RangePositioner<char, &'a str> for Position {
    fn update_range(&mut self, range: &&'a str) {
        range.chars().for_each(|c| self.update(&c));
    }
}
