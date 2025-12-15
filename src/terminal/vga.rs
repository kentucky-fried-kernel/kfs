use core::ptr::write_volatile;

use crate::{
    boot::KERNEL_BASE,
    terminal::{Screen, cursor::Cursor, entry::Entry},
};

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

const VGA_BUFFER_ADDR: *mut u16 = (KERNEL_BASE + 0xB8000) as *mut u16;

#[derive(Debug)]
pub struct Buffer {
    pub entries: [[Entry; BUFFER_WIDTH]; BUFFER_HEIGHT],
    pub cursor: Option<Cursor>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            entries: [[Entry::new(b' '); BUFFER_WIDTH]; BUFFER_HEIGHT],
            cursor: None,
        }
    }
}

impl Buffer {
    pub fn from_screen(screen: &mut Screen, rows_scrolled_up: usize) -> Self {
        let lines_push_up = BUFFER_HEIGHT - screen.lines().rev().take(BUFFER_HEIGHT).count();
        let mut new = Self::default();
        for (line_index, l) in screen.lines().rev().skip(rows_scrolled_up).enumerate().take(BUFFER_HEIGHT) {
            for (char_index, c) in l.into_iter().enumerate().take(BUFFER_WIDTH) {
                new.entries[BUFFER_HEIGHT - line_index - 1 - lines_push_up][char_index] = *c;
            }
        }

        new.cursor = if rows_scrolled_up > 0 {
            None
        } else if let Some(cursor_line) = screen.lines().next_back() {
            if let Some((last_char_index, _)) = cursor_line.into_iter().enumerate().last() {
                Some(Cursor {
                    x: last_char_index as u16,
                    y: (BUFFER_HEIGHT - lines_push_up - 1) as u16,
                })
            } else {
                None
            }
        } else {
            None
        };
        new
    }

    pub fn flush(&self) {
        for (line_index, line) in self.entries.iter().enumerate() {
            for (character_index, c) in line.iter().enumerate() {
                let index = line_index * BUFFER_WIDTH + character_index;
                // SAFETY: This is safe because the iterators will never go over
                // BUFFER_HEIGHT * BUFFER_WIDTH which is the max len of the VGA BUFFER
                unsafe { write_volatile(VGA_BUFFER_ADDR.add(index), u16::from(*c)) }
            }
        }

        match self.cursor {
            None => Cursor::hide(),
            Some(cursor) => {
                // SAFETY: This is safe because we don't call flush in usermode right now
                unsafe {
                    cursor.flush_pos();
                };
                Cursor::show();
            }
        }
    }
}
pub struct LinesIterator<'a> {
    screen: &'a Screen,
    index: usize,
    index_back: usize,
}

impl<'a> LinesIterator<'a> {
    #[must_use]
    pub fn new(screen: &'a Screen) -> Self {
        let len = screen.len;
        Self {
            screen,
            index: 0,
            index_back: len - 1,
        }
    }
}

impl<'a> Iterator for LinesIterator<'a> {
    type Item = Line<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.index_back {
            return None;
        }

        let mut c = 0;
        for e in self.screen.into_iter().skip(self.index).take(BUFFER_WIDTH) {
            if e.character() == b'\n' {
                break;
            }
            c += 1;
        }

        let next = self.screen as *const Screen;

        // SAFETY: This lifetime is valid because it is linked
        // to the Lifetime of LinesIterator<'a> which itself is dependend on Screen
        unsafe {
            let next = Line::new(&*next, self.index, c);
            self.index += c;

            let new_line_found = c != BUFFER_WIDTH;
            if new_line_found {
                self.index += 1;
            }
            Some(next)
        }
    }
}

impl<'a> DoubleEndedIterator for LinesIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index_back <= self.index {
            return None;
        }

        let mut start_of_line = self.index_back + 1;
        let mut found_new_line = false;
        let screen_len = self.screen.len;
        for c in self.screen.into_iter().rev().skip(screen_len - (self.index_back + 1)) {
            if c.character() == b'\n' {
                found_new_line = true;
                break;
            }
            start_of_line = start_of_line.saturating_sub(1);
        }

        let start;
        let line_len;

        if start_of_line == self.index_back {
            start = start_of_line;
            line_len = 0;
            self.index_back -= 1;
        } else {
            let len_all = (self.index_back + 1) - start_of_line;
            if len_all > BUFFER_WIDTH {
                let mut len_inner = len_all;
                let mut counter = 0;
                loop {
                    if len_inner > BUFFER_WIDTH {
                        len_inner -= BUFFER_WIDTH;
                        counter += 1;
                    } else {
                        break;
                    }
                }
                let start_inner = start_of_line + counter * BUFFER_WIDTH;
                start = start_inner;
                line_len = len_inner;
                if self.index_back >= len_inner {
                    self.index_back -= len_inner;
                } else {
                    self.index_back = 0;
                }
            } else {
                let len_inner = len_all;
                let start_inner = start_of_line;
                start = start_inner;
                line_len = len_inner;
                if self.index_back >= len_inner {
                    self.index_back -= len_inner;
                } else {
                    self.index_back = 0;
                }
                if found_new_line {
                    self.index_back -= 1;
                }
            }
        }
        let next = self.screen as *const Screen;

        // SAFETY: This lifetime is valid because it is linked
        // to the Lifetime of LinesIterator<'a> which itself is dependend on Screen
        unsafe {
            let next = Line::new(&*next, start, line_len);
            Some(next)
        }
    }
}

pub struct Line<'a> {
    screen: &'a Screen,
    start: usize,
    len: usize,
}

impl<'a> Line<'a> {
    #[must_use]
    pub fn new(screen: &'a Screen, start: usize, len: usize) -> Self {
        Self { screen, start, len }
    }
}

pub struct LineIterator<'a> {
    line: Line<'a>,
    index: usize,
}

impl<'a> IntoIterator for Line<'a> {
    type Item = &'a Entry;
    type IntoIter = LineIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LineIterator { line: self, index: 0 }
    }
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.line.len {
            None
        } else {
            let idx: usize = (self.line.screen.head + self.line.start + self.index) % self.line.screen.entries.len();

            // SAFETY: This lifetime is valid because it is linked
            // to the Lifetime of Line<'a> which itself is dependend on Screen
            unsafe {
                let next = &raw const self.line.screen.entries[idx];
                self.index += 1;
                Some(&*next)
            }
        }
    }
}
