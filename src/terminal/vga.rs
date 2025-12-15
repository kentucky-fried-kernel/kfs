use core::ptr::write_volatile;

use crate::{
    boot::KERNEL_BASE,
    serial_print, serial_println,
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
            for (char_index, c) in l.enumerate().take(BUFFER_WIDTH) {
                new.entries[BUFFER_HEIGHT - line_index - 1 - lines_push_up][char_index] = *c;
            }
        }

        // new.cursor = if rows_scrolled_up > 0 {
        //     None
        // } else if let Some((cursor_line_index, cursor_line)) =
        // screen.lines().skip(line_viewable_first_index).enumerate().take(BUFFER_HEIGHT).last() {
        //     if let Some((last_char_index, _)) = cursor_line.enumerate().last() {
        //         Some(Cursor {
        //             x: last_char_index as u16,
        //             y: cursor_line_index as u16,
        //         })
        //     } else {
        //         None
        //     }
        // } else {
        //     None
        // };
        new
    }

    pub fn flush(&self) {
        for (line_index, line) in self.entries.iter().enumerate() {
            for (character_index, c) in line.iter().enumerate() {
                let index = line_index * BUFFER_WIDTH + character_index;
                // SAFETY: This is safe because the iterators will never go over
                // BUFFER_HEIGHT * BUFFER_WIDTH which is the max len of the VGA BUFFER
                unsafe { write_volatile(VGA_BUFFER_ADDR.add(index), c.to_u16()) }
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
    screen: &'a mut Screen,
    index: usize,
    index_back: usize,
}

impl<'a> LinesIterator<'a> {
    pub fn new(screen: &'a mut Screen) -> Self {
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
            if e.get_character() == b'\n' {
                break;
            }
            c += 1;
        }

        let next = self.screen as *mut Screen;

        // SAFETY: This lifetime is valid because it is linked
        // to the Lifetime of LinesIterator<'a> which itself is dependend on Screen
        unsafe {
            let next = Line::new(&mut *next, self.index, c);
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
        serial_println!("hello");
        serial_println!("index_back {}", self.index_back);
        if self.index_back <= self.index {
            return None;
        }

        let mut start_of_line = self.index_back + 1;
        let mut found_new_line = false;
        let mut screel_len = self.screen.len;
        for c in self.screen.into_iter().rev().skip(screel_len - (self.index_back + 1)) {
            if c.get_character() == b'\n' {
                found_new_line = true;
                break;
            }
            if start_of_line > 0 {
                start_of_line -= 1;
            }
            // serial_println!("start_of_line {}", start_of_line);
        }

        let mut start = 0;
        let mut line_len = 0;

        if start_of_line == self.index_back {
            start = start_of_line;
            line_len = 0;
            self.index_back -= 1;
        } else {
            let len_all = (self.index_back + 1) - start_of_line;
            if len_all > BUFFER_WIDTH {
                // serial_println!("hello");
                let len_inner = len_all % BUFFER_WIDTH;
                let start_inner = start_of_line + (len_all / BUFFER_WIDTH) * BUFFER_WIDTH;
                start = start_inner;
                line_len = len_inner;
                // serial_println!("start_of_line {}", start_of_line);
                // serial_println!("len {}", len_all);
                // serial_println!("len_all {}", len_all);
                // serial_println!("start_inner {}", start_inner);
                // serial_println!("len_inner {}", len_inner);
                // serial_println!("index_back {}", self.index_back);
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

        // if start_of_line == 0 {
        //     start = self.index_back;
        //     line_len = 0;
        //     self.index_back -= 1;
        // } else {
        //     if self.index_back == last_new_line_index {
        //         if last_new_line_index >= BUFFER_WIDTH {
        //             last_new_line_index %= BUFFER_WIDTH;
        //             start = self.index_back - last_new_line_index;
        //             line_len = last_new_line_index + 1;
        //             self.index_back -= last_new_line_index + 1;
        //         } else {
        //             start = 0;
        //             line_len = last_new_line_index + 1;
        //             self.index_back = 0;
        //         }
        //     } else {
        //         let loong = last_new_line_index == BUFFER_WIDTH - 1;
        //         last_new_line_index %= BUFFER_WIDTH;
        //         start = self.index_back - last_new_line_index + 1;
        //         line_len = last_new_line_index;
        //         if loong {
        //             line_len += 1;
        //         }
        //         self.index_back -= last_new_line_index + 1;
        //     }
        // }
        //
        serial_println!("start {}", start);
        serial_println!("line_len {}", line_len);
        serial_println!("index_back {}", self.index_back);
        serial_println!("");
        // let mut c = 0;
        // for e in self.screen.into_iter().skip(self.index).take(BUFFER_WIDTH) {
        //     if e.get_character() == b'\n' {
        //         break;
        //     }
        //     c += 1;
        // }
        let next = self.screen as *mut Screen;

        // SAFETY: This lifetime is valid because it is linked
        // to the Lifetime of LinesIterator<'a> which itself is dependend on Screen
        unsafe {
            let next = Line::new(&mut *next, start, line_len);
            Some(next)
        }
    }
}

pub struct Line<'a> {
    screen: &'a mut Screen,
    start: usize,
    len: usize,
    index: usize,
}

impl<'a> Line<'a> {
    pub fn new(screen: &'a mut Screen, start: usize, len: usize) -> Self {
        Self { screen, start, len, index: 0 }
    }
}

impl<'a> Iterator for Line<'a> {
    type Item = &'a mut Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let idx: usize = (self.screen.head + self.start + self.index) % self.screen.entries.len();

            // SAFETY: This lifetime is valid because it is linked
            // to the Lifetime of Line<'a> which itself is dependend on Screen
            unsafe {
                let next = &raw mut self.screen.entries[idx];
                self.index += 1;
                Some(&mut *next)
            }
        }
    }
}
