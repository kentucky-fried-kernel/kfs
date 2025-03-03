#![no_std]
use gdt::set_gdt;
use terminal::SCREEN;

mod conv;
mod gdt;
mod panic;
mod print;
mod shell;
mod terminal;

#[no_mangle]
pub extern "C" fn kernel_main() {
    set_gdt();
    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut SCREEN });
}
