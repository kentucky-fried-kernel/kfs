#![allow(unused_imports)]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use kfs::{printkln, serial_println};

    printkln!("KERNEL PANIC: {:?}\n", info.message());

    serial_println!("KERNEL PANIC: {:?}", info.message());

    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info);
}
