#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use kfs::boot::MultibootInfo;
use kfs::{
    kassert, kassert_eq,
    vmm::{
        self,
        allocators::{
            backend::buddy::BUDDY_ALLOCATOR_SIZE,
            kmalloc::{buddy_allocator_alloc, buddy_allocator_free},
        },
        paging::PAGE_SIZE,
    },
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info)
}

#[test_case]
fn full_cache_usable() -> Result<(), &'static str> {
    let mut ptrs = [core::ptr::null() as *const u8; 8];
    for idx in 0..8 {
        let ptr = buddy_allocator_alloc(BUDDY_ALLOCATOR_SIZE / 8).map_err(|_| "Allocation failed when it should have been able to service the request")?;

        ptrs[idx] = ptr;
    }

    Ok(())
}

#[test_case]
fn alloc_free_alloc() -> Result<(), &'static str> {
    let p1 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Allocation failed")?;
    buddy_allocator_free(p1).map_err(|_| "Free failed")?;

    let p2 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Allocation failed")?;
    let p3 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Allocation failed")?;

    kassert_eq!(p1, p2);
    kassert!(p1 != p3, "buddy_allocator_alloc allocated the same address twice");

    buddy_allocator_free(p1).map_err(|_| "Free failed")?;
    buddy_allocator_free(p3).map_err(|_| "Free failed")?;

    let p = buddy_allocator_alloc(BUDDY_ALLOCATOR_SIZE).map_err(|_| "Allocation failed")?;
    buddy_allocator_free(p).map_err(|_| "Free failed")?;

    Ok(())
}

#[test_case]
fn alloc_full_size() -> Result<(), &'static str> {
    let ptr = buddy_allocator_alloc(BUDDY_ALLOCATOR_SIZE).map_err(|_| "Could not allocate full size of Buddy Allocator buffer")?;

    buddy_allocator_free(ptr).map_err(|_| "Free failed")?;

    Ok(())
}

#[test_case]
fn alloc_page_size() -> Result<(), &'static str> {
    let p1 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Could not allocate full size of Buddy Allocator buffer")?;
    let p2 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Could not allocate full size of Buddy Allocator buffer")?;

    kassert_eq!(p2 as usize - p1 as usize, PAGE_SIZE);

    buddy_allocator_free(p1).map_err(|_| "Free failed")?;
    buddy_allocator_free(p2).map_err(|_| "Free failed")?;

    Ok(())
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{arch, qemu, vmm::paging::init::init_memory};

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    if let Err(_) = vmm::allocators::kmalloc::init_buddy_allocator() {
        panic!("Failed to initialize buddy allocator");
    }

    test_main();

    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
