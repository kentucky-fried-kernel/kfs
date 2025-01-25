#![allow(unused_imports)]
use core::panic::PanicInfo;

use crate::terminal::vga::Color;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use crate::terminal::{vga::Buffer, Terminal};

    let mut t = Terminal::default();
    t.write_color_str("Paniced!", Color::Error as u8);
    let b = Buffer::from_screen(t.active_screen());
    b.flush();
    loop {}
}
