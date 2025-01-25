use super::{
    ps2::Key,
    vga::{Color, Entry},
};

pub const BUFFER_SIZE: usize = 1000;

#[derive(Clone, Copy)]
pub struct Screen {
    pub buffer: [u16; BUFFER_SIZE],
    pub cursor: usize,
    pub last_entry_index: usize,
    pub rows_scrolled: usize,
}

impl Screen {
    pub fn default() -> Self {
        Screen {
            buffer: [Entry::new(b' ').to_u16(); BUFFER_SIZE],
            cursor: 0,
            last_entry_index: 0,
            rows_scrolled: 0,
        }
    }

    pub fn handle_key(&mut self, key: Key) {
        use Key::*;
        match key {
            Tab => {}
            Enter => self.write(b'\n'),
            Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                self.remove_entry_at(self.cursor);
            }
            ArrowUp => self.scroll(1),
            ArrowDown => self.scroll(-1),
            ArrowLeft => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            ArrowRight => {
                if self.cursor < BUFFER_SIZE - 1 && self.cursor <= self.last_entry_index {
                    self.cursor += 1;
                }
            }
            _ => self.write(key as u8),
        }
    }

    pub fn scroll(&mut self, delta: isize) {
        if delta >= 0 {
            self.rows_scrolled += delta as usize;
        } else if delta < 0 && delta.unsigned_abs() <= self.rows_scrolled {
            self.rows_scrolled -= delta.unsigned_abs();
        } else {
            self.rows_scrolled = 0;
        }
    }

    pub fn write(&mut self, character: u8) {
        self.write_color(character, Color::Default as u8);
    }

    pub fn write_color(&mut self, character: u8, color: u8) {
        if self.cursor >= BUFFER_SIZE - 1 {
            return;
        }
        let mut index = BUFFER_SIZE - 2;
        while index + 1 > self.cursor  && index > 0{
            self.buffer[index + 1] = self.buffer[index];
            index -= 1;
        }

        self.last_entry_index += 1;
        self.buffer[self.cursor] = Entry::new_with_color(character, color).to_u16();

        self.cursor += 1;
    }

    pub fn write_str(&mut self, string: &str) {
        for &c in string.as_bytes().iter() {
            self.write(c);
        }
    }

    #[allow(dead_code)]
    pub fn write_color_str(&mut self, string: &str, color: u8) {
        for &c in string.as_bytes().iter() {
            self.write_color(c, color);
        }
    }

    fn remove_entry_at(&mut self, mut index: usize) {
        while (index + 1) < BUFFER_SIZE {
            self.buffer[index] = self.buffer[index + 1];
            index += 1;
        }
        self.last_entry_index -= 1;
        self.buffer[index] = Entry::new(b' ').to_u16();
    }
}
