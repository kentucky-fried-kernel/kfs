use super::{
    ps2::Key,
    vga::{Color, Entry},
};

pub const BUFFER_SIZE: usize = 50000;

#[derive(Clone, Copy)]
pub struct Screen {
    pub buffer: [u16; BUFFER_SIZE],
    pub cursor: usize,
    pub last_entry_index: usize,
    pub rows_scrolled: usize,
}

/// This is a temporary fix until we have an allocator.
#[link_section = ".data"]
pub static mut SCREEN: Screen = Screen::default();

impl Screen {
    pub const fn default() -> Self {
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
                    self.remove_entry_at(self.cursor);
                }
            }
            ArrowUp => self.scroll(1),
            ArrowDown => self.scroll(-1),
            ArrowLeft => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            ArrowRight => {
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

    pub fn move_cursor_to_end(&mut self) {
        for _ in 0..BUFFER_SIZE {
            self.handle_key(Key::ArrowRight);
        }
        self.rows_scrolled = 0;
    }

    /// Writes a single byte in hexadecimal notation (little-endian).
    pub fn write_hex_byte(&mut self, byte: u8) {
        let high_nibble = (byte >> 4) & 0xF;
        let low_nibble = byte & 0xF;

        self.write(if high_nibble < 10 { b'0' + high_nibble } else { b'a' + (high_nibble - 10) });
        self.write(if low_nibble < 10 { b'0' + low_nibble } else { b'a' + (low_nibble - 10) });
    }

    /// Writes `value` in hexadecimal notation, left-padded with zeros.
    pub fn write_hex(&mut self, val: u32) {
        let mut nibble;
        for i in (0..8).rev() {
            nibble = ((val >> (i * 4)) & 0xF) as u8;
            self.write(if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) });
        }
    }
}
