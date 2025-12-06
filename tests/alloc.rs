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
use kfs::{kassert_eq, printkln, serial_println};

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
fn pointers() -> Result<(), &'static str> {
    {
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
    }

    for _ in 0..12 {
        let p = Vec::<u8>::with_capacity(PAGE_SIZE);

        let p_ptr = p.as_ptr();
        serial_println!("p:  {:x}", p_ptr as usize);
    }

    Ok(())
}

#[test_case]
fn drop() -> Result<(), &'static str> {
    {
        let v: Vec<u8> = Vec::with_capacity(BUDDY_ALLOCATOR_SIZE / 2);
        // print to prevent compiler from optimizing out
        printkln!("{:?}", v);
    }

    {
        let v: Vec<u8> = Vec::with_capacity(BUDDY_ALLOCATOR_SIZE / 2);
        // print to prevent compiler from optimizing out
        printkln!("{:?}", v);
    }

    {
        let v: Vec<u8> = Vec::with_capacity(BUDDY_ALLOCATOR_SIZE / 2);
        // print to prevent compiler from optimizing out
        printkln!("{:?}", v);
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

    if let Err(_) = vmm::allocators::kmalloc::init() {
        panic!("Failed to initialize dynamic memory allocation");
    }

    test_main();

    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
