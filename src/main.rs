#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::{
    boot::MultibootInfo,
    serial_println,
    terminal::{entry::Entry, vga::Buffer},
};

mod panic;

extern crate alloc;

/// # Panics
/// This function will panic if initialization of dynamic memory allocation fails.
// #[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{
        arch,
        terminal::{self, Screen},
        vmm::{self, paging::init::init_memory},
    };

    init_memory(info);
    arch::x86::gdt::init();
    arch::x86::idt::init();

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    let mut s = Screen::default();
    for i in 0..20 {
        for _ in 0..60 {
            s.push(Entry::new(b'a'));
        }
        s.push(Entry::new(b'\n'));
    }
    for i in 0..20 {
        for _ in 0..50 {
            s.push(Entry::new(b'a'));
        }
        s.push(Entry::new(b'\n'));
    }

    let b = Buffer::from_screen(&mut s);

    b.flush();
    // #[allow(static_mut_refs)]
    // shell::launch(unsafe { &mut terminal::SCREEN });
}

// /// # Panics
// /// This function will panic if initialization of dynamic memory allocation fails.
// #[cfg(test)]
// #[unsafe(no_mangle)]
// pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
//     use kfs::{arch, qemu, vmm};
//
//     arch::x86::gdt::init();
//     arch::x86::idt::init();
//
//     vmm::paging::init::init_memory(info);
//
//     if vmm::allocators::kmalloc::init().is_err() {
//         panic!("Failed to initialize kmalloc");
//     }
//
//     test_main();
//
//     unsafe { qemu::exit(qemu::ExitCode::Success) };
// }
