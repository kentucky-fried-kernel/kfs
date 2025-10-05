#![no_std]
use terminal::SCREEN;

mod arch;
mod conv;
mod panic;
mod port;
mod printk;
mod ps2;
mod qemu;
mod shell;
mod terminal;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    if let Err(e) = ps2::init() {
        panic!("could not initialize PS/2: {}", e);
    }
    arch::x86::gdt::init();
    #[cfg(not(test))]
    arch::x86::set_idt();
    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut SCREEN });
}
