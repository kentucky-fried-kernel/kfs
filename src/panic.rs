#![allow(unused_imports)]
use core::panic::PanicInfo;

use crate::terminal::vga::Color;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use crate::terminal::{vga::Buffer, Screen};

    let mut s = Screen::default();
    s.write_color_str("Panicked!", Color::Error as u8);
    let b = Buffer::from_screen(&s);
    b.flush();
    loop {}
}
