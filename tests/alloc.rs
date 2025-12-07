#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use kfs::alloc::vec::Vec;
use kfs::boot::MultibootInfo;
use kfs::{
    alloc::string::String,
    vmm::{self, allocators::backend::buddy::BUDDY_ALLOCATOR_SIZE, paging::PAGE_SIZE},
};
use kfs::{kassert, kassert_eq};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info)
}

#[test_case]
fn alloc_string() -> Result<(), &'static str> {
    let mut s = String::from("a");

    for _ in 0..(PAGE_SIZE * 16) {
        s.push_str("LET THE BUFFER GROW");
    }

    Ok(())
}

#[test_case]
fn consecutive_allocations() -> Result<(), &'static str> {
    let p1 = Vec::<u8>::with_capacity(PAGE_SIZE);
    let p2 = Vec::<u8>::with_capacity(PAGE_SIZE);
    let p3 = Vec::<u8>::with_capacity(PAGE_SIZE);
    let p4 = Vec::<u8>::with_capacity(PAGE_SIZE);

    let p1_ptr = p1.as_ptr();
    let p2_ptr = p2.as_ptr();
    let p3_ptr = p3.as_ptr();
    let p4_ptr = p4.as_ptr();

    kassert_eq!(p2_ptr as usize - p1_ptr as usize, PAGE_SIZE);
    kassert_eq!(p3_ptr as usize - p2_ptr as usize, PAGE_SIZE);
    kassert_eq!(p4_ptr as usize - p3_ptr as usize, PAGE_SIZE);

    Ok(())
}

#[test_case]
fn reuse_after_free() -> Result<(), &'static str> {
    let mut addresses = [0usize; 10];

    for a in addresses.iter_mut() {
        let v = Vec::<u8>::with_capacity(PAGE_SIZE);
        core::hint::black_box(&v);
        *a = v.as_ptr() as usize;
    }

    let unique_count = {
        let mut sorted = addresses;
        sorted.sort_unstable();
        sorted.windows(2).filter(|w| w[0] != w[1]).count() + 1
    };

    kassert!(unique_count < 5);

    Ok(())
}

#[test_case]
fn drop() -> Result<(), &'static str> {
    {
        let v: Vec<u8> = Vec::with_capacity(BUDDY_ALLOCATOR_SIZE / 2);
        core::hint::black_box(&v);
    }

    {
        let v: Vec<u8> = Vec::with_capacity(BUDDY_ALLOCATOR_SIZE / 2);
        core::hint::black_box(&v);
    }

    {
        let v: Vec<u8> = Vec::with_capacity(BUDDY_ALLOCATOR_SIZE / 2);
        core::hint::black_box(&v);
    }

    Ok(())
}

#[test_case]
fn mixed_size_allocations() -> Result<(), &'static str> {
    let v1 = Vec::<u8>::with_capacity(PAGE_SIZE);
    let v2 = Vec::<u8>::with_capacity(PAGE_SIZE * 2);
    let v3 = Vec::<u8>::with_capacity(PAGE_SIZE * 4);
    let v4 = Vec::<u8>::with_capacity(PAGE_SIZE * 8);

    let p1 = v1.as_ptr() as usize;
    let p2 = v2.as_ptr() as usize;
    let p3 = v3.as_ptr() as usize;
    let p4 = v4.as_ptr() as usize;

    kassert!(p2 >= p1 + PAGE_SIZE || p1 >= p2 + PAGE_SIZE * 2);
    kassert!(p3 >= p2 + PAGE_SIZE * 2 || p2 >= p3 + PAGE_SIZE * 4);
    kassert!(p4 >= p3 + PAGE_SIZE * 4 || p3 >= p4 + PAGE_SIZE * 8);

    Ok(())
}

#[test_case]
fn stress_test_many_allocations() -> Result<(), &'static str> {
    for _ in 0..10000 {
        let v = Vec::<u8>::with_capacity(PAGE_SIZE);
        core::hint::black_box(&v);
    }

    Ok(())
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{arch, qemu, vmm::paging::init::init_memory};

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize dynamic memory allocation");
    }

    test_main();

    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
