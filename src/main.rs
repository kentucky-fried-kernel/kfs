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
    shell::Shell,
    vmm::paging::{
        PAGE_SIZE,
        mmap::{Mode, mmap},
    },
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
    arch::x86::idt::init();

    init_memory(info);

    kfs::ps2::init();

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    let mode = Mode::Continous;
    let space = mmap(None, PAGE_SIZE, vmm::paging::Permissions::ReadWrite, vmm::paging::Access::User, &mode).unwrap();
    let space = mmap(None, PAGE_SIZE, vmm::paging::Permissions::ReadWrite, vmm::paging::Access::User, &mode).unwrap();

    kfs::serial_println!("space {:?}", space);

    loop {}
    // #[allow(static_mut_refs)]
    // let mut shell = Shell::default(unsafe { &mut kfs::terminal::SCREEN },
    // Keyboard::new(Layout::new(map_qwerty))); shell.launch();
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn user_program(intno: u32, stack_ptr: u32) {
    core::arch::naked_asm!("a:", "jmp a")
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
