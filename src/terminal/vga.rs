use core::ptr::write_volatile;

use crate::{
    boot::KERNEL_BASE,
    serial_println,
    terminal::{Screen, entry::Entry},
};

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

const VGA_BUFFER_ADDR: *mut u16 = (KERNEL_BASE + 0xB8000) as *mut u16;

#[derive(Debug)]
pub struct Buffer {
    pub entries: [[Entry; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    pub fn default() -> Self {
        Self {
            entries: [[Entry::new(b' '); BUFFER_WIDTH]; BUFFER_HEIGHT],
        }
    }

    pub fn from_screen(screen: &mut Screen) -> Self {
        let line_viewable_first_index = if screen.lines().count() > BUFFER_HEIGHT {
            screen.lines().count() - BUFFER_HEIGHT
        } else {
            0
        };

        let mut new = Self::default();
        for (line_index, l) in screen.lines().skip(line_viewable_first_index).enumerate().take(BUFFER_HEIGHT) {
            for (char_index, c) in l.enumerate() {
                new.entries[line_index][char_index] = *c;
            }
        }

        new
    }

    pub fn flush(&self) {
        for (line_index, line) in self.entries.iter().enumerate() {
            for (character_index, c) in line.iter().enumerate() {
                let index = line_index * BUFFER_WIDTH + character_index;
                unsafe { write_volatile(VGA_BUFFER_ADDR.add(index), c.to_u16()) }
            }
        }
    }
}
pub struct LinesIterator<'a> {
    screen: &'a mut Screen,
    index: usize,
}

impl<'a> LinesIterator<'a> {
    pub fn new(screen: &'a mut Screen) -> Self {
        Self { screen, index: 0 }
    }
}

impl<'a> Iterator for LinesIterator<'a> {
    type Item = Line<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.screen.into_iter().count() {
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

            unsafe {
                let next = &mut self.screen.entries[idx] as *mut Entry;
                self.index += 1;
                Some(&mut *next)
            }
        }
    }
}
