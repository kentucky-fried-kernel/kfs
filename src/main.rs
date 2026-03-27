#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::{
    boot::MultibootInfo,
    keyboard::{
        Keyboard,
        layout::{Layout, map_qwerty},
    },
    serial_println,
    shell::Shell,
    vmm::{page::PAGE_SIZE, page_allocator::PAGE_ALLOCATOR},
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
        vmm::{self},
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    unsafe {
        #[allow(static_mut_refs)]
        let a = PAGE_ALLOCATOR.alloc_at(0x1000, 1).unwrap();
        kfs::printkln!("hello {:x}", a as usize);
    }

    loop {}

    // kfs::ps2::init();
}
//
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
//     kfs::ps2::init();
//
//     if vmm::allocators::kmalloc::init().is_err() {
//         panic!("Failed to initialize kmalloc");
//     }
//
//     test_main();
//
//     unsafe { qemu::exit(qemu::ExitCode::Success) };
// }
