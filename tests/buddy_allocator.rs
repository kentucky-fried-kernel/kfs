#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use kfs::boot::MultibootInfo;
use kfs::vmm::{
    allocators::buddy::BuddyAllocator,
    paging::{
        Access, Permissions,
        mmap::{Mode, mmap},
    },
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info)
}

#[test_case]
fn full_cache_usable() -> Result<(), &'static str> {
    let cache_memory = mmap(None, 1 << 20, Permissions::ReadWrite, Access::Root, Mode::Continous).map_err(|_| "Error::MmapFailure")?;
    let mut bm = BuddyAllocator::new(cache_memory as *const u8, 4096 * 8);
    for _ in 0..8 {
        if bm.alloc(4096).is_none() {
            return Err("Allocation failed when it should have been able to service the request");
        }
    }
    Ok(())
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{
        arch, qemu,
        vmm::{self, paging::init::init_memory},
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    if let Err(_) = vmm::allocators::kmalloc::init() {
        panic!("Failed to initialize kmalloc");
    }

    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
