#![no_std]

use print::{slice_to_str, u64_to_base};
use terminal::vga::Buffer;

mod gdt;
mod panic;
mod print;
mod shell;
mod terminal;

#[no_mangle]
pub extern "C" fn kernel_main() {
    let mut t = terminal::Terminal::default();
    shell::launch(&mut t);
}
