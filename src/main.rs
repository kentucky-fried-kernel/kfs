#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::boot::MultibootInfo;

mod panic;

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(magic: usize, info: &MultibootInfo) {
    use kfs::{
        arch,
        kmalloc::{self, kmalloc},
        printkln, shell, terminal, vmm,
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    kmalloc::init().unwrap();

    vmm::init_memory(info.mem_upper as usize, info.mem_lower as usize);

    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{arch, qemu, vmm};

    arch::x86::gdt::init();
    arch::x86::idt::init();

    vmm::init_memory(info.mem_upper as usize, info.mem_lower as usize);

    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
