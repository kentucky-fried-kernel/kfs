#![no_std]

use gdt::set_gdt;
use terminal::Screen;

mod conv;
mod gdt;
mod panic;
mod print;
mod shell;
mod terminal;

#[no_mangle]
pub extern "C" fn kernel_main() {
    set_gdt();
    let mut s = Screen::default();
    shell::launch(&mut s);
}
