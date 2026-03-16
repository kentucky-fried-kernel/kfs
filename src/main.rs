#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::asm;

use kfs::{
    arch::x86::{idt::InterruptRegisters, interrupts::irq},
    boot::MultibootInfo,
    keyboard::{
        Keyboard,
        layout::{Layout, map_qwerty},
    },
    ps2, scheduler, serial_println,
    shell::Shell,
    vmm::paging::PAGE_SIZE,
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
        vmm::{self, paging::init::init_memory},
    };

    arch::x86::gdt::init();

    init_memory(info);

    // kfs::ps2::init();

    // if vmm::allocators::kmalloc::init().is_err() {
    //     panic!("Failed to initialize kmalloc");
    // }

    // let mut k = Keyboard::new(Layout::new(map_qwerty));
    // while let None = k.next() {}
    // serial_println!("hello");

    // scheduler::sys_execve(scheduler::forked_function, PAGE_SIZE * 100);
    scheduler::sys_execve(scheduler::print_green, PAGE_SIZE * 100);
    scheduler::sys_execve(scheduler::print_red, PAGE_SIZE * 100);
    scheduler::sys_execve(scheduler::print_red, PAGE_SIZE * 100);
    arch::x86::idt::init();
    kfs::scheduler::init();
    loop {}
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
