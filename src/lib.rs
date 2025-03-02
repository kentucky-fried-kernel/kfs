#![no_std]
use terminal::SCREEN;

mod arch;
mod conv;
mod panic;
mod printk;
mod shell;
mod terminal;

#[no_mangle]
pub extern "C" fn kernel_main() {
    arch::x86::set_gdt();
    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut SCREEN });
}
