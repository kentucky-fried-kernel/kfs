#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use terminal::SCREEN;

mod arch;
mod conv;
mod macros;
mod panic;
mod port;
mod printk;
mod ps2;
mod qemu;
mod serial;
mod shell;
mod terminal;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[cfg(not(test))]
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

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    test_main();
    unsafe {
        qemu::exit(qemu::ExitCode::Success);
    }
    // if let Err(e) = ps2::init() {
    //     panic!("could not initialize PS/2: {}", e);
    // }
    // arch::x86::gdt::init();
    // #[cfg(not(test))]
    // arch::x86::set_idt();
    // #[allow(static_mut_refs)]
    // shell::launch(unsafe { &mut SCREEN });
}
