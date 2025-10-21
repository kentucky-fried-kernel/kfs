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

#[doc(hidden)]
#[allow(static_mut_refs)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    unsafe { PRINTK_WRITER.write_fmt(args).expect("Printing to VGA failed") };
}

/// `printk!` flushes when the buffer (`1KB`) fills up or when encountering a `\n`.
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {{
        $crate::printk::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! printkln {
    () => ($crate::printk!(concat!($fmt, "\n")));
    ($fmt:expr) => ($crate::printk!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::printk!(concat!($fmt, "\n"), $($arg)*));
}
