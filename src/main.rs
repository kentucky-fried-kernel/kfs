#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod panic;

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    panic!("yoyoyo");

    use kfs::{arch, printkln, ps2, shell, terminal};
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
pub extern "C" fn kmain() {
    use kfs::qemu;
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
