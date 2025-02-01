#![no_std]

use core::arch::asm;

use gdt::set_gdt;
use print::u64_to_base;
use terminal::Screen;

mod gdt;
mod panic;
mod print;
mod shell;
mod terminal;

fn store_gdt() -> u32 {
    let mut out: u32 = 0;
    unsafe {
        asm!(
            "sgdt [eax]",
            out("eax") out,
        )
    };
    out
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    set_gdt();
    // let out = store_gdt();

    let mut s = Screen::default();
    // let nbr = u64_to_base(out as u64, 16).unwrap();
    // for i in nbr.0.iter() {
    //     if *i == 0 {
    //         continue;
    //     }
    //     s.write(*i);
    // }
    // s.write(b'\n');
    shell::launch(&mut s);
}
