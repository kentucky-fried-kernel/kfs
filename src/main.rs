#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::boot::MultibootInfo;

mod panic;

extern crate alloc;

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use alloc::{string::String, vec};
    use kfs::{
        arch, printkln, shell, terminal,
        vmm::{self, paging::init::init_memory},
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    {
        let s = String::from("asd");
        printkln!("Heap-allocated String {:?}", s);
    }
    let v = vec![1, 2, 3];
    printkln!("Heap-allocated Vec {:?}", v);
    let mut hm = alloc::collections::btree_map::BTreeMap::new();

    for i in 0..3 {
        use alloc::format;

        hm.insert(format!("i{i}"), i);
    }
    printkln!("Heap-allocated BTreeMap {:?}", hm);

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
