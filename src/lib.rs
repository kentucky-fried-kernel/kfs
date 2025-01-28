#![no_std]

use terminal::Screen;

mod gdt;
mod panic;
mod print;
mod shell;
mod terminal;

#[no_mangle]
pub extern "C" fn kernel_main() {
    let mut s = Screen::default();
    shell::launch(&mut s);
}
