use crate::{
    serial_println,
    terminal::{
        entry::{Color, Entry},
        vga::LinesIterator,
    },
};

pub const BUFFER_SIZE: usize = 0x10000;

pub static mut SCREEN: Screen = Screen::default();

#[derive(Debug)]
pub struct Screen {
    pub entries: [Entry; BUFFER_SIZE],
    pub head: usize,
    pub len: usize,
}

impl Screen {
    #[must_use]
    pub const fn default() -> Self {
        Self {
            entries: [Entry::new(b' '); BUFFER_SIZE],
            head: 0,
            len: 0,
        }
    }
    pub fn push(&mut self, e: Entry) {
        if self.entries.len() <= self.len {
            self.entries[self.head] = e;
            self.head += 1;
            self.head %= BUFFER_SIZE;
        } else {
            self.entries[(self.head + self.len) % self.entries.len()] = e;
            self.len += 1;
        }
    }

    pub fn write(&mut self, str: &str) {
        for c in str.chars() {
            self.push(Entry::new(c as u8));
        }
    }

    pub fn write_color(&mut self, str: &str, color: Color) {
        for c in str.chars() {
            self.push(Entry::new_with_color(c as u8, color as u8));
        }
    }

    pub fn remove_last(&mut self) -> Option<Entry> {
        if self.len == 0 {
            None
        } else {
            let idx = (self.head + self.len - 1) % self.entries.len();
            let e = self.entries[idx];
            self.len -= 1;
            Some(e)
        }
    }

    pub fn lines<'a>(&'a mut self) -> LinesIterator<'a> {
        LinesIterator::new(self)
    }
}

impl<'a> IntoIterator for &'a mut Screen {
    type Item = &'a mut Entry;
    type IntoIter = ScreenIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ScreenIterator { screen: self, index: 0 }
    }
}

pub struct ScreenIterator<'a> {
    screen: &'a mut Screen,
    index: usize,
}

impl<'a> Iterator for ScreenIterator<'a> {
    type Item = &'a mut Entry;
    fn next(&mut self) -> Option<&'a mut Entry> {
        if self.index >= self.screen.len {
            None
        } else {
            let idx: usize = (self.screen.head + self.index) % self.screen.entries.len();
            // SAFETY: This is safe because ScreenIterator is liked to the lifetime
            // of Screen which this returned element depends on
            unsafe {
                let next = &raw mut self.screen.entries[idx];
                self.index += 1;
                Some(&mut *next)
            }
        }
    }
}

impl<'a> DoubleEndedIterator for ScreenIterator<'a> {
    fn next_back(&mut self) -> Option<&'a mut Entry> {
        if self.index >= self.screen.len {
            None
        } else {
            let offset = self.screen.len - self.index - 1;
            let idx: usize = (self.screen.head + offset) % self.screen.entries.len();
            // SAFETY: This is safe because ScreenIterator is liked to the lifetime
            // of Screen which this returned element depends on
            unsafe {
                let next = &raw mut self.screen.entries[idx];
                self.index += 1;
                Some(&mut *next)
            }
        }
    }
}
