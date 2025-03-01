#![no_std]

use gdt::set_gdt;
use terminal::{ps2, Screen};

mod conv;
mod gdt;
mod panic;
mod print;
mod shell;
mod terminal;

#[no_mangle]
pub extern "C" fn kernel_main() {
    set_gdt();

    if let Err(e) = ps2::init() {
        panic!("{}", );
    }

    let mut s = Screen::default();
    shell::launch(&mut s);
}
