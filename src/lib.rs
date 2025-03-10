#![no_std]
use terminal::SCREEN;

mod arch;
mod conv;
mod panic;
mod printk;
mod ps2;
mod shell;
mod terminal;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    if let Err(e) = ps2::init() {
        panic!("could not initialize PS/2: {}", e);
    }

    arch::x86::set_gdt();
    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut SCREEN });
}
