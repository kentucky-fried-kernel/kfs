#![allow(unused_imports)]
use core::panic::PanicInfo;

use crate::terminal::vga::Color;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use crate::terminal::{Screen, vga::Buffer};

    let mut s = Screen::default();
    s.write_color_str("Panicked!\n", Color::Error as u8);
    s.write_str(_info.message().as_str().unwrap_or("???"));
    let b = Buffer::from_screen(&s);
    b.flush();
    loop {}
}
