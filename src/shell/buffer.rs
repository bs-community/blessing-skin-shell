pub struct Buffer {
    text: String,
    cursor: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            text: String::with_capacity(18),
            cursor: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.len() {
            self.cursor += 1;
        }
    }

    pub fn move_to_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_to_end(&mut self) {
        self.cursor = self.len();
    }

    pub fn insert(&mut self, data: &str) {
        self.insert_without_moving(data);
        self.cursor += data.len();
    }

    pub fn insert_without_moving(&mut self, data: &str) {
        if self.text.len() == self.cursor {
            self.text.push_str(data);
        } else {
            self.text.insert_str(self.cursor, data);
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    pub fn delete_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
        }
    }

    pub fn delete_right(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    #[inline]
    pub fn get(&self) -> &str {
        &self.text
    }

    pub fn set(&mut self, data: String) {
        self.cursor = data.len();
        self.text = data;
    }

    #[inline]
    pub fn get_cursor(&self) -> usize {
        self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut buffer = Buffer::new();
        buffer.insert("kumiko");

        assert_eq!(6, buffer.get_cursor());
        assert_eq!(6, buffer.len());
    }

    #[test]
    fn insert_without_moving_cursor() {
        let mut buffer = Buffer::new();
        buffer.insert_without_moving("kumiko");

        assert_eq!(0, buffer.get_cursor());
        assert_eq!(6, buffer.len());
    }

    #[test]
    fn check_empty() {
        let mut buffer = Buffer::new();
        assert!(buffer.is_empty());

        buffer.insert("t");
        assert!(!buffer.is_empty());
    }

    #[test]
    fn move_left() {
        let mut buffer = Buffer::new();
        buffer.move_left();
        assert_eq!(0, buffer.get_cursor());

        buffer.insert("kumiko");
        buffer.move_left();
        assert_eq!(5, buffer.get_cursor());
    }

    #[test]
    fn move_right() {
        let mut buffer = Buffer::new();
        buffer.insert("kumiko");

        buffer.move_right();
        assert_eq!(6, buffer.get_cursor());

        buffer.move_left();
        buffer.move_left();
        buffer.move_left();
        buffer.move_right();
        assert_eq!(4, buffer.get_cursor());
    }

    #[test]
    fn move_to_start() {
        let mut buffer = Buffer::new();
        buffer.insert("kumiko");
        assert_eq!(6, buffer.get_cursor());

        buffer.move_to_start();
        assert_eq!(0, buffer.get_cursor());
    }

    #[test]
    fn move_to_end() {
        let mut buffer = Buffer::new();
        buffer.insert("kumiko");
        buffer.move_left();
        buffer.move_left();
        assert_eq!(4, buffer.get_cursor());

        buffer.move_to_end();
        assert_eq!(6, buffer.get_cursor());
    }

    #[test]
    fn clear_buffer() {
        let mut buffer = Buffer::new();
        buffer.insert("kumiko");
        buffer.clear();

        assert_eq!(0, buffer.get_cursor());
        assert!(buffer.is_empty());
    }

    #[test]
    fn delete_left() {
        let mut buffer = Buffer::new();
        buffer.insert("kumiko");
        buffer.delete_left();

        assert_eq!("kumik", buffer.get());
        assert_eq!(5, buffer.get_cursor());
    }

    #[test]
    fn delete_right() {
        let mut buffer = Buffer::new();
        buffer.insert_without_moving("kumiko");
        buffer.delete_right();

        assert_eq!("umiko", buffer.get());
        assert_eq!(0, buffer.get_cursor());
    }

    #[test]
    fn get_buffer_content() {
        let mut buffer = Buffer::new();
        buffer.insert("kumiko");

        assert_eq!("kumiko", buffer.get());
    }

    #[test]
    fn set_buffer_content() {
        let mut buffer = Buffer::new();
        buffer.set("kumiko".to_string());

        assert_eq!("kumiko", buffer.get());
        assert_eq!(6, buffer.get_cursor());
    }
}
