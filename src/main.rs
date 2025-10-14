#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[test_case]
fn foo() -> Result<(), &'static str> {
    Ok(())
}

#[test_case]
fn bar() -> Result<(), &'static str> {
    Ok(())
}

pub mod arch;
pub mod conv;
pub mod macros;
pub mod panic;
pub mod port;
pub mod printk;
pub mod ps2;
pub mod qemu;
pub mod serial;
pub mod shell;
pub mod terminal;
pub mod tester;

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    use crate::terminal;
    if let Err(e) = ps2::init() {
        panic!("could not initialize PS/2: {}", e);
    }
    arch::x86::gdt::init();
    #[cfg(not(test))]
    arch::x86::set_idt();
    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
