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
        arch, shell, terminal,
        vmm::{self, paging::init::init_memory},
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    // let mut ps = [core::ptr::null(); 16];

    // use kfs::vmm::allocators::kmalloc::{kfree, kmalloc};
    // // 256 bytes objects are stored on order 0 slabs, which means one slab contains a maximum of
    // // 15 allocations. The 16th allocation should successfully move to the next slab instead of
    // // failing.
    // for p in ps.iter_mut() {
    //     use kfs::serial_println;

    //     *p = kmalloc(256).unwrap();
    //     serial_println!("{:p}", *p);
    // }

    // for p in ps {
    //     let _ = unsafe { kfree(p) };
    // }

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
