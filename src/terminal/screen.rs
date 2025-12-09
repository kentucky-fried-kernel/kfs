use crate::ps2::Key;

use super::vga::{Color, Entry};

pub const BUFFER_SIZE: usize = 50000;

#[derive(Clone, Copy)]
pub struct Screen {
    pub buffer: [u16; BUFFER_SIZE],
    pub cursor: usize,
    pub last_entry_index: usize,
    pub rows_scrolled: usize,
}

/// This is a temporary fix until we have an allocator.
#[unsafe(link_section = ".data")]
pub static mut SCREEN: Screen = Screen::default();

impl Screen {
    #[must_use]
    pub const fn default() -> Self {
        Screen {
            buffer: [Entry::new(b' ').to_u16(); BUFFER_SIZE],
            cursor: 0,
            last_entry_index: 0,
            rows_scrolled: 0,
        }
    }

    pub fn handle_key(&mut self, key: Key) {
        match key {
            Key::Tab => {}
            Key::Enter => self.write(b'\n'),
            Key::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.remove_entry_at(self.cursor);
                }
            }
            Key::ArrowUp => self.scroll(1),
            Key::ArrowDown => self.scroll(-1),
            Key::ArrowLeft => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            Key::ArrowRight => {
                if self.cursor < BUFFER_SIZE - 1 && self.cursor < self.last_entry_index {
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
        while index + 1 > self.cursor && index > 0 {
            self.buffer[index + 1] = self.buffer[index];
            index -= 1;
        }

        self.last_entry_index += 1;
        self.buffer[self.cursor] = Entry::new_with_color(character, color).to_u16();

        self.cursor += 1;
    }

    pub fn write_str(&mut self, string: &str) {
        for &c in string.as_bytes() {
            self.write(c);
        }
    }

    #[allow(dead_code)]
    pub fn write_color_str(&mut self, string: &str, color: u8) {
        for &c in string.as_bytes() {
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

    #[allow(unused)]
    pub fn move_cursor_to_end(&mut self) {
        for _ in 0..BUFFER_SIZE {
            self.handle_key(Key::ArrowRight);
        }
        self.rows_scrolled = 0;
    }
}
