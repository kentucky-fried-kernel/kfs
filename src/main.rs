#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::boot::MultibootInfo;

mod panic;

extern crate alloc;

/// # Panics
/// This function will panic if initialization of dynamic memory allocation fails.
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{
        arch, serial_println, shell, terminal,
        vmm::{
            self,
            paging::{Access, Permissions, init::init_memory, mmap::Mode},
        },
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }
    let e = vmm::paging::mmap::mmap(None, 4096, Permissions::ReadWrite, Access::Root, &Mode::Continous);
    serial_println!("{:?}", e);

    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

/// # Panics
/// This function will panic if initialization of dynamic memory allocation fails.
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{arch, qemu, vmm};

    arch::x86::gdt::init();
    arch::x86::idt::init();

    vmm::paging::init::init_memory(info);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    test_main();

    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
