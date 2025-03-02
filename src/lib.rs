#![no_std]

use gdt::set_gdt;
use terminal::{Screen, vga::Buffer};

mod conv;
mod gdt;
mod panic;
mod print;
mod ps2;
mod shell;
mod terminal;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    set_gdt();
    let mut s = Screen::default();
    let b = Buffer::from_screen(&s);

    if let Err(e) = ps2::init() {
        let mut s = Screen::default();
        s.write_str(e);
        b.flush();
    }

    shell::launch(&mut s);
}
