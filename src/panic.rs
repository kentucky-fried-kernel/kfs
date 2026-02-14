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
    use kfs::{clear_regs, cli, hlt, printkln, serial_println, stack_print_serial::print_stack_to_serial};

    cli!();

    printkln!("KERNEL PANIC: {:?}\n", info.message());

    serial_println!("KERNEL PANIC: {:?}", info.message());

    print_stack_to_serial();
    unsafe {
        clear_regs!();
    }
    hlt!();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info);
}
