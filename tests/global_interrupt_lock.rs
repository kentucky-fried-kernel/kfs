#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use kfs::serial_println;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    kfs::tester::should_panic_panic_handler()
}

#[test_case]
fn panics_when_locking_lock_twice() -> Result<(), &'static str> {
    use kfs::arch::x86::interrupts::lock::GlobalInterruptLock;
    let _lock1 = GlobalInterruptLock::lock();
    let _lock2 = GlobalInterruptLock::lock();

    Err("GlobalInterruptLock::lock should have panicked when attempting to lock for the second time")
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    test_main();
    unsafe { kfs::qemu::exit(kfs::qemu::ExitCode::Failed) };
}
