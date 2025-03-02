use core::fmt;

use crate::terminal::{SCREEN, Screen, vga::Buffer};

const BUFFER_SIZE: usize = 1024;

pub struct PrintkWriter {
    screen: &'static mut Screen,
    buffer: [u8; BUFFER_SIZE],
    position: usize,
}

impl PrintkWriter {
    const fn new() -> Self {
        Self {
            #[allow(static_mut_refs)]
            screen: unsafe { &mut SCREEN },
            buffer: [0; BUFFER_SIZE],
            position: 0,
        }
    }

    fn write_byte(&mut self, byte: u8) {
        if self.position >= BUFFER_SIZE {
            self.flush();
        }
        self.buffer[self.position] = byte;
        self.position += 1;

        if byte == b'\n' {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        let b = Buffer::from_screen(self.screen);
        for byte in &self.buffer[..self.position] {
            self.screen.write(*byte);
        }
        b.flush();
        self.position = 0;
    }
}

#[unsafe(link_section = ".data")]
pub static mut PRINTK_WRITER: PrintkWriter = PrintkWriter::new();

impl fmt::Write for PrintkWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

/// Prints the formatted arguments to the screen. This macro needs to be wrapped in an unsafe
/// block, as we could inadvertedly run unchecked code through it otherwise.
///
/// `printk!` flushes when the buffer (`1KB`) fills up or when encountering a `\n`.
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {{
        use core::fmt::Write;

        use $crate::printk::PRINTK_WRITER;
        #[allow(static_mut_refs)]
        let _ = write!(PRINTK_WRITER, $($arg)*);
    }};
}
