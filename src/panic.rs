#![allow(unused_imports)]
use core::panic::PanicInfo;

use kfs::{
    boot::{STACK, STACK_SIZE},
    serial_print, serial_println,
    terminal::entry::Color,
};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use kfs::{clear_regs, cli, hlt, printkln, serial_println};

    cli!();

    printkln!("KERNEL PANIC: {:?}\n", info.message());

    serial_println!("KERNEL PANIC: {:?}", info.message());

    unsafe {
        clear_regs!();
    }
    loop {
        hlt!();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info);
}
