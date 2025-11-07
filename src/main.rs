#![no_std]
#![no_main]

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

pub const STACK_SIZE: usize = 2 << 20;

#[repr(align(4096))]
struct Stack([u8; STACK_SIZE]);

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

// #[unsafe(naked)]
// #[unsafe(no_mangle)]
// unsafe extern "C" fn _start() {
//     core::arch::naked_asm!("push eax", "push ebx", "cli", "call kernel_main");
// }
