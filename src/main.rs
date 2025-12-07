#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::boot::MultibootInfo;

mod panic;

extern crate alloc;

/// # Panics
/// This function will panic initialization of dynamic memory allocation fails.
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use alloc::string::String;
    use kfs::{
        arch, serial_println, shell, terminal,
        vmm::{
            self,
            paging::{PAGE_SIZE, init::init_memory},
        },
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    let mut s = String::from("a");

    for _ in 0..(PAGE_SIZE * 16) {
        s.push_str("LET THE BUFFER GROW");
    }

    serial_println!("{}", s);

    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{arch, qemu, vmm};

    arch::x86::gdt::init();
    arch::x86::idt::init();

    vmm::paging::init::init_memory(info.mem_upper as usize, info.mem_lower as usize);

    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
