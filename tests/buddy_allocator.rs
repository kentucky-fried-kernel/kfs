#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use kfs::boot::MultibootInfo;
use kfs::{
    kassert_eq,
    vmm::{
        self,
        allocators::kmalloc::{kfree, kfs::vmm::allocators::backend::slab_allocator::BUDDY_ALLOCATOR_SIZE, kmalloc},
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
        let ptr = kmalloc(BUDDY_ALLOCATOR_SIZE / 8);
        if ptr.is_err() {
            return Err("Allocation failed when it should have been able to service the request");
        }
        ptrs[idx] = ptr.unwrap();
    }

    Ok(())
}

#[test_case]
fn alloc_free_alloc() -> Result<(), &'static str> {
    let p1 = kmalloc(0x1000).map_err(|_| "Allocation failed")?;
    kfree(p1).map_err(|_| "Free failed")?;

    let p2 = kmalloc(0x1000).map_err(|_| "Allocation failed")?;

    kassert_eq!(p1, p2);

    kfree(p1).map_err(|_| "Free failed")?;

    Ok(())
}

#[test_case]
fn alloc_full_size() -> Result<(), &'static str> {
    let ptr = kmalloc(BUDDY_ALLOCATOR_SIZE);
    if ptr.is_err() {
        return Err("Could not allocate full size of Buddy Allocator buffer");
    }

    kfree(ptr.unwrap()).map_err(|_| "Free failed")?;

    Ok(())
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    if let Err(_) = vmm::allocators::kmalloc::init() {
        panic!("Failed to initialize kmalloc");
    }

    use kfs::{arch, qemu, vmm::paging::init::init_memory};

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
