#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use kfs::{arch::x86::interrupts::lock::IRQLock, serial_println};

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    kfs::tester::should_panic_panic_handler()
}

#[test_case]
fn panics_when_locking_lock_twice() -> Result<(), &'static str> {
    use kfs::arch::x86::interrupts::lock::IRQLock;
    let _lock1 = IRQLock::lock(1);
    let _lock2 = IRQLock::lock(1);

    Err("IRQLock::lock should have panicked when attempting to lock the same IRQ for the second time")
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    test_main();
    unsafe { kfs::qemu::exit(kfs::qemu::ExitCode::Failed) };
}
