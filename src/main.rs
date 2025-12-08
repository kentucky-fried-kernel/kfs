#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::{boot::MultibootInfo, printkln, serial_println, vmm::paging::PAGE_SIZE};

mod panic;

extern crate alloc;

fn alloc_bunch() {
    use alloc::{string::String, vec::Vec};
    let s = String::from("12345678");
    let v: Vec<u8> = Vec::with_capacity(4096);
    core::hint::black_box(&s);
    core::hint::black_box(&v);

    for size in [8, 64, 256, 2048, 8192, 32768, 1280000] {
        printkln!("before alloc");
        let mut v1: Vec<usize> = Vec::with_capacity(size);
        let mut v2: Vec<usize> = Vec::with_capacity(size);
        printkln!("after alloc");
        core::hint::black_box(&v1);
        core::hint::black_box(&v2);

        for idx in 0..size {
            v1.push(idx * 4);
            v2.push(idx * 1000);
        }

        for (idx, (e1, e2)) in v1.iter().zip(&v2).enumerate() {
            assert!(*e1 == idx * 4, "Memory corruption error");
            assert!(*e2 == idx * 1000, "Memory corruption error");
        }

        serial_println!("v1: {:?}", &v1[size - 1]);
        serial_println!("v2: {:?}", &v2[size - 1]);
    }
    for _ in 0..10 {
        let v = Vec::<u8>::with_capacity(PAGE_SIZE);
        core::hint::black_box(&v);
    }

    let p1 = Vec::<u8>::with_capacity(PAGE_SIZE);
    let p2 = Vec::<u8>::with_capacity(PAGE_SIZE);
    let p3 = Vec::<u8>::with_capacity(PAGE_SIZE);
    let p4 = Vec::<u8>::with_capacity(PAGE_SIZE);

    let p1_ptr = p1.as_ptr();
    let p2_ptr = p2.as_ptr();
    let p3_ptr = p3.as_ptr();
    let p4_ptr = p4.as_ptr();

    assert_eq!(p2_ptr as usize - p1_ptr as usize, PAGE_SIZE);
    assert_eq!(p3_ptr as usize - p2_ptr as usize, PAGE_SIZE);
    assert_eq!(p4_ptr as usize - p3_ptr as usize, PAGE_SIZE);

    let mut s = String::from("a");
    core::hint::black_box(&s);
    for _ in 0..8192 {
        s.push('L');
        core::hint::black_box(&s);
    }
}

/// # Panics
/// This function will panic initialization of dynamic memory allocation fails.
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{
        arch, shell, terminal,
        vmm::{self, allocators::kmalloc::kmalloc, paging::init::init_memory},
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    alloc_bunch();
    let ptr = kmalloc(8).unwrap();
    printkln!("{:p}", ptr);
    let ptr = kmalloc(16).unwrap();
    printkln!("{:p}", ptr);
    let ptr = kmalloc(32).unwrap();
    printkln!("{:p}", ptr);
    let ptr = kmalloc(64).unwrap();
    printkln!("{:p}", ptr);
    let ptr = kmalloc(128).unwrap();
    printkln!("{:p}", ptr);
    let ptr = kmalloc(256).unwrap();
    printkln!("{:p}", ptr);
    let ptr = kmalloc(512).unwrap();
    printkln!("{:p}", ptr);
    let ptr = kmalloc(1024).unwrap();
    printkln!("{:p}", ptr);
    let ptr = kmalloc(2048).unwrap();
    printkln!("{:p}", ptr);
    const SIZE: usize = 4096 * 64;
    let ptr = kmalloc(SIZE).unwrap();
    printkln!("{:p} - {:x}", ptr, ptr as usize + SIZE);

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
