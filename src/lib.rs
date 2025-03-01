#![no_std]

use gdt::set_gdt;
use terminal::Screen;

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

    if let Err(e) = ps2::init() {
        let mut s = Screen::default();
        s.write_str(e);
    }

    let mut s = Screen::default();
    shell::launch(&mut s);
}
