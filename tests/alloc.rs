#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use kfs::alloc::vec::Vec;
use kfs::boot::MultibootInfo;
use kfs::printkln;
use kfs::{
    alloc::string::String,
    vmm::{self, allocators::backend::buddy::BUDDY_ALLOCATOR_SIZE, paging::PAGE_SIZE},
};

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
