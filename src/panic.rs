#![allow(unused_imports)]
use core::panic::PanicInfo;

use kfs::terminal::entry::Color;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use kfs::terminal::{Screen, vga::Buffer};

    let mut s = Screen::default();
    s.write_color("KERNEL PANIC\n", Color::Error);
    s.write_color(_info.message().as_str().unwrap_or("???"), Color::Error);
    let b = Buffer::from_screen(&mut s, 0);
    b.flush();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info);
}
