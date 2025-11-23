#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(not(test))]
use kfs::boot::MultibootInfo;

mod panic;

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(magic: usize, info: &MultibootInfo) {
    use kfs::{arch, printkln, shell, terminal};

    printkln!("Multiboot Magic: {:x}", magic);
    printkln!("Multiboot Info :  {}", info);
    printkln!("_start       : 0x{:x}", kfs::boot::_start as *const () as usize);
    printkln!("kmain        : 0x{:x}", kmain as *const () as usize);
    printkln!("idt::init    : 0x{:x}", arch::x86::idt::init as *const () as usize);
    printkln!("higher_half  : 0x{:x}", kfs::boot::higher_half as *const () as usize);

    arch::x86::gdt::init();
    arch::x86::idt::init();

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
