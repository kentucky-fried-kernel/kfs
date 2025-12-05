#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::boot::MultibootInfo;

mod panic;

extern crate alloc;

// #[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use alloc::{string::String, vec};
    use kfs::{
        arch, serial_println, shell, terminal,
        vmm::{self, paging::init::init_memory},
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    let mut hm: alloc::collections::btree_map::BTreeMap<usize, usize> = alloc::collections::btree_map::BTreeMap::new();
    // hm.insert(String::from(":"), 1);

    // let s = vec![1, 2, 3];
    let s = String::from("asd");
    serial_println!("{:?}", s);
    for i in 0..2 {
        hm.insert(i, i);
    }
    serial_println!("{:?}", hm);

    // printkln!("{:?}", hm);

    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

// #[cfg(test)]
// #[unsafe(no_mangle)]
// pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
//     use kfs::{arch, qemu, vmm};

//     arch::x86::gdt::init();
//     arch::x86::idt::init();

//     vmm::paging::init::init_memory(info.mem_upper as usize, info.mem_lower as usize);

//     test_main();
//     unsafe { qemu::exit(qemu::ExitCode::Success) };
// }
