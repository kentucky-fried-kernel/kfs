#![no_std]

use print::{slice_to_str, u64_to_base};
use terminal::vga::Buffer;

mod gdt;
mod panic;
mod print;
mod terminal;

#[no_mangle]
pub extern "C" fn kernel_main() {
    let mut t = terminal::Terminal::default();
    let (slice, len) = u64_to_base(42_u64, 10).unwrap();
    let string = slice_to_str((&slice, len)).unwrap();
    t.write_str(string);
    t.write_str("\n");
    let b = Buffer::from_screen(t.active_screen());
    b.flush();
    loop {
        if let Some(key) = terminal::ps2::read_if_ready() {
            t.handle_key(key);
            let b = Buffer::from_screen(t.active_screen());
            b.flush();
        }
    }
}
