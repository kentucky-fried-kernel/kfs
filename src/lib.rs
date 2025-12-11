#![no_std]
#![no_main]
#![feature(const_ops)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_trait_impl)]
#![feature(iter_map_windows)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(pointer_is_aligned_to)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use core::panic::PanicInfo;

pub extern crate alloc;

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
