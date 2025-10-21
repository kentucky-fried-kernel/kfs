#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use core::panic::PanicInfo;

pub mod arch;
pub mod boot;
pub mod conv;
pub mod macros;
pub mod port;
pub mod printk;
pub mod ps2;
pub mod qemu;
pub mod serial;
pub mod shell;
pub mod terminal;
pub mod tester;

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    use crate::qemu;
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::tester::panic_handler(info);
}
