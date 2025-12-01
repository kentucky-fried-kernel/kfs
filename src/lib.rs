#![no_std]
#![no_main]
#![feature(const_ops)]
#![feature(const_convert)]
#![feature(const_trait_impl)]
#![deny(clippy::unwrap_used)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use core::panic::PanicInfo;

pub mod arch;
pub mod bitmap;
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
pub mod vmm;

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    use crate::qemu;
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::tester::panic_handler(info);
}
