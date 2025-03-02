use core::fmt;

use crate::terminal::{vga::Buffer, Screen, SCREEN};

pub struct PrintkWriter {
    screen: &'static mut Screen,
}

impl PrintkWriter {
    const fn new() -> Self {
        Self {
            #[allow(static_mut_refs)]
            screen: unsafe { &mut SCREEN },
        }
    }

    pub fn flush(&self) {
        let b = Buffer::from_screen(self.screen);
        b.flush();
    }
}

pub static mut PRINTK_WRITER: PrintkWriter = PrintkWriter::new();

impl fmt::Write for PrintkWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.screen.write_str(s);
        Ok(())
    }
}

/// Prints the formatted arguments to the screen. This macro needs to be wrapped in an unsafe
/// block, as we could inadvertedly run unchecked code through it otherwise.
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {{
        use core::fmt::Write;

        use $crate::printk::PRINTK_WRITER;
        #[allow(static_mut_refs)]
        let _ = write!(PRINTK_WRITER, $($arg)*);
        #[allow(static_mut_refs)]
        PRINTK_WRITER.flush();
    }};
}
